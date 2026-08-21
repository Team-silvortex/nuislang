use super::*;
use crate::final_executable_macho_input::{
    ParsedMachOObjectLinkage, ParsedMachOSection, ParsedMachOSymbol,
};
use std::collections::BTreeSet;

#[test]
fn placement_and_binding_are_deterministic_across_input_order() {
    let program = linkage(
        0,
        vec![
            defined_symbol(0, "_entry", 1, 0),
            undefined_symbol(1, "_runtime"),
            undefined_symbol(2, "_puts"),
        ],
    );
    let runtime = linkage(0, vec![defined_symbol(0, "_runtime", 1, 0)]);
    let forward = [
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
    ];
    let reversed = [
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
    ];

    let report = build_macho_placement_binding_report(&forward).unwrap();
    let reordered = build_macho_placement_binding_report(&reversed).unwrap();

    assert_eq!(report, reordered);
    assert_eq!(report.contract, MACHO_PLACEMENT_BINDING_CONTRACT);
    assert_eq!(report.merged_sections.len(), 1);
    assert_eq!(report.section_placements.len(), 2);
    assert_eq!(report.image_span_bytes, 16);
    assert_eq!(report.section_placements[0].object_id, "host.program");
    assert_eq!(report.section_placements[0].output_offset, 0);
    assert_eq!(report.section_placements[1].object_id, "host.runtime");
    assert_eq!(report.section_placements[1].output_offset, 8);
    assert_eq!(report.symbol_bindings.len(), 2);
    assert_eq!(report.symbol_bindings[0].symbol, "_runtime");
    assert_eq!(report.symbol_bindings[0].status, "internal");
    assert_eq!(
        report.symbol_bindings[0].target_object_id.as_deref(),
        Some("host.runtime")
    );
    assert_eq!(report.symbol_bindings[0].target_output_offset, Some(8));
    assert_eq!(report.symbol_bindings[1].symbol, "_puts");
    assert_eq!(report.symbol_bindings[1].status, "external-compatibility");
    assert_eq!(report.internally_bound_symbol_count, 1);
    assert_eq!(report.external_compatibility_symbol_count, 1);
}

#[test]
fn duplicate_external_definitions_are_rejected() {
    let program = linkage(0, vec![defined_symbol(0, "_duplicate", 1, 0)]);
    let runtime = linkage(0, vec![defined_symbol(0, "_duplicate", 1, 0)]);
    let objects = [
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
    ];

    let error = build_macho_placement_binding_report(&objects).unwrap_err();
    assert!(error.contains("duplicate external Mach-O definition `_duplicate`"));
    assert!(error.contains("host.program"));
    assert!(error.contains("host.runtime"));
}

#[test]
fn incompatible_merged_section_flags_are_rejected() {
    let program = linkage(0, vec![]);
    let runtime = linkage(1, vec![]);
    let objects = [
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
    ];

    let error = build_macho_placement_binding_report(&objects).unwrap_err();
    assert!(error.contains("incompatible flags"), "{error}");
}

#[test]
fn referenced_common_definition_gets_provider_owned_storage() {
    let program = linkage(0, vec![undefined_symbol(0, "_shared")]);
    let runtime = linkage(0, vec![common_symbol(0, "_shared")]);
    let objects = [
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
    ];

    let report = build_macho_placement_binding_report(&objects).unwrap();

    assert_eq!(report.image_span_bytes, 24);
    assert_eq!(report.merged_sections.len(), 2);
    let section = &report.merged_sections[1];
    assert_eq!(section.segment_name, "__DATA");
    assert_eq!(section.section_name, "__nuis_common");
    assert_eq!(section.flags, 1);
    assert!(section.zero_fill);
    assert_eq!(report.common_allocations.len(), 1);
    let allocation = &report.common_allocations[0];
    assert_eq!(allocation.symbol, "_shared");
    assert_eq!(allocation.owner_object_id, "host.runtime");
    assert_eq!(allocation.size_bytes, 8);
    assert_eq!(allocation.alignment, 8);
    assert_eq!(allocation.output_offset, 16);
    assert_eq!(report.symbol_bindings.len(), 2);
    assert!(report
        .symbol_bindings
        .iter()
        .all(|binding| binding.target_output_offset == Some(16)));
}

#[test]
fn duplicate_common_declarations_coalesce_by_max_shape_deterministically() {
    let program = linkage(0, vec![common_symbol_with_shape(0, "_shared", 4, 4)]);
    let runtime = linkage(0, vec![common_symbol_with_shape(0, "_shared", 24, 16)]);
    let forward = [
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
    ];
    let reversed = [
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
    ];

    let report = build_macho_placement_binding_report(&forward).unwrap();
    assert_eq!(
        report,
        build_macho_placement_binding_report(&reversed).unwrap()
    );
    let allocation = &report.common_allocations[0];
    assert_eq!(allocation.owner_object_id, "host.program");
    assert_eq!(allocation.declaration_count, 2);
    assert_eq!(allocation.size_bytes, 24);
    assert_eq!(allocation.alignment, 16);
    assert_eq!(allocation.output_offset, 16);
    assert_eq!(report.image_span_bytes, 40);
}

#[test]
fn strong_definition_overrides_common_without_allocating_storage() {
    let program = linkage(0, vec![common_symbol(0, "_shared")]);
    let runtime = linkage(0, vec![defined_symbol(0, "_shared", 1, 0)]);
    let objects = [
        MachOLayoutObject {
            object_id: "host.program",
            role: "program-llvm",
            linkage: &program,
        },
        MachOLayoutObject {
            object_id: "host.runtime",
            role: "runtime-shim",
            linkage: &runtime,
        },
    ];

    let report = build_macho_placement_binding_report(&objects).unwrap();

    assert!(report.common_allocations.is_empty());
    assert_eq!(report.merged_sections.len(), 1);
    assert_eq!(report.symbol_bindings.len(), 1);
    assert_eq!(
        report.symbol_bindings[0].target_kind.as_deref(),
        Some("section")
    );
    assert_eq!(
        report.symbol_bindings[0].target_object_id.as_deref(),
        Some("host.runtime")
    );
    assert_eq!(report.symbol_bindings[0].target_output_offset, Some(8));
}

#[test]
fn input_cannot_claim_the_provider_owned_common_section() {
    let mut program = linkage(1, vec![common_symbol(0, "_shared")]);
    program.sections[0].segment_name = "__DATA".to_owned();
    program.sections[0].name = "__nuis_common".to_owned();
    program.sections[0].zero_fill = true;
    let objects = [MachOLayoutObject {
        object_id: "host.program",
        role: "program-llvm",
        linkage: &program,
    }];

    let error = build_macho_placement_binding_report(&objects).unwrap_err();

    assert!(error.contains("reserves provider-owned section"), "{error}");
    assert!(error.contains("__DATA,__nuis_common"), "{error}");
}

fn linkage(flags: u32, symbols: Vec<ParsedMachOSymbol>) -> ParsedMachOObjectLinkage {
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
    ParsedMachOObjectLinkage {
        section_count: 1,
        symbol_count: symbols.len(),
        relocation_count: 0,
        defined_symbol_count: external_definitions.len(),
        undefined_symbol_count: external_undefined.len(),
        external_definitions,
        external_undefined,
        sections: vec![ParsedMachOSection {
            ordinal: 1,
            segment_name: "__TEXT".to_owned(),
            name: "__text".to_owned(),
            address: 0,
            size: 8,
            alignment: 4,
            flags,
            zero_fill: false,
            payload_offset: 0,
            relocation_offset: 0,
            relocation_count: 0,
        }],
        symbols,
        relocations: vec![],
    }
}

fn defined_symbol(
    index: usize,
    name: &str,
    section_ordinal: usize,
    value: u64,
) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "section".to_owned(),
        external: true,
        defined: true,
        section_ordinal: Some(section_ordinal),
        value,
        common_alignment: None,
        indirect_target: None,
    }
}

fn undefined_symbol(index: usize, name: &str) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "undefined".to_owned(),
        external: true,
        defined: false,
        section_ordinal: None,
        value: 0,
        common_alignment: None,
        indirect_target: None,
    }
}

fn common_symbol(index: usize, name: &str) -> ParsedMachOSymbol {
    common_symbol_with_shape(index, name, 8, 8)
}

fn common_symbol_with_shape(
    index: usize,
    name: &str,
    size: u64,
    alignment: u64,
) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "common".to_owned(),
        external: true,
        defined: true,
        section_ordinal: None,
        value: size,
        common_alignment: Some(alignment),
        indirect_target: None,
    }
}
