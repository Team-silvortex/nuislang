use super::reports::NsldObjectFileLayoutReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOHeaderPlan {
    pub(crate) magic: u32,
    pub(crate) cpu_type: u32,
    pub(crate) cpu_subtype: u32,
    pub(crate) file_type: u32,
    pub(crate) load_command_count: u32,
    pub(crate) load_command_size: u32,
    pub(crate) flags: u32,
    pub(crate) reserved: u32,
}

pub(crate) fn mach_o_arm64_header_plan(
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<NsldMachOHeaderPlan> {
    (file_layout.backend_kind == "mach-o-arm64").then(|| NsldMachOHeaderPlan {
        magic: 0xfeedfacf,
        cpu_type: 0x0100000c,
        cpu_subtype: 0,
        file_type: 1,
        load_command_count: 2,
        load_command_size: mach_o_load_command_size(file_layout),
        flags: 0,
        reserved: 0,
    })
}

pub(crate) fn encode_mach_o_header(header: &NsldMachOHeaderPlan) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    write_u32_le(&mut bytes, 0, header.magic);
    write_u32_le(&mut bytes, 4, header.cpu_type);
    write_u32_le(&mut bytes, 8, header.cpu_subtype);
    write_u32_le(&mut bytes, 12, header.file_type);
    write_u32_le(&mut bytes, 16, header.load_command_count);
    write_u32_le(&mut bytes, 20, header.load_command_size);
    write_u32_le(&mut bytes, 24, header.flags);
    write_u32_le(&mut bytes, 28, header.reserved);
    bytes
}

fn mach_o_load_command_size(file_layout: &NsldObjectFileLayoutReport) -> u32 {
    file_layout
        .records
        .iter()
        .find(|record| record.record_kind == "macho-load-commands")
        .map(|record| record.size_bytes as u32)
        .unwrap_or(0)
}

fn write_u32_le(bytes: &mut [u8; 32], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::{encode_mach_o_header, mach_o_arm64_header_plan};
    use crate::{
        main_test_support::empty_link_plan, object_file_layout::nsld_object_file_layout_report,
    };
    use std::path::Path;

    #[test]
    fn plans_and_encodes_minimal_mach_o_arm64_header() {
        let plan = empty_link_plan();
        let file_layout = nsld_object_file_layout_report(Path::new("manifest.toml"), &plan);
        let header = mach_o_arm64_header_plan(&file_layout).unwrap();
        let bytes = encode_mach_o_header(&header);

        assert_eq!(header.magic, 0xfeedfacf);
        assert_eq!(header.cpu_type, 0x0100000c);
        assert_eq!(header.file_type, 1);
        assert_eq!(header.load_command_count, 2);
        assert_eq!(bytes.len(), 32);
        assert_eq!(&bytes[0..4], &[0xcf, 0xfa, 0xed, 0xfe]);
        assert_eq!(&bytes[4..8], &[0x0c, 0x00, 0x00, 0x01]);
    }
}
