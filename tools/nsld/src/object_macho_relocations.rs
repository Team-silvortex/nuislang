use super::{
    object_plan::nsld_object_plan_report,
    reports::{NsldObjectFileLayoutReport, NsldObjectRelocationSeedDiagnostic},
};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachORelocationTablePlan {
    pub(crate) relocation_count: usize,
    pub(crate) table_size: usize,
    pub(crate) relocations: Vec<NsldMachORelocationPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachORelocationPlan {
    pub(crate) address: u32,
    pub(crate) symbol_index: u32,
    pub(crate) pc_relative: bool,
    pub(crate) length_power: u8,
    pub(crate) external: bool,
    pub(crate) relocation_type: u8,
}

pub(crate) fn mach_o_arm64_relocation_table_plan(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<NsldMachORelocationTablePlan> {
    if file_layout.writer_backend_kind != "mach-o-arm64" {
        return None;
    }
    let object_plan = nsld_object_plan_report(manifest, plan);
    let expected_size = file_layout
        .records
        .iter()
        .find(|record| record.record_kind == "macho-relocation-table")?
        .size_bytes;
    let relocations = object_plan
        .relocation_seeds
        .iter()
        .enumerate()
        .map(|(index, seed)| relocation_plan(index, seed))
        .collect::<Vec<_>>();

    Some(NsldMachORelocationTablePlan {
        relocation_count: relocations.len(),
        table_size: expected_size,
        relocations,
    })
}

pub(crate) fn encode_mach_o_relocations(plan: &NsldMachORelocationTablePlan) -> Vec<u8> {
    let mut bytes = vec![0u8; plan.table_size];
    for (index, relocation) in plan.relocations.iter().enumerate() {
        let offset = index * 8;
        write_u32_le(&mut bytes, offset, relocation.address);
        write_u32_le(&mut bytes, offset + 4, relocation_word(relocation));
    }
    bytes
}

fn relocation_plan(
    index: usize,
    seed: &NsldObjectRelocationSeedDiagnostic,
) -> NsldMachORelocationPlan {
    NsldMachORelocationPlan {
        address: seed.source_offset_seed as u32,
        symbol_index: index.saturating_add(1) as u32,
        pc_relative: false,
        length_power: 3,
        external: true,
        relocation_type: 0,
    }
}

fn relocation_word(relocation: &NsldMachORelocationPlan) -> u32 {
    let pc_relative = u32::from(relocation.pc_relative);
    let external = u32::from(relocation.external);
    (relocation.symbol_index & 0x00ff_ffff)
        | (pc_relative << 24)
        | ((relocation.length_power as u32 & 0x3) << 25)
        | (external << 27)
        | ((relocation.relocation_type as u32 & 0xf) << 28)
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::{encode_mach_o_relocations, mach_o_arm64_relocation_table_plan};
    use crate::{
        main_test_support::empty_link_plan, object_file_layout::nsld_object_file_layout_report,
    };
    use std::path::Path;

    #[test]
    fn plans_and_encodes_mach_o_relocation_table() {
        let plan = empty_link_plan();
        let manifest = Path::new("manifest.toml");
        let file_layout = nsld_object_file_layout_report(manifest, &plan);
        let relocations =
            mach_o_arm64_relocation_table_plan(manifest, &plan, &file_layout).unwrap();
        let bytes = encode_mach_o_relocations(&relocations);

        assert_eq!(relocations.relocation_count, 4);
        assert_eq!(relocations.table_size, 32);
        assert_eq!(bytes.len(), 32);
        assert_eq!(&bytes[0..4], &[0, 0, 0, 0]);
        assert_eq!(&bytes[4..8], &[1, 0, 0, 0x0e]);
    }
}
