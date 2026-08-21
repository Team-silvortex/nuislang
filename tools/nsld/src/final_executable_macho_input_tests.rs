use super::*;

#[test]
fn parses_sections_symbols_and_external_relocations() {
    let parsed = parse_macho_arm64_object_linkage(&sample_object()).unwrap();

    assert_eq!(parsed.section_count, 1);
    assert_eq!(parsed.symbol_count, 2);
    assert_eq!(parsed.relocation_count, 1);
    assert_eq!(parsed.defined_symbol_count, 1);
    assert_eq!(parsed.undefined_symbol_count, 1);
    assert_eq!(
        parsed.external_definitions,
        BTreeSet::from(["_defined".to_owned()])
    );
    assert_eq!(
        parsed.external_undefined,
        BTreeSet::from(["_missing".to_owned()])
    );
    assert_eq!(parsed.sections[0].ordinal, 1);
    assert_eq!(parsed.sections[0].segment_name, "__TEXT");
    assert_eq!(parsed.sections[0].name, "__text");
    assert_eq!(parsed.sections[0].size, 8);
    assert_eq!(parsed.sections[0].alignment, 1);
    assert_eq!(parsed.symbols[0].name, "_defined");
    assert_eq!(parsed.symbols[0].kind, "section");
    assert_eq!(parsed.symbols[0].section_ordinal, Some(1));
    assert_eq!(parsed.symbols[1].name, "_missing");
    assert_eq!(parsed.symbols[1].kind, "undefined");
    assert_eq!(parsed.relocations[0].section_ordinal, 1);
    assert_eq!(parsed.relocations[0].symbol_number, 1);
    assert_eq!(parsed.relocations[0].width_bytes, 4);
    assert!(parsed.relocations[0].pc_relative);
    assert!(parsed.relocations[0].external);
    assert_eq!(parsed.relocations[0].relocation_type, 2);
}

#[test]
fn rejects_executable_at_relocatable_object_boundary() {
    let mut bytes = sample_object();
    write_u32(&mut bytes, 12, 2);

    assert!(parse_macho_arm64_object_linkage(&bytes)
        .unwrap_err()
        .contains("expected MH_OBJECT"));
}

#[test]
fn rejects_relocation_symbol_index_outside_symbol_table() {
    let mut bytes = sample_object();
    let relocation_word = 7 | (1 << 24) | (2 << 25) | (1 << 27) | (2 << 28);
    write_u32(&mut bytes, 220, relocation_word);

    let error = parse_macho_arm64_object_linkage(&bytes).unwrap_err();
    assert!(error.contains("missing symbol index 7"), "{error}");
}

#[test]
fn rejects_relocation_table_outside_object() {
    let mut bytes = sample_object();
    let object_size = bytes.len() as u32;
    write_u32(&mut bytes, 160, object_size);

    let error = parse_macho_arm64_object_linkage(&bytes).unwrap_err();
    assert!(
        error.contains("relocation table exceeds object boundary"),
        "{error}"
    );
}

#[test]
fn accepts_arm64_addend_payload_in_symbol_number_bits() {
    let mut bytes = sample_object();
    let relocation_word = 40 | (2 << 25) | (ARM64_RELOCATION_ADDEND << 28);
    write_u32(&mut bytes, 220, relocation_word);

    assert_eq!(
        parse_macho_arm64_object_linkage(&bytes)
            .unwrap()
            .relocation_count,
        1
    );
}

#[test]
fn parses_common_symbol_size_and_alignment_as_a_tentative_definition() {
    let mut bytes = sample_object();
    let common_offset = 224 + NLIST_64_SIZE;
    bytes[common_offset + 6..common_offset + 8].copy_from_slice(&(3u16 << 8).to_le_bytes());
    write_u64(&mut bytes, common_offset + 8, 16);

    let parsed = parse_macho_arm64_object_linkage(&bytes).unwrap();
    let common = &parsed.symbols[1];

    assert_eq!(common.name, "_missing");
    assert_eq!(common.kind, "common");
    assert!(common.defined);
    assert_eq!(common.section_ordinal, None);
    assert_eq!(common.value, 16);
    assert_eq!(common.common_alignment, Some(8));
    assert_eq!(parsed.defined_symbol_count, 2);
    assert_eq!(parsed.undefined_symbol_count, 0);
    assert!(parsed.external_definitions.contains("_missing"));
    assert!(parsed.external_undefined.is_empty());
}

#[test]
fn rejects_non_terminated_symbol_name() {
    let mut bytes = sample_object();
    *bytes.last_mut().unwrap() = b'x';

    let error = parse_macho_arm64_object_linkage(&bytes).unwrap_err();
    assert!(error.contains("name is not NUL-terminated"), "{error}");
}

#[test]
fn requires_symbol_table_command() {
    let mut bytes = sample_object();
    write_u32(&mut bytes, 184, 0x0b);

    assert!(parse_macho_arm64_object_linkage(&bytes)
        .unwrap_err()
        .contains("no LC_SYMTAB"));
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn parses_real_nuisc_program_and_runtime_objects() {
    let dir = unique_temp_dir("nsld-real-macho-input");
    let input = dir.join("main.ns");
    let output = dir.join("out");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&input, "mod cpu Main { fn main() -> i64 { return 0; } }\n").unwrap();

    nuisc::run(nuisc::cli::CommandKind::Compile {
        input: input.clone(),
        output_dir: output.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: Some("native-cpu-llvm".to_owned()),
    })
    .unwrap();
    let artifact =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    let parsed = artifact
        .host_objects
        .iter()
        .map(|object| parse_macho_arm64_object_linkage(&object.bytes).unwrap())
        .collect::<Vec<_>>();
    let section_count = parsed.iter().map(|item| item.section_count).sum::<usize>();
    let symbol_count = parsed.iter().map(|item| item.symbol_count).sum::<usize>();
    let relocation_count = parsed
        .iter()
        .map(|item| item.relocation_count)
        .sum::<usize>();
    let definitions = parsed
        .iter()
        .flat_map(|item| item.external_definitions.iter().cloned())
        .collect::<BTreeSet<_>>();
    let undefined = parsed
        .iter()
        .flat_map(|item| item.external_undefined.iter().cloned())
        .collect::<BTreeSet<_>>();

    assert_eq!(artifact.host_objects.len(), 2);
    assert!(section_count >= 8, "section_count={section_count}");
    assert!(symbol_count >= 100, "symbol_count={symbol_count}");
    assert!(
        relocation_count >= 100,
        "relocation_count={relocation_count}"
    );
    assert!(definitions.contains("_nuis_yir_entry"));
    assert!(undefined.contains("_nuis_yir_entry"));
    assert!(undefined.iter().any(|symbol| symbol == "_malloc"));

    let cache_key = nuisc::cache::compute_compile_cache_key(&input, None).unwrap();
    let _ = std::fs::remove_dir_all(cache_key.root.join(cache_key.key));
    std::fs::remove_dir_all(dir).unwrap();
}

fn sample_object() -> Vec<u8> {
    const SEGMENT_OFFSET: usize = 32;
    const SECTION_OFFSET: usize = 104;
    const SYMTAB_OFFSET: usize = 184;
    const PAYLOAD_OFFSET: usize = 208;
    const RELOCATION_OFFSET: usize = 216;
    const SYMBOL_OFFSET: usize = 224;
    const STRING_OFFSET: usize = 256;
    const STRINGS: &[u8] = b"\0_defined\0_missing\0";

    let mut bytes = vec![0u8; STRING_OFFSET + STRINGS.len()];
    bytes[..4].copy_from_slice(&MACH_O_64_LE_MAGIC);
    write_u32(&mut bytes, 4, MACH_O_CPU_TYPE_ARM64);
    write_u32(&mut bytes, 12, MACH_O_FILE_TYPE_OBJECT);
    write_u32(&mut bytes, 16, 2);
    write_u32(&mut bytes, 20, 176);

    write_u32(&mut bytes, SEGMENT_OFFSET, LC_SEGMENT_64);
    write_u32(&mut bytes, SEGMENT_OFFSET + 4, 152);
    write_u32(&mut bytes, SEGMENT_OFFSET + 64, 1);
    bytes[SECTION_OFFSET..SECTION_OFFSET + 6].copy_from_slice(b"__text");
    bytes[SECTION_OFFSET + 16..SECTION_OFFSET + 22].copy_from_slice(b"__TEXT");
    write_u64(&mut bytes, SECTION_OFFSET + 40, 8);
    write_u32(&mut bytes, SECTION_OFFSET + 48, PAYLOAD_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 56, RELOCATION_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 60, 1);

    write_u32(&mut bytes, SYMTAB_OFFSET, LC_SYMTAB);
    write_u32(&mut bytes, SYMTAB_OFFSET + 4, SYMTAB_COMMAND_SIZE as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 8, SYMBOL_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 12, 2);
    write_u32(&mut bytes, SYMTAB_OFFSET + 16, STRING_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 20, STRINGS.len() as u32);

    write_u32(&mut bytes, RELOCATION_OFFSET, 0);
    let relocation_word = 1 | (1 << 24) | (2 << 25) | (1 << 27) | (2 << 28);
    write_u32(&mut bytes, RELOCATION_OFFSET + 4, relocation_word);

    write_u32(&mut bytes, SYMBOL_OFFSET, 1);
    bytes[SYMBOL_OFFSET + 4] = N_SECT | N_EXT;
    bytes[SYMBOL_OFFSET + 5] = 1;
    write_u32(&mut bytes, SYMBOL_OFFSET + NLIST_64_SIZE, 10);
    bytes[SYMBOL_OFFSET + NLIST_64_SIZE + 4] = N_UNDF | N_EXT;
    bytes[STRING_OFFSET..].copy_from_slice(STRINGS);
    bytes
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
