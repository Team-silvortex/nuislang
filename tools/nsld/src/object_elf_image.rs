use super::{
    object_plan::nsld_object_plan_report,
    reports::{
        NsldObjectFileLayoutRecordDiagnostic, NsldObjectFileLayoutReport,
        NsldObjectImageRelocationRecordDiagnostic, NsldObjectRelocationSeedDiagnostic,
        NsldRelocationLoweringRuleDiagnostic,
    },
};
use std::{collections::BTreeMap, fs, path::Path};

const ELF_HEADER_SIZE: usize = 64;
const ELF_SECTION_HEADER_SIZE: usize = 64;
const ELF_SYMBOL_SIZE: usize = 24;
const ELF_RELOCATION_SIZE: usize = 24;
const R_X86_64_64: u32 = 1;

pub(crate) fn encode_elf_amd64_image(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<Vec<u8>> {
    if file_layout.writer_backend_kind != "elf-amd64" {
        return None;
    }
    let object_plan = nsld_object_plan_report(manifest, plan);
    let mut bytes = vec![0u8; file_layout.total_file_size_bytes];
    let sections = elf_sections(file_layout, &object_plan)?;
    let section_names = string_table(sections.iter().map(|section| section.name.as_str()));
    let symbol_names = string_table(["__nuis_entry"].into_iter());
    let symbol_table_index = section_index_by_kind(&sections, "elf-symbol-table")?;
    let string_table_index = section_index_by_kind(&sections, "elf-string-table")?;
    let section_name_table_index = section_index_by_kind(&sections, "elf-section-name-table")?;
    let section_headers = record_by_kind(file_layout, "elf-section-header-table")?;

    encode_header(
        &mut bytes,
        section_headers.file_offset,
        sections.len() + 1,
        section_name_table_index,
    )?;
    write_section_payloads(&mut bytes, &object_plan, file_layout)?;
    write_named_record(
        &mut bytes,
        file_layout,
        "elf-string-table",
        &symbol_names.bytes,
    )?;
    write_named_record(
        &mut bytes,
        file_layout,
        "elf-section-name-table",
        &section_names.bytes,
    )?;
    write_symbols(&mut bytes, file_layout, &sections, &symbol_names)?;
    write_relocations(
        &mut bytes,
        file_layout,
        &sections,
        &object_plan.relocation_seeds,
    )?;
    write_section_headers(
        &mut bytes,
        section_headers,
        &sections,
        &section_names,
        symbol_table_index,
        string_table_index,
    )?;
    Some(bytes)
}

pub(crate) fn elf_amd64_relocation_lowering_rules() -> Vec<NsldRelocationLoweringRuleDiagnostic> {
    [
        "bootstrap-entry-seed",
        "metadata-address-seed",
        "data-address-seed",
        "extension-address-seed",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, seed)| NsldRelocationLoweringRuleDiagnostic {
        rule_id: format!("elf-amd64-reloc-rule-{index:04}"),
        source_seed_kind: seed.to_owned(),
        target_relocation_kind: "x86-64-absolute64".to_owned(),
        pc_relative: false,
        length_power: 3,
        external: false,
        relocation_type: R_X86_64_64 as u8,
    })
    .collect()
}

pub(crate) fn elf_amd64_relocation_records(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Vec<NsldObjectImageRelocationRecordDiagnostic> {
    let object_plan = nsld_object_plan_report(manifest, plan);
    object_plan
        .relocation_seeds
        .iter()
        .enumerate()
        .map(|(index, seed)| NsldObjectImageRelocationRecordDiagnostic {
            record_id: format!("elf-amd64-reloc-record-{index:04}"),
            relocation_seed_id: seed.relocation_seed_id.clone(),
            source_section_id: seed.source_section_id.clone(),
            source_offset: seed.source_offset_seed,
            source_seed_kind: seed.relocation_seed_kind.clone(),
            target_relocation_kind: "x86-64-absolute64".to_owned(),
            symbol_index: payload_symbol_index(file_layout, &seed.source_section_id).unwrap_or(0),
            pc_relative: false,
            length_power: 3,
            external: false,
            relocation_type: R_X86_64_64 as u8,
        })
        .collect()
}

pub(crate) fn elf_amd64_relocation_resolution_issues(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Vec<String> {
    let object_plan = nsld_object_plan_report(manifest, plan);
    object_plan
        .relocation_seeds
        .iter()
        .filter_map(|seed| {
            if !registered_seed_kind(&seed.relocation_seed_kind) {
                Some(format!(
                    "elf-relocation:{}:unsupported-seed-kind:{}",
                    seed.relocation_seed_id, seed.relocation_seed_kind
                ))
            } else if payload_symbol_index(file_layout, &seed.source_section_id).is_none() {
                Some(format!(
                    "elf-relocation:{}:unresolved-section-symbol:{}",
                    seed.relocation_seed_id, seed.source_section_id
                ))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Clone)]
struct ElfSection<'a> {
    name: String,
    record: &'a NsldObjectFileLayoutRecordDiagnostic,
    target_section_index: usize,
}

fn elf_sections<'a>(
    file_layout: &'a NsldObjectFileLayoutReport,
    object_plan: &super::reports::NsldObjectPlanReport,
) -> Option<Vec<ElfSection<'a>>> {
    let payload_names = object_plan
        .object_sections
        .iter()
        .map(|section| {
            (
                section.source_section_id.as_str(),
                section.object_section_name.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let payload_indices = file_layout
        .records
        .iter()
        .filter(|record| record.record_kind == "section-payload")
        .enumerate()
        .map(|(index, record)| (source_section_id(record), index + 1))
        .collect::<BTreeMap<_, _>>();
    file_layout
        .records
        .iter()
        .filter(|record| {
            !matches!(
                record.record_kind.as_str(),
                "elf-header" | "elf-section-header-table"
            )
        })
        .map(|record| {
            let (name, target_section_index) = match record.record_kind.as_str() {
                "section-payload" => (payload_names.get(source_section_id(record))?.to_string(), 0),
                "elf-relocation-table" => {
                    let source = record.record_id.strip_prefix("elf.relocations.")?;
                    (
                        format!(".rela{}", payload_names.get(source)?),
                        *payload_indices.get(source)?,
                    )
                }
                "elf-symbol-table" => (".symtab".to_owned(), 0),
                "elf-string-table" => (".strtab".to_owned(), 0),
                "elf-section-name-table" => (".shstrtab".to_owned(), 0),
                _ => return None,
            };
            Some(ElfSection {
                name,
                record,
                target_section_index,
            })
        })
        .collect()
}

struct EncodedStringTable {
    bytes: Vec<u8>,
    offsets: BTreeMap<String, u32>,
}

fn string_table<'a>(names: impl IntoIterator<Item = &'a str>) -> EncodedStringTable {
    let mut bytes = vec![0u8];
    let mut offsets = BTreeMap::new();
    for name in names {
        offsets.entry(name.to_owned()).or_insert_with(|| {
            let offset = bytes.len() as u32;
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            offset
        });
    }
    EncodedStringTable { bytes, offsets }
}

fn encode_header(
    image: &mut [u8],
    section_header_offset: usize,
    section_count: usize,
    section_name_table_index: usize,
) -> Option<()> {
    if image.len() < ELF_HEADER_SIZE {
        return None;
    }
    image[0..16].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    write_u16(image, 16, 1)?;
    write_u16(image, 18, 62)?;
    write_u32(image, 20, 1)?;
    write_u64(image, 40, section_header_offset as u64)?;
    write_u16(image, 52, ELF_HEADER_SIZE as u16)?;
    write_u16(image, 58, ELF_SECTION_HEADER_SIZE as u16)?;
    write_u16(image, 60, section_count as u16)?;
    write_u16(image, 62, section_name_table_index as u16)?;
    Some(())
}

fn write_section_payloads(
    image: &mut [u8],
    object_plan: &super::reports::NsldObjectPlanReport,
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<()> {
    for record in file_layout
        .records
        .iter()
        .filter(|record| record.record_kind == "section-payload")
    {
        let section = object_plan
            .object_sections
            .iter()
            .find(|section| section.source_section_id == source_section_id(record))?;
        let Ok(payload) = fs::read(&section.source_path) else {
            continue;
        };
        write_record(image, record, &payload)?;
    }
    Some(())
}

fn write_symbols(
    image: &mut [u8],
    file_layout: &NsldObjectFileLayoutReport,
    sections: &[ElfSection<'_>],
    strings: &EncodedStringTable,
) -> Option<()> {
    let record = record_by_kind(file_layout, "elf-symbol-table")?;
    let payload_sections = sections
        .iter()
        .enumerate()
        .filter(|(_, section)| section.record.record_kind == "section-payload")
        .collect::<Vec<_>>();
    let mut table = vec![0u8; record.size_bytes];
    for (symbol_offset, (section_index, _)) in payload_sections.iter().enumerate() {
        let offset = (symbol_offset + 1) * ELF_SYMBOL_SIZE;
        table[offset + 4] = 3;
        write_u16(&mut table, offset + 6, (*section_index + 1) as u16)?;
    }
    let global_offset = (payload_sections.len() + 1) * ELF_SYMBOL_SIZE;
    write_u32(
        &mut table,
        global_offset,
        *strings.offsets.get("__nuis_entry")?,
    )?;
    table[global_offset + 4] = 0x10;
    write_u16(&mut table, global_offset + 6, 1)?;
    write_record(image, record, &table)
}

fn write_relocations(
    image: &mut [u8],
    file_layout: &NsldObjectFileLayoutReport,
    sections: &[ElfSection<'_>],
    seeds: &[NsldObjectRelocationSeedDiagnostic],
) -> Option<()> {
    for section in sections
        .iter()
        .filter(|section| section.record.record_kind == "elf-relocation-table")
    {
        let source = section.record.record_id.strip_prefix("elf.relocations.")?;
        let source_seeds = seeds
            .iter()
            .filter(|seed| seed.source_section_id == source)
            .collect::<Vec<_>>();
        let mut table = vec![0u8; section.record.size_bytes];
        let symbol_index = payload_symbol_index(file_layout, source)?;
        for (index, seed) in source_seeds.iter().enumerate() {
            let offset = index * ELF_RELOCATION_SIZE;
            write_u64(&mut table, offset, seed.source_offset_seed as u64)?;
            write_u64(
                &mut table,
                offset + 8,
                (u64::from(symbol_index) << 32) | u64::from(R_X86_64_64),
            )?;
            write_i64(&mut table, offset + 16, seed.addend as i64)?;
        }
        write_record(image, section.record, &table)?;
    }
    Some(())
}

fn write_section_headers(
    image: &mut [u8],
    record: &NsldObjectFileLayoutRecordDiagnostic,
    sections: &[ElfSection<'_>],
    names: &EncodedStringTable,
    symbol_table_index: usize,
    string_table_index: usize,
) -> Option<()> {
    let mut table = vec![0u8; record.size_bytes];
    for (index, section) in sections.iter().enumerate() {
        let offset = (index + 1) * ELF_SECTION_HEADER_SIZE;
        write_u32(&mut table, offset, *names.offsets.get(&section.name)?)?;
        let (kind, link, info, entry_size) = match section.record.record_kind.as_str() {
            "section-payload" => (1, 0, 0, 0),
            "elf-relocation-table" => (
                4,
                symbol_table_index as u32,
                section.target_section_index as u32,
                ELF_RELOCATION_SIZE as u64,
            ),
            "elf-symbol-table" => (
                2,
                string_table_index as u32,
                payload_section_count(sections) as u32 + 1,
                ELF_SYMBOL_SIZE as u64,
            ),
            "elf-string-table" | "elf-section-name-table" => (3, 0, 0, 0),
            _ => return None,
        };
        write_u32(&mut table, offset + 4, kind)?;
        write_u64(&mut table, offset + 24, section.record.file_offset as u64)?;
        write_u64(&mut table, offset + 32, section.record.size_bytes as u64)?;
        write_u32(&mut table, offset + 40, link)?;
        write_u32(&mut table, offset + 44, info)?;
        write_u64(
            &mut table,
            offset + 48,
            section.record.alignment.max(1) as u64,
        )?;
        write_u64(&mut table, offset + 56, entry_size)?;
    }
    write_record(image, record, &table)
}

fn payload_section_count(sections: &[ElfSection<'_>]) -> usize {
    sections
        .iter()
        .filter(|section| section.record.record_kind == "section-payload")
        .count()
}

fn section_index_by_kind(sections: &[ElfSection<'_>], kind: &str) -> Option<usize> {
    sections
        .iter()
        .position(|section| section.record.record_kind == kind)
        .map(|index| index + 1)
}

fn payload_symbol_index(
    file_layout: &NsldObjectFileLayoutReport,
    requested_section_id: &str,
) -> Option<u32> {
    file_layout
        .records
        .iter()
        .filter(|record| record.record_kind == "section-payload")
        .position(|record| source_section_id(record) == requested_section_id)
        .map(|index| (index + 1) as u32)
}

fn registered_seed_kind(seed: &str) -> bool {
    matches!(
        seed,
        "bootstrap-entry-seed"
            | "metadata-address-seed"
            | "data-address-seed"
            | "extension-address-seed"
    )
}

fn source_section_id(record: &NsldObjectFileLayoutRecordDiagnostic) -> &str {
    record
        .record_id
        .strip_prefix("section.")
        .unwrap_or(&record.record_id)
}

fn record_by_kind<'a>(
    file_layout: &'a NsldObjectFileLayoutReport,
    kind: &str,
) -> Option<&'a NsldObjectFileLayoutRecordDiagnostic> {
    file_layout
        .records
        .iter()
        .find(|record| record.record_kind == kind)
}

fn write_named_record(
    image: &mut [u8],
    file_layout: &NsldObjectFileLayoutReport,
    kind: &str,
    payload: &[u8],
) -> Option<()> {
    write_record(image, record_by_kind(file_layout, kind)?, payload)
}

fn write_record(
    image: &mut [u8],
    record: &NsldObjectFileLayoutRecordDiagnostic,
    payload: &[u8],
) -> Option<()> {
    if payload.len() > record.size_bytes {
        return None;
    }
    let end = record.file_offset.checked_add(payload.len())?;
    image
        .get_mut(record.file_offset..end)?
        .copy_from_slice(payload);
    Some(())
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) -> Option<()> {
    bytes
        .get_mut(offset..offset + 2)?
        .copy_from_slice(&value.to_le_bytes());
    Some(())
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Option<()> {
    bytes
        .get_mut(offset..offset + 4)?
        .copy_from_slice(&value.to_le_bytes());
    Some(())
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) -> Option<()> {
    bytes
        .get_mut(offset..offset + 8)?
        .copy_from_slice(&value.to_le_bytes());
    Some(())
}

fn write_i64(bytes: &mut [u8], offset: usize, value: i64) -> Option<()> {
    bytes
        .get_mut(offset..offset + 8)?
        .copy_from_slice(&value.to_le_bytes());
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        main_test_support::empty_link_plan, object_file_layout::nsld_object_file_layout_report,
    };

    #[test]
    fn encodes_elf64_amd64_sections_symbols_and_relocations() {
        let mut plan = empty_link_plan();
        plan.cpu_target.machine_arch = "x86_64".to_owned();
        plan.cpu_target.machine_os = "linux".to_owned();
        plan.cpu_target.object_format = "elf".to_owned();
        let layout = nsld_object_file_layout_report(Path::new("manifest.toml"), &plan);
        let image = encode_elf_amd64_image(Path::new("manifest.toml"), &plan, &layout).unwrap();

        assert_eq!(&image[..4], b"\x7fELF");
        assert_eq!(u16::from_le_bytes(image[18..20].try_into().unwrap()), 62);
        assert_eq!(
            u16::from_le_bytes(image[60..62].try_into().unwrap()) as usize,
            layout
                .records
                .iter()
                .filter(|record| {
                    !matches!(
                        record.record_kind.as_str(),
                        "elf-header" | "elf-section-header-table"
                    )
                })
                .count()
                + 1
        );
        assert!(image
            .windows("__nuis_entry".len())
            .any(|window| window == b"__nuis_entry"));
        assert_eq!(
            elf_amd64_relocation_records(Path::new("manifest.toml"), &plan, &layout).len(),
            4
        );
    }
}
