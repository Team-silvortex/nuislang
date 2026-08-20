use crate::{reports::NsldMachOArm64MaterializationPreviewReport, toml};
use std::fmt::Write as _;

pub(crate) fn render_macho_materialization_preview(
    out: &mut String,
    report: &NsldMachOArm64MaterializationPreviewReport,
) {
    writeln!(
        out,
        "finalizer_input_materialization_contract = \"{}\"",
        toml::escape_toml_string(&report.contract)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_status = \"{}\"",
        toml::escape_toml_string(&report.status)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_placement_plan_hash = \"{}\"",
        toml::escape_toml_string(&report.placement_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_relocation_plan_hash = \"{}\"",
        toml::escape_toml_string(&report.relocation_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_image_span_bytes = {}",
        report.image_span_bytes
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_copied_bytes = {}",
        report.copied_bytes
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_zero_fill_bytes = {}",
        report.zero_fill_bytes
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_image_hash = \"{}\"",
        toml::escape_toml_string(&report.image_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_patch_plan_hash = \"{}\"",
        toml::escape_toml_string(&report.patch_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_planned_direct_count = {}",
        report.planned_direct_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_previewed_patch_count = {}",
        report.previewed_patch_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_deferred_patch_count = {}",
        report.deferred_patch_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_metadata_record_count = {}",
        report.metadata_record_count
    )
    .unwrap();
    let sections = report
        .section_audits
        .iter()
        .map(|section| {
            format!(
                "{}|{}|{}|{}|{}|{}",
                section.section_id,
                section.output_offset,
                section.size_bytes,
                section.copied_bytes,
                section.zero_fill_bytes,
                section.content_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_materialization_sections = [{}]",
        toml::toml_string_array_literal(&sections)
    )
    .unwrap();
    let patches = report
        .patches
        .iter()
        .map(|patch| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                patch.relocation_id,
                patch.relocation_kind,
                patch.source_output_offset,
                patch.width_bytes,
                patch.target_output_offset,
                patch.effective_addend,
                patch.source_bytes_hex,
                patch.encoded_bytes_hex,
                patch.source_bytes_hash,
                patch.encoded_bytes_hash,
                patch.audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_materialization_patches = [{}]",
        toml::toml_string_array_literal(&patches)
    )
    .unwrap();
}
