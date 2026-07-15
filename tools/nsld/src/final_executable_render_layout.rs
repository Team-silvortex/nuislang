use super::{reports::NsldFinalExecutableLayoutPlanReport, toml};

pub(crate) fn render_final_executable_layout_plan(
    report: &NsldFinalExecutableLayoutPlanReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-layout-plan-v1\"\n");
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
        "layout_hash = \"{}\"\n",
        toml::escape_toml_string(&report.layout_hash)
    ));
    out.push_str(&format!(
        "final_stage_plan_hash = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_plan_hash)
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "platform_envelope_family = \"{}\"\n",
        toml::escape_toml_string(&report.platform_envelope_family)
    ));
    out.push_str(&format!(
        "platform_envelope_policy = \"{}\"\n",
        toml::escape_toml_string(&report.platform_envelope_policy)
    ));
    out.push_str(&format!(
        "internal_binary_format = \"{}\"\n",
        toml::escape_toml_string(&report.internal_binary_format)
    ));
    out.push_str(&format!(
        "lifecycle_entry_hook = \"{}\"\n",
        toml::escape_toml_string(&report.lifecycle_entry_hook)
    ));
    out.push_str(&format!(
        "scheduler_contract = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_contract)
    ));
    out.push_str(&format!(
        "scheduler_metadata_payload = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_metadata_payload)
    ));
    out.push_str(&format!(
        "scheduler_metadata_lifecycle_hook = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_metadata_lifecycle_hook)
    ));
    out.push_str(&format!(
        "scheduler_hetero_node_count = {}\n",
        report.scheduler_hetero_node_count
    ));
    out.push_str(&format!(
        "scheduler_wait_event_count = {}\n",
        report.scheduler_wait_event_count
    ));
    out.push_str(&format!(
        "scheduler_emit_event_count = {}\n",
        report.scheduler_emit_event_count
    ));
    out.push_str(&format!(
        "data_segment_ordering = \"{}\"\n",
        toml::escape_toml_string(&report.data_segment_ordering)
    ));
    out.push_str(&format!(
        "relocation_application_strategy = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_strategy)
    ));
    out.push_str(&format!(
        "relocation_application_table_source = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_table_source)
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
        "native_object_path = \"{}\"\n",
        toml::escape_toml_string(&report.native_object_path)
    ));
    out.push_str(&format!(
        "native_object_required = {}\n",
        report.native_object_required
    ));
    out.push_str(&format!(
        "native_object_present = {}\n",
        report.native_object_present
    ));
    out.push_str(&format!(
        "compatibility_domain = \"{}\"\n",
        toml::escape_toml_string(&report.compatibility_domain)
    ));
    out.push_str(&format!(
        "compatibility_lifecycle_hook = \"{}\"\n",
        toml::escape_toml_string(&report.compatibility_lifecycle_hook)
    ));
    out.push_str(&format!("payload_count = {}\n", report.payload_count));
    out.push_str(&format!(
        "payloads = [{}]\n",
        toml::toml_string_array_literal(&report.payload_names)
    ));
    out.push_str(&format!("byte_alignment = {}\n", report.byte_alignment));
    out.push_str(&format!("byte_span = {}\n", report.byte_span));
    out.push_str(&format!(
        "byte_map_hash = \"{}\"\n",
        toml::escape_toml_string(&report.byte_map_hash)
    ));
    out.push_str(&format!(
        "notes = [{}]\n",
        toml::toml_string_array_literal(&report.notes)
    ));
    for payload in &report.payloads {
        out.push_str("\n[[payload]]\n");
        out.push_str(&format!("order_index = {}\n", payload.order_index));
        out.push_str(&format!(
            "payload_id = \"{}\"\n",
            toml::escape_toml_string(&payload.payload_id)
        ));
        out.push_str(&format!(
            "payload_kind = \"{}\"\n",
            toml::escape_toml_string(&payload.payload_kind)
        ));
        out.push_str(&format!(
            "lifecycle_hook = \"{}\"\n",
            toml::escape_toml_string(&payload.lifecycle_hook)
        ));
        out.push_str(&format!(
            "path = \"{}\"\n",
            toml::escape_toml_string(&payload.path)
        ));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            toml::escape_toml_string(&payload.content_hash)
        ));
        out.push_str(&format!("required = {}\n", payload.required));
        out.push_str(&format!("present = {}\n", payload.present));
    }
    for entry in &report.byte_map_entries {
        out.push_str("\n[[byte_map_entry]]\n");
        out.push_str(&format!("order_index = {}\n", entry.order_index));
        out.push_str(&format!(
            "payload_id = \"{}\"\n",
            toml::escape_toml_string(&entry.payload_id)
        ));
        out.push_str(&format!(
            "payload_kind = \"{}\"\n",
            toml::escape_toml_string(&entry.payload_kind)
        ));
        out.push_str(&format!("offset = {}\n", entry.offset));
        out.push_str(&format!("size_bytes = {}\n", entry.size_bytes));
        out.push_str(&format!("alignment = {}\n", entry.alignment));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            toml::escape_toml_string(&entry.content_hash)
        ));
    }
    for record in &report.relocation_applications {
        out.push_str("\n[[relocation_application]]\n");
        out.push_str(&format!("order_index = {}\n", record.order_index));
        out.push_str(&format!(
            "relocation_id = \"{}\"\n",
            toml::escape_toml_string(&record.relocation_id)
        ));
        out.push_str(&format!(
            "relocation_kind = \"{}\"\n",
            toml::escape_toml_string(&record.relocation_kind)
        ));
        out.push_str(&format!(
            "source_payload_id = \"{}\"\n",
            toml::escape_toml_string(&record.source_payload_id)
        ));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            toml::escape_toml_string(&record.source_section_id)
        ));
        out.push_str(&format!("source_offset = {}\n", record.source_offset));
        out.push_str(&format!("image_offset = {}\n", record.image_offset));
        out.push_str(&format!(
            "target_symbol_id = \"{}\"\n",
            toml::escape_toml_string(&record.target_symbol_id)
        ));
        out.push_str(&format!("addend = {}\n", record.addend));
        out.push_str(&format!(
            "application_status = \"{}\"\n",
            toml::escape_toml_string(&record.application_status)
        ));
    }
    out
}
