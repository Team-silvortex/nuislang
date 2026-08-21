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
        elf_program_object, elf_runtime_object, elf_unrelated_runtime_object, R_X86_64_PLT32,
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

struct Fixture {
    program_bytes: Vec<u8>,
    runtime_bytes: Vec<u8>,
    program_hash: String,
    runtime_hash: String,
    objects: Vec<ElfAmd64ObjectLinkage>,
}

impl Fixture {
    fn new(runtime_bytes: Vec<u8>) -> Self {
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

    fn pipeline(
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
