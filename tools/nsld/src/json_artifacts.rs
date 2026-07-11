use super::{json_fields::*, reports::*};

pub(crate) fn nsld_artifact_chain_report_json(report: &NsldArtifactChainReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_artifact_chain"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_dir", &report.output_dir),
        json_bool_field("valid", report.valid),
        json_usize_field("stage_count", report.stage_count),
        json_usize_field("present_count", report.present_count),
        json_usize_field("required_count", report.required_count),
        json_usize_field("missing_required_count", report.missing_required_count),
        json_usize_field("optional_present_count", report.optional_present_count),
        json_optional_string_field(
            "first_missing_required_stage",
            report.first_missing_required_stage.as_deref(),
        ),
        json_optional_string_field("next_required_stage", report.next_required_stage.as_deref()),
        json_optional_string_field(
            "suggested_command_id",
            report.suggested_command_id.as_deref(),
        ),
        json_optional_string_field("suggested_command", report.suggested_command.as_deref()),
        json_optional_string_field(
            "suggested_command_resolved",
            report.suggested_command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "suggested_command_reason",
            report.suggested_command_reason.as_deref(),
        ),
        json_optional_string_field("next_optional_stage", report.next_optional_stage.as_deref()),
        json_optional_string_field(
            "next_optional_command_id",
            report.next_optional_command_id.as_deref(),
        ),
        json_optional_string_field(
            "next_optional_command",
            report.next_optional_command.as_deref(),
        ),
        json_optional_string_field(
            "next_optional_command_resolved",
            report.next_optional_command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "next_optional_command_reason",
            report.next_optional_command_reason.as_deref(),
        ),
        json_optional_string_field("advisory_command_id", report.advisory_command_id.as_deref()),
        json_optional_string_field("advisory_command", report.advisory_command.as_deref()),
        json_optional_string_field(
            "advisory_command_resolved",
            report.advisory_command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "advisory_command_reason",
            report.advisory_command_reason.as_deref(),
        ),
        json_optional_string_field(
            "next_action_command_id",
            report.next_action_command_id.as_deref(),
        ),
        json_optional_string_field("next_action_command", report.next_action_command.as_deref()),
        json_optional_string_field(
            "next_action_command_resolved",
            report.next_action_command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "next_action_command_reason",
            report.next_action_command_reason.as_deref(),
        ),
        json_optional_string_field("next_action_source", report.next_action_source.as_deref()),
        json_bool_field("next_action_available", report.next_action_available),
        format!(
            "\"stages\":[{}]",
            artifact_chain_stage_diagnostics_json(&report.stages)
        ),
        json_string_array_field("advisories", &report.advisories),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_units_emit_report_json(report: &NsldLinkUnitsEmitReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_units_verify_report_json(report: &NsldLinkUnitsVerifyReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field("expected_unit_count", report.expected_unit_count),
        json_usize_field(
            "expected_hetero_unit_count",
            report.expected_hetero_unit_count,
        ),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_string_field("expected_unit_table_hash", &report.expected_unit_table_hash),
        json_optional_usize_field("actual_unit_count", report.actual_unit_count),
        json_optional_usize_field("actual_hetero_unit_count", report.actual_hetero_unit_count),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_string_field(
            "actual_unit_table_hash",
            report.actual_unit_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_report_json(report: &NsldLinkBundleReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle"),
        json_string_field("manifest", &report.manifest),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("compiled_artifact_path", &report.compiled_artifact_path),
        json_string_field("native_output_path", &report.native_output_path),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_emit_report_json(report: &NsldLinkBundleEmitReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_verify_report_json(report: &NsldLinkBundleVerifyReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_bundle_id", &report.expected_bundle_id),
        json_string_field("expected_bundle_hash", &report.expected_bundle_hash),
        json_optional_string_field("actual_bundle_id", report.actual_bundle_id.as_deref()),
        json_optional_string_field("actual_bundle_hash", report.actual_bundle_hash.as_deref()),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn artifact_chain_stage_diagnostics_json(stages: &[NsldArtifactStageDiagnostic]) -> String {
    stages
        .iter()
        .map(|stage| {
            let fields = [
                json_usize_field("order_index", stage.order_index),
                json_string_field("stage_id", &stage.stage_id),
                json_string_field("file_name", &stage.file_name),
                json_string_field("path", &stage.path),
                json_bool_field("required", stage.required),
                json_bool_field("present", stage.present),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}
