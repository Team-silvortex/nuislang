use super::*;
use crate::{
    final_executable_elf_input::{
        parse_elf64_amd64_object_linkage, ParsedElfObjectLinkage, ParsedElfSection, ParsedElfSymbol,
    },
    final_executable_elf_test_fixture::{elf_program_object, elf_runtime_object, R_X86_64_PLT32},
};
use std::collections::BTreeSet;

#[test]
fn places_real_host_objects_and_binds_cross_object_reference() {
    let objects = fixture_objects();

    let report = build_elf_amd64_placement_binding(&objects).unwrap();

    assert_eq!(report.contract, ELF_AMD64_PLACEMENT_BINDING_CONTRACT);
    assert_eq!(report.status, "placement-and-internal-binding-ready");
    assert_eq!(report.image_base, 0x0040_0000);
    assert_eq!(report.payload_file_offset, 0x1000);
    assert_eq!(report.file_span_bytes, 0x1011);
    assert_eq!(report.memory_span_bytes, 0x1011);
    assert_eq!(report.merged_sections.len(), 1);
    assert_eq!(report.merged_sections[0].output_section_name, ".text");
    assert_eq!(report.merged_sections[0].size_bytes, 17);
    assert_eq!(report.section_placements.len(), 2);
    assert_eq!(report.section_placements[0].object_role, "program-llvm");
    assert_eq!(report.section_placements[0].file_offset, Some(0x1000));
    assert_eq!(report.section_placements[0].virtual_address, 0x401000);
    assert_eq!(report.section_placements[1].object_role, "runtime-shim");
    assert_eq!(report.section_placements[1].file_offset, Some(0x1010));
    assert_eq!(report.section_placements[1].virtual_address, 0x401010);
    let runtime_reference = binding(&report, "host.program-llvm", "nuis_runtime_entry");
    assert_eq!(runtime_reference.status, "internal");
    assert_eq!(
        runtime_reference.target_object_id.as_deref(),
        Some("host.runtime-shim")
    );
    assert_eq!(runtime_reference.target_virtual_address, Some(0x401010));
    assert_eq!(report.internally_bound_symbol_count, 1);
    assert_eq!(report.external_compatibility_symbol_count, 0);
}

#[test]
fn places_all_runtime_classes_and_binds_common_and_absolute_symbols() {
    let objects = semantic_matrix_objects();

    let report = build_elf_amd64_placement_binding(&objects).unwrap();

    assert_eq!(
        report
            .merged_sections
            .iter()
            .map(|section| section.class.as_str())
            .collect::<Vec<_>>(),
        ["text", "rodata", "data", "bss", "common"]
    );
    assert_eq!(report.file_span_bytes, 0x3004);
    assert_eq!(report.memory_span_bytes, 0x5020);
    let bss = report
        .merged_sections
        .iter()
        .find(|section| section.class == "bss")
        .unwrap();
    assert_eq!(bss.file_offset, None);
    assert_eq!(bss.image_offset, 0x4000);
    let common = &report.common_allocations[0];
    assert_eq!(common.symbol, "shared");
    assert_eq!(common.declaration_count, 2);
    assert_eq!(common.size_bytes, 32);
    assert_eq!(common.alignment, 16);
    assert_eq!(common.image_offset, 0x5000);
    assert_eq!(common.virtual_address, 0x405000);
    let entry = binding(&report, "host.program-llvm", "entry");
    assert_eq!(entry.target_virtual_address, Some(0x401001));
    let absolute = binding(&report, "host.program-llvm", "absolute");
    assert_eq!(absolute.target_kind.as_deref(), Some("absolute"));
    assert_eq!(absolute.target_absolute_value, Some(0x1234));
    assert_eq!(absolute.target_virtual_address, Some(0x1234));
    let runtime = binding(&report, "host.program-llvm", "runtime");
    assert_eq!(runtime.status, "internal");
    assert_eq!(runtime.target_virtual_address, Some(0x401010));
    for shared in report
        .symbol_bindings
        .iter()
        .filter(|binding| binding.symbol == "shared")
    {
        assert_eq!(shared.status, "common-allocation");
        assert_eq!(shared.target_virtual_address, Some(0x405000));
    }
}

#[test]
fn placement_plan_is_independent_of_input_object_order() {
    let mut objects = semantic_matrix_objects();
    let forward = build_elf_amd64_placement_binding(&objects).unwrap();
    objects.reverse();

    let reverse = build_elf_amd64_placement_binding(&objects).unwrap();

    assert_eq!(reverse.plan_hash, forward.plan_hash);
    assert_eq!(reverse.merged_sections, forward.merged_sections);
    assert_eq!(reverse.section_placements, forward.section_placements);
    assert_eq!(reverse.common_allocations, forward.common_allocations);
    assert_eq!(reverse.symbol_bindings, forward.symbol_bindings);
}

#[test]
fn preserves_unresolved_system_symbols_as_an_explicit_compatibility_boundary() {
    let object = object(
        "host.program-llvm",
        "program-llvm",
        vec![section(1, ".text", 0x6, 1, 16, false)],
        vec![null_symbol(), undefined_symbol(1, "puts")],
    );

    let report = build_elf_amd64_placement_binding(&[object]).unwrap();

    assert_eq!(
        report.status,
        "placement-ready-with-external-compatibility-boundary"
    );
    assert_eq!(report.internally_bound_symbol_count, 0);
    assert_eq!(report.external_compatibility_symbol_count, 1);
    let reference = binding(&report, "host.program-llvm", "puts");
    assert_eq!(reference.status, "external-compatibility");
    assert_eq!(reference.target_virtual_address, None);
}

#[test]
fn resolves_an_unmatched_weak_reference_to_zero() {
    let object = object(
        "host.program-llvm",
        "program-llvm",
        vec![section(1, ".text", 0x6, 1, 16, false)],
        vec![null_symbol(), weak_undefined_symbol(1, "optional_hook")],
    );

    let report = build_elf_amd64_placement_binding(&[object]).unwrap();

    assert_eq!(report.status, "placement-and-internal-binding-ready");
    assert_eq!(report.external_compatibility_symbol_count, 0);
    let reference = binding(&report, "host.program-llvm", "optional_hook");
    assert_eq!(reference.status, "weak-zero");
    assert_eq!(reference.target_kind.as_deref(), Some("weak-zero"));
    assert_eq!(reference.target_absolute_value, Some(0));
    assert_eq!(reference.target_virtual_address, Some(0));
}

#[test]
fn rejects_alignment_that_would_create_an_unbounded_image_gap() {
    let object = object(
        "host.program-llvm",
        "program-llvm",
        vec![section(1, ".text", 0x6, 1, 0x40_0000, false)],
        vec![null_symbol()],
    );

    let error = build_elf_amd64_placement_binding(&[object]).unwrap_err();

    assert!(error.contains("outside the supported power-of-two range"));
}

#[test]
fn rejects_tls_until_the_runtime_has_a_tls_placement_contract() {
    let object = object(
        "host.program-llvm",
        "program-llvm",
        vec![section(1, ".tdata", 0x403, 8, 8, false)],
        vec![null_symbol()],
    );

    let error = build_elf_amd64_placement_binding(&[object]).unwrap_err();

    assert!(error.contains("unsupported TLS placement"));
}

fn fixture_objects() -> Vec<ElfAmd64ObjectLinkage> {
    vec![
        ElfAmd64ObjectLinkage {
            object_id: "host.program-llvm".to_owned(),
            role: "program-llvm".to_owned(),
            linkage: parse_elf64_amd64_object_linkage(&elf_program_object(R_X86_64_PLT32)).unwrap(),
        },
        ElfAmd64ObjectLinkage {
            object_id: "host.runtime-shim".to_owned(),
            role: "runtime-shim".to_owned(),
            linkage: parse_elf64_amd64_object_linkage(&elf_runtime_object()).unwrap(),
        },
    ]
}

fn semantic_matrix_objects() -> Vec<ElfAmd64ObjectLinkage> {
    vec![
        object(
            "host.program-llvm",
            "program-llvm",
            vec![
                section(1, ".text", 0x6, 3, 16, false),
                section(2, ".rodata", 0x2, 5, 8, false),
                section(3, ".data", 0x3, 4, 4, false),
                section(4, ".bss", 0x3, 8, 8, true),
            ],
            vec![
                null_symbol(),
                defined_symbol(1, "entry", 1, 1, 1),
                absolute_symbol(2, "absolute", 0x1234),
                common_symbol(3, "shared", 16, 24),
                undefined_symbol(4, "runtime"),
            ],
        ),
        object(
            "host.runtime-shim",
            "runtime-shim",
            vec![section(1, ".text", 0x6, 1, 16, false)],
            vec![
                null_symbol(),
                defined_symbol(1, "runtime", 1, 0, 1),
                common_symbol(2, "shared", 8, 32),
            ],
        ),
    ]
}

fn object(
    object_id: &str,
    role: &str,
    sections: Vec<ParsedElfSection>,
    symbols: Vec<ParsedElfSymbol>,
) -> ElfAmd64ObjectLinkage {
    let external_definitions = symbols
        .iter()
        .filter(|symbol| symbol.external && symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect::<BTreeSet<_>>();
    let external_undefined = symbols
        .iter()
        .filter(|symbol| symbol.external && !symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect::<BTreeSet<_>>();
    let defined_symbol_count = symbols
        .iter()
        .skip(1)
        .filter(|symbol| symbol.defined)
        .count();
    let undefined_symbol_count = symbols
        .iter()
        .skip(1)
        .filter(|symbol| !symbol.defined)
        .count();
    ElfAmd64ObjectLinkage {
        object_id: object_id.to_owned(),
        role: role.to_owned(),
        linkage: ParsedElfObjectLinkage {
            section_count: sections.len() + 1,
            symbol_count: symbols.len(),
            relocation_count: 0,
            defined_symbol_count,
            undefined_symbol_count,
            external_definitions,
            external_undefined,
            sections,
            symbols,
            relocations: Vec::new(),
        },
    }
}

fn section(
    index: usize,
    name: &str,
    flags: u64,
    size: usize,
    alignment: u64,
    zero_fill: bool,
) -> ParsedElfSection {
    ParsedElfSection {
        index,
        name: name.to_owned(),
        section_type: if zero_fill { 8 } else { 1 },
        flags,
        size,
        alignment,
        payload_offset: (!zero_fill).then_some(0),
        zero_fill,
    }
}

fn null_symbol() -> ParsedElfSymbol {
    ParsedElfSymbol {
        index: 0,
        name: String::new(),
        binding: 0,
        symbol_type: 0,
        external: false,
        weak: false,
        defined: false,
        section_index: None,
        absolute: false,
        common: false,
        value: 0,
        size: 0,
    }
}

fn defined_symbol(
    index: usize,
    name: &str,
    section_index: usize,
    value: u64,
    size: u64,
) -> ParsedElfSymbol {
    ParsedElfSymbol {
        index,
        name: name.to_owned(),
        binding: 1,
        symbol_type: 2,
        external: true,
        weak: false,
        defined: true,
        section_index: Some(section_index),
        absolute: false,
        common: false,
        value,
        size,
    }
}

fn absolute_symbol(index: usize, name: &str, value: u64) -> ParsedElfSymbol {
    ParsedElfSymbol {
        section_index: None,
        absolute: true,
        value,
        size: 0,
        ..defined_symbol(index, name, 1, 0, 0)
    }
}

fn common_symbol(index: usize, name: &str, alignment: u64, size: u64) -> ParsedElfSymbol {
    ParsedElfSymbol {
        section_index: None,
        common: true,
        value: alignment,
        size,
        ..defined_symbol(index, name, 1, 0, 0)
    }
}

fn undefined_symbol(index: usize, name: &str) -> ParsedElfSymbol {
    ParsedElfSymbol {
        index,
        name: name.to_owned(),
        binding: 1,
        symbol_type: 2,
        external: true,
        weak: false,
        defined: false,
        section_index: None,
        absolute: false,
        common: false,
        value: 0,
        size: 0,
    }
}

fn weak_undefined_symbol(index: usize, name: &str) -> ParsedElfSymbol {
    ParsedElfSymbol {
        binding: 2,
        weak: true,
        ..undefined_symbol(index, name)
    }
}

fn binding<'a>(
    report: &'a ElfAmd64PlacementBindingReport,
    object_id: &str,
    symbol: &str,
) -> &'a ElfAmd64SymbolBinding {
    report
        .symbol_bindings
        .iter()
        .find(|binding| binding.reference_object_id == object_id && binding.symbol == symbol)
        .unwrap()
}
