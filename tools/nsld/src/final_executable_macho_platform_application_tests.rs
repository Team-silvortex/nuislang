use super::*;
use crate::reports::{
    NsldMachOArm64PatchApplicationReport, NsldMachOArm64RelocationApplication,
    NsldMachOArm64RelocationApplicationReport, NsldMachOPlacementBindingReport,
};

#[test]
fn applies_stubs_got_entries_deferred_patches_and_bind_records_deterministically() {
    let (placement, relocations, applied) = platform_fixture();
    let plan = build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
        .unwrap();

    let first =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &plan).unwrap();
    let second =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &plan).unwrap();

    assert_eq!(first.bytes, second.bytes);
    assert_eq!(first.report, second.report);
    assert_eq!(first.bytes.len(), 48);
    assert_eq!(&first.bytes[0..4], &0x9400_0004u32.to_le_bytes());
    assert_eq!(&first.bytes[4..8], &0x9000_0000u32.to_le_bytes());
    assert_eq!(&first.bytes[8..12], &0xf940_1400u32.to_le_bytes());
    assert_eq!(
        &first.bytes[16..28],
        &[0x10, 0x00, 0x00, 0x90, 0x10, 0x12, 0x40, 0xf9, 0x00, 0x02, 0x1f, 0xd6,]
    );
    assert_eq!(&first.bytes[28..40], &[0; 12]);
    assert_eq!(&first.bytes[40..48], &12u64.to_le_bytes());

    let report = &first.report;
    assert_eq!(
        report.status,
        "platform-patches-applied-with-unresolved-binds"
    );
    assert_eq!(report.base_image_span_bytes, 16);
    assert_eq!(report.platform_image_span_bytes, 48);
    assert_eq!(report.expected_deferred_patch_count, 3);
    assert_eq!(report.applied_deferred_patch_count, 3);
    assert_eq!(report.stub_write_count, 1);
    assert_eq!(report.got_write_count, 2);
    assert_eq!(report.unresolved_bind_count, 1);
    assert_eq!(report.write_once_span_count, 6);
    assert_eq!(report.structure_writes.len(), 3);
    assert_eq!(report.patches.len(), 3);
    assert_eq!(report.bind_records.len(), 1);
    assert_eq!(report.bind_records[0].target_symbol, "_puts");
    assert_eq!(report.bind_records[0].got_output_offset, 32);
    assert_eq!(report.bind_records[0].status, "unresolved-external");
    assert!(report
        .structure_writes
        .iter()
        .any(|write| write.write_kind == "arm64-branch-stub"
            && write.encoded_bytes_hex == "10000090101240f900021fd6"));
    assert!(report.structure_writes.iter().any(|write| write.write_kind
        == "internal-image-relative-got"
        && write.output_offset == 40));
    assert_eq!(crate::fnv1a64_hex(&first.bytes), report.platform_image_hash);
}

#[test]
fn plan_and_base_image_drift_fail_closed() {
    let (placement, relocations, applied) = platform_fixture();
    let mut plan =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
            .unwrap();
    plan.plan_hash = "0xdeadbeefdeadbeef".to_owned();
    let plan_error =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &plan)
            .unwrap_err();
    assert!(plan_error.contains("plan drift"));

    let valid_plan =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
            .unwrap();
    let mut drifted = applied;
    drifted.bytes[0] ^= 0xff;
    let image_error =
        apply_macho_arm64_platform_structure(&placement, &relocations, &drifted, &valid_plan)
            .unwrap_err();
    assert!(image_error.contains("base image drift"));
}

#[test]
fn overlapping_deferred_relocations_are_rejected_by_write_once_policy() {
    let placement = placement(8);
    let relocations = relocation_report(vec![
        application(
            "reloc-first",
            "arm64-branch26",
            "rewrite-branch26",
            0,
            "external-compatibility",
            "_first",
            None,
        ),
        application(
            "reloc-second",
            "arm64-branch26",
            "rewrite-branch26",
            0,
            "external-compatibility",
            "_second",
            None,
        ),
    ]);
    let applied = applied_image(
        vec![0x00, 0x00, 0x00, 0x94, 0, 0, 0, 0],
        &placement,
        &relocations,
    );
    let plan = build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
        .unwrap();

    let error = apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &plan)
        .unwrap_err();
    assert!(error.contains("overlaps a previously committed span"));
}

#[test]
fn invalid_platform_instruction_fails_before_publication() {
    let placement = placement(4);
    let relocations = relocation_report(vec![application(
        "reloc-branch",
        "arm64-branch26",
        "rewrite-branch26",
        0,
        "external-compatibility",
        "_puts",
        None,
    )]);
    let applied = applied_image(vec![0, 0, 0, 0], &placement, &relocations);
    let plan = build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
        .unwrap();

    let error = apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &plan)
        .unwrap_err();
    assert!(error.contains("is not B/BL"));
}

#[test]
fn empty_platform_plan_preserves_the_direct_applied_image() {
    let placement = placement(4);
    let relocations = relocation_report(Vec::new());
    let applied = applied_image(vec![1, 2, 3, 4], &placement, &relocations);
    let plan = build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
        .unwrap();

    let output =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &plan).unwrap();

    assert_eq!(output.bytes, applied.bytes);
    assert_eq!(output.report.status, "not-required");
    assert_eq!(output.report.platform_image_span_bytes, 4);
    assert_eq!(output.report.write_once_span_count, 0);
    assert!(output.report.structure_writes.is_empty());
    assert!(output.report.patches.is_empty());
    assert!(output.report.bind_records.is_empty());
}

fn platform_fixture() -> (
    NsldMachOPlacementBindingReport,
    NsldMachOArm64RelocationApplicationReport,
    MachOArm64AppliedImage,
) {
    let placement = placement(16);
    let relocations = relocation_report(vec![
        application(
            "reloc-branch",
            "arm64-branch26",
            "rewrite-branch26",
            0,
            "external-compatibility",
            "_puts",
            None,
        ),
        application(
            "reloc-got-page",
            "arm64-got-load-page21",
            "rewrite-got-load-page21",
            4,
            "internal-symbol",
            "_shared",
            Some(12),
        ),
        application(
            "reloc-got-offset",
            "arm64-got-load-pageoff12",
            "rewrite-got-load-pageoff12",
            8,
            "internal-symbol",
            "_shared",
            Some(12),
        ),
    ]);
    let bytes = [
        0x00, 0x00, 0x00, 0x94, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x40, 0xf9, 0, 0, 0, 0,
    ]
    .to_vec();
    let applied = applied_image(bytes, &placement, &relocations);
    (placement, relocations, applied)
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

fn applied_image(
    bytes: Vec<u8>,
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
) -> MachOArm64AppliedImage {
    let image_hash = crate::fnv1a64_hex(&bytes);
    MachOArm64AppliedImage {
        bytes,
        report: NsldMachOArm64PatchApplicationReport {
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
            original_image_hash: image_hash.clone(),
            applied_image_hash: image_hash,
            image_span_bytes: placement.image_span_bytes,
            expected_patch_count: 0,
            applied_patch_count: 0,
            deferred_patch_count: relocations.platform_structure_count,
            write_once_span_count: 0,
            application_ledger_hash: "0x6000000000000006".to_owned(),
            patches: Vec::new(),
        },
    }
}

fn application(
    relocation_id: &str,
    relocation_kind: &str,
    action_kind: &str,
    source_output_offset: usize,
    resolver_status: &str,
    target_symbol: &str,
    target_output_offset: Option<usize>,
) -> NsldMachOArm64RelocationApplication {
    let internal = resolver_status != "external-compatibility";
    let relocation_type = match relocation_kind {
        "arm64-branch26" => 2,
        "arm64-got-load-page21" => 5,
        "arm64-got-load-pageoff12" => 6,
        _ => 0,
    };
    NsldMachOArm64RelocationApplication {
        relocation_id: relocation_id.to_owned(),
        object_id: "host.program".to_owned(),
        object_role: "program-llvm".to_owned(),
        input_section_ordinal: 1,
        source_section_id: "macho-section-0000".to_owned(),
        source_offset: source_output_offset,
        source_output_offset,
        width_bytes: 4,
        pc_relative: relocation_kind.ends_with("page21") || relocation_kind == "arm64-branch26",
        external: true,
        relocation_type,
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
