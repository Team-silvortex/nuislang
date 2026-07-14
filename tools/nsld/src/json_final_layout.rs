use super::{json_fields::*, reports::*};

pub(crate) fn nsld_final_executable_layout_plan_report_json(
    report: &NsldFinalExecutableLayoutPlanReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_layout_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("layout_hash", &report.layout_hash),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_string_field("platform_envelope_family", &report.platform_envelope_family),
        json_string_field("platform_envelope_policy", &report.platform_envelope_policy),
        json_string_field("internal_binary_format", &report.internal_binary_format),
        json_string_field("lifecycle_entry_hook", &report.lifecycle_entry_hook),
        json_string_field("scheduler_contract", &report.scheduler_contract),
        json_string_field(
            "scheduler_metadata_payload",
            &report.scheduler_metadata_payload,
        ),
        json_string_field(
            "scheduler_metadata_lifecycle_hook",
            &report.scheduler_metadata_lifecycle_hook,
        ),
        json_usize_field(
            "scheduler_hetero_node_count",
            report.scheduler_hetero_node_count,
        ),
        json_usize_field(
            "scheduler_wait_event_count",
            report.scheduler_wait_event_count,
        ),
        json_usize_field(
            "scheduler_emit_event_count",
            report.scheduler_emit_event_count,
        ),
        json_string_field("data_segment_ordering", &report.data_segment_ordering),
        json_string_field(
            "relocation_application_strategy",
            &report.relocation_application_strategy,
        ),
        json_string_field(
            "relocation_application_table_source",
            &report.relocation_application_table_source,
        ),
        json_usize_field(
            "relocation_application_count",
            report.relocation_application_count,
        ),
        json_string_field(
            "relocation_application_table_hash",
            &report.relocation_application_table_hash,
        ),
        json_string_field("native_object_path", &report.native_object_path),
        json_bool_field("native_object_required", report.native_object_required),
        json_bool_field("native_object_present", report.native_object_present),
        json_string_field("compatibility_domain", &report.compatibility_domain),
        json_string_field(
            "compatibility_lifecycle_hook",
            &report.compatibility_lifecycle_hook,
        ),
        json_usize_field("payload_count", report.payload_count),
        json_string_array_field("payloads", &report.payload_names),
        json_usize_field("byte_alignment", report.byte_alignment),
        json_usize_field("byte_span", report.byte_span),
        json_string_field("byte_map_hash", &report.byte_map_hash),
        format!(
            "\"payload_diagnostics\":[{}]",
            final_executable_payload_diagnostics_json(&report.payloads)
        ),
        format!(
            "\"byte_map_entries\":[{}]",
            final_executable_byte_map_entries_json(&report.byte_map_entries)
        ),
        format!(
            "\"relocation_applications\":[{}]",
            final_executable_relocation_applications_json(&report.relocation_applications)
        ),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_layout_plan_emit_report_json(
    report: &NsldFinalExecutableLayoutPlanEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_layout_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("layout_hash", &report.layout_hash),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_usize_field("payload_count", report.payload_count),
        json_bool_field("native_object_present", report.native_object_present),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_layout_plan_verify_report_json(
    report: &NsldFinalExecutableLayoutPlanVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_layout_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_layout_hash", &report.expected_layout_hash),
        json_optional_string_field("actual_layout_hash", report.actual_layout_hash.as_deref()),
        json_usize_field("expected_payload_count", report.expected_payload_count),
        json_optional_usize_field("actual_payload_count", report.actual_payload_count),
        json_string_array_field("expected_payloads", &report.expected_payloads),
        json_string_array_field("actual_payloads", &report.actual_payloads),
        json_usize_field(
            "expected_payload_entry_count",
            report.expected_payload_entry_count,
        ),
        json_usize_field(
            "actual_payload_entry_count",
            report.actual_payload_entry_count,
        ),
        json_usize_field(
            "expected_byte_map_entry_count",
            report.expected_byte_map_entry_count,
        ),
        json_usize_field(
            "actual_byte_map_entry_count",
            report.actual_byte_map_entry_count,
        ),
        json_usize_field("expected_byte_span", report.expected_byte_span),
        json_optional_usize_field("actual_byte_span", report.actual_byte_span),
        json_string_field("expected_byte_map_hash", &report.expected_byte_map_hash),
        json_optional_string_field(
            "actual_byte_map_hash",
            report.actual_byte_map_hash.as_deref(),
        ),
        json_string_field(
            "expected_lifecycle_entry_hook",
            &report.expected_lifecycle_entry_hook,
        ),
        json_optional_string_field(
            "actual_lifecycle_entry_hook",
            report.actual_lifecycle_entry_hook.as_deref(),
        ),
        json_usize_field(
            "expected_scheduler_hetero_node_count",
            report.expected_scheduler_hetero_node_count,
        ),
        json_optional_usize_field(
            "actual_scheduler_hetero_node_count",
            report.actual_scheduler_hetero_node_count,
        ),
        json_string_field(
            "expected_relocation_application_strategy",
            &report.expected_relocation_application_strategy,
        ),
        json_optional_string_field(
            "actual_relocation_application_strategy",
            report.actual_relocation_application_strategy.as_deref(),
        ),
        json_usize_field(
            "expected_relocation_application_count",
            report.expected_relocation_application_count,
        ),
        json_optional_usize_field(
            "actual_relocation_application_count",
            report.actual_relocation_application_count,
        ),
        json_string_field(
            "expected_relocation_application_table_hash",
            &report.expected_relocation_application_table_hash,
        ),
        json_optional_string_field(
            "actual_relocation_application_table_hash",
            report.actual_relocation_application_table_hash.as_deref(),
        ),
        json_string_field(
            "expected_platform_envelope_family",
            &report.expected_platform_envelope_family,
        ),
        json_optional_string_field(
            "actual_platform_envelope_family",
            report.actual_platform_envelope_family.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn final_executable_payload_diagnostics_json(
    payloads: &[NsldFinalExecutablePayloadDiagnostic],
) -> String {
    payloads
        .iter()
        .map(|payload| {
            let fields = [
                json_usize_field("order_index", payload.order_index),
                json_string_field("payload_id", &payload.payload_id),
                json_string_field("payload_kind", &payload.payload_kind),
                json_string_field("lifecycle_hook", &payload.lifecycle_hook),
                json_string_field("path", &payload.path),
                json_string_field("content_hash", &payload.content_hash),
                json_bool_field("required", payload.required),
                json_bool_field("present", payload.present),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn final_executable_byte_map_entries_json(entries: &[NsldFinalExecutableByteMapEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            let fields = [
                json_usize_field("order_index", entry.order_index),
                json_string_field("payload_id", &entry.payload_id),
                json_string_field("payload_kind", &entry.payload_kind),
                json_usize_field("offset", entry.offset),
                json_usize_field("size_bytes", entry.size_bytes),
                json_usize_field("alignment", entry.alignment),
                json_string_field("content_hash", &entry.content_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn final_executable_relocation_applications_json(
    records: &[NsldFinalExecutableRelocationApplicationRecord],
) -> String {
    records
        .iter()
        .map(|record| {
            let fields = [
                json_usize_field("order_index", record.order_index),
                json_string_field("relocation_id", &record.relocation_id),
                json_string_field("relocation_kind", &record.relocation_kind),
                json_string_field("source_payload_id", &record.source_payload_id),
                json_string_field("source_section_id", &record.source_section_id),
                json_usize_field("source_offset", record.source_offset),
                json_usize_field("image_offset", record.image_offset),
                json_string_field("target_symbol_id", &record.target_symbol_id),
                json_isize_field("addend", record.addend),
                json_string_field("application_status", &record.application_status),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}
