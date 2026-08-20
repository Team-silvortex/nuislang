use crate::{
    final_executable_macho_application::apply_macho_arm64_patch_previews,
    final_executable_macho_input::parse_macho_arm64_object_linkage,
    final_executable_macho_layout::{build_macho_placement_binding_report, MachOLayoutObject},
    final_executable_macho_materialization::{
        build_macho_arm64_materialization_preview, MachOImageObject,
    },
    final_executable_macho_platform::build_macho_arm64_platform_structure_plan,
    final_executable_macho_relocation::build_macho_arm64_relocation_application_report,
    reports::NsldExecutableFinalizerInputSummary,
};
use std::collections::BTreeSet;

pub(crate) const MACHO_HOST_OBJECT_LINKAGE_CONTRACT: &str =
    "nuis-nsld-macho-host-object-linkage-v1";
const REQUIRED_HOST_OBJECT_ROLES: [&str; 2] = ["program-llvm", "runtime-shim"];

pub(crate) fn validate_macho_host_object_handoff(
    artifact: &nuisc::aot::NuisCompiledArtifact,
    plan: &nuisc::linker::LinkPlan,
) -> Result<(), String> {
    summarize_macho_host_object_handoff(artifact, plan).map(|_| ())
}

pub(crate) fn summarize_macho_host_object_handoff(
    artifact: &nuisc::aot::NuisCompiledArtifact,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldExecutableFinalizerInputSummary, String> {
    if artifact.host_objects.is_empty() {
        return Err("compiled artifact has no relocatable host objects".to_owned());
    }
    if artifact.host_objects.len() != plan.compiled_artifact.host_objects.len() {
        return Err(format!(
            "host object count mismatch: plan={}, artifact={}",
            plan.compiled_artifact.host_objects.len(),
            artifact.host_objects.len()
        ));
    }

    let mut object_ids = BTreeSet::new();
    let mut section_count = 0usize;
    let mut symbol_count = 0usize;
    let mut relocation_count = 0usize;
    let mut defined_symbol_count = 0usize;
    let mut undefined_symbol_count = 0usize;
    let mut external_definitions = BTreeSet::new();
    let mut external_undefined = BTreeSet::new();
    let mut parsed_objects = Vec::with_capacity(artifact.host_objects.len());
    for object in &artifact.host_objects {
        if !object_ids.insert(object.object_id.as_str()) {
            return Err(format!("duplicate host object id `{}`", object.object_id));
        }
        if canonical_object_format(&object.object_format)
            != canonical_object_format(&plan.cpu_target.object_format)
        {
            return Err(format!(
                "host object `{}` format mismatch: object={}, target={}",
                object.object_id, object.object_format, plan.cpu_target.object_format
            ));
        }
        validate_plan_identity(object, plan)?;
        let parsed = parse_macho_arm64_object_linkage(&object.bytes)
            .map_err(|error| format!("host object `{}`: {error}", object.object_id))?;
        section_count += parsed.section_count;
        symbol_count += parsed.symbol_count;
        relocation_count += parsed.relocation_count;
        defined_symbol_count += parsed.defined_symbol_count;
        undefined_symbol_count += parsed.undefined_symbol_count;
        external_definitions.extend(parsed.external_definitions.iter().cloned());
        external_undefined.extend(parsed.external_undefined.iter().cloned());
        parsed_objects.push((object, parsed));
    }

    for required_role in REQUIRED_HOST_OBJECT_ROLES {
        let count = artifact
            .host_objects
            .iter()
            .filter(|object| object.role == required_role)
            .count();
        if count != 1 {
            return Err(format!(
                "required host object role `{required_role}` must appear exactly once; found {count}"
            ));
        }
    }
    let internally_resolved = external_undefined
        .intersection(&external_definitions)
        .cloned()
        .collect::<BTreeSet<_>>();
    let unresolved_external_symbols = external_undefined
        .difference(&external_definitions)
        .cloned()
        .collect::<Vec<_>>();
    let status = if unresolved_external_symbols.is_empty() {
        "verified-internal-closure"
    } else {
        "verified-with-external-compatibility-boundary"
    };
    let placement_inputs = parsed_objects
        .iter()
        .map(|(object, linkage)| MachOLayoutObject {
            object_id: &object.object_id,
            role: &object.role,
            linkage,
        })
        .collect::<Vec<_>>();
    let placement_binding = build_macho_placement_binding_report(&placement_inputs)?;
    let relocation_application =
        build_macho_arm64_relocation_application_report(&placement_inputs, &placement_binding)?;
    if relocation_application.relocation_count != relocation_count {
        return Err(format!(
            "Mach-O relocation application coverage drift: parsed={relocation_count}, planned={}",
            relocation_application.relocation_count
        ));
    }
    let image_inputs = parsed_objects
        .iter()
        .map(|(object, linkage)| MachOImageObject {
            object_id: &object.object_id,
            role: &object.role,
            bytes: &object.bytes,
            linkage,
        })
        .collect::<Vec<_>>();
    let materialization_preview = build_macho_arm64_materialization_preview(
        &image_inputs,
        &placement_binding,
        &relocation_application,
    )?;
    let applied_image = apply_macho_arm64_patch_previews(
        &image_inputs,
        &placement_binding,
        &relocation_application,
        &materialization_preview,
    )?;
    if crate::fnv1a64_hex(&applied_image.bytes) != applied_image.report.applied_image_hash {
        return Err("Mach-O applied image handoff hash drift".to_owned());
    }
    let platform_structure_plan = build_macho_arm64_platform_structure_plan(
        &placement_binding,
        &relocation_application,
        &applied_image.report,
    )?;
    Ok(NsldExecutableFinalizerInputSummary {
        contract: MACHO_HOST_OBJECT_LINKAGE_CONTRACT.to_owned(),
        status: status.to_owned(),
        object_count: artifact.host_objects.len(),
        section_count,
        symbol_count,
        relocation_count,
        defined_symbol_count,
        undefined_symbol_count,
        internally_resolved_symbol_count: internally_resolved.len(),
        unresolved_external_symbol_count: unresolved_external_symbols.len(),
        unresolved_external_symbols,
        placement_binding,
        relocation_application,
        materialization_preview,
        patch_application: applied_image.report,
        platform_structure_plan,
    })
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
                "host object `{}` is absent from the link plan",
                object.object_id
            )
        })?;
    let actual_hash = crate::fnv1a64_hex(&object.bytes);
    if planned.role != object.role
        || canonical_object_format(&planned.object_format)
            != canonical_object_format(&object.object_format)
        || planned.bytes != object.bytes.len()
        || planned.content_hash != actual_hash
    {
        return Err(format!(
            "host object `{}` identity drift: plan role={} format={} bytes={} hash={}, artifact role={} format={} bytes={} hash={}",
            object.object_id,
            planned.role,
            planned.object_format,
            planned.bytes,
            planned.content_hash,
            object.role,
            object.object_format,
            object.bytes.len(),
            actual_hash
        ));
    }
    Ok(())
}

fn canonical_object_format(object_format: &str) -> &str {
    match object_format.trim().to_ascii_lowercase().as_str() {
        "mach-o" | "macho" => "mach-o",
        "elf" => "elf",
        "coff" | "pe" | "pe-coff" | "pe/coff" => "pe-coff",
        _ => "unknown",
    }
}
