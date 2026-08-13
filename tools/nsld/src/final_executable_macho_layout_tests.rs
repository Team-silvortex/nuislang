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
fn referenced_non_section_definition_is_rejected() {
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

    let error = build_macho_placement_binding_report(&objects).unwrap_err();
    assert!(
        error.contains("definition `_shared` kind `common`"),
        "{error}"
    );
    assert!(error.contains("not section-backed"), "{error}");
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
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "common".to_owned(),
        external: true,
        defined: true,
        section_ordinal: None,
        value: 8,
        common_alignment: Some(8),
        indirect_target: None,
    }
}
