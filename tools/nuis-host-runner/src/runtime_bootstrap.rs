use crate::container::ContainerLoaderSummary;
use nuis_runtime::{
    plan_lifecycle_bootstrap, prepare_lifecycle_bootstrap_execution, AppliedRelocationFacts,
    CompiledEntryTransferResult, LifecycleBootstrapFacts, LifecycleBootstrapPlan,
    MappedSectionFacts, OwnedAppliedRelocationHandle, OwnedImageMapping, OwnedMappedSectionHandle,
    OwnedRuntimeServiceHandle, RuntimeServiceBindingFacts, CLOCK_ROOT_BINDING_ID,
    GLM_ROOT_BINDING_ID,
};

pub(super) struct RuntimeBootstrapHandoff {
    pub(super) plan: LifecycleBootstrapPlan,
    pub(super) execution_protocol: &'static str,
    pub(super) transfer: CompiledEntryTransferResult,
}

pub(super) fn runtime_bootstrap_handoff(
    container: &ContainerLoaderSummary,
    image_verified: bool,
    mapped_image_hash: &str,
    mapped_image_size_bytes: usize,
    scheduler_entry: &str,
    lifecycle_hook: &str,
) -> RuntimeBootstrapHandoff {
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
    let facts = LifecycleBootstrapFacts {
        image_verified,
        container_handoff_ready: container.handoff_ready,
        scheduler_entry: scheduler_entry.to_owned(),
        process_lifecycle_hook: lifecycle_hook.to_owned(),
        loader_entry_kind: container.loader_entry_kind.clone(),
        loader_entry_abi_contract: container.loader_entry_abi_contract.clone(),
        loader_entry_machine_arch: container.loader_entry_machine_arch.clone(),
        loader_entry_symbol: container.loader_entry_symbol.clone(),
        loader_entry_section_id: container.loader_entry_section_id.clone(),
        loader_symbol_status: container.loader_symbol.status.clone(),
        loader_symbol_kind: container.loader_symbol.symbol_kind.clone(),
        loader_symbol_name: container.loader_symbol.symbol_name.clone(),
        loader_symbol_lifecycle_hook: container.loader_symbol.lifecycle_hook.clone(),
        loader_symbol_section_id: container.loader_symbol.section_id.clone(),
        loader_symbol_offset: container.loader_symbol.offset,
        loader_symbol_size_bytes: container.loader_symbol.size_bytes,
        loader_symbol_payload_hash: container.loader_symbol.payload_hash.clone(),
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
    };
    let plan = plan_lifecycle_bootstrap(&facts);
    let section_handles = facts
        .mapped_sections
        .iter()
        .map(|facts| OwnedMappedSectionHandle::from_facts(&plan.identity_hash, facts))
        .collect();
    let relocation_handles = facts
        .applied_relocations
        .iter()
        .map(|facts| OwnedAppliedRelocationHandle::from_facts(&plan.identity_hash, facts))
        .collect();
    let service_handles = facts
        .runtime_service_bindings
        .iter()
        .map(|facts| OwnedRuntimeServiceHandle::from_facts(&plan.identity_hash, facts))
        .collect();
    let preparation = prepare_lifecycle_bootstrap_execution(
        &facts,
        OwnedImageMapping::new(mapped_image_hash, mapped_image_size_bytes),
        section_handles,
        relocation_handles,
        service_handles,
    );
    let execution_protocol = preparation.protocol;
    let transfer = preparation.transfer();
    RuntimeBootstrapHandoff {
        plan,
        execution_protocol,
        transfer,
    }
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
