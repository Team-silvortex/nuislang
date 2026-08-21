use super::*;
use crate::reports::{
    NsldMachOArm64PatchApplicationReport, NsldMachOArm64RelocationApplication,
    NsldMachOArm64RelocationApplicationReport, NsldMachOPlacementBindingReport,
};

#[test]
fn got_pair_shares_one_deterministic_target_slot() {
    let placement = placement(32);
    let relocations = relocation_report(vec![
        application(
            "reloc-page21",
            "arm64-got-load-page21",
            "rewrite-got-load-page21",
            "internal-symbol",
            "_shared",
            Some(8),
        ),
        application(
            "reloc-pageoff12",
            "arm64-got-load-pageoff12",
            "rewrite-got-load-pageoff12",
            "internal-symbol",
            "_shared",
            Some(8),
        ),
    ]);
    let applied = applied_report(&placement, &relocations);

    let report =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied).unwrap();
    let repeated =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied).unwrap();

    assert_eq!(report, repeated);
    assert_eq!(report.status, "allocated-ready-for-platform-patching");
    assert_eq!(report.deferred_relocation_count, 2);
    assert_eq!(report.target_count, 1);
    assert_eq!(report.stub_entry_count, 0);
    assert_eq!(report.got_entry_count, 1);
    assert_eq!(report.got_region_offset, 32);
    assert_eq!(report.planned_image_span_bytes, 40);
    assert_eq!(report.targets[0].target_symbol, "_shared");
    assert_eq!(report.targets[0].got_slot_index, Some(0));
    assert_eq!(report.targets[0].got_output_offset, Some(32));
    assert_eq!(report.targets[0].relocation_ids.len(), 2);
    assert!(report
        .relocation_bindings
        .iter()
        .all(|binding| binding.patch_target_kind == "got-entry"
            && binding.patch_target_output_offset == 32));
}

#[test]
fn external_branch_targets_are_sorted_but_bindings_keep_relocation_order() {
    let placement = placement(16);
    let relocations = relocation_report(vec![
        application(
            "reloc-zeta",
            "arm64-branch26",
            "rewrite-branch26",
            "external-compatibility",
            "_zeta",
            None,
        ),
        application(
            "reloc-alpha",
            "arm64-branch26",
            "rewrite-branch26",
            "external-compatibility",
            "_alpha",
            None,
        ),
    ]);
    let applied = applied_report(&placement, &relocations);

    let report =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied).unwrap();

    assert_eq!(report.target_count, 2);
    assert_eq!(report.stub_region_offset, 16);
    assert_eq!(report.stub_region_bytes, 24);
    assert_eq!(report.got_region_offset, 40);
    assert_eq!(report.got_region_bytes, 16);
    assert_eq!(report.planned_image_span_bytes, 56);
    assert_eq!(report.targets[0].target_symbol, "_alpha");
    assert_eq!(report.targets[0].stub_output_offset, Some(16));
    assert_eq!(report.targets[0].got_output_offset, Some(40));
    assert_eq!(report.targets[1].target_symbol, "_zeta");
    assert_eq!(report.targets[1].stub_output_offset, Some(28));
    assert_eq!(report.targets[1].got_output_offset, Some(48));
    assert_eq!(report.relocation_bindings[0].relocation_id, "reloc-zeta");
    assert_eq!(report.relocation_bindings[0].patch_target_output_offset, 28);
    assert_eq!(report.relocation_bindings[1].relocation_id, "reloc-alpha");
    assert_eq!(report.relocation_bindings[1].patch_target_output_offset, 16);
}

#[test]
fn empty_platform_boundary_does_not_extend_the_applied_image() {
    let placement = placement(24);
    let relocations = relocation_report(Vec::new());
    let applied = applied_report(&placement, &relocations);

    let report =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied).unwrap();

    assert_eq!(report.status, "not-required");
    assert_eq!(report.base_image_span_bytes, 24);
    assert_eq!(report.planned_image_span_bytes, 24);
    assert_eq!(report.target_count, 0);
    assert!(report.targets.is_empty());
    assert!(report.relocation_bindings.is_empty());
}

#[test]
fn unsupported_deferred_shape_and_application_drift_fail_closed() {
    let placement = placement(16);
    let relocations = relocation_report(vec![application(
        "reloc-unsupported",
        "arm64-unsigned",
        "write-absolute",
        "external-compatibility",
        "_external_data",
        None,
    )]);
    let applied = applied_report(&placement, &relocations);

    let unsupported =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied).unwrap_err();
    assert!(unsupported.contains("has no registered platform structure rule"));

    let supported = relocation_report(vec![application(
        "reloc-branch",
        "arm64-branch26",
        "rewrite-branch26",
        "external-compatibility",
        "_external_function",
        None,
    )]);
    let mut drifted = applied_report(&placement, &supported);
    drifted.application_ledger_hash.clear();
    let drift =
        build_macho_arm64_platform_structure_plan(&placement, &supported, &drifted).unwrap_err();
    assert!(drift.contains("patch application envelope drift"));
}

#[test]
fn plan_hash_binds_the_patch_application_ledger() {
    let placement = placement(16);
    let relocations = relocation_report(vec![application(
        "reloc-branch",
        "arm64-branch26",
        "rewrite-branch26",
        "external-compatibility",
        "_external_function",
        None,
    )]);
    let first_applied = applied_report(&placement, &relocations);
    let mut second_applied = first_applied.clone();
    second_applied.application_ledger_hash = "0x1111111111111111".to_owned();

    let first = build_macho_arm64_platform_structure_plan(&placement, &relocations, &first_applied)
        .unwrap();
    let second =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &second_applied)
            .unwrap();
    assert_ne!(first.plan_hash, second.plan_hash);
}

#[test]
fn unreachable_external_branch_slot_fails_during_planning() {
    let placement = placement(0x0800_0004);
    let relocations = relocation_report(vec![application(
        "reloc-branch",
        "arm64-branch26",
        "rewrite-branch26",
        "external-compatibility",
        "_far_function",
        None,
    )]);
    let applied = applied_report(&placement, &relocations);

    let error =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied).unwrap_err();
    assert!(error.contains("displacement"));
    assert!(error.contains("out of range"));
}

fn placement(image_span_bytes: usize) -> NsldMachOPlacementBindingReport {
    NsldMachOPlacementBindingReport {
        contract: MACHO_PLACEMENT_BINDING_CONTRACT.to_owned(),
        status: "placement-ready-with-external-compatibility-boundary".to_owned(),
        plan_hash: "0x1000000000000001".to_owned(),
        image_span_bytes,
        merged_sections: Vec::new(),
        section_placements: Vec::new(),
        common_allocations: Vec::new(),
        symbol_bindings: Vec::new(),
        internally_bound_symbol_count: 0,
        external_compatibility_symbol_count: 0,
    }
}

fn relocation_report(
    applications: Vec<NsldMachOArm64RelocationApplication>,
) -> NsldMachOArm64RelocationApplicationReport {
    let platform_structure_count = applications.len();
    let external_compatibility_count = applications
        .iter()
        .filter(|item| item.resolver_status == "external-compatibility")
        .count();
    NsldMachOArm64RelocationApplicationReport {
        contract: MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT.to_owned(),
        status: if applications.is_empty() {
            "ready-for-byte-encoding"
        } else {
            "planned-with-platform-structure-boundary"
        }
        .to_owned(),
        plan_hash: "0x2000000000000002".to_owned(),
        placement_plan_hash: "0x1000000000000001".to_owned(),
        relocation_count: applications.len(),
        registered_kind_count: applications.len(),
        ready_application_count: 0,
        platform_structure_count,
        external_compatibility_count,
        metadata_record_count: 0,
        applications,
    }
}

fn applied_report(
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
) -> NsldMachOArm64PatchApplicationReport {
    NsldMachOArm64PatchApplicationReport {
        contract: MACHO_ARM64_PATCH_APPLICATION_CONTRACT.to_owned(),
        status: if relocations.platform_structure_count == 0 {
            "direct-patches-applied"
        } else {
            "direct-patches-applied-with-platform-structure-boundary"
        }
        .to_owned(),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        patch_plan_hash: "0x3000000000000003".to_owned(),
        original_image_hash: "0x4000000000000004".to_owned(),
        applied_image_hash: "0x5000000000000005".to_owned(),
        image_span_bytes: placement.image_span_bytes,
        expected_patch_count: 0,
        applied_patch_count: 0,
        deferred_patch_count: relocations.platform_structure_count,
        write_once_span_count: 0,
        application_ledger_hash: "0x6000000000000006".to_owned(),
        patches: Vec::new(),
    }
}

fn application(
    relocation_id: &str,
    relocation_kind: &str,
    action_kind: &str,
    resolver_status: &str,
    target_symbol: &str,
    target_output_offset: Option<usize>,
) -> NsldMachOArm64RelocationApplication {
    let internal = resolver_status != "external-compatibility";
    NsldMachOArm64RelocationApplication {
        relocation_id: relocation_id.to_owned(),
        object_id: "host.program".to_owned(),
        object_role: "program-llvm".to_owned(),
        input_section_ordinal: 1,
        source_section_id: "macho-section-0000".to_owned(),
        source_offset: 0,
        source_output_offset: 0,
        width_bytes: 4,
        pc_relative: relocation_kind.ends_with("page21") || relocation_kind == "arm64-branch26",
        external: true,
        relocation_type: 0,
        relocation_kind: relocation_kind.to_owned(),
        action_kind: action_kind.to_owned(),
        target_symbol: Some(target_symbol.to_owned()),
        target_symbol_index: Some(0),
        target_object_id: internal.then(|| "host.runtime".to_owned()),
        target_section_id: internal.then(|| "macho-section-0000".to_owned()),
        target_output_offset,
        explicit_addend: None,
        pair_relocation_id: None,
        resolver_status: resolver_status.to_owned(),
        application_status: "planned-platform-structure".to_owned(),
    }
}
