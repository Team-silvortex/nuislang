use super::{json_fields::*, json_final_fragments::final_stage_inputs_json, reports::*};

pub(crate) fn nsld_final_executable_writer_plan_report_json(
    report: &NsldFinalExecutableWriterPlanReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_writer_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_kind", &report.writer_kind),
        json_string_field("writer_status", &report.writer_status),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("input_count", report.input_count),
        format!("\"inputs\":[{}]", final_stage_inputs_json(&report.inputs)),
        json_string_array_field("writer_steps", &report.writer_steps),
        json_string_array_field("writer_blockers", &report.writer_blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_writer_input_emit_report_json(
    report: &NsldFinalExecutableWriterInputEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_writer_input_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_input_hash", &report.writer_input_hash),
        json_string_field("writer_kind", &report.writer_kind),
        json_string_field("writer_status", &report.writer_status),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("command_arg_count", report.command_arg_count),
        json_string_array_field("writer_blockers", &report.writer_blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_writer_input_verify_report_json(
    report: &NsldFinalExecutableWriterInputVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_writer_input_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_writer_input_hash",
            &report.expected_writer_input_hash,
        ),
        json_optional_string_field(
            "actual_writer_input_hash",
            report.actual_writer_input_hash.as_deref(),
        ),
        json_string_field(
            "expected_final_stage_plan_hash",
            &report.expected_final_stage_plan_hash,
        ),
        json_optional_string_field(
            "actual_final_stage_plan_hash",
            report.actual_final_stage_plan_hash.as_deref(),
        ),
        json_string_field("expected_writer_kind", &report.expected_writer_kind),
        json_optional_string_field("actual_writer_kind", report.actual_writer_kind.as_deref()),
        json_string_field("expected_writer_status", &report.expected_writer_status),
        json_optional_string_field(
            "actual_writer_status",
            report.actual_writer_status.as_deref(),
        ),
        json_usize_field(
            "expected_command_arg_count",
            report.expected_command_arg_count,
        ),
        json_optional_usize_field("actual_command_arg_count", report.actual_command_arg_count),
        json_string_array_field("expected_command_args", &report.expected_command_args),
        json_string_array_field("actual_command_args", &report.actual_command_args),
        json_string_array_field("expected_writer_blockers", &report.expected_writer_blockers),
        json_string_array_field("actual_writer_blockers", &report.actual_writer_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_dry_run_report_json(
    report: &NsldFinalExecutableHostDryRunReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_bool_field("writer_input_valid", report.writer_input_valid),
        json_optional_string_field("writer_input_hash", report.writer_input_hash.as_deref()),
        json_string_field("driver", &report.driver),
        json_bool_field("driver_available", report.driver_available),
        json_optional_string_field(
            "driver_resolved_path",
            report.driver_resolved_path.as_deref(),
        ),
        json_usize_field("command_arg_count", report.command_arg_count),
        json_string_array_field("command_args", &report.command_args),
        json_bool_field("environment_ready", report.environment_ready),
        json_string_field("invocation_policy", &report.invocation_policy),
        json_string_field("invocation_policy_reason", &report.invocation_policy_reason),
        json_bool_field(
            "can_invoke_host_finalizer",
            report.can_invoke_host_finalizer,
        ),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_invoke_plan_report_json(
    report: &NsldFinalExecutableHostInvokePlanReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_invoke_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_string_field("invocation_kind", &report.invocation_kind),
        json_string_field("invocation_policy", &report.invocation_policy),
        json_string_field("invocation_policy_reason", &report.invocation_policy_reason),
        json_bool_field("requires_explicit_allow", report.requires_explicit_allow),
        json_bool_field("explicit_allow_present", report.explicit_allow_present),
        json_bool_field("environment_ready", report.environment_ready),
        json_bool_field("driver_available", report.driver_available),
        json_optional_string_field(
            "driver_resolved_path",
            report.driver_resolved_path.as_deref(),
        ),
        json_bool_field(
            "can_invoke_host_finalizer",
            report.can_invoke_host_finalizer,
        ),
        json_bool_field("would_invoke", report.would_invoke),
        json_usize_field("command_arg_count", report.command_arg_count),
        json_string_array_field("command_args", &report.command_args),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_invoke_plan_emit_report_json(
    report: &NsldFinalExecutableHostInvokePlanEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_invoke_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("invoke_plan_hash", &report.invoke_plan_hash),
        json_string_field("invocation_policy", &report.invocation_policy),
        json_bool_field("requires_explicit_allow", report.requires_explicit_allow),
        json_bool_field("explicit_allow_present", report.explicit_allow_present),
        json_bool_field("would_invoke", report.would_invoke),
        json_usize_field("blocker_count", report.blocker_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_invoke_plan_verify_report_json(
    report: &NsldFinalExecutableHostInvokePlanVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_invoke_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_invoke_plan_hash",
            &report.expected_invoke_plan_hash,
        ),
        json_optional_string_field(
            "actual_invoke_plan_hash",
            report.actual_invoke_plan_hash.as_deref(),
        ),
        json_string_field(
            "expected_invocation_policy",
            &report.expected_invocation_policy,
        ),
        json_optional_string_field(
            "actual_invocation_policy",
            report.actual_invocation_policy.as_deref(),
        ),
        json_bool_field(
            "expected_requires_explicit_allow",
            report.expected_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "actual_requires_explicit_allow",
            report.actual_requires_explicit_allow,
        ),
        json_bool_field(
            "expected_explicit_allow_present",
            report.expected_explicit_allow_present,
        ),
        json_optional_bool_field(
            "actual_explicit_allow_present",
            report.actual_explicit_allow_present,
        ),
        json_bool_field("expected_would_invoke", report.expected_would_invoke),
        json_optional_bool_field("actual_would_invoke", report.actual_would_invoke),
        json_usize_field(
            "expected_command_arg_count",
            report.expected_command_arg_count,
        ),
        json_optional_usize_field("actual_command_arg_count", report.actual_command_arg_count),
        json_string_array_field("expected_command_args", &report.expected_command_args),
        json_string_array_field("actual_command_args", &report.actual_command_args),
        json_usize_field("expected_blocker_count", report.expected_blocker_count),
        json_optional_usize_field("actual_blocker_count", report.actual_blocker_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
