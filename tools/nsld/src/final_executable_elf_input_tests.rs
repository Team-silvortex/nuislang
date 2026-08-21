use super::*;
use crate::final_executable_elf_test_fixture::{
    elf_program_object, elf_runtime_object, R_X86_64_PLT32,
};

#[test]
fn parses_sections_symbols_and_registered_rela() {
    let parsed = parse_elf64_amd64_object_linkage(&elf_program_object(R_X86_64_PLT32))
        .expect("program object should parse");

    assert_eq!(parsed.section_count, 5);
    assert_eq!(parsed.symbol_count, 3);
    assert_eq!(parsed.defined_symbol_count, 1);
    assert_eq!(parsed.undefined_symbol_count, 1);
    assert_eq!(parsed.relocation_count, 1);
    assert!(parsed.external_definitions.contains("__nuis_entry"));
    assert!(parsed.external_undefined.contains("nuis_runtime_entry"));
    assert_eq!(parsed.sections[0].name, ".text");
    assert_eq!(parsed.sections[0].flags, 0x6);
    assert_eq!(parsed.symbols[1].section_index, Some(1));
    assert_eq!(parsed.relocations[0].target_section_index, 1);
    assert_eq!(parsed.relocations[0].symbol_index, 2);
    assert_eq!(parsed.relocations[0].relocation_type, R_X86_64_PLT32);
    assert_eq!(parsed.relocations[0].width_bytes, 4);
    assert!(parsed.relocations[0].pc_relative);
}

#[test]
fn parses_definition_only_runtime_object() {
    let parsed = parse_elf64_amd64_object_linkage(&elf_runtime_object()).unwrap();

    assert_eq!(parsed.section_count, 4);
    assert_eq!(parsed.symbol_count, 2);
    assert_eq!(parsed.relocation_count, 0);
    assert!(parsed.external_definitions.contains("nuis_runtime_entry"));
    assert!(parsed.external_undefined.is_empty());
}

#[test]
fn rejects_unregistered_relocation_type() {
    let error = parse_elf64_amd64_object_linkage(&elf_program_object(0x7fff)).unwrap_err();

    assert!(error.contains("unsupported R_X86_64 type 32767"));
}

#[test]
fn registered_relocation_shapes_are_explicit() {
    assert_eq!(relocation_shape(R_X86_64_NONE), Some((0, false)));
    assert_eq!(relocation_shape(R_X86_64_64), Some((8, false)));
    assert_eq!(relocation_shape(R_X86_64_PC32), Some((4, true)));
    assert_eq!(relocation_shape(R_X86_64_PLT32), Some((4, true)));
    assert_eq!(relocation_shape(R_X86_64_32), Some((4, false)));
    assert_eq!(relocation_shape(R_X86_64_32S), Some((4, false)));
}

#[test]
fn rejects_truncated_section_table() {
    let mut bytes = elf_runtime_object();
    let invalid_offset = bytes.len() as u64 - 8;
    bytes[40..48].copy_from_slice(&invalid_offset.to_le_bytes());

    let error = parse_elf64_amd64_object_linkage(&bytes).unwrap_err();

    assert!(error.contains("ELF section table exceeds image bounds"));
}
