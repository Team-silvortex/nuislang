use super::*;
use crate::{
    ExecutableEntryRequest, ExecutableMemoryAdapter, NativeEntryInvocationPermit,
    NativeHostExecutableMemoryAdapter, CLOCK_ROOT_BINDING_ID, CLOCK_ROOT_CONTRACT,
    GLM_ROOT_BINDING_ID, GLM_ROOT_CONTRACT,
};

fn ready_facts() -> LifecycleBootstrapFacts {
    LifecycleBootstrapFacts {
        image_verified: true,
        container_handoff_ready: true,
        scheduler_entry: "nuis.scheduler.loop.v1".to_owned(),
        process_lifecycle_hook: "on_process_start".to_owned(),
        loader_entry_kind: Some("lifecycle-bootstrap".to_owned()),
        loader_entry_abi_contract: Some(crate::NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1.to_owned()),
        loader_entry_machine_arch: Some(
            crate::native_host_machine_arch()
                .unwrap_or(crate::NUIS_MACHINE_ARCH_AARCH64)
                .to_owned(),
        ),
        loader_entry_symbol: Some("main".to_owned()),
        loader_entry_section_id: Some("sec0001.nuis-native-entry-code".to_owned()),
        loader_symbol_status: "parsed".to_owned(),
        loader_symbol_kind: Some("lifecycle-bootstrap".to_owned()),
        loader_symbol_name: Some("main".to_owned()),
        loader_symbol_lifecycle_hook: Some("on_lifecycle_bootstrap".to_owned()),
        loader_symbol_section_id: Some("sec0001.nuis-native-entry-code".to_owned()),
        loader_symbol_offset: Some(136),
        loader_symbol_size_bytes: Some(8),
        loader_symbol_payload_hash: Some(fnv1a64_hex(&[0; 8])),
        relocation_targets_loader_symbol: true,
        relocation_source_matches_loader_symbol: true,
        source_section_count: 2,
        source_section_table_hash: "0x3333333333333333".to_owned(),
        mapped_sections: vec![
            MappedSectionFacts {
                section_id: "sec0000.compiled-artifact".to_owned(),
                section_kind: "compiled-artifact".to_owned(),
                offset: 0,
                size_bytes: 128,
                payload_hash: "0x4444444444444444".to_owned(),
                required: true,
                mapping_status: "mapped".to_owned(),
            },
            MappedSectionFacts {
                section_id: "sec0001.nuis-native-entry-code".to_owned(),
                section_kind: crate::NUIS_NATIVE_ENTRY_SECTION_KIND.to_owned(),
                offset: 128,
                size_bytes: 16,
                payload_hash: "0x7777777777777777".to_owned(),
                required: true,
                mapping_status: "mapped".to_owned(),
            },
        ],
        source_relocation_count: 1,
        source_relocation_table_hash: "0x5555555555555555".to_owned(),
        applied_relocations: vec![AppliedRelocationFacts {
            relocation_id: "rel0000.lifecycle-entry".to_owned(),
            relocation_kind: "lifecycle-entry-binding".to_owned(),
            source_section_id: "sec0001.nuis-native-entry-code".to_owned(),
            source_offset: 128,
            target_symbol_id: "sym0000.loader-entry".to_owned(),
            addend: 0,
            application_status: "applied".to_owned(),
        }],
        runtime_service_bindings: vec![
            RuntimeServiceBindingFacts {
                binding_id: CLOCK_ROOT_BINDING_ID.to_owned(),
                contract: CLOCK_ROOT_CONTRACT.to_owned(),
                value_count: 3,
                value_hash: "0x1111111111111111".to_owned(),
                validation_status: "verified".to_owned(),
                required: true,
            },
            RuntimeServiceBindingFacts {
                binding_id: GLM_ROOT_BINDING_ID.to_owned(),
                contract: GLM_ROOT_CONTRACT.to_owned(),
                value_count: 2,
                value_hash: "0x2222222222222222".to_owned(),
                validation_status: "verified".to_owned(),
                required: true,
            },
        ],
        provider_dispatch_status: "verified-empty".to_owned(),
    }
}

fn owned_inputs(
    facts: &LifecycleBootstrapFacts,
) -> (
    OwnedImageMapping,
    Vec<OwnedMappedSectionHandle>,
    Vec<OwnedAppliedRelocationHandle>,
    Vec<OwnedRuntimeServiceHandle>,
) {
    let plan_hash = plan_lifecycle_bootstrap(facts).identity_hash;
    (
        OwnedImageMapping::new("0x9999999999999999", 256),
        facts
            .mapped_sections
            .iter()
            .map(|facts| OwnedMappedSectionHandle::from_facts(&plan_hash, facts))
            .collect(),
        facts
            .applied_relocations
            .iter()
            .map(|facts| OwnedAppliedRelocationHandle::from_facts(&plan_hash, facts))
            .collect(),
        facts
            .runtime_service_bindings
            .iter()
            .map(|facts| OwnedRuntimeServiceHandle::from_facts(&plan_hash, facts))
            .collect(),
    )
}

#[test]
fn ready_context_consumes_every_capability_before_entry_transfer() {
    let facts = ready_facts();
    let (image, sections, relocations, services) = owned_inputs(&facts);
    let preparation =
        prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services);
    assert!(preparation.ready);
    assert_eq!(preparation.status, "ready");
    assert!(valid_hash(&preparation.execution_identity_hash));

    let transfer = preparation.transfer();
    assert!(transfer.ready);
    assert_eq!(transfer.status, "transfer-ready");
    assert_eq!(transfer.consumed_mapped_section_count, 2);
    assert_eq!(transfer.consumed_applied_relocation_count, 1);
    assert_eq!(
        transfer.entry_section_kind.as_deref(),
        Some(crate::NUIS_NATIVE_ENTRY_SECTION_KIND)
    );
    assert_eq!(transfer.entry_section_offset, Some(128));
    assert_eq!(transfer.entry_section_size_bytes, Some(16));
    assert_eq!(
        transfer.entry_section_payload_hash.as_deref(),
        Some("0x7777777777777777")
    );
    assert_eq!(
        transfer.entry_abi_contract.as_deref(),
        Some(crate::NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1)
    );
    assert_eq!(
        transfer.entry_machine_arch.as_deref(),
        crate::native_host_machine_arch()
    );
    assert_eq!(transfer.entry_symbol_offset, Some(136));
    assert_eq!(transfer.entry_symbol_size_bytes, Some(8));
    assert_eq!(
        transfer.entry_symbol_payload_hash.as_deref(),
        Some(fnv1a64_hex(&[0; 8]).as_str())
    );
    assert_eq!(transfer.activated_service_ids.len(), 2);
    assert_eq!(
        transfer.trace,
        vec![
            "consume-mapped-section:sec0000.compiled-artifact",
            "consume-mapped-section:sec0001.nuis-native-entry-code",
            "consume-applied-relocation:rel0000.lifecycle-entry",
            "activate-runtime-service:runtime.clock-root",
            "activate-runtime-service:runtime.glm-root",
            "transfer-compiled-entry:main@sec0001.nuis-native-entry-code",
        ]
    );
    let request = ExecutableEntryRequest::from_transfer(&transfer, &[0; 8]).unwrap();
    let context = crate::NativeLifecycleEntryContextV1::from_transfer(&transfer).unwrap();
    let permit = NativeEntryInvocationPermit::from_transfer(&transfer, &context).unwrap();
    assert_eq!(
        permit.protocol(),
        crate::NATIVE_ENTRY_INVOCATION_PERMIT_PROTOCOL
    );
    let native = NativeHostExecutableMemoryAdapter.prepare(&request);
    assert!(native.ready, "{:?}", native.blockers);
    assert_eq!(native.protection_status, "sealed-read-execute");
    assert!(native.authorize(permit, context).is_ok());
}

#[test]
fn section_capability_drift_fails_closed_without_consuming_anything() {
    let facts = ready_facts();
    let (image, mut sections, relocations, services) = owned_inputs(&facts);
    sections[0].payload_hash = "0xaaaaaaaaaaaaaaaa".to_owned();

    let transfer =
        prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services)
            .transfer();
    assert!(!transfer.ready);
    assert_eq!(transfer.status, "blocked");
    assert_eq!(transfer.consumed_mapped_section_count, 0);
    assert_eq!(transfer.consumed_applied_relocation_count, 0);
    assert!(transfer.activated_service_ids.is_empty());
    assert!(transfer.trace.is_empty());
    assert!(transfer.blockers.iter().any(|blocker| {
        blocker == "runtime-bootstrap-execution:section-handle-invalid:sec0000.compiled-artifact"
    }));
}

#[test]
fn capabilities_cannot_cross_plan_identity_boundaries() {
    let facts = ready_facts();
    let (image, sections, mut relocations, services) = owned_inputs(&facts);
    relocations[0].plan_identity_hash = "0xaaaaaaaaaaaaaaaa".to_owned();

    let transfer =
        prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services)
            .transfer();
    assert!(!transfer.ready);
    assert!(transfer.blockers.iter().any(|blocker| {
        blocker == "runtime-bootstrap-execution:relocation-handle-invalid:rel0000.lifecycle-entry"
    }));
}

#[test]
fn undersized_mapping_fails_closed_before_section_consumption() {
    let facts = ready_facts();
    let (_, sections, relocations, services) = owned_inputs(&facts);
    let transfer = prepare_lifecycle_bootstrap_execution(
        &facts,
        OwnedImageMapping::new("0x9999999999999999", 64),
        sections,
        relocations,
        services,
    )
    .transfer();

    assert!(!transfer.ready);
    assert_eq!(transfer.image_hash, None);
    assert!(transfer.blockers.iter().any(|blocker| {
        blocker == "runtime-bootstrap-execution:section-handle-invalid:sec0000.compiled-artifact"
    }));
}

#[test]
fn blocked_plan_never_materializes_an_execution_context() {
    let mut facts = ready_facts();
    facts.image_verified = false;
    let (image, sections, relocations, services) = owned_inputs(&facts);
    let transfer =
        prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services)
            .transfer();

    assert!(!transfer.ready);
    assert_eq!(transfer.plan_identity_hash, "none");
    assert!(transfer
        .blockers
        .contains(&"runtime-bootstrap:image-unverified".to_owned()));
}

#[test]
fn dispatch_import_identity_crosses_execution_transfer_context_and_permit() {
    let mut facts = ready_facts();
    facts.loader_entry_abi_contract = Some(crate::NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2.to_owned());
    let import_facts = crate::RuntimeDispatchImportFacts {
        declarations: vec![crate::RuntimeDispatchImportDeclaration {
            import_kind: crate::NATIVE_RUNTIME_DISPATCH_IMPORT_KIND.to_owned(),
            import_name: crate::NATIVE_RUNTIME_DISPATCH_IMPORT_NAME.to_owned(),
            provider: crate::NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER.to_owned(),
            required: true,
        }],
    };
    let resolution = crate::resolve_runtime_dispatch_import(
        crate::NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2,
        &import_facts,
    );
    assert!(resolution.ready);
    let (image, sections, relocations, services) = owned_inputs(&facts);
    let transfer = prepare_lifecycle_bootstrap_execution_with_dispatch(
        &facts,
        image,
        sections,
        relocations,
        services,
        &resolution,
    )
    .transfer();

    assert!(transfer.ready, "{:?}", transfer.blockers);
    assert_eq!(
        transfer.execution_identity_contract,
        LIFECYCLE_BOOTSTRAP_DISPATCH_EXECUTION_IDENTITY_CONTRACT
    );
    let binding = transfer
        .runtime_dispatch_import
        .as_ref()
        .expect("dispatch-aware transfer owns resolved import");
    assert!(binding.table_identity.is_some());
    assert_eq!(
        binding.capability_mask,
        crate::NATIVE_RUNTIME_DISPATCH_KNOWN_CAPABILITIES
    );
    assert!(transfer.trace.iter().any(|event| {
        event
            == &format!(
                "bind-runtime-dispatch-import:{}",
                binding.import_identity_hash
            )
    }));

    let context = crate::NativeLifecycleEntryContextV1::from_transfer(&transfer).unwrap();
    assert_eq!(
        binding.table_identity,
        Some(context.dispatch_table_identity())
    );
    let permit = crate::NativeEntryInvocationPermit::from_transfer(&transfer, &context).unwrap();
    assert_eq!(
        permit.runtime_dispatch_import_identity_hash(),
        Some(binding.import_identity_hash.as_str())
    );

    let mut tampered = transfer.clone();
    tampered.runtime_dispatch_import.as_mut().unwrap().provider = "host-special-case".to_owned();
    assert_eq!(
        crate::NativeLifecycleEntryContextV1::from_transfer(&tampered).unwrap_err(),
        "runtime-dispatch-binding:contract-mismatch"
    );
}
