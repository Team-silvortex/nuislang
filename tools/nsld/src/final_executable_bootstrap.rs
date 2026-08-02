use super::{
    container_pipeline::nsld_container_report, reports::NsldFinalExecutableImageDryRunVerifyReport,
};
use nuis_runtime::{
    plan_lifecycle_bootstrap, AppliedRelocationFacts, LifecycleBootstrapFacts,
    LifecycleBootstrapPlan, MappedSectionFacts, RuntimeServiceBindingFacts,
};
use std::path::Path;

pub(crate) fn nsld_final_output_bootstrap_plan(
    manifest: &Path,
    link_plan: &nuisc::linker::LinkPlan,
    image_verified: bool,
    container_handoff_ready: bool,
    image: &NsldFinalExecutableImageDryRunVerifyReport,
) -> LifecycleBootstrapPlan {
    let container = nsld_container_report(manifest, link_plan);
    let loader_symbol = container.loader_symbols.first();
    let entry_relocation = container.relocations.first();
    let mapping_ready = image_verified && container_handoff_ready && container.ready;
    let relocation_ready = mapping_ready
        && image.actual_relocation_patch_application_status.as_deref() == Some("applied")
        && image.actual_relocation_patch_application_count == Some(container.relocations.len())
        && image.actual_relocation_patch_byte_audit_status.as_deref() == Some("verified")
        && image.actual_relocation_patch_byte_audit_count == Some(container.relocations.len());
    let mapped_sections = container
        .sections
        .iter()
        .map(|section| MappedSectionFacts {
            section_id: section.section_id.clone(),
            section_kind: section.section_kind.clone(),
            offset: section.offset,
            size_bytes: section.size_bytes,
            payload_hash: section.payload_hash.clone(),
            required: section.required,
            mapping_status: if mapping_ready { "mapped" } else { "blocked" }.to_owned(),
        })
        .collect();
    let applied_relocations = container
        .relocations
        .iter()
        .map(|relocation| AppliedRelocationFacts {
            relocation_id: relocation.relocation_id.clone(),
            relocation_kind: relocation.relocation_kind.clone(),
            source_section_id: relocation.source_section_id.clone(),
            source_offset: relocation.source_offset,
            target_symbol_id: relocation.target_symbol_id.clone(),
            addend: relocation.addend,
            application_status: if relocation_ready {
                "applied"
            } else {
                "blocked"
            }
            .to_owned(),
        })
        .collect();
    let runtime_service_bindings = container
        .metadata_bindings
        .iter()
        .filter(|binding| {
            matches!(
                binding.binding_id.as_str(),
                nuis_runtime::CLOCK_ROOT_BINDING_ID | nuis_runtime::GLM_ROOT_BINDING_ID
            )
        })
        .map(|binding| RuntimeServiceBindingFacts {
            binding_id: binding.binding_id.clone(),
            contract: binding.contract.clone(),
            value_count: binding.value_count,
            value_hash: binding.value_hash.clone(),
            validation_status: binding.validation_status.clone(),
            required: binding.required,
        })
        .collect();
    plan_lifecycle_bootstrap(&LifecycleBootstrapFacts {
        image_verified,
        container_handoff_ready,
        scheduler_entry: "nuis.scheduler.loop.v1".to_owned(),
        process_lifecycle_hook: "on_process_start".to_owned(),
        loader_entry_kind: Some(container.loader_entry_kind.clone()),
        loader_entry_abi_contract: Some(container.loader_entry_abi_contract.clone()),
        loader_entry_machine_arch: Some(container.loader_entry_machine_arch.clone()),
        loader_entry_symbol: Some(container.loader_entry_symbol.clone()),
        loader_entry_section_id: Some(container.loader_entry_section_id.clone()),
        loader_symbol_status: if loader_symbol.is_some() {
            "parsed"
        } else {
            "missing"
        }
        .to_owned(),
        loader_symbol_kind: loader_symbol.map(|symbol| symbol.symbol_kind.clone()),
        loader_symbol_name: loader_symbol.map(|symbol| symbol.symbol_name.clone()),
        loader_symbol_lifecycle_hook: loader_symbol.map(|symbol| symbol.lifecycle_hook.clone()),
        loader_symbol_section_id: loader_symbol.map(|symbol| symbol.section_id.clone()),
        loader_symbol_offset: loader_symbol.map(|symbol| symbol.offset),
        loader_symbol_size_bytes: loader_symbol.map(|symbol| symbol.size_bytes),
        loader_symbol_payload_hash: loader_symbol.map(|symbol| symbol.payload_hash.clone()),
        relocation_targets_loader_symbol: entry_relocation
            .zip(loader_symbol)
            .is_some_and(|(relocation, symbol)| relocation.target_symbol_id == symbol.symbol_id),
        relocation_source_matches_loader_symbol: entry_relocation
            .zip(loader_symbol)
            .is_some_and(|(relocation, symbol)| relocation.source_section_id == symbol.section_id),
        source_section_count: container.section_count,
        source_section_table_hash: container.container_section_table_hash,
        mapped_sections,
        source_relocation_count: container.relocations.len(),
        source_relocation_table_hash: container.relocation_table_hash,
        applied_relocations,
        runtime_service_bindings,
        provider_dispatch_status: container.provider_dispatch_validation_status,
    })
}
