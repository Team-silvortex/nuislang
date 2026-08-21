use crate::{
    final_executable_elf_input::{parse_elf64_amd64_object_linkage, ParsedElfObjectLinkage},
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_relocation::build_elf_amd64_relocation_application,
    final_executable_elf_relocation_report::ElfAmd64RelocationApplicationReport,
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
