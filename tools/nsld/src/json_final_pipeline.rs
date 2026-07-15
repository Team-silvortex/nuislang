use super::{
    json_fields::{
        json_bool_field, json_optional_bool_field, json_optional_string_field,
        json_optional_usize_field, json_string_array_field, json_string_field, json_usize_field,
    },
    reports::{NsldFinalExecutablePipelineEmitReport, NsldFinalExecutablePipelineVerifyReport},
};

pub(crate) fn nsld_final_executable_pipeline_emit_report_json(
    report: &NsldFinalExecutablePipelineEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_pipeline_emit"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_string_field("final_stage_plan_path", &report.final_stage_plan_path),
        json_string_field("final_output_path", &report.final_output_path),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_string_field("host_invoke_plan_path", &report.host_invoke_plan_path),
        json_string_field("layout_plan_path", &report.layout_plan_path),
        json_string_field("image_dry_run_path", &report.image_dry_run_path),
        json_string_field(
            "final_executable_blocked_path",
            &report.final_executable_blocked_path,
        ),
        json_string_field("launcher_manifest_path", &report.launcher_manifest_path),
        json_string_field("launcher_dry_run_path", &report.launcher_dry_run_path),
        json_bool_field("final_executable_emitted", report.final_executable_emitted),
        json_bool_field("launcher_manifest_ready", report.launcher_manifest_ready),
        json_bool_field("launcher_dry_run_ready", report.launcher_dry_run_ready),
        json_bool_field(
            "would_enter_lifecycle_hook",
            report.would_enter_lifecycle_hook,
        ),
        json_string_field("self_owned_image_status", &report.self_owned_image_status),
        json_string_field(
            "entrypoint_materialization_status",
            &report.entrypoint_materialization_status,
        ),
        json_string_field(
            "entrypoint_materialization_kind",
            &report.entrypoint_materialization_kind,
        ),
        json_optional_string_field(
            "entrypoint_materialization_path",
            report.entrypoint_materialization_path.as_deref(),
        ),
        json_bool_field(
            "entrypoint_materialization_ready",
            report.entrypoint_materialization_ready,
        ),
        json_optional_string_field(
            "entrypoint_materialization_first_blocker",
            report.entrypoint_materialization_first_blocker.as_deref(),
        ),
        json_optional_bool_field(
            "entrypoint_materialization_present",
            report.entrypoint_materialization_present,
        ),
        json_optional_string_field(
            "entrypoint_materialization_hash",
            report.entrypoint_materialization_hash.as_deref(),
        ),
        json_optional_string_field(
            "entrypoint_materialization_runner_command",
            report.entrypoint_materialization_runner_command.as_deref(),
        ),
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
        json_optional_string_field(
            "scheduler_metadata_payload_id",
            report.scheduler_metadata_payload_id.as_deref(),
        ),
        json_optional_bool_field(
            "scheduler_metadata_present",
            report.scheduler_metadata_present,
        ),
        json_optional_string_field(
            "scheduler_metadata_hash",
            report.scheduler_metadata_hash.as_deref(),
        ),
        json_usize_field(
            "required_stage_path_count",
            report.required_stage_path_count,
        ),
        json_usize_field(
            "required_stage_path_present_count",
            report.required_stage_path_present_count,
        ),
        json_string_array_field(
            "missing_required_stage_paths",
            &report.missing_required_stage_paths,
        ),
        json_usize_field("blocker_count", report.blocker_count),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_pipeline_verify_report_json(
    report: &NsldFinalExecutablePipelineVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_pipeline_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_pipeline_hash", &report.expected_pipeline_hash),
        json_optional_string_field(
            "actual_pipeline_hash",
            report.actual_pipeline_hash.as_deref(),
        ),
        json_bool_field("expected_valid", report.expected_valid),
        json_optional_bool_field("actual_valid", report.actual_valid),
        json_bool_field(
            "expected_final_executable_emitted",
            report.expected_final_executable_emitted,
        ),
        json_optional_bool_field(
            "actual_final_executable_emitted",
            report.actual_final_executable_emitted,
        ),
        json_bool_field(
            "expected_launcher_manifest_ready",
            report.expected_launcher_manifest_ready,
        ),
        json_optional_bool_field(
            "actual_launcher_manifest_ready",
            report.actual_launcher_manifest_ready,
        ),
        json_bool_field(
            "expected_launcher_dry_run_ready",
            report.expected_launcher_dry_run_ready,
        ),
        json_optional_bool_field(
            "actual_launcher_dry_run_ready",
            report.actual_launcher_dry_run_ready,
        ),
        json_bool_field(
            "expected_would_enter_lifecycle_hook",
            report.expected_would_enter_lifecycle_hook,
        ),
        json_optional_bool_field(
            "actual_would_enter_lifecycle_hook",
            report.actual_would_enter_lifecycle_hook,
        ),
        json_string_field(
            "expected_self_owned_image_status",
            &report.expected_self_owned_image_status,
        ),
        json_optional_string_field(
            "actual_self_owned_image_status",
            report.actual_self_owned_image_status.as_deref(),
        ),
        json_string_field(
            "expected_entrypoint_materialization_status",
            &report.expected_entrypoint_materialization_status,
        ),
        json_optional_string_field(
            "actual_entrypoint_materialization_status",
            report.actual_entrypoint_materialization_status.as_deref(),
        ),
        json_string_field(
            "expected_entrypoint_materialization_kind",
            &report.expected_entrypoint_materialization_kind,
        ),
        json_optional_string_field(
            "actual_entrypoint_materialization_kind",
            report.actual_entrypoint_materialization_kind.as_deref(),
        ),
        json_optional_string_field(
            "expected_entrypoint_materialization_path",
            report.expected_entrypoint_materialization_path.as_deref(),
        ),
        json_optional_string_field(
            "actual_entrypoint_materialization_path",
            report.actual_entrypoint_materialization_path.as_deref(),
        ),
        json_bool_field(
            "expected_entrypoint_materialization_ready",
            report.expected_entrypoint_materialization_ready,
        ),
        json_optional_bool_field(
            "actual_entrypoint_materialization_ready",
            report.actual_entrypoint_materialization_ready,
        ),
        json_optional_string_field(
            "expected_entrypoint_materialization_first_blocker",
            report
                .expected_entrypoint_materialization_first_blocker
                .as_deref(),
        ),
        json_optional_string_field(
            "actual_entrypoint_materialization_first_blocker",
            report
                .actual_entrypoint_materialization_first_blocker
                .as_deref(),
        ),
        json_optional_bool_field(
            "expected_entrypoint_materialization_present",
            report.expected_entrypoint_materialization_present,
        ),
        json_optional_bool_field(
            "actual_entrypoint_materialization_present",
            report.actual_entrypoint_materialization_present,
        ),
        json_optional_string_field(
            "expected_entrypoint_materialization_hash",
            report.expected_entrypoint_materialization_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_entrypoint_materialization_hash",
            report.actual_entrypoint_materialization_hash.as_deref(),
        ),
        json_optional_string_field(
            "expected_entrypoint_materialization_runner_command",
            report
                .expected_entrypoint_materialization_runner_command
                .as_deref(),
        ),
        json_optional_string_field(
            "actual_entrypoint_materialization_runner_command",
            report
                .actual_entrypoint_materialization_runner_command
                .as_deref(),
        ),
        json_string_field(
            "expected_execution_handoff_contract",
            &report.expected_execution_handoff_contract,
        ),
        json_optional_string_field(
            "actual_execution_handoff_contract",
            report.actual_execution_handoff_contract.as_deref(),
        ),
        json_bool_field(
            "expected_execution_handoff_ready",
            report.expected_execution_handoff_ready,
        ),
        json_optional_bool_field(
            "actual_execution_handoff_ready",
            report.actual_execution_handoff_ready,
        ),
        json_string_field(
            "expected_execution_handoff_status",
            &report.expected_execution_handoff_status,
        ),
        json_optional_string_field(
            "actual_execution_handoff_status",
            report.actual_execution_handoff_status.as_deref(),
        ),
        json_string_field(
            "expected_execution_handoff_target",
            &report.expected_execution_handoff_target,
        ),
        json_optional_string_field(
            "actual_execution_handoff_target",
            report.actual_execution_handoff_target.as_deref(),
        ),
        json_string_field(
            "expected_execution_handoff_evidence_status",
            &report.expected_execution_handoff_evidence_status,
        ),
        json_optional_string_field(
            "actual_execution_handoff_evidence_status",
            report.actual_execution_handoff_evidence_status.as_deref(),
        ),
        json_optional_string_field(
            "expected_execution_handoff_first_blocker",
            report.expected_execution_handoff_first_blocker.as_deref(),
        ),
        json_optional_string_field(
            "actual_execution_handoff_first_blocker",
            report.actual_execution_handoff_first_blocker.as_deref(),
        ),
        json_string_field(
            "expected_execution_handoff_decision_code",
            &report.expected_execution_handoff_decision_code,
        ),
        json_optional_string_field(
            "actual_execution_handoff_decision_code",
            report.actual_execution_handoff_decision_code.as_deref(),
        ),
        json_optional_string_field(
            "expected_scheduler_metadata_payload_id",
            report.expected_scheduler_metadata_payload_id.as_deref(),
        ),
        json_optional_string_field(
            "actual_scheduler_metadata_payload_id",
            report.actual_scheduler_metadata_payload_id.as_deref(),
        ),
        json_optional_bool_field(
            "expected_scheduler_metadata_present",
            report.expected_scheduler_metadata_present,
        ),
        json_optional_bool_field(
            "actual_scheduler_metadata_present",
            report.actual_scheduler_metadata_present,
        ),
        json_optional_string_field(
            "expected_scheduler_metadata_hash",
            report.expected_scheduler_metadata_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_scheduler_metadata_hash",
            report.actual_scheduler_metadata_hash.as_deref(),
        ),
        json_usize_field(
            "expected_required_stage_path_count",
            report.expected_required_stage_path_count,
        ),
        json_optional_usize_field(
            "actual_required_stage_path_count",
            report.actual_required_stage_path_count,
        ),
        json_usize_field(
            "expected_required_stage_path_present_count",
            report.expected_required_stage_path_present_count,
        ),
        json_optional_usize_field(
            "actual_required_stage_path_present_count",
            report.actual_required_stage_path_present_count,
        ),
        json_string_array_field(
            "expected_missing_required_stage_paths",
            &report.expected_missing_required_stage_paths,
        ),
        json_string_array_field(
            "actual_missing_required_stage_paths",
            &report.actual_missing_required_stage_paths,
        ),
        json_usize_field("expected_blocker_count", report.expected_blocker_count),
        json_optional_usize_field("actual_blocker_count", report.actual_blocker_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
