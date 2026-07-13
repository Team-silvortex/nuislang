use super::{json_fields::*, reports::*};

pub(crate) fn nsld_final_executable_emit_report_json(
    report: &NsldFinalExecutableEmitReport,
) -> String {
    nsld_final_executable_report_json_with_kind(report, "nsld_final_executable_emit")
}

pub(crate) fn nsld_final_executable_readiness_report_json(
    report: &NsldFinalExecutableEmitReport,
) -> String {
    nsld_final_executable_report_json_with_kind(report, "nsld_final_executable_readiness")
}

fn nsld_final_executable_report_json_with_kind(
    report: &NsldFinalExecutableEmitReport,
    kind: &str,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", kind),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("blocked_report_path", &report.blocked_report_path),
        json_bool_field("emitted", report.emitted),
        json_bool_field(
            "can_emit_final_executable",
            report.can_emit_final_executable,
        ),
        json_bool_field("final_stage_ready", report.final_stage_ready),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("writer_kind", &report.writer_kind),
        json_string_field("writer_status", &report.writer_status),
        json_string_array_field("writer_blockers", &report.writer_blockers),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_optional_bool_field("writer_input_valid", report.writer_input_valid),
        json_optional_string_field("writer_input_hash", report.writer_input_hash.as_deref()),
        json_string_array_field("writer_input_issues", &report.writer_input_issues),
        json_optional_bool_field(
            "host_dry_run_environment_ready",
            report.host_dry_run_environment_ready,
        ),
        json_optional_bool_field(
            "host_dry_run_driver_available",
            report.host_dry_run_driver_available,
        ),
        json_optional_string_field(
            "host_dry_run_driver_resolved_path",
            report.host_dry_run_driver_resolved_path.as_deref(),
        ),
        json_optional_bool_field("host_dry_run_can_invoke", report.host_dry_run_can_invoke),
        json_optional_string_field(
            "host_dry_run_invocation_policy",
            report.host_dry_run_invocation_policy.as_deref(),
        ),
        json_optional_string_field(
            "host_dry_run_invocation_policy_reason",
            report.host_dry_run_invocation_policy_reason.as_deref(),
        ),
        json_usize_field(
            "host_dry_run_command_arg_count",
            report.host_dry_run_command_arg_count,
        ),
        json_string_array_field(
            "host_dry_run_command_args",
            &report.host_dry_run_command_args,
        ),
        json_usize_field(
            "host_dry_run_blocker_count",
            report.host_dry_run_blocker_count,
        ),
        json_string_array_field("host_dry_run_blockers", &report.host_dry_run_blockers),
        json_string_field("host_invoke_plan_path", &report.host_invoke_plan_path),
        json_optional_bool_field("host_invoke_plan_valid", report.host_invoke_plan_valid),
        json_optional_string_field(
            "host_invoke_plan_hash",
            report.host_invoke_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "host_invoke_plan_invocation_policy",
            report.host_invoke_plan_invocation_policy.as_deref(),
        ),
        json_optional_bool_field(
            "host_invoke_plan_requires_explicit_allow",
            report.host_invoke_plan_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "host_invoke_plan_explicit_allow_present",
            report.host_invoke_plan_explicit_allow_present,
        ),
        json_optional_bool_field(
            "host_invoke_plan_would_invoke",
            report.host_invoke_plan_would_invoke,
        ),
        json_optional_usize_field(
            "host_invoke_plan_blocker_count",
            report.host_invoke_plan_blocker_count,
        ),
        json_string_array_field("host_invoke_plan_issues", &report.host_invoke_plan_issues),
        json_string_field("layout_plan_path", &report.layout_plan_path),
        json_optional_bool_field("layout_plan_valid", report.layout_plan_valid),
        json_optional_string_field("layout_plan_hash", report.layout_plan_hash.as_deref()),
        json_string_array_field("layout_plan_issues", &report.layout_plan_issues),
        json_string_field("image_dry_run_path", &report.image_dry_run_path),
        json_string_field("image_dry_run_bytes_path", &report.image_dry_run_bytes_path),
        json_optional_bool_field("image_dry_run_valid", report.image_dry_run_valid),
        json_optional_string_field("image_dry_run_hash", report.image_dry_run_hash.as_deref()),
        json_optional_usize_field("image_dry_run_size_bytes", report.image_dry_run_size_bytes),
        json_string_array_field("image_dry_run_issues", &report.image_dry_run_issues),
        json_bool_field("final_output_checked", report.final_output_checked),
        json_bool_field("final_output_present", report.final_output_present),
        json_optional_usize_field("final_output_size_bytes", report.final_output_size_bytes),
        json_optional_string_field("final_output_hash", report.final_output_hash.as_deref()),
        json_optional_bool_field(
            "final_output_image_header_valid",
            report.final_output_image_header_valid,
        ),
        json_optional_bool_field(
            "final_output_runnable_candidate",
            report.final_output_runnable_candidate,
        ),
        json_usize_field("input_count", report.input_count),
        json_usize_field("blocker_count", report.blockers.len()),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_emit_verify_report_json(
    report: &NsldFinalExecutableEmitVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_emit_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_final_stage_plan_hash",
            &report.expected_final_stage_plan_hash,
        ),
        json_optional_string_field(
            "actual_final_stage_plan_hash",
            report.actual_final_stage_plan_hash.as_deref(),
        ),
        json_bool_field("expected_emitted", report.expected_emitted),
        json_optional_bool_field("actual_emitted", report.actual_emitted),
        json_optional_bool_field(
            "expected_writer_input_valid",
            report.expected_writer_input_valid,
        ),
        json_optional_bool_field(
            "actual_writer_input_valid",
            report.actual_writer_input_valid,
        ),
        json_optional_string_field(
            "expected_writer_input_hash",
            report.expected_writer_input_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_writer_input_hash",
            report.actual_writer_input_hash.as_deref(),
        ),
        json_string_array_field(
            "expected_writer_input_issues",
            &report.expected_writer_input_issues,
        ),
        json_string_array_field(
            "actual_writer_input_issues",
            &report.actual_writer_input_issues,
        ),
        json_optional_bool_field(
            "expected_host_dry_run_environment_ready",
            report.expected_host_dry_run_environment_ready,
        ),
        json_optional_bool_field(
            "actual_host_dry_run_environment_ready",
            report.actual_host_dry_run_environment_ready,
        ),
        json_optional_bool_field(
            "expected_host_dry_run_driver_available",
            report.expected_host_dry_run_driver_available,
        ),
        json_optional_bool_field(
            "actual_host_dry_run_driver_available",
            report.actual_host_dry_run_driver_available,
        ),
        json_optional_bool_field(
            "expected_host_dry_run_can_invoke",
            report.expected_host_dry_run_can_invoke,
        ),
        json_optional_bool_field(
            "actual_host_dry_run_can_invoke",
            report.actual_host_dry_run_can_invoke,
        ),
        json_optional_string_field(
            "expected_host_dry_run_driver_resolved_path",
            report.expected_host_dry_run_driver_resolved_path.as_deref(),
        ),
        json_optional_string_field(
            "actual_host_dry_run_driver_resolved_path",
            report.actual_host_dry_run_driver_resolved_path.as_deref(),
        ),
        json_optional_string_field(
            "expected_host_dry_run_invocation_policy",
            report.expected_host_dry_run_invocation_policy.as_deref(),
        ),
        json_optional_string_field(
            "actual_host_dry_run_invocation_policy",
            report.actual_host_dry_run_invocation_policy.as_deref(),
        ),
        json_optional_string_field(
            "expected_host_dry_run_invocation_policy_reason",
            report
                .expected_host_dry_run_invocation_policy_reason
                .as_deref(),
        ),
        json_optional_string_field(
            "actual_host_dry_run_invocation_policy_reason",
            report
                .actual_host_dry_run_invocation_policy_reason
                .as_deref(),
        ),
        json_usize_field(
            "expected_host_dry_run_command_arg_count",
            report.expected_host_dry_run_command_arg_count,
        ),
        json_optional_usize_field(
            "actual_host_dry_run_command_arg_count",
            report.actual_host_dry_run_command_arg_count,
        ),
        json_string_array_field(
            "expected_host_dry_run_command_args",
            &report.expected_host_dry_run_command_args,
        ),
        json_string_array_field(
            "actual_host_dry_run_command_args",
            &report.actual_host_dry_run_command_args,
        ),
        json_usize_field(
            "expected_host_dry_run_blocker_count",
            report.expected_host_dry_run_blocker_count,
        ),
        json_optional_usize_field(
            "actual_host_dry_run_blocker_count",
            report.actual_host_dry_run_blocker_count,
        ),
        json_string_array_field(
            "expected_host_dry_run_blockers",
            &report.expected_host_dry_run_blockers,
        ),
        json_string_array_field(
            "actual_host_dry_run_blockers",
            &report.actual_host_dry_run_blockers,
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_valid",
            report.expected_host_invoke_plan_valid,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_valid",
            report.actual_host_invoke_plan_valid,
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_would_invoke",
            report.expected_host_invoke_plan_would_invoke,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_would_invoke",
            report.actual_host_invoke_plan_would_invoke,
        ),
        json_optional_string_field(
            "expected_host_invoke_plan_hash",
            report.expected_host_invoke_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_host_invoke_plan_hash",
            report.actual_host_invoke_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "expected_host_invoke_plan_invocation_policy",
            report
                .expected_host_invoke_plan_invocation_policy
                .as_deref(),
        ),
        json_optional_string_field(
            "actual_host_invoke_plan_invocation_policy",
            report.actual_host_invoke_plan_invocation_policy.as_deref(),
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_requires_explicit_allow",
            report.expected_host_invoke_plan_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_requires_explicit_allow",
            report.actual_host_invoke_plan_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_explicit_allow_present",
            report.expected_host_invoke_plan_explicit_allow_present,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_explicit_allow_present",
            report.actual_host_invoke_plan_explicit_allow_present,
        ),
        json_optional_usize_field(
            "expected_host_invoke_plan_blocker_count",
            report.expected_host_invoke_plan_blocker_count,
        ),
        json_optional_usize_field(
            "actual_host_invoke_plan_blocker_count",
            report.actual_host_invoke_plan_blocker_count,
        ),
        json_string_array_field(
            "expected_host_invoke_plan_issues",
            &report.expected_host_invoke_plan_issues,
        ),
        json_string_array_field(
            "actual_host_invoke_plan_issues",
            &report.actual_host_invoke_plan_issues,
        ),
        json_optional_bool_field(
            "expected_layout_plan_valid",
            report.expected_layout_plan_valid,
        ),
        json_optional_bool_field("actual_layout_plan_valid", report.actual_layout_plan_valid),
        json_optional_string_field(
            "expected_layout_plan_hash",
            report.expected_layout_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_layout_plan_hash",
            report.actual_layout_plan_hash.as_deref(),
        ),
        json_string_array_field(
            "expected_layout_plan_issues",
            &report.expected_layout_plan_issues,
        ),
        json_string_array_field(
            "actual_layout_plan_issues",
            &report.actual_layout_plan_issues,
        ),
        json_optional_bool_field(
            "expected_image_dry_run_valid",
            report.expected_image_dry_run_valid,
        ),
        json_optional_bool_field(
            "actual_image_dry_run_valid",
            report.actual_image_dry_run_valid,
        ),
        json_optional_string_field(
            "expected_image_dry_run_hash",
            report.expected_image_dry_run_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_image_dry_run_hash",
            report.actual_image_dry_run_hash.as_deref(),
        ),
        json_optional_usize_field(
            "expected_image_dry_run_size_bytes",
            report.expected_image_dry_run_size_bytes,
        ),
        json_optional_usize_field(
            "actual_image_dry_run_size_bytes",
            report.actual_image_dry_run_size_bytes,
        ),
        json_string_array_field(
            "expected_image_dry_run_issues",
            &report.expected_image_dry_run_issues,
        ),
        json_string_array_field(
            "actual_image_dry_run_issues",
            &report.actual_image_dry_run_issues,
        ),
        json_bool_field(
            "expected_final_output_checked",
            report.expected_final_output_checked,
        ),
        json_optional_bool_field(
            "actual_final_output_checked",
            report.actual_final_output_checked,
        ),
        json_bool_field(
            "expected_final_output_present",
            report.expected_final_output_present,
        ),
        json_optional_bool_field(
            "actual_final_output_present",
            report.actual_final_output_present,
        ),
        json_optional_usize_field(
            "expected_final_output_size_bytes",
            report.expected_final_output_size_bytes,
        ),
        json_optional_usize_field(
            "actual_final_output_size_bytes",
            report.actual_final_output_size_bytes,
        ),
        json_optional_string_field(
            "expected_final_output_hash",
            report.expected_final_output_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_final_output_hash",
            report.actual_final_output_hash.as_deref(),
        ),
        json_optional_bool_field(
            "expected_final_output_image_header_valid",
            report.expected_final_output_image_header_valid,
        ),
        json_optional_bool_field(
            "actual_final_output_image_header_valid",
            report.actual_final_output_image_header_valid,
        ),
        json_optional_bool_field(
            "expected_final_output_runnable_candidate",
            report.expected_final_output_runnable_candidate,
        ),
        json_optional_bool_field(
            "actual_final_output_runnable_candidate",
            report.actual_final_output_runnable_candidate,
        ),
        json_usize_field("expected_blocker_count", report.expected_blocker_count),
        json_optional_usize_field("actual_blocker_count", report.actual_blocker_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_output_report_json(
    report: &NsldFinalExecutableOutputReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_output"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("output_kind", &report.output_kind),
        json_string_field("output_validation_mode", &report.output_validation_mode),
        json_string_field("boundary_status", &report.boundary_status),
        json_string_field("materialization_status", &report.materialization_status),
        json_string_field(
            "execution_handoff_contract",
            &report.execution_handoff_contract,
        ),
        json_bool_field("execution_handoff_ready", report.execution_handoff_ready),
        json_string_field("execution_handoff_status", &report.execution_handoff_status),
        json_string_field("execution_handoff_target", &report.execution_handoff_target),
        json_string_field(
            "execution_handoff_evidence_status",
            &report.execution_handoff_evidence_status,
        ),
        json_optional_string_field(
            "execution_handoff_first_blocker",
            report.execution_handoff_first_blocker.as_deref(),
        ),
        json_string_field(
            "execution_handoff_decision_code",
            &report.execution_handoff_decision_code,
        ),
        json_string_field("recommended_next_action", &report.recommended_next_action),
        json_bool_field("path_present", report.path_present),
        json_bool_field("nsld_owned_output", report.nsld_owned_output),
        json_bool_field("present", report.present),
        json_optional_usize_field("size_bytes", report.size_bytes),
        json_optional_string_field("output_hash", report.output_hash.as_deref()),
        json_bool_field(
            "output_image_header_required",
            report.output_image_header_required,
        ),
        json_bool_field(
            "output_image_header_valid",
            report.output_image_header_valid,
        ),
        json_optional_string_field("output_image_magic", report.output_image_magic.as_deref()),
        json_optional_usize_field("output_image_version", report.output_image_version),
        json_optional_usize_field("output_image_header_size", report.output_image_header_size),
        json_optional_usize_field(
            "output_payload_byte_offset",
            report.output_payload_byte_offset,
        ),
        json_optional_usize_field("output_payload_byte_span", report.output_payload_byte_span),
        json_optional_string_field("output_layout_hash", report.output_layout_hash.as_deref()),
        json_optional_string_field(
            "output_byte_map_hash",
            report.output_byte_map_hash.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_metadata_payload_id",
            report.scheduler_metadata_payload_id.as_deref(),
        ),
        json_optional_bool_field(
            "scheduler_metadata_present",
            report.scheduler_metadata_present,
        ),
        json_optional_usize_field(
            "scheduler_metadata_offset",
            report.scheduler_metadata_offset,
        ),
        json_optional_string_field(
            "scheduler_metadata_hash",
            report.scheduler_metadata_hash.as_deref(),
        ),
        json_optional_usize_field(
            "expected_image_size_bytes",
            report.expected_image_size_bytes,
        ),
        json_optional_string_field("expected_image_hash", report.expected_image_hash.as_deref()),
        json_bool_field("matches_expected_image", report.matches_expected_image),
        json_bool_field("final_stage_plan_valid", report.final_stage_plan_valid),
        json_optional_string_field(
            "final_stage_plan_hash",
            report.final_stage_plan_hash.as_deref(),
        ),
        json_bool_field(
            "final_executable_emit_valid",
            report.final_executable_emit_valid,
        ),
        json_optional_bool_field("final_executable_emitted", report.final_executable_emitted),
        json_optional_usize_field(
            "final_executable_blocker_count",
            report.final_executable_blocker_count,
        ),
        json_bool_field("runnable_candidate", report.runnable_candidate),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
