use super::*;
use crate::{
    final_executable_elf_input::parse_elf64_amd64_object_linkage,
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_materialization::{
        application::{apply_elf_amd64_patch_previews, ElfAmd64PatchApplicationReport},
        build_elf_amd64_materialization_preview, ElfAmd64ImageObject,
    },
    final_executable_elf_object::ElfAmd64ObjectLinkage,
    final_executable_elf_relocation::build_elf_amd64_relocation_application,
    final_executable_elf_test_fixture::{
        elf_program_object, elf_program_object_two_plt32_calls, elf_runtime_object, R_X86_64_PLT32,
    },
};

#[test]
fn plans_real_external_plt_got_and_rela_without_mutating_the_applied_ledger() {
    let fixture = Fixture::new(elf_program_object(R_X86_64_PLT32));
    let (placement, relocations, applied) = fixture.external_reports();

    let report =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap();
    let repeated =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap();

    assert_eq!(report, repeated);
    assert_eq!(report.contract, ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT);
    assert_eq!(report.status, "allocated-ready-for-platform-patching");
    assert_eq!(report.base_memory_span_bytes, 0x1006);
    assert_eq!(report.plt_region_image_offset, 0x2000);
    assert_eq!(report.plt_region_bytes, 16);
    assert_eq!(report.got_region_image_offset, 0x3000);
    assert_eq!(report.got_region_bytes, 8);
    assert_eq!(report.metadata_region_image_offset, 0x4000);
    assert_eq!(report.dynamic_symbol_entry_count, 2);
    assert_eq!(report.dynamic_string_region_bytes, 20);
    assert_eq!(report.dynamic_relocation_region_image_offset, 0x4048);
    assert_eq!(report.dynamic_relocation_region_bytes, 24);
    assert_eq!(report.planned_memory_span_bytes, 0x4060);
    assert_eq!(report.target_count, 1);
    assert_eq!(report.deferred_relocation_count, 1);
    assert_eq!(report.targets[0].target_symbol, "nuis_runtime_entry");
    assert_eq!(report.targets[0].dynamic_symbol_index, 1);
    assert_eq!(report.targets[0].dynamic_string_offset, 1);
    assert_eq!(report.targets[0].plt_virtual_address, Some(0x402000));
    assert_eq!(report.targets[0].got_virtual_address, Some(0x403000));
    assert_eq!(report.targets[0].plt_got_displacement, Some(0xffa));
    assert_eq!(
        report.targets[0].dynamic_relocation_kind.as_deref(),
        Some("r-x86-64-jump-slot")
    );
    assert_eq!(report.targets[0].dynamic_relocation_type, Some(7));
    assert_eq!(report.targets[0].dynamic_relocation_offset, Some(0x403000));
    assert_eq!(
        report.targets[0].dynamic_relocation_info,
        Some((1u64 << 32) | 7)
    );
    assert_eq!(report.relocation_bindings[0].computed_value, 0xffb);
    assert_eq!(
        report.relocation_bindings[0].encoded_bytes,
        [0xfb, 0x0f, 0, 0]
    );
    assert_eq!(
        report.patch_application_ledger_hash,
        applied.application_ledger_hash
    );
    assert_eq!(
        report.plan_hash,
        crate::fnv1a64_hex(report.canonical_plan().as_bytes())
    );
}

#[test]
fn repeated_external_calls_share_one_target_but_keep_distinct_bindings() {
    let fixture = Fixture::new(elf_program_object_two_plt32_calls());
    let (placement, relocations, applied) = fixture.external_reports();

    let report =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap();

    assert_eq!(report.deferred_relocation_count, 2);
    assert_eq!(report.target_count, 1);
    assert_eq!(report.plt_entry_count, 1);
    assert_eq!(report.got_entry_count, 1);
    assert_eq!(report.dynamic_relocation_entry_count, 1);
    assert_eq!(report.targets[0].relocation_ids.len(), 2);
    assert_eq!(report.relocation_bindings.len(), 2);
    assert!(report
        .relocation_bindings
        .iter()
        .all(|binding| binding.structure_id == report.targets[0].structure_id));
    assert_eq!(
        report.relocation_bindings[0].encoded_bytes,
        [0xfb, 0x0f, 0, 0]
    );
    assert_eq!(
        report.relocation_bindings[1].encoded_bytes,
        [0xf6, 0x0f, 0, 0]
    );
}

#[test]
fn internally_closed_image_does_not_gain_platform_regions() {
    let fixture = Fixture::new(elf_program_object(R_X86_64_PLT32));
    let (placement, relocations, applied) = fixture.closed_reports();

    let report =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap();

    assert_eq!(report.status, "not-required");
    assert_eq!(report.base_file_span_bytes, report.planned_file_span_bytes);
    assert_eq!(
        report.base_memory_span_bytes,
        report.planned_memory_span_bytes
    );
    assert_eq!(report.target_count, 0);
    assert_eq!(report.plt_entry_count, 0);
    assert_eq!(report.got_entry_count, 0);
    assert_eq!(report.dynamic_symbol_entry_count, 0);
    assert_eq!(report.dynamic_relocation_entry_count, 0);
    assert!(report.targets.is_empty());
    assert!(report.relocation_bindings.is_empty());
}

#[test]
fn unregistered_external_shape_and_ledger_drift_fail_closed() {
    let fixture = Fixture::new(elf_program_object(R_X86_64_PLT32));
    let (placement, mut relocations, mut applied) = fixture.external_reports();
    relocations.applications[0].relocation_kind = "x86_64-pc32".to_owned();
    relocations.applications[0].action_kind = "write-pc-relative-32".to_owned();
    relocations.plan_hash = crate::fnv1a64_hex(relocations.canonical_plan().as_bytes());
    applied.relocation_plan_hash = relocations.plan_hash.clone();
    applied.application_ledger_hash = crate::fnv1a64_hex(applied.canonical_ledger().as_bytes());

    let unsupported =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap_err();
    assert!(
        unsupported.contains("has no registered platform structure rule"),
        "{unsupported}"
    );

    let (placement, relocations, mut applied) = fixture.external_reports();
    applied.application_ledger_hash.push('0');
    let drift =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap_err();
    assert!(drift.contains("patch application ledger drift"), "{drift}");

    let (placement, relocations, mut applied) = fixture.closed_reports();
    applied.patches[0].encoded_bytes_hash = crate::fnv1a64_hex(&[0, 0, 0, 0]);
    applied.patches[0].post_write_bytes_hash = applied.patches[0].encoded_bytes_hash.clone();
    applied.application_ledger_hash = crate::fnv1a64_hex(applied.canonical_ledger().as_bytes());
    let audit_drift =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap_err();
    assert!(audit_drift.contains("patch audit"), "{audit_drift}");
}

#[test]
fn overlapping_deferred_sources_are_rejected_before_slot_assignment() {
    let fixture = Fixture::new(elf_program_object_two_plt32_calls());
    let (placement, mut relocations, mut applied) = fixture.external_reports();
    let first = relocations.applications[0].clone();
    let second = &mut relocations.applications[1];
    second.source_offset = first.source_offset;
    second.source_file_offset = first.source_file_offset;
    second.source_image_offset = first.source_image_offset;
    second.source_virtual_address = first.source_virtual_address;
    relocations.plan_hash = crate::fnv1a64_hex(relocations.canonical_plan().as_bytes());
    applied.relocation_plan_hash = relocations.plan_hash.clone();
    applied.application_ledger_hash = crate::fnv1a64_hex(applied.canonical_ledger().as_bytes());

    let error =
        build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied).unwrap_err();

    assert!(error.contains("overlaps"), "{error}");
}

struct Fixture {
    program_bytes: Vec<u8>,
    runtime_bytes: Vec<u8>,
    program_hash: String,
    runtime_hash: String,
    objects: Vec<ElfAmd64ObjectLinkage>,
}

impl Fixture {
    fn new(program_bytes: Vec<u8>) -> Self {
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

    fn external_reports(
        &self,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64PatchApplicationReport,
    ) {
        self.reports(false)
    }

    fn closed_reports(
        &self,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64PatchApplicationReport,
    ) {
        self.reports(true)
    }

    fn reports(
        &self,
        include_runtime: bool,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64PatchApplicationReport,
    ) {
        let objects = if include_runtime {
            self.objects.as_slice()
        } else {
            &self.objects[..1]
        };
        let placement = build_elf_amd64_placement_binding(objects).unwrap();
        let relocations = build_elf_amd64_relocation_application(objects, &placement).unwrap();
        let inputs = self.image_objects(include_runtime);
        let preview =
            build_elf_amd64_materialization_preview(&inputs, &placement, &relocations).unwrap();
        let applied =
            apply_elf_amd64_patch_previews(&inputs, &placement, &relocations, &preview).unwrap();
        (placement, relocations, applied.report)
    }

    fn image_objects(&self, include_runtime: bool) -> Vec<ElfAmd64ImageObject<'_>> {
        let mut inputs = vec![ElfAmd64ImageObject {
            object_id: &self.objects[0].object_id,
            role: &self.objects[0].role,
            bytes: &self.program_bytes,
            planned_size_bytes: self.program_bytes.len(),
            planned_source_hash: &self.program_hash,
            linkage: &self.objects[0].linkage,
        }];
        if include_runtime {
            inputs.push(ElfAmd64ImageObject {
                object_id: &self.objects[1].object_id,
                role: &self.objects[1].role,
                bytes: &self.runtime_bytes,
                planned_size_bytes: self.runtime_bytes.len(),
                planned_source_hash: &self.runtime_hash,
                linkage: &self.objects[1].linkage,
            });
        }
        inputs
    }
}
