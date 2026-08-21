use super::*;
use crate::{
    final_executable_elf_input::parse_elf64_amd64_object_linkage,
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_materialization::{
        application::apply_elf_amd64_patch_previews, build_elf_amd64_materialization_preview,
        ElfAmd64ImageObject,
    },
    final_executable_elf_object::ElfAmd64ObjectLinkage,
    final_executable_elf_relocation::build_elf_amd64_relocation_application,
    final_executable_elf_test_fixture::{
        elf_program_object, elf_program_object_two_plt32_calls, elf_runtime_object, R_X86_64_PLT32,
    },
};

#[test]
fn emits_real_plt_got_dynamic_records_and_deferred_patch_bytes() {
    let fixture = Fixture::new(elf_program_object(R_X86_64_PLT32));
    let (placement, relocations, applied, plan) = fixture.external_pipeline();

    let output =
        apply_elf_amd64_platform_structure_plan(&placement, &relocations, &applied, &plan).unwrap();
    let repeated =
        apply_elf_amd64_platform_structure_plan(&placement, &relocations, &applied, &plan).unwrap();

    assert_eq!(output.bytes, repeated.bytes);
    assert_eq!(output.report, repeated.report);
    assert_eq!(output.bytes.len(), 0x4060);
    assert_eq!(
        &output.bytes[0x1000..0x1006],
        &[0xe8, 0xfb, 0x0f, 0, 0, 0xc3]
    );
    assert_eq!(
        &output.bytes[0x2000..0x2010],
        &[
            0xff, 0x25, 0xfa, 0x0f, 0, 0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
            0x90
        ]
    );
    assert_eq!(&output.bytes[0x3000..0x3008], &[0; 8]);
    assert_eq!(&output.bytes[0x4000..0x4018], &[0; 24]);
    let mut expected_symbol = [0; 24];
    expected_symbol[0] = 1;
    expected_symbol[4] = 0x12;
    assert_eq!(&output.bytes[0x4018..0x4030], &expected_symbol);
    assert_eq!(&output.bytes[0x4030..0x4044], b"\0nuis_runtime_entry\0");
    assert_eq!(&output.bytes[0x4048..0x4050], &0x403000u64.to_le_bytes());
    assert_eq!(
        &output.bytes[0x4050..0x4058],
        &((1u64 << 32) | 7).to_le_bytes()
    );
    assert_eq!(&output.bytes[0x4058..0x4060], &[0; 8]);

    let report = &output.report;
    assert_eq!(
        report.contract,
        ELF_AMD64_PLATFORM_PATCH_APPLICATION_CONTRACT
    );
    assert_eq!(
        report.status,
        "platform-structures-and-deferred-patches-applied-with-unresolved-dynamic-binds"
    );
    assert_eq!(report.expected_structure_write_count, 7);
    assert_eq!(report.applied_structure_write_count, 7);
    assert_eq!(report.applied_deferred_patch_count, 1);
    assert_eq!(report.plt_write_count, 1);
    assert_eq!(report.got_write_count, 1);
    assert_eq!(report.dynamic_symbol_write_count, 2);
    assert_eq!(report.dynamic_string_write_count, 2);
    assert_eq!(report.dynamic_relocation_write_count, 1);
    assert_eq!(report.unresolved_dynamic_bind_count, 1);
    assert_eq!(report.write_once_span_count, 8);
    assert_eq!(
        report.dynamic_bind_records[0].target_symbol,
        "nuis_runtime_entry"
    );
    assert_eq!(report.dynamic_bind_records[0].relocation_type, 7);
    assert_eq!(report.dynamic_bind_records[0].relocation_offset, 0x403000);
    assert_eq!(
        report.applied_memory_image_hash,
        crate::fnv1a64_hex(&output.bytes)
    );
    assert_eq!(
        report.application_ledger_hash,
        crate::fnv1a64_hex(report.canonical_ledger().as_bytes())
    );
}

#[test]
fn repeated_calls_share_structures_but_each_source_is_patched_once() {
    let fixture = Fixture::new(elf_program_object_two_plt32_calls());
    let (placement, relocations, applied, plan) = fixture.external_pipeline();

    let output =
        apply_elf_amd64_platform_structure_plan(&placement, &relocations, &applied, &plan).unwrap();

    assert_eq!(&output.bytes[0x1001..0x1005], &[0xfb, 0x0f, 0, 0]);
    assert_eq!(&output.bytes[0x1006..0x100a], &[0xf6, 0x0f, 0, 0]);
    assert_eq!(output.report.applied_structure_write_count, 7);
    assert_eq!(output.report.applied_deferred_patch_count, 2);
    assert_eq!(output.report.write_once_span_count, 9);
    assert_eq!(output.report.dynamic_bind_records.len(), 1);
    assert_eq!(
        output.report.patches[0].structure_id,
        output.report.patches[1].structure_id
    );
    assert_ne!(
        output.report.patches[0].write_audit_hash,
        output.report.patches[1].write_audit_hash
    );
}

#[test]
fn closed_image_is_preserved_without_platform_writes() {
    let fixture = Fixture::new(elf_program_object(R_X86_64_PLT32));
    let (placement, relocations, applied, plan) = fixture.closed_pipeline();

    let output =
        apply_elf_amd64_platform_structure_plan(&placement, &relocations, &applied, &plan).unwrap();

    assert_eq!(output.bytes, applied.bytes);
    assert_eq!(output.report.status, "not-required-image-preserved");
    assert_eq!(output.report.expected_structure_write_count, 0);
    assert_eq!(output.report.applied_deferred_patch_count, 0);
    assert_eq!(output.report.write_once_span_count, 0);
    assert!(output.report.structure_writes.is_empty());
    assert!(output.report.patches.is_empty());
    assert!(output.report.dynamic_bind_records.is_empty());
    assert_eq!(
        output.report.base_applied_memory_image_hash,
        output.report.applied_memory_image_hash
    );
}

#[test]
fn base_image_and_plan_drift_fail_before_platform_publication() {
    let fixture = Fixture::new(elf_program_object(R_X86_64_PLT32));
    let (placement, relocations, applied, mut plan) = fixture.external_pipeline();
    plan.plan_hash.push('0');
    let plan_error =
        apply_elf_amd64_platform_structure_plan(&placement, &relocations, &applied, &plan)
            .unwrap_err();
    assert!(plan_error.contains("structure plan drift"), "{plan_error}");

    let (placement, relocations, mut applied, plan) = fixture.external_pipeline();
    applied.bytes[0x1000] ^= 1;
    let image_error =
        apply_elf_amd64_platform_structure_plan(&placement, &relocations, &applied, &plan)
            .unwrap_err();
    assert!(image_error.contains("base image drift"), "{image_error}");
}

#[test]
fn platform_write_once_primitive_rejects_overlap_and_source_drift() {
    let mut image = vec![0; 8];
    let mut occupied = vec![false; image.len()];
    apply_write_once(&mut image, &mut occupied, 1, &[0, 0], &[1, 2], "first").unwrap();

    let overlap =
        apply_write_once(&mut image, &mut occupied, 2, &[2, 0], &[3, 4], "overlap").unwrap_err();
    assert!(overlap.contains("overlaps a previously committed span"));

    let drift = apply_write_once(&mut image, &mut occupied, 4, &[1], &[2], "drift").unwrap_err();
    assert!(drift.contains("source drift"));
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

    fn external_pipeline(
        &self,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64AppliedImage,
        ElfAmd64PlatformStructurePlanReport,
    ) {
        self.pipeline(false)
    }

    fn closed_pipeline(
        &self,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64AppliedImage,
        ElfAmd64PlatformStructurePlanReport,
    ) {
        self.pipeline(true)
    }

    fn pipeline(
        &self,
        include_runtime: bool,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64AppliedImage,
        ElfAmd64PlatformStructurePlanReport,
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
        let plan =
            build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied.report)
                .unwrap();
        (placement, relocations, applied, plan)
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
