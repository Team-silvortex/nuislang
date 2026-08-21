use super::*;
use crate::{
    final_executable_elf_input::parse_elf64_amd64_object_linkage,
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_layout_report::ElfAmd64MergedSectionPlan,
    final_executable_elf_object::ElfAmd64ObjectLinkage,
    final_executable_elf_relocation::build_elf_amd64_relocation_application,
    final_executable_elf_test_fixture::{elf_program_object, elf_runtime_object, R_X86_64_PLT32},
};

#[test]
fn materializes_real_sections_and_previews_plt32_without_mutation() {
    let fixture = Fixture::new();
    let placement = build_elf_amd64_placement_binding(&fixture.objects).unwrap();
    let relocations = build_elf_amd64_relocation_application(&fixture.objects, &placement).unwrap();
    let inputs = fixture.image_objects();

    let image = build_elf_amd64_merged_image(&inputs, &placement).unwrap();
    let report =
        build_elf_amd64_materialization_preview(&inputs, &placement, &relocations).unwrap();

    assert_eq!(&image.bytes[0x1000..0x1006], &[0xe8, 0, 0, 0, 0, 0xc3]);
    assert!(image.bytes[0x1006..0x1010].iter().all(|byte| *byte == 0));
    assert_eq!(image.bytes[0x1010], 0xc3);
    assert_eq!(report.contract, ELF_AMD64_MATERIALIZATION_PREVIEW_CONTRACT);
    assert_eq!(report.status, "preview-ready");
    assert_eq!(report.placement_plan_hash, placement.plan_hash);
    assert_eq!(report.relocation_plan_hash, relocations.plan_hash);
    assert_eq!(report.file_span_bytes, 0x1011);
    assert_eq!(report.memory_span_bytes, 0x1011);
    assert_eq!(report.copied_bytes, 7);
    assert_eq!(report.zero_fill_bytes, 0);
    assert_eq!(report.input_object_count, 2);
    assert_eq!(report.section_audit_count, 2);
    assert_eq!(report.merged_section_audit_count, 1);
    assert_eq!(report.previewed_patch_count, 1);
    assert!(report
        .object_audits
        .iter()
        .all(|audit| audit.status == "verified-plan-bound"));
    assert_eq!(report.patches[0].source_image_offset, 0x1001);
    assert_eq!(report.patches[0].source_bytes, [0, 0, 0, 0]);
    assert_eq!(report.patches[0].encoded_bytes, [0x0b, 0, 0, 0]);
    assert_eq!(report.patches[0].status, "write-once-preview");
    assert_eq!(
        report.plan_hash,
        crate::fnv1a64_hex(report.canonical_plan().as_bytes())
    );
}

#[test]
fn unresolved_system_target_remains_deferred_after_materialization() {
    let fixture = Fixture::new();
    let objects = &fixture.objects[..1];
    let placement = build_elf_amd64_placement_binding(objects).unwrap();
    let relocations = build_elf_amd64_relocation_application(objects, &placement).unwrap();
    let inputs = fixture.program_image_object();

    let report =
        build_elf_amd64_materialization_preview(&inputs, &placement, &relocations).unwrap();

    assert_eq!(
        report.status,
        "preview-ready-with-platform-structure-boundary"
    );
    assert_eq!(report.copied_bytes, 6);
    assert_eq!(report.planned_direct_count, 0);
    assert_eq!(report.previewed_patch_count, 0);
    assert_eq!(report.deferred_patch_count, 1);
    assert!(report.patches.is_empty());
}

#[test]
fn materialization_is_independent_of_image_object_input_order() {
    let fixture = Fixture::new();
    let placement = build_elf_amd64_placement_binding(&fixture.objects).unwrap();
    let relocations = build_elf_amd64_relocation_application(&fixture.objects, &placement).unwrap();
    let forward =
        build_elf_amd64_materialization_preview(&fixture.image_objects(), &placement, &relocations)
            .unwrap();
    let mut reversed_inputs = fixture.image_objects();
    reversed_inputs.reverse();

    let reversed =
        build_elf_amd64_materialization_preview(&reversed_inputs, &placement, &relocations)
            .unwrap();

    assert_eq!(reversed, forward);
}

#[test]
fn source_hash_and_parsed_linkage_drift_fail_before_copy() {
    let mut hash_drift = Fixture::new();
    let placement = build_elf_amd64_placement_binding(&hash_drift.objects).unwrap();
    let relocations =
        build_elf_amd64_relocation_application(&hash_drift.objects, &placement).unwrap();
    hash_drift.program_bytes[64] ^= 0x01;

    let error = build_elf_amd64_materialization_preview(
        &hash_drift.image_objects(),
        &placement,
        &relocations,
    )
    .unwrap_err();

    assert!(error.contains("source hash drift"), "{error}");

    let mut linkage_drift = Fixture::new();
    let placement = build_elf_amd64_placement_binding(&linkage_drift.objects).unwrap();
    let relocations =
        build_elf_amd64_relocation_application(&linkage_drift.objects, &placement).unwrap();
    linkage_drift.objects[0].linkage.sections[0]
        .name
        .push_str(".drift");

    let error = build_elf_amd64_materialization_preview(
        &linkage_drift.image_objects(),
        &placement,
        &relocations,
    )
    .unwrap_err();

    assert!(error.contains("source/linkage drift"), "{error}");
}

#[test]
fn plan_hash_drift_and_overlapping_direct_patches_fail_closed() {
    let fixture = Fixture::new();
    let placement = build_elf_amd64_placement_binding(&fixture.objects).unwrap();
    let relocations = build_elf_amd64_relocation_application(&fixture.objects, &placement).unwrap();
    let inputs = fixture.image_objects();

    let mut bad_placement = placement.clone();
    bad_placement.plan_hash.push('0');
    let error =
        build_elf_amd64_materialization_preview(&inputs, &bad_placement, &relocations).unwrap_err();
    assert!(error.contains("placement hash mismatch"), "{error}");

    let mut bad_relocations = relocations.clone();
    bad_relocations.plan_hash.push('0');
    let error =
        build_elf_amd64_materialization_preview(&inputs, &placement, &bad_relocations).unwrap_err();
    assert!(error.contains("relocation hash mismatch"), "{error}");

    let mut bad_encoding = relocations.clone();
    bad_encoding.applications[0].encoded_bytes[0] ^= 1;
    bad_encoding.plan_hash = crate::fnv1a64_hex(bad_encoding.canonical_plan().as_bytes());
    let error =
        build_elf_amd64_materialization_preview(&inputs, &placement, &bad_encoding).unwrap_err();
    assert!(error.contains("encoded byte drift"), "{error}");

    let mut unknown_status = relocations.clone();
    unknown_status.applications[0].application_status = "unknown".to_owned();
    unknown_status.direct_preview_count = 0;
    unknown_status.plan_hash = crate::fnv1a64_hex(unknown_status.canonical_plan().as_bytes());
    let error =
        build_elf_amd64_materialization_preview(&inputs, &placement, &unknown_status).unwrap_err();
    assert!(error.contains("unclassified relocation status"), "{error}");

    let mut overlapping = relocations.clone();
    let mut duplicate = overlapping.applications[0].clone();
    duplicate.relocation_id = "elf-amd64-reloc-overlap".to_owned();
    overlapping.applications.push(duplicate);
    overlapping.relocation_count += 1;
    overlapping.direct_preview_count += 1;
    overlapping.plan_hash = crate::fnv1a64_hex(overlapping.canonical_plan().as_bytes());
    let error =
        build_elf_amd64_materialization_preview(&inputs, &placement, &overlapping).unwrap_err();
    assert!(error.contains("overlaps an earlier span"), "{error}");
}

#[test]
fn merged_zero_fill_sections_are_audited_separately() {
    let fixture = Fixture::new();
    let mut placement = build_elf_amd64_placement_binding(&fixture.objects).unwrap();
    placement.memory_span_bytes = 0x2008;
    placement.merged_sections.push(ElfAmd64MergedSectionPlan {
        section_id: "elf-section-zero-test".to_owned(),
        output_section_name: ".bss".to_owned(),
        class: "bss".to_owned(),
        alignment: 0x1000,
        file_offset: None,
        image_offset: 0x2000,
        virtual_address: 0x402000,
        size_bytes: 8,
        contribution_count: 0,
        zero_fill: true,
    });
    let mut bytes = vec![0u8; placement.memory_span_bytes];

    let (audits, zero_fill_bytes) = audit_merged_sections(&bytes, &placement).unwrap();

    assert_eq!(zero_fill_bytes, 8);
    assert_eq!(audits.last().unwrap().status, "verified-zero-fill");
    bytes[0x2003] = 1;
    let error = audit_merged_sections(&bytes, &placement).unwrap_err();
    assert!(error.contains("contains materialized nonzero bytes"));
}

struct Fixture {
    program_bytes: Vec<u8>,
    runtime_bytes: Vec<u8>,
    program_hash: String,
    runtime_hash: String,
    objects: Vec<ElfAmd64ObjectLinkage>,
}

impl Fixture {
    fn new() -> Self {
        let program_bytes = elf_program_object(R_X86_64_PLT32);
        let runtime_bytes = elf_runtime_object();
        let program_hash = crate::fnv1a64_hex(&program_bytes);
        let runtime_hash = crate::fnv1a64_hex(&runtime_bytes);
        let objects = vec![
            ElfAmd64ObjectLinkage {
                object_id: "host.program-llvm".to_owned(),
                role: "program-llvm".to_owned(),
                linkage: parse_elf64_amd64_object_linkage(&program_bytes).unwrap(),
            },
            ElfAmd64ObjectLinkage {
                object_id: "host.runtime-shim".to_owned(),
                role: "runtime-shim".to_owned(),
                linkage: parse_elf64_amd64_object_linkage(&runtime_bytes).unwrap(),
            },
        ];
        Self {
            program_bytes,
            runtime_bytes,
            program_hash,
            runtime_hash,
            objects,
        }
    }

    fn image_objects(&self) -> Vec<ElfAmd64ImageObject<'_>> {
        vec![self.program_input(), self.runtime_input()]
    }

    fn program_image_object(&self) -> Vec<ElfAmd64ImageObject<'_>> {
        vec![self.program_input()]
    }

    fn program_input(&self) -> ElfAmd64ImageObject<'_> {
        ElfAmd64ImageObject {
            object_id: &self.objects[0].object_id,
            role: &self.objects[0].role,
            bytes: &self.program_bytes,
            planned_size_bytes: self.program_bytes.len(),
            planned_source_hash: &self.program_hash,
            linkage: &self.objects[0].linkage,
        }
    }

    fn runtime_input(&self) -> ElfAmd64ImageObject<'_> {
        ElfAmd64ImageObject {
            object_id: &self.objects[1].object_id,
            role: &self.objects[1].role,
            bytes: &self.runtime_bytes,
            planned_size_bytes: self.runtime_bytes.len(),
            planned_source_hash: &self.runtime_hash,
            linkage: &self.objects[1].linkage,
        }
    }
}
