use super::{
    object_macho_symbols::mach_o_arm64_section_symbol_index,
    object_plan::nsld_object_plan_report,
    reports::{
        NsldObjectFileLayoutReport, NsldObjectRelocationSeedDiagnostic,
        NsldRelocationLoweringRuleDiagnostic,
    },
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
    pub(crate) seed_kind: String,
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
        .map(|seed| relocation_plan(seed, file_layout))
        .collect::<Vec<_>>();

    Some(NsldMachORelocationTablePlan {
        relocation_count: relocations.len(),
        table_size: expected_size,
        relocations,
    })
}

pub(crate) fn mach_o_arm64_relocation_resolution_issues(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Vec<String> {
    if file_layout.writer_backend_kind != "mach-o-arm64" {
        return Vec::new();
    }
    let object_plan = nsld_object_plan_report(manifest, plan);
    object_plan
        .relocation_seeds
        .iter()
        .filter(|seed| {
            mach_o_arm64_section_symbol_index(file_layout, &seed.source_section_id).is_none()
        })
        .map(|seed| {
            format!(
                "mach-o-relocation:{}:unresolved-section-symbol:{}",
                seed.relocation_seed_id, seed.source_section_id
            )
        })
        .chain(
            object_plan
                .relocation_seeds
                .iter()
                .filter(|seed| mach_o_arm64_relocation_kind(&seed.relocation_seed_kind).is_none())
                .map(|seed| {
                    format!(
                        "mach-o-relocation:{}:unsupported-seed-kind:{}",
                        seed.relocation_seed_id, seed.relocation_seed_kind
                    )
                }),
        )
        .collect()
}

pub(crate) fn mach_o_arm64_relocation_lowering_rule_count() -> usize {
    mach_o_arm64_relocation_lowering_rules().len()
}

pub(crate) fn mach_o_arm64_relocation_lowering_rules() -> Vec<NsldRelocationLoweringRuleDiagnostic>
{
    [
        "bootstrap-entry-seed",
        "metadata-address-seed",
        "data-address-seed",
        "extension-address-seed",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, seed_kind)| {
        let kind = mach_o_arm64_relocation_kind(seed_kind)
            .expect("registered Mach-O relocation seed kind must lower");
        NsldRelocationLoweringRuleDiagnostic {
            rule_id: format!("macho-arm64-reloc-rule-{index:04}"),
            source_seed_kind: seed_kind.to_owned(),
            target_relocation_kind: "arm64-unsigned-pointer".to_owned(),
            pc_relative: kind.pc_relative,
            length_power: kind.length_power,
            external: kind.external,
            relocation_type: kind.relocation_type,
        }
    })
    .collect()
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
    seed: &NsldObjectRelocationSeedDiagnostic,
    file_layout: &NsldObjectFileLayoutReport,
) -> NsldMachORelocationPlan {
    let kind = mach_o_arm64_relocation_kind(&seed.relocation_seed_kind)
        .unwrap_or_else(mach_o_arm64_unknown_relocation_kind);
    NsldMachORelocationPlan {
        address: seed.source_offset_seed as u32,
        symbol_index: mach_o_arm64_section_symbol_index(file_layout, &seed.source_section_id)
            .unwrap_or(0) as u32,
        seed_kind: seed.relocation_seed_kind.clone(),
        pc_relative: kind.pc_relative,
        length_power: kind.length_power,
        external: kind.external,
        relocation_type: kind.relocation_type,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NsldMachORelocationKind {
    pc_relative: bool,
    length_power: u8,
    external: bool,
    relocation_type: u8,
}

fn mach_o_arm64_relocation_kind(seed_kind: &str) -> Option<NsldMachORelocationKind> {
    match seed_kind {
        "bootstrap-entry-seed"
        | "metadata-address-seed"
        | "data-address-seed"
        | "extension-address-seed" => Some(NsldMachORelocationKind {
            pc_relative: false,
            length_power: 3,
            external: true,
            relocation_type: 0,
        }),
        _ => None,
    }
}

fn mach_o_arm64_unknown_relocation_kind() -> NsldMachORelocationKind {
    NsldMachORelocationKind {
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
    use super::{
        encode_mach_o_relocations, mach_o_arm64_relocation_lowering_rules,
        mach_o_arm64_relocation_resolution_issues, mach_o_arm64_relocation_table_plan,
    };
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
        assert_eq!(relocations.relocations[0].seed_kind, "bootstrap-entry-seed");
        assert_eq!(relocations.relocations[0].relocation_type, 0);
        assert_eq!(relocations.relocations[0].length_power, 3);
        assert_eq!(relocations.relocations[0].symbol_index, 1);
        assert_eq!(relocations.relocations[3].symbol_index, 4);
        assert_eq!(bytes.len(), 32);
        assert_eq!(&bytes[0..4], &[0, 0, 0, 0]);
        assert_eq!(&bytes[4..8], &[1, 0, 0, 0x0e]);
    }

    #[test]
    fn exposes_mach_o_relocation_lowering_rules() {
        let rules = mach_o_arm64_relocation_lowering_rules();

        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].source_seed_kind, "bootstrap-entry-seed");
        assert_eq!(rules[0].target_relocation_kind, "arm64-unsigned-pointer");
        assert_eq!(rules[0].relocation_type, 0);
        assert_eq!(rules[0].length_power, 3);
    }

    #[test]
    fn reports_unresolved_section_symbol_for_missing_relocation_target() {
        let plan = empty_link_plan();
        let manifest = Path::new("manifest.toml");
        let mut file_layout = nsld_object_file_layout_report(manifest, &plan);
        file_layout
            .records
            .retain(|record| record.record_id != "section.sec0000.compiled-artifact");

        let issues = mach_o_arm64_relocation_resolution_issues(manifest, &plan, &file_layout);

        assert!(issues.iter().any(|issue| {
            issue == "mach-o-relocation:orel0000.compiled_artifact:unresolved-section-symbol:sec0000.compiled-artifact"
        }));
    }
}
