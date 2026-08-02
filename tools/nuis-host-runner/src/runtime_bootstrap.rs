use crate::container::ContainerLoaderSummary;
use nuis_runtime::{
    plan_lifecycle_bootstrap, AppliedRelocationFacts, LifecycleBootstrapFacts,
    LifecycleBootstrapPlan, MappedSectionFacts, RuntimeServiceBindingFacts, CLOCK_ROOT_BINDING_ID,
    GLM_ROOT_BINDING_ID,
};

pub(super) fn runtime_bootstrap_plan(
    container: &ContainerLoaderSummary,
    image_verified: bool,
    scheduler_entry: &str,
    lifecycle_hook: &str,
) -> LifecycleBootstrapPlan {
    let mapping_ready = image_verified && container.handoff_ready;
    let mapped_sections = container
        .container_section
        .entries
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
        .relocation
        .entries
        .iter()
        .map(|relocation| AppliedRelocationFacts {
            relocation_id: relocation.relocation_id.clone(),
            relocation_kind: relocation.relocation_kind.clone(),
            source_section_id: relocation.source_section_id.clone(),
            source_offset: relocation.source_offset,
            target_symbol_id: relocation.target_symbol_id.clone(),
            addend: relocation.addend,
            application_status: if mapping_ready { "applied" } else { "blocked" }.to_owned(),
        })
        .collect();
    plan_lifecycle_bootstrap(&LifecycleBootstrapFacts {
        image_verified,
        container_handoff_ready: container.handoff_ready,
        scheduler_entry: scheduler_entry.to_owned(),
        process_lifecycle_hook: lifecycle_hook.to_owned(),
        loader_entry_kind: container.loader_entry_kind.clone(),
        loader_entry_symbol: container.loader_entry_symbol.clone(),
        loader_entry_section_id: container.loader_entry_section_id.clone(),
        loader_symbol_status: container.loader_symbol.status.clone(),
        loader_symbol_kind: container.loader_symbol.symbol_kind.clone(),
        loader_symbol_name: container.loader_symbol.symbol_name.clone(),
        loader_symbol_lifecycle_hook: container.loader_symbol.lifecycle_hook.clone(),
        loader_symbol_section_id: container.loader_symbol.section_id.clone(),
        relocation_targets_loader_symbol: container.relocation.first_targets_loader_symbol,
        relocation_source_matches_loader_symbol: container
            .relocation
            .first_source_matches_loader_symbol,
        source_section_count: container.container_section.declared_count.unwrap_or(0),
        source_section_table_hash: container
            .container_section_table_hash
            .clone()
            .unwrap_or_default(),
        mapped_sections,
        source_relocation_count: container.relocation.declared_count.unwrap_or(0),
        source_relocation_table_hash: container.relocation_table_hash.clone().unwrap_or_default(),
        applied_relocations,
        runtime_service_bindings: runtime_service_binding_facts(&container.metadata_binding),
        provider_dispatch_status: container.provider_dispatch.status.clone(),
    })
}

fn runtime_service_binding_facts(
    summary: &crate::container_metadata_binding::MetadataBindingSummary,
) -> Vec<RuntimeServiceBindingFacts> {
    let mut bindings = Vec::new();
    if let (Some(contract), Some(value_count), Some(value_hash), Some(validation_status)) = (
        summary.clock_root_contract.clone(),
        summary.clock_root_count,
        summary.clock_root_hash.clone(),
        summary.clock_root_status.clone(),
    ) {
        bindings.push(RuntimeServiceBindingFacts {
            binding_id: CLOCK_ROOT_BINDING_ID.to_owned(),
            contract,
            value_count,
            value_hash,
            validation_status,
            required: summary.clock_root_required.unwrap_or(false),
        });
    }
    if let (Some(contract), Some(value_count), Some(value_hash), Some(validation_status)) = (
        summary.glm_root_contract.clone(),
        summary.glm_root_count,
        summary.glm_root_hash.clone(),
        summary.glm_root_status.clone(),
    ) {
        bindings.push(RuntimeServiceBindingFacts {
            binding_id: GLM_ROOT_BINDING_ID.to_owned(),
            contract,
            value_count,
            value_hash,
            validation_status,
            required: summary.glm_root_required.unwrap_or(false),
        });
    }
    bindings
}
