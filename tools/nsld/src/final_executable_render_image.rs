use super::{
    final_executable_render::optional_usize_toml, reports::NsldFinalExecutableImageDryRunReport,
    toml,
};

pub(crate) fn render_final_executable_image_dry_run(
    report: &NsldFinalExecutableImageDryRunReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-image-dry-run-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        toml::escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "image_path = \"{}\"\n",
        toml::escape_toml_string(&report.image_path)
    ));
    out.push_str(&format!(
        "image_format = \"{}\"\n",
        toml::escape_toml_string(&report.image_format)
    ));
    out.push_str(&format!(
        "image_magic = \"{}\"\n",
        toml::escape_toml_string(&report.image_magic)
    ));
    out.push_str(&format!(
        "image_header_size = {}\n",
        report.image_header_size
    ));
    out.push_str(&format!(
        "payload_byte_offset = {}\n",
        report.payload_byte_offset
    ));
    out.push_str(&format!(
        "payload_byte_span = {}\n",
        report.payload_byte_span
    ));
    out.push_str(&format!(
        "layout_hash = \"{}\"\n",
        toml::escape_toml_string(&report.layout_hash)
    ));
    out.push_str(&format!(
        "byte_map_hash = \"{}\"\n",
        toml::escape_toml_string(&report.byte_map_hash)
    ));
    out.push_str(&format!("payload_count = {}\n", report.payload_count));
    out.push_str(&format!("byte_span = {}\n", report.byte_span));
    out.push_str(&format!(
        "scheduler_metadata_payload_id = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_metadata_payload_id)
    ));
    out.push_str(&format!(
        "scheduler_metadata_present = {}\n",
        report.scheduler_metadata_present
    ));
    out.push_str(&format!(
        "scheduler_metadata_offset = {}\n",
        optional_usize_toml(report.scheduler_metadata_offset)
    ));
    out.push_str(&format!(
        "scheduler_metadata_hash = \"{}\"\n",
        toml::escape_toml_string(report.scheduler_metadata_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "backend_artifact_payload_count = {}\n",
        report.backend_artifact_payload_count
    ));
    out.push_str(&format!(
        "backend_artifact_payload_present_count = {}\n",
        report.backend_artifact_payload_present_count
    ));
    out.push_str(&format!(
        "backend_artifact_payload_role_status = \"{}\"\n",
        toml::escape_toml_string(&report.backend_artifact_payload_role_status)
    ));
    out.push_str(&format!(
        "backend_artifact_payload_ids = [{}]\n",
        toml::toml_string_array_literal(&report.backend_artifact_payload_ids)
    ));
    out.push_str(&format!(
        "backend_artifact_payload_kinds = [{}]\n",
        toml::toml_string_array_literal(&report.backend_artifact_payload_kinds)
    ));
    out.push_str(&format!(
        "backend_artifact_payload_first_missing = \"{}\"\n",
        toml::escape_toml_string(
            report
                .backend_artifact_payload_first_missing
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "relocation_application_strategy = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_strategy)
    ));
    out.push_str(&format!(
        "relocation_application_count = {}\n",
        report.relocation_application_count
    ));
    out.push_str(&format!(
        "relocation_application_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_table_hash)
    ));
    out.push_str(&format!(
        "relocation_application_audit_status = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_audit_status)
    ));
    out.push_str(&format!(
        "relocation_application_audit_count = {}\n",
        report.relocation_application_audit_count
    ));
    out.push_str(&format!(
        "relocation_application_audit_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_audit_table_hash)
    ));
    out.push_str(&format!(
        "relocation_application_audit_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.relocation_application_audit_blockers)
    ));
    out.push_str(&format!(
        "relocation_patch_preview_status = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_preview_status)
    ));
    out.push_str(&format!(
        "relocation_patch_preview_count = {}\n",
        report.relocation_patch_preview_count
    ));
    out.push_str(&format!(
        "relocation_patch_preview_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_preview_table_hash)
    ));
    out.push_str(&format!(
        "relocation_patch_application_status = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_application_status)
    ));
    out.push_str(&format!(
        "relocation_patch_application_count = {}\n",
        report.relocation_patch_application_count
    ));
    out.push_str(&format!(
        "relocation_patch_application_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_application_table_hash)
    ));
    out.push_str(&format!(
        "relocation_patch_application_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.relocation_patch_application_blockers)
    ));
    out.push_str(&format!(
        "image_constructed = {}\n",
        report.image_constructed
    ));
    out.push_str(&format!("image_ready = {}\n", report.image_ready));
    out.push_str(&format!(
        "image_size_bytes = {}\n",
        optional_usize_toml(report.image_size_bytes)
    ));
    out.push_str(&format!(
        "image_hash = \"{}\"\n",
        toml::escape_toml_string(report.image_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml::toml_string_array_literal(&report.blockers)
    ));
    for record in &report.relocation_patch_previews {
        out.push_str("\n[[relocation_patch_preview]]\n");
        out.push_str(&format!("order_index = {}\n", record.order_index));
        out.push_str(&format!(
            "relocation_id = \"{}\"\n",
            toml::escape_toml_string(&record.relocation_id)
        ));
        out.push_str(&format!(
            "patch_kind = \"{}\"\n",
            toml::escape_toml_string(&record.patch_kind)
        ));
        out.push_str(&format!("patch_offset = {}\n", record.patch_offset));
        out.push_str(&format!(
            "patch_width_bytes = {}\n",
            record.patch_width_bytes
        ));
        out.push_str(&format!(
            "resolved_patch_value = {}\n",
            optional_usize_toml(record.resolved_patch_value)
        ));
        out.push_str(&format!(
            "patch_value_hash = \"{}\"\n",
            toml::escape_toml_string(&record.patch_value_hash)
        ));
        out.push_str(&format!(
            "target_symbol_id = \"{}\"\n",
            toml::escape_toml_string(&record.target_symbol_id)
        ));
        out.push_str(&format!(
            "target_symbol_image_offset = {}\n",
            optional_usize_toml(record.target_symbol_image_offset)
        ));
        out.push_str(&format!(
            "preview_status = \"{}\"\n",
            toml::escape_toml_string(&record.preview_status)
        ));
        out.push_str(&format!(
            "resolver_status = \"{}\"\n",
            toml::escape_toml_string(&record.resolver_status)
        ));
    }
    out
}
