use super::reports::{NsldObjectFileLayoutRecordDiagnostic, NsldObjectFileLayoutReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOLoadCommandsPlan {
    pub(crate) command_size: usize,
    pub(crate) section_count: usize,
    pub(crate) symbol_count: usize,
    pub(crate) string_table_offset: usize,
    pub(crate) string_table_size: usize,
    pub(crate) records: Vec<NsldMachOSectionCommandPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOSectionCommandPlan {
    pub(crate) section_name: String,
    pub(crate) file_offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) alignment_power: u32,
}

pub(crate) fn mach_o_arm64_load_commands_plan(
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<NsldMachOLoadCommandsPlan> {
    if file_layout.writer_backend_kind != "mach-o-arm64" {
        return None;
    }
    let load_commands = record_by_kind(file_layout, "macho-load-commands")?;
    let symbols = record_by_kind(file_layout, "macho-symbol-table")?;
    let strings = record_by_kind(file_layout, "macho-string-table")?;
    let records = file_layout
        .records
        .iter()
        .filter(|record| record.record_kind == "section-payload")
        .map(|record| NsldMachOSectionCommandPlan {
            section_name: mach_o_section_name(record),
            file_offset: record.file_offset,
            size_bytes: record.size_bytes,
            alignment_power: alignment_power(record.alignment),
        })
        .collect::<Vec<_>>();

    Some(NsldMachOLoadCommandsPlan {
        command_size: load_commands.size_bytes,
        section_count: records.len(),
        symbol_count: symbols.size_bytes / 16,
        string_table_offset: strings.file_offset,
        string_table_size: strings.size_bytes,
        records,
    })
}

pub(crate) fn encode_mach_o_load_commands(plan: &NsldMachOLoadCommandsPlan) -> Vec<u8> {
    let mut bytes = vec![0u8; plan.command_size];
    let segment_size = 72 + plan.section_count * 80;
    write_u32_le(&mut bytes, 0, 0x19);
    write_u32_le(&mut bytes, 4, segment_size as u32);
    write_u64_le(&mut bytes, 24, 0);
    write_u64_le(&mut bytes, 32, total_section_size(plan) as u64);
    write_u64_le(&mut bytes, 40, first_section_offset(plan) as u64);
    write_u64_le(&mut bytes, 48, total_section_size(plan) as u64);
    write_u32_le(&mut bytes, 56, 7);
    write_u32_le(&mut bytes, 60, 7);
    write_u32_le(&mut bytes, 64, plan.section_count as u32);
    write_u32_le(&mut bytes, 68, 0);

    let mut offset = 72;
    for section in &plan.records {
        write_fixed_str(&mut bytes, offset, 16, &section.section_name);
        write_u64_le(&mut bytes, offset + 32, 0);
        write_u64_le(&mut bytes, offset + 40, section.size_bytes as u64);
        write_u32_le(&mut bytes, offset + 48, section.file_offset as u32);
        write_u32_le(&mut bytes, offset + 52, section.alignment_power);
        write_u32_le(&mut bytes, offset + 56, 0);
        write_u32_le(&mut bytes, offset + 60, 0);
        write_u32_le(&mut bytes, offset + 64, 0);
        write_u32_le(&mut bytes, offset + 68, 0);
        write_u32_le(&mut bytes, offset + 72, 0);
        write_u32_le(&mut bytes, offset + 76, 0);
        offset += 80;
    }

    write_u32_le(&mut bytes, offset, 0x2);
    write_u32_le(&mut bytes, offset + 4, 24);
    write_u32_le(&mut bytes, offset + 8, 0);
    write_u32_le(&mut bytes, offset + 12, plan.symbol_count as u32);
    write_u32_le(&mut bytes, offset + 16, plan.string_table_offset as u32);
    write_u32_le(&mut bytes, offset + 20, plan.string_table_size as u32);
    bytes
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

fn mach_o_section_name(record: &NsldObjectFileLayoutRecordDiagnostic) -> String {
    let raw = record
        .record_id
        .strip_prefix("section.")
        .unwrap_or(&record.record_id);
    let mut name = raw.replace('.', "_");
    if !name.starts_with("__") {
        name = format!("__{name}");
    }
    name.chars().take(16).collect()
}

fn alignment_power(alignment: usize) -> u32 {
    if alignment <= 1 {
        return 0;
    }
    alignment.next_power_of_two().trailing_zeros()
}

fn first_section_offset(plan: &NsldMachOLoadCommandsPlan) -> usize {
    plan.records
        .iter()
        .map(|record| record.file_offset)
        .min()
        .unwrap_or(0)
}

fn total_section_size(plan: &NsldMachOLoadCommandsPlan) -> usize {
    let start = first_section_offset(plan);
    let end = plan
        .records
        .iter()
        .map(|record| record.file_offset.saturating_add(record.size_bytes))
        .max()
        .unwrap_or(start);
    end.saturating_sub(start)
}

fn write_fixed_str(bytes: &mut [u8], offset: usize, len: usize, value: &str) {
    let raw = value.as_bytes();
    let copy_len = raw.len().min(len);
    bytes[offset..offset + copy_len].copy_from_slice(&raw[..copy_len]);
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64_le(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::{encode_mach_o_load_commands, mach_o_arm64_load_commands_plan};
    use crate::{
        main_test_support::empty_link_plan, object_file_layout::nsld_object_file_layout_report,
    };
    use std::path::Path;

    #[test]
    fn plans_and_encodes_mach_o_load_commands() {
        let plan = empty_link_plan();
        let file_layout = nsld_object_file_layout_report(Path::new("manifest.toml"), &plan);
        let commands = mach_o_arm64_load_commands_plan(&file_layout).unwrap();
        let bytes = encode_mach_o_load_commands(&commands);

        assert_eq!(commands.section_count, 4);
        assert_eq!(commands.command_size, 416);
        assert_eq!(commands.symbol_count, 5);
        assert_eq!(bytes.len(), commands.command_size);
        assert_eq!(&bytes[0..4], &[0x19, 0x00, 0x00, 0x00]);
        assert_eq!(&bytes[4..8], &[0x88, 0x01, 0x00, 0x00]);
        assert_eq!(&bytes[72..74], b"__");
        assert_eq!(&bytes[392..396], &[0x02, 0x00, 0x00, 0x00]);
        assert_eq!(&bytes[396..400], &[0x18, 0x00, 0x00, 0x00]);
    }
}
