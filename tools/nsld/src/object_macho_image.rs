use super::{
    object_macho_header::{encode_mach_o_header, mach_o_arm64_header_plan},
    object_macho_load_commands::{encode_mach_o_load_commands, mach_o_arm64_load_commands_plan},
    object_macho_relocations::{encode_mach_o_relocations, mach_o_arm64_relocation_table_plan},
    object_macho_symbols::{
        encode_mach_o_string_table, encode_mach_o_symbols, mach_o_arm64_symbol_table_plan,
    },
    object_plan::nsld_object_plan_report,
    reports::{NsldObjectFileLayoutRecordDiagnostic, NsldObjectFileLayoutReport},
};
use std::{fs, path::Path};

pub(crate) fn encode_mach_o_arm64_image(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<Vec<u8>> {
    if file_layout.writer_backend_kind != "mach-o-arm64" {
        return None;
    }
    let mut bytes = vec![0u8; file_layout.total_file_size_bytes];
    write_record(
        &mut bytes,
        record_by_kind(file_layout, "macho-header")?,
        &encode_mach_o_header(&mach_o_arm64_header_plan(file_layout)?),
    )?;
    write_record(
        &mut bytes,
        record_by_kind(file_layout, "macho-load-commands")?,
        &encode_mach_o_load_commands(&mach_o_arm64_load_commands_plan(file_layout)?),
    )?;
    let relocations = mach_o_arm64_relocation_table_plan(manifest, plan, file_layout)?;
    write_record(
        &mut bytes,
        record_by_kind(file_layout, "macho-relocation-table")?,
        &encode_mach_o_relocations(&relocations),
    )?;
    let symbols = mach_o_arm64_symbol_table_plan(file_layout)?;
    write_record(
        &mut bytes,
        record_by_kind(file_layout, "macho-symbol-table")?,
        &encode_mach_o_symbols(&symbols),
    )?;
    write_record(
        &mut bytes,
        record_by_kind(file_layout, "macho-string-table")?,
        &encode_mach_o_string_table(&symbols),
    )?;
    write_section_payloads(&mut bytes, manifest, plan, file_layout)?;
    Some(bytes)
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
    if end > image.len() {
        return None;
    }
    image[record.file_offset..end].copy_from_slice(payload);
    Some(())
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

fn write_section_payloads(
    image: &mut [u8],
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<()> {
    let object_plan = nsld_object_plan_report(manifest, plan);
    for record in file_layout
        .records
        .iter()
        .filter(|record| record.record_kind == "section-payload")
    {
        let Some(source_section_id) = record.record_id.strip_prefix("section.") else {
            return None;
        };
        let Some(section) = object_plan
            .object_sections
            .iter()
            .find(|section| section.source_section_id == source_section_id)
        else {
            return None;
        };
        let Ok(payload) = fs::read(&section.source_path) else {
            continue;
        };
        write_record(image, record, &payload)?;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::encode_mach_o_arm64_image;
    use crate::{
        main_test_support::empty_link_plan, object_file_layout::nsld_object_file_layout_report,
    };
    use std::path::Path;

    #[test]
    fn encodes_mach_o_image_dry_run_from_file_layout() {
        let plan = empty_link_plan();
        let manifest = Path::new("manifest.toml");
        let file_layout = nsld_object_file_layout_report(manifest, &plan);
        let image = encode_mach_o_arm64_image(manifest, &plan, &file_layout).unwrap();

        assert_eq!(image.len(), file_layout.total_file_size_bytes);
        assert_eq!(&image[0..4], &[0xcf, 0xfa, 0xed, 0xfe]);
        let load_commands = file_layout
            .records
            .iter()
            .find(|record| record.record_kind == "macho-load-commands")
            .unwrap();
        assert_eq!(
            &image[load_commands.file_offset..load_commands.file_offset + 4],
            &[0x19, 0x00, 0x00, 0x00]
        );
        let relocations = file_layout
            .records
            .iter()
            .find(|record| record.record_kind == "macho-relocation-table")
            .unwrap();
        assert_eq!(
            &image[relocations.file_offset + 4..relocations.file_offset + 8],
            &[1, 0, 0, 0x0e]
        );
        let strings = file_layout
            .records
            .iter()
            .find(|record| record.record_kind == "macho-string-table")
            .unwrap();
        assert_eq!(image[strings.file_offset], 0);
        assert!(
            image[strings.file_offset..strings.file_offset + strings.size_bytes]
                .windows("__nuis_entry".len())
                .any(|window| window == b"__nuis_entry")
        );
    }
}
