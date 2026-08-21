use crate::{
    final_executable_elf_input::{parse_elf64_amd64_object_linkage, ParsedElfObjectLinkage},
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_materialization::{
        application::{
            apply_elf_amd64_patch_previews,
            platform::{
                application::{
                    apply_elf_amd64_platform_structure_plan, ElfAmd64PlatformPatchApplicationReport,
                },
                build_elf_amd64_platform_structure_plan, ElfAmd64PlatformStructurePlanReport,
            },
            ElfAmd64PatchApplicationReport,
        },
        build_elf_amd64_materialization_preview, ElfAmd64ImageObject,
    },
    final_executable_elf_materialization_report::ElfAmd64MaterializationPreviewReport,
    final_executable_elf_relocation::build_elf_amd64_relocation_application,
    final_executable_elf_relocation_report::ElfAmd64RelocationApplicationReport,
    final_executable_elf_shell::{
        build_elf_amd64_shell_layout_plan, serialize_elf_amd64_shell_image,
        ElfAmd64ShellImageSerializationReport, ElfAmd64ShellLayoutPlanReport,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ELF_AMD64_HOST_OBJECT_LINKAGE_CONTRACT: &str =
    "nuis-nsld-elf-amd64-host-object-linkage-v1";
const REQUIRED_HOST_OBJECT_ROLES: [&str; 2] = ["program-llvm", "runtime-shim"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64HostObjectLinkageSummary {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) object_count: usize,
    pub(crate) section_count: usize,
    pub(crate) symbol_count: usize,
    pub(crate) relocation_count: usize,
    pub(crate) defined_symbol_count: usize,
    pub(crate) undefined_symbol_count: usize,
    pub(crate) internally_resolved_symbols: Vec<String>,
    pub(crate) unresolved_external_symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ObjectLinkage {
    pub(crate) object_id: String,
    pub(crate) role: String,
    pub(crate) linkage: ParsedElfObjectLinkage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64HostObjectLinkage {
    pub(crate) summary: ElfAmd64HostObjectLinkageSummary,
    pub(crate) objects: Vec<ElfAmd64ObjectLinkage>,
    pub(crate) placement_binding: ElfAmd64PlacementBindingReport,
    pub(crate) relocation_application: ElfAmd64RelocationApplicationReport,
    pub(crate) materialization_preview: ElfAmd64MaterializationPreviewReport,
    pub(crate) patch_application: ElfAmd64PatchApplicationReport,
    pub(crate) platform_structure_plan: ElfAmd64PlatformStructurePlanReport,
    pub(crate) platform_patch_application: ElfAmd64PlatformPatchApplicationReport,
    pub(crate) shell_layout_plan: ElfAmd64ShellLayoutPlanReport,
    pub(crate) shell_image_serialization: ElfAmd64ShellImageSerializationReport,
    pub(crate) private_shell_image: Vec<u8>,
}

pub(crate) fn build_elf_amd64_host_object_linkage(
    artifact: &nuisc::aot::NuisCompiledArtifact,
    plan: &nuisc::linker::LinkPlan,
) -> Result<ElfAmd64HostObjectLinkage, String> {
    validate_object_set_shape(artifact, plan)?;
    let mut object_ids = BTreeSet::new();
    let mut roles = BTreeSet::new();
    let mut section_count = 0usize;
    let mut symbol_count = 0usize;
    let mut relocation_count = 0usize;
    let mut defined_symbol_count = 0usize;
    let mut undefined_symbol_count = 0usize;
    let mut external_definitions = BTreeSet::new();
    let mut external_undefined = BTreeSet::new();
    let mut strong_definitions = BTreeMap::<String, String>::new();
    let mut objects = Vec::with_capacity(artifact.host_objects.len());

    for object in &artifact.host_objects {
        if !object_ids.insert(object.object_id.as_str()) {
            return Err(format!(
                "ELF native handoff contains duplicate object id `{}`",
                object.object_id
            ));
        }
        if !roles.insert(object.role.as_str()) {
            return Err(format!(
                "ELF native handoff contains duplicate role `{}`",
                object.role
            ));
        }
        if canonical_object_format(&object.object_format) != "elf" {
            return Err(format!(
                "ELF host object `{}` declares format `{}`",
                object.object_id, object.object_format
            ));
        }
        validate_plan_identity(object, plan)?;
        let linkage = parse_elf64_amd64_object_linkage(&object.bytes).map_err(|error| {
            format!("ELF host object `{}` is invalid: {error}", object.object_id)
        })?;
        validate_relocation_symbols(&object.object_id, &linkage)?;
        for symbol in linkage
            .symbols
            .iter()
            .filter(|symbol| symbol.external && symbol.defined && !symbol.weak)
        {
            if let Some(previous) =
                strong_definitions.insert(symbol.name.clone(), object.object_id.clone())
            {
                return Err(format!(
                    "ELF strong symbol `{}` is defined by both `{previous}` and `{}`",
                    symbol.name, object.object_id
                ));
            }
        }
        section_count += linkage.section_count;
        symbol_count += linkage.symbol_count;
        relocation_count += linkage.relocation_count;
        defined_symbol_count += linkage.defined_symbol_count;
        undefined_symbol_count += linkage.undefined_symbol_count;
        external_definitions.extend(linkage.external_definitions.iter().cloned());
        external_undefined.extend(linkage.external_undefined.iter().cloned());
        objects.push(ElfAmd64ObjectLinkage {
            object_id: object.object_id.clone(),
            role: object.role.clone(),
            linkage,
        });
    }
    validate_required_roles(&roles)?;
    let internally_resolved_symbols = external_undefined
        .intersection(&external_definitions)
        .cloned()
        .collect::<Vec<_>>();
    let unresolved_external_symbols = external_undefined
        .difference(&external_definitions)
        .cloned()
        .collect::<Vec<_>>();
    let status = if unresolved_external_symbols.is_empty() {
        "verified-internal-closure"
    } else {
        "verified-with-external-compatibility-boundary"
    };
    let placement_binding = build_elf_amd64_placement_binding(&objects)?;
    let relocation_application =
        build_elf_amd64_relocation_application(&objects, &placement_binding)?;
    let image_objects = objects
        .iter()
        .map(|object| {
            let source = artifact
                .host_objects
                .iter()
                .find(|source| source.object_id == object.object_id)
                .ok_or_else(|| {
                    format!(
                        "ELF materialization source object `{}` is missing",
                        object.object_id
                    )
                })?;
            let planned = plan
                .compiled_artifact
                .host_objects
                .iter()
                .find(|planned| planned.object_id == object.object_id)
                .ok_or_else(|| {
                    format!(
                        "ELF materialization planned object `{}` is missing",
                        object.object_id
                    )
                })?;
            Ok(ElfAmd64ImageObject {
                object_id: &object.object_id,
                role: &object.role,
                bytes: &source.bytes,
                planned_size_bytes: planned.bytes,
                planned_source_hash: &planned.content_hash,
                linkage: &object.linkage,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let materialization_preview = build_elf_amd64_materialization_preview(
        &image_objects,
        &placement_binding,
        &relocation_application,
    )?;
    let applied_image = apply_elf_amd64_patch_previews(
        &image_objects,
        &placement_binding,
        &relocation_application,
        &materialization_preview,
    )?;
    if crate::fnv1a64_hex(&applied_image.bytes) != applied_image.report.applied_memory_image_hash {
        return Err("ELF applied image handoff hash drift".to_owned());
    }
    let platform_structure_plan = build_elf_amd64_platform_structure_plan(
        &placement_binding,
        &relocation_application,
        &applied_image.report,
    )?;
    let platform_applied_image = apply_elf_amd64_platform_structure_plan(
        &placement_binding,
        &relocation_application,
        &applied_image,
        &platform_structure_plan,
    )?;
    if crate::fnv1a64_hex(&platform_applied_image.bytes)
        != platform_applied_image.report.applied_memory_image_hash
    {
        return Err("ELF platform-applied image handoff hash drift".to_owned());
    }
    let shell_layout_plan = build_elf_amd64_shell_layout_plan(
        &objects,
        &placement_binding,
        &relocation_application,
        &platform_structure_plan,
        &platform_applied_image,
    )?;
    let shell_image = serialize_elf_amd64_shell_image(
        &objects,
        &placement_binding,
        &relocation_application,
        &platform_structure_plan,
        &platform_applied_image,
        &shell_layout_plan,
    )?;
    if shell_image.bytes.len() != shell_image.report.shell_image_span_bytes
        || crate::fnv1a64_hex(&shell_image.bytes) != shell_image.report.shell_image_hash
        || shell_image.report.serialization_ledger_hash
            != crate::fnv1a64_hex(shell_image.report.canonical_ledger().as_bytes())
    {
        return Err("ELF private shell image/report handoff drift".to_owned());
    }
    Ok(ElfAmd64HostObjectLinkage {
        summary: ElfAmd64HostObjectLinkageSummary {
            contract: ELF_AMD64_HOST_OBJECT_LINKAGE_CONTRACT,
            status: status.to_owned(),
            object_count: objects.len(),
            section_count,
            symbol_count,
            relocation_count,
            defined_symbol_count,
            undefined_symbol_count,
            internally_resolved_symbols,
            unresolved_external_symbols,
        },
        objects,
        placement_binding,
        relocation_application,
        materialization_preview,
        patch_application: applied_image.report,
        platform_structure_plan,
        platform_patch_application: platform_applied_image.report,
        shell_layout_plan,
        shell_image_serialization: shell_image.report,
        private_shell_image: shell_image.bytes,
    })
}

fn validate_object_set_shape(
    artifact: &nuisc::aot::NuisCompiledArtifact,
    plan: &nuisc::linker::LinkPlan,
) -> Result<(), String> {
    if artifact.host_objects.len() != REQUIRED_HOST_OBJECT_ROLES.len() {
        return Err(format!(
            "ELF native handoff requires two host objects, found {}",
            artifact.host_objects.len()
        ));
    }
    if plan.compiled_artifact.host_objects.len() != artifact.host_objects.len() {
        return Err(format!(
            "ELF host-object count mismatch: plan={}, artifact={}",
            plan.compiled_artifact.host_objects.len(),
            artifact.host_objects.len()
        ));
    }
    Ok(())
}

fn validate_required_roles(roles: &BTreeSet<&str>) -> Result<(), String> {
    let expected = BTreeSet::from(REQUIRED_HOST_OBJECT_ROLES);
    if *roles == expected {
        return Ok(());
    }
    Err(format!(
        "ELF native handoff roles must be program-llvm and runtime-shim, found {}",
        roles.iter().copied().collect::<Vec<_>>().join(",")
    ))
}

fn validate_plan_identity(
    object: &nuisc::aot::NuisCompiledArtifactHostObject,
    plan: &nuisc::linker::LinkPlan,
) -> Result<(), String> {
    let planned = plan
        .compiled_artifact
        .host_objects
        .iter()
        .find(|planned| planned.object_id == object.object_id)
        .ok_or_else(|| {
            format!(
                "ELF host object `{}` is missing from the link plan",
                object.object_id
            )
        })?;
    let actual_hash = crate::fnv1a64_hex(&object.bytes);
    if planned.role != object.role {
        return Err(format!(
            "ELF host object `{}` role mismatch: plan={}, artifact={}",
            object.object_id, planned.role, object.role
        ));
    }
    if canonical_object_format(&planned.object_format) != "elf" {
        return Err(format!(
            "ELF host object `{}` plan format is `{}`",
            object.object_id, planned.object_format
        ));
    }
    if planned.bytes != object.bytes.len() {
        return Err(format!(
            "ELF host object `{}` size mismatch: plan={}, artifact={}",
            object.object_id,
            planned.bytes,
            object.bytes.len()
        ));
    }
    if planned.content_hash != actual_hash {
        return Err(format!(
            "ELF host object `{}` hash mismatch: plan={}, artifact={actual_hash}",
            object.object_id, planned.content_hash
        ));
    }
    Ok(())
}

fn validate_relocation_symbols(
    object_id: &str,
    linkage: &ParsedElfObjectLinkage,
) -> Result<(), String> {
    for relocation in &linkage.relocations {
        let symbol = &linkage.symbols[relocation.symbol_index];
        if relocation.relocation_type != 0 && !symbol.defined && symbol.name.is_empty() {
            return Err(format!(
                "ELF host object `{object_id}` relocation section {} references unnamed undefined symbol {}",
                relocation.relocation_section_index, symbol.index
            ));
        }
    }
    Ok(())
}

fn canonical_object_format(object_format: &str) -> &str {
    match object_format.trim().to_ascii_lowercase().as_str() {
        "elf" => "elf",
        _ => "unknown",
    }
}

#[cfg(test)]
#[path = "final_executable_elf_object_tests.rs"]
mod tests;
