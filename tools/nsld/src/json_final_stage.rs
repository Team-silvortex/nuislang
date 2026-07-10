use super::{json_fields::*, json_final_fragments::final_stage_inputs_json, reports::*};

pub(crate) fn nsld_final_stage_plan_report_json(report: &NsldFinalStagePlanReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_stage_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("plan_hash", &report.plan_hash),
        json_string_field("final_stage_kind", &report.final_stage_kind),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_string_field("final_output_path", &report.final_output_path),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("compatibility_mode", &report.compatibility_mode),
        json_usize_field("input_count", report.input_count),
        format!("\"inputs\":[{}]", final_stage_inputs_json(&report.inputs)),
        json_string_field("container_hash", &report.container_hash),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_field("linker_contract_hash", &report.linker_contract_hash),
        json_bool_field("native_object_required", report.native_object_required),
        json_bool_field("native_object_present", report.native_object_present),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_stage_plan_emit_report_json(
    report: &NsldFinalStagePlanEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_stage_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_string_field("plan_hash", &report.plan_hash),
        json_usize_field("input_count", report.input_count),
        json_usize_field("blocker_count", report.blocker_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_stage_plan_verify_report_json(
    report: &NsldFinalStagePlanVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_stage_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_plan_hash", &report.expected_plan_hash),
        json_optional_string_field("actual_plan_hash", report.actual_plan_hash.as_deref()),
        json_usize_field("expected_input_count", report.expected_input_count),
        json_optional_usize_field("actual_input_count", report.actual_input_count),
        json_string_array_field("expected_input_ids", &report.expected_input_ids),
        json_string_array_field("actual_input_ids", &report.actual_input_ids),
        json_usize_field(
            "expected_input_entry_count",
            report.expected_input_entry_count,
        ),
        json_usize_field("actual_input_entry_count", report.actual_input_entry_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("expected_notes", &report.expected_notes),
        json_string_array_field("actual_notes", &report.actual_notes),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
