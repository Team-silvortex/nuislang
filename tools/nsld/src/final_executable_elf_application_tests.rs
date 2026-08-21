use super::*;
use crate::{
    final_executable_elf_input::parse_elf64_amd64_object_linkage,
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_object::ElfAmd64ObjectLinkage,
    final_executable_elf_relocation::build_elf_amd64_relocation_application,
    final_executable_elf_test_fixture::{elf_program_object, elf_runtime_object, R_X86_64_PLT32},
};

#[test]
fn applies_real_plt32_patch_once_and_emits_a_bound_ledger() {
    let fixture = Fixture::new();
    let (placement, relocations, preview) = fixture.plans();
    let inputs = fixture.image_objects();

    let applied =
        apply_elf_amd64_patch_previews(&inputs, &placement, &relocations, &preview).unwrap();

    assert_eq!(&applied.bytes[0x1000..0x1006], &[0xe8, 0x0b, 0, 0, 0, 0xc3]);
    assert_eq!(applied.bytes[0x1010], 0xc3);
    assert_eq!(
        applied.report.contract,
        ELF_AMD64_PATCH_APPLICATION_CONTRACT
    );
    assert_eq!(applied.report.status, "direct-patches-applied");
    assert_eq!(applied.report.materialization_plan_hash, preview.plan_hash);
    assert_eq!(applied.report.expected_patch_count, 1);
    assert_eq!(applied.report.applied_patch_count, 1);
    assert_eq!(applied.report.write_once_span_count, 1);
    assert_ne!(
        applied.report.source_memory_image_hash,
        applied.report.applied_memory_image_hash
    );
    assert_eq!(
        applied.report.applied_memory_image_hash,
        crate::fnv1a64_hex(&applied.bytes)
    );
    assert_eq!(applied.report.patches[0].status, "applied-write-once");
    assert_eq!(
        applied.report.patches[0].post_write_bytes_hash,
        applied.report.patches[0].encoded_bytes_hash
    );
    assert_eq!(
        applied.report.application_ledger_hash,
        crate::fnv1a64_hex(applied.report.canonical_ledger().as_bytes())
    );
}

#[test]
fn unresolved_system_target_stays_deferred_and_preserves_the_image() {
    let fixture = Fixture::new();
    let objects = &fixture.objects[..1];
    let placement = build_elf_amd64_placement_binding(objects).unwrap();
    let relocations = build_elf_amd64_relocation_application(objects, &placement).unwrap();
    let inputs = fixture.program_image_object();
    let preview =
        build_elf_amd64_materialization_preview(&inputs, &placement, &relocations).unwrap();

    let applied =
        apply_elf_amd64_patch_previews(&inputs, &placement, &relocations, &preview).unwrap();

    assert_eq!(
        applied.report.status,
        "direct-patches-applied-with-platform-structure-boundary"
    );
    assert_eq!(applied.report.expected_patch_count, 0);
    assert_eq!(applied.report.applied_patch_count, 0);
    assert_eq!(applied.report.deferred_patch_count, 1);
    assert_eq!(
        applied.report.source_memory_image_hash,
        applied.report.applied_memory_image_hash
    );
}

#[test]
fn application_is_independent_of_image_object_input_order() {
    let fixture = Fixture::new();
    let (placement, relocations, preview) = fixture.plans();
    let forward = apply_elf_amd64_patch_previews(
        &fixture.image_objects(),
        &placement,
        &relocations,
        &preview,
    )
    .unwrap();
    let mut reversed_inputs = fixture.image_objects();
    reversed_inputs.reverse();

    let reversed =
        apply_elf_amd64_patch_previews(&reversed_inputs, &placement, &relocations, &preview)
            .unwrap();

    assert_eq!(reversed.bytes, forward.bytes);
    assert_eq!(reversed.report, forward.report);
}

#[test]
fn source_and_preview_drift_fail_before_any_write() {
    let mut source_drift = Fixture::new();
    let (placement, relocations, preview) = source_drift.plans();
    source_drift.program_bytes[64] ^= 1;

    let error = apply_elf_amd64_patch_previews(
        &source_drift.image_objects(),
        &placement,
        &relocations,
        &preview,
    )
    .unwrap_err();
    assert!(error.contains("source hash drift"), "{error}");

    let fixture = Fixture::new();
    let (placement, relocations, mut preview) = fixture.plans();
    preview.patches[0].encoded_bytes[0] ^= 1;
    preview.plan_hash = crate::fnv1a64_hex(preview.canonical_plan().as_bytes());
    let error = apply_elf_amd64_patch_previews(
        &fixture.image_objects(),
        &placement,
        &relocations,
        &preview,
    )
    .unwrap_err();
    assert!(error.contains("materialization preview drift"), "{error}");

    let (_, _, mut bad_hash) = fixture.plans();
    bad_hash.plan_hash.push('0');
    let error = apply_elf_amd64_patch_previews(
        &fixture.image_objects(),
        &placement,
        &relocations,
        &bad_hash,
    )
    .unwrap_err();
    assert!(error.contains("preview plan hash drift"), "{error}");
}

#[test]
fn write_once_primitive_rejects_overlap_and_source_drift() {
    let mut image = vec![1, 2, 3, 4, 5];
    let mut written = vec![false; image.len()];
    apply_write_once(&mut image, &mut written, 1, &[2, 3], &[8, 9], "first").unwrap();
    assert_eq!(image, [1, 8, 9, 4, 5]);

    let overlap =
        apply_write_once(&mut image, &mut written, 2, &[9, 4], &[7, 6], "overlap").unwrap_err();
    assert!(overlap.contains("overlaps a previously applied patch"));

    let drift = apply_write_once(&mut image, &mut written, 3, &[7], &[6], "drift").unwrap_err();
    assert!(drift.contains("write source drift"));
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

    fn plans(
        &self,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64MaterializationPreviewReport,
    ) {
        let placement = build_elf_amd64_placement_binding(&self.objects).unwrap();
        let relocations =
            build_elf_amd64_relocation_application(&self.objects, &placement).unwrap();
        let preview = build_elf_amd64_materialization_preview(
            &self.image_objects(),
            &placement,
            &relocations,
        )
        .unwrap();
        (placement, relocations, preview)
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
