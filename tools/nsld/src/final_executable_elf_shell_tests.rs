use super::*;
use crate::{
    final_executable_elf_input::parse_elf64_amd64_object_linkage,
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_materialization::{
        application::{
            apply_elf_amd64_patch_previews,
            platform::{
                application::{
                    apply_elf_amd64_platform_structure_plan, ElfAmd64PlatformAppliedImage,
                },
                build_elf_amd64_platform_structure_plan, ElfAmd64PlatformStructurePlanReport,
            },
        },
        build_elf_amd64_materialization_preview, ElfAmd64ImageObject,
    },
    final_executable_elf_relocation::build_elf_amd64_relocation_application,
    final_executable_elf_test_fixture::{
        elf_program_object, elf_runtime_object, elf_runtime_object_with_bss,
        elf_unrelated_runtime_object, R_X86_64_PLT32,
    },
};

#[test]
fn plans_static_shell_with_file_backed_executable_entry() {
    let fixture = Fixture::new(elf_runtime_object());
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();

    let report = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();

    assert_eq!(report.contract, ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT);
    assert_eq!(report.status, "static-closure-layout-planned");
    assert_eq!(report.image_base, 0x0040_0000);
    assert_eq!(report.elf_header_size_bytes, 64);
    assert_eq!(report.program_header_entry_size_bytes, 56);
    assert_eq!(report.program_header_count, 3);
    assert_eq!(report.load_segment_count, 2);
    assert_eq!(report.section_header_count, 3);
    assert_eq!(report.entry_rule_id, "amd64.elf.program-entry.v1");
    assert_eq!(report.entry_symbol, "__nuis_entry");
    assert_eq!(report.entry_source_object_id, "host.program-llvm");
    assert_eq!(report.entry_source_image_offset, 0x1000);
    assert_eq!(report.entry_file_offset, 0x1000);
    assert_eq!(report.entry_virtual_address, 0x401000);
    assert_eq!(report.dynamic_table_entry_count, 0);
    assert!(report.dynamic_entries.is_empty());
    assert_eq!(
        report.plan_hash,
        crate::fnv1a64_hex(report.canonical_plan().as_bytes())
    );
    assert!(report
        .program_headers
        .iter()
        .any(|header| header.program_kind == "load" && header.flags == 5));
}

#[test]
fn plans_permission_segments_dynamic_table_and_section_links() {
    let fixture = Fixture::new(elf_unrelated_runtime_object());
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();

    let report = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();

    assert_eq!(
        report.status,
        "layout-planned-with-external-resolution-boundary"
    );
    assert_eq!(report.applied_memory_span_bytes, 0x4060);
    assert_eq!(report.dynamic_table_file_offset, Some(0x5000));
    assert_eq!(report.dynamic_table_virtual_address, Some(0x405000));
    assert_eq!(report.dynamic_table_entry_count, 12);
    assert_eq!(report.dynamic_table_bytes, 192);
    assert_eq!(report.planned_memory_span_bytes, 0x50c0);
    assert_eq!(report.load_segment_count, 6);
    assert_eq!(report.program_header_count, 8);
    assert_eq!(report.section_header_count, 9);
    assert_eq!(
        report
            .sections
            .iter()
            .map(|section| section.section_name.as_str())
            .collect::<Vec<_>>(),
        [
            "",
            ".text",
            ".plt",
            ".got.plt",
            ".dynsym",
            ".dynstr",
            ".rela.plt",
            ".dynamic",
            ".shstrtab",
        ]
    );
    let dynstr_index = section(&report, ".dynstr").section_index;
    let dynsym_index = section(&report, ".dynsym").section_index;
    let got_index = section(&report, ".got.plt").section_index;
    assert_eq!(section(&report, ".dynsym").link_section_index, dynstr_index);
    assert_eq!(
        section(&report, ".dynamic").link_section_index,
        dynstr_index
    );
    assert_eq!(
        section(&report, ".rela.plt").link_section_index,
        dynsym_index
    );
    assert_eq!(section(&report, ".rela.plt").info_section_index, got_index);
    assert_eq!(report.dynamic_entries.last().unwrap().tag_name, "DT_NULL");
    assert!(report
        .program_headers
        .iter()
        .any(|header| header.program_kind == "dynamic-table" && header.program_type == 2));
    assert_nonoverlapping_loads(&report);
}

#[test]
fn serializes_static_shell_into_a_deterministic_private_elf_image() {
    let fixture = Fixture::new(elf_runtime_object());
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();
    let shell = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();

    let first = serialize_elf_amd64_shell_image(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
        &shell,
    )
    .unwrap();
    let second = serialize_elf_amd64_shell_image(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
        &shell,
    )
    .unwrap();

    assert_eq!(second, first);
    assert_eq!(first.bytes.get(..4), Some(b"\x7fELF".as_slice()));
    assert_eq!(read_u16(&first.bytes, 16), 2);
    assert_eq!(read_u16(&first.bytes, 18), 62);
    assert_eq!(read_u64(&first.bytes, 24), shell.entry_virtual_address);
    assert_eq!(read_u64(&first.bytes, 32) as usize, 64);
    assert_eq!(
        read_u64(&first.bytes, 40) as usize,
        shell.section_header_table_file_offset
    );
    assert_eq!(read_u16(&first.bytes, 54), 56);
    assert_eq!(
        read_u16(&first.bytes, 56) as usize,
        shell.program_header_count
    );
    assert_eq!(read_u16(&first.bytes, 58), 64);
    assert_eq!(
        read_u16(&first.bytes, 60) as usize,
        shell.section_header_count
    );
    assert_eq!(
        read_u16(&first.bytes, 62) as usize,
        shell.section_name_table_section_index
    );
    assert_eq!(
        first.report.contract,
        super::image::ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT
    );
    assert_eq!(first.report.status, "serialized-static-private-image");
    assert_eq!(first.report.publication_status, "private-not-published");
    assert_eq!(first.report.applied_shell_write_count, 4);
    assert_eq!(first.report.dynamic_table_bytes, 0);
    assert_eq!(first.report.file_backed_source_span_count, 1);
    assert_eq!(first.report.zero_fill_source_span_count, 0);
    assert_eq!(first.report.source_preservation_count, 1);
    assert_eq!(
        first.report.serialization_ledger_hash,
        crate::fnv1a64_hex(first.report.canonical_ledger().as_bytes())
    );
    assert_eq!(
        first.report.shell_image_hash,
        crate::fnv1a64_hex(&first.bytes)
    );
    assert_eq!(
        first.bytes[shell.entry_file_offset],
        platform_applied.bytes[shell.entry_source_image_offset]
    );
    assert!(first
        .report
        .writes
        .iter()
        .all(|write| write.status == "applied-write-once"));
    assert!(first
        .report
        .source_preservations
        .iter()
        .all(|audit| audit.status == "preserved-byte-for-byte"));
    assert_section_names(&first.bytes, &shell);
}

#[test]
fn serializes_dynamic_tags_and_preserves_platform_records() {
    let fixture = Fixture::new(elf_unrelated_runtime_object());
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();
    let shell = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();

    let image = serialize_elf_amd64_shell_image(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
        &shell,
    )
    .unwrap();

    assert_eq!(
        image.report.status,
        "serialized-private-image-with-external-resolution-boundary"
    );
    assert_eq!(image.report.applied_shell_write_count, 5);
    assert_eq!(image.report.dynamic_table_bytes, shell.dynamic_table_bytes);
    assert_eq!(image.report.file_backed_source_span_count, 6);
    assert_eq!(image.report.zero_fill_source_span_count, 0);
    let dynamic_offset = shell.dynamic_table_file_offset.unwrap();
    for entry in &shell.dynamic_entries {
        let offset = dynamic_offset + entry.dynamic_entry_index * 16;
        assert_eq!(read_i64(&image.bytes, offset), entry.tag);
        assert_eq!(read_u64(&image.bytes, offset + 8), entry.value);
    }
    let dynamic_header = shell
        .program_headers
        .iter()
        .find(|header| header.program_kind == "dynamic-table")
        .unwrap();
    let header_offset = shell.program_header_table_file_offset
        + dynamic_header.program_header_index * shell.program_header_entry_size_bytes;
    assert_eq!(read_u32(&image.bytes, header_offset), 2);
    assert_eq!(
        read_u64(&image.bytes, header_offset + 8) as usize,
        dynamic_offset
    );
    assert_eq!(
        image.report.preserved_file_source_bytes,
        image
            .report
            .source_preservations
            .iter()
            .map(|audit| audit.source_size_bytes)
            .sum::<usize>()
    );
    assert!(image
        .report
        .source_preservations
        .iter()
        .all(|audit| audit.source_bytes_hash == audit.result_bytes_hash));
    assert_section_names(&image.bytes, &shell);
}

#[test]
fn serializes_zero_fill_as_nobits_without_file_payload() {
    let fixture = Fixture::new(elf_runtime_object_with_bss());
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();
    let shell = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();

    let image = serialize_elf_amd64_shell_image(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
        &shell,
    )
    .unwrap();

    let bss = section(&shell, ".bss");
    let bss_audit = image
        .report
        .source_preservations
        .iter()
        .find(|audit| audit.section_name == ".bss")
        .unwrap();
    assert_eq!(bss.section_type, 8);
    assert_eq!(bss.file_size_bytes, 0);
    assert_eq!(bss.memory_size_bytes, 32);
    assert_eq!(image.report.zero_fill_source_span_count, 1);
    assert_eq!(image.report.preserved_zero_fill_bytes, 32);
    assert_eq!(bss_audit.preservation_kind, "nobits-zero-fill-span");
    assert_eq!(bss_audit.result_file_offset, None);
    assert_eq!(bss_audit.status, "preserved-as-nobits-zero-fill");
    let section_header = shell.section_header_table_file_offset
        + bss.section_index * shell.section_header_entry_size_bytes;
    assert_eq!(read_u32(&image.bytes, section_header + 4), 8);
    assert_eq!(read_u64(&image.bytes, section_header + 32), 32);
}

#[test]
fn serializer_rejects_platform_image_drift() {
    let fixture = Fixture::new(elf_runtime_object());
    let (placement, relocations, platform_plan, mut platform_applied) = fixture.pipeline();
    let shell = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();
    platform_applied.bytes[shell.entry_source_image_offset] ^= 1;

    let error = serialize_elf_amd64_shell_image(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
        &shell,
    )
    .unwrap_err();

    assert!(error.contains("platform image drift"), "{error}");
}

#[test]
fn serializer_rebuilds_and_rejects_rehashed_layout_drift() {
    let fixture = Fixture::new(elf_runtime_object());
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();
    let mut shell = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();
    shell.entry_virtual_address += 1;
    shell.plan_hash = crate::fnv1a64_hex(shell.canonical_plan().as_bytes());

    let error = serialize_elf_amd64_shell_image(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
        &shell,
    )
    .unwrap_err();

    assert!(error.contains("layout plan drift"), "{error}");
}

#[test]
fn shell_plan_is_independent_of_object_input_order() {
    let fixture = Fixture::new(elf_unrelated_runtime_object());
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();
    let forward = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();
    let mut reversed = fixture.objects.clone();
    reversed.reverse();

    let reverse = build_elf_amd64_shell_layout_plan(
        &reversed,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap();

    assert_eq!(reverse, forward);
}

#[test]
fn zero_fill_load_coordinates_remain_inside_the_planned_file_envelope() {
    let fixture = Fixture::new(elf_runtime_object());
    let (mut placement, _, mut platform_plan, _) = fixture.pipeline();
    placement.merged_sections.push(
        crate::final_executable_elf_layout_report::ElfAmd64MergedSectionPlan {
            section_id: "elf-section-zero-fill-probe".to_owned(),
            output_section_name: ".bss".to_owned(),
            class: "bss".to_owned(),
            alignment: 0x1000,
            file_offset: None,
            image_offset: 0x2000,
            virtual_address: 0x402000,
            size_bytes: 32,
            contribution_count: 1,
            zero_fill: true,
        },
    );
    placement.memory_span_bytes = 0x2020;
    platform_plan.base_memory_span_bytes = placement.memory_span_bytes;
    platform_plan.planned_memory_span_bytes = placement.memory_span_bytes;

    let layout =
        layout::build_elf_amd64_shell_layout(&placement, &platform_plan, "ledger").unwrap();
    let bss = layout
        .sections
        .iter()
        .find(|section| section.section_name == ".bss")
        .unwrap();
    let bss_load = layout
        .program_headers
        .iter()
        .find(|header| bss.load_segment_id.as_deref() == Some(header.program_header_id.as_str()))
        .unwrap();

    assert_eq!(bss.file_size_bytes, 0);
    assert_eq!(bss_load.file_size_bytes, 0);
    assert!(bss_load.file_offset <= layout.planned_file_span_bytes);
    assert!(layout.section_name_table_file_offset >= placement.memory_span_bytes);
}

#[test]
fn platform_application_ledger_drift_fails_before_shell_planning() {
    let fixture = Fixture::new(elf_unrelated_runtime_object());
    let (placement, relocations, platform_plan, mut platform_applied) = fixture.pipeline();
    platform_applied.report.application_ledger_hash.push('0');

    let error = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap_err();

    assert!(error.contains("hash or ledger drift"), "{error}");
}

#[test]
fn missing_registered_entry_fails_closed() {
    let fixture = Fixture::without_program_entry();
    let (placement, relocations, platform_plan, platform_applied) = fixture.pipeline();

    let error = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform_applied,
    )
    .unwrap_err();

    assert!(error.contains("entry registry found no supported definition"));
}

fn section<'a>(
    report: &'a ElfAmd64ShellLayoutPlanReport,
    name: &str,
) -> &'a report::ElfAmd64ShellSectionPlan {
    report
        .sections
        .iter()
        .find(|section| section.section_name == name)
        .unwrap()
}

fn assert_nonoverlapping_loads(report: &ElfAmd64ShellLayoutPlanReport) {
    let loads = report
        .program_headers
        .iter()
        .filter(|header| header.program_kind == "load")
        .collect::<Vec<_>>();
    for (index, load) in loads.iter().enumerate() {
        let load_end = load.virtual_address + load.memory_size_bytes as u64;
        for other in loads.iter().skip(index + 1) {
            let other_end = other.virtual_address + other.memory_size_bytes as u64;
            assert!(load_end <= other.virtual_address || other_end <= load.virtual_address);
        }
    }
}

fn assert_section_names(bytes: &[u8], report: &ElfAmd64ShellLayoutPlanReport) {
    for section in &report.sections {
        let start = report.section_name_table_file_offset + section.section_name_offset;
        let end = start + section.section_name.len();
        assert_eq!(&bytes[start..end], section.section_name.as_bytes());
        assert_eq!(bytes[end], 0);
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn read_i64(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

pub(super) struct Fixture {
    program_bytes: Vec<u8>,
    runtime_bytes: Vec<u8>,
    program_hash: String,
    runtime_hash: String,
    pub(super) objects: Vec<ElfAmd64ObjectLinkage>,
}

impl Fixture {
    pub(super) fn new(runtime_bytes: Vec<u8>) -> Self {
        Self::from_bytes(elf_program_object(R_X86_64_PLT32), runtime_bytes)
    }

    fn without_program_entry() -> Self {
        Self::from_bytes(elf_unrelated_runtime_object(), elf_runtime_object())
    }

    fn from_bytes(program_bytes: Vec<u8>, runtime_bytes: Vec<u8>) -> Self {
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

    pub(super) fn pipeline(
        &self,
    ) -> (
        ElfAmd64PlacementBindingReport,
        ElfAmd64RelocationApplicationReport,
        ElfAmd64PlatformStructurePlanReport,
        ElfAmd64PlatformAppliedImage,
    ) {
        let placement = build_elf_amd64_placement_binding(&self.objects).unwrap();
        let relocations =
            build_elf_amd64_relocation_application(&self.objects, &placement).unwrap();
        let inputs = self.image_objects();
        let preview =
            build_elf_amd64_materialization_preview(&inputs, &placement, &relocations).unwrap();
        let applied =
            apply_elf_amd64_patch_previews(&inputs, &placement, &relocations, &preview).unwrap();
        let platform_plan =
            build_elf_amd64_platform_structure_plan(&placement, &relocations, &applied.report)
                .unwrap();
        let platform_applied = apply_elf_amd64_platform_structure_plan(
            &placement,
            &relocations,
            &applied,
            &platform_plan,
        )
        .unwrap();
        (placement, relocations, platform_plan, platform_applied)
    }

    fn image_objects(&self) -> Vec<ElfAmd64ImageObject<'_>> {
        vec![
            ElfAmd64ImageObject {
                object_id: &self.objects[0].object_id,
                role: &self.objects[0].role,
                bytes: &self.program_bytes,
                planned_size_bytes: self.program_bytes.len(),
                planned_source_hash: &self.program_hash,
                linkage: &self.objects[0].linkage,
            },
            ElfAmd64ImageObject {
                object_id: &self.objects[1].object_id,
                role: &self.objects[1].role,
                bytes: &self.runtime_bytes,
                planned_size_bytes: self.runtime_bytes.len(),
                planned_source_hash: &self.runtime_hash,
                linkage: &self.objects[1].linkage,
            },
        ]
    }
}
