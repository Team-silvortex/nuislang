pub(super) fn nsld_tail_json_fields(
    final_tail_summary: Option<&crate::workflow::NsldFinalExecutableTailSummary>,
    final_tail_stage_records: &[String],
    final_output_summary: Option<&crate::workflow::NsldFinalExecutableOutputBoundarySummary>,
) -> Vec<String> {
    vec![
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_command",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.pipeline_command.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_tail_ready",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        crate::json_usize_field(
            "nsld_final_executable_tail_stage_count",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        crate::json_usize_field(
            "nsld_final_executable_tail_present_count",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_tail_next_missing_stage",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        crate::json_object_array_field(
            "nsld_final_executable_tail_stage_records",
            &final_tail_stage_records,
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_valid",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.pipeline_valid),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_final_executable_emitted",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.final_executable_emitted),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_manifest_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.launcher_manifest_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_dry_run_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_would_enter_lifecycle_hook",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.would_enter_lifecycle_hook),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_blocker_count",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.blocker_count),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_first_blocker",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_contract",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_contract.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_execution_handoff_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_status",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_status.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_target",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_target.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_evidence_status.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_decision_code.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_kind",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_kind.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_path",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_first_blocker",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_first_blocker.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_present",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_present),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_hash",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_hash.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_runner_command",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_runner_command.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_payload_id",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_payload_id.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_scheduler_metadata_present",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_present),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_hash",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_hash.as_deref()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_count",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.required_stage_path_count),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_present_count",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.required_stage_path_present_count),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_first_missing_required_stage_path",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.first_missing_required_stage_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_ready),
        ),
        crate::json_optional_string_field(
            "nsld_self_owned_image_status",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.self_owned_image_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_entrypoint_materialization_status",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.entrypoint_materialization_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_self_owned_image_path",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_present",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_present),
        ),
        crate::json_optional_string_field(
            "nsld_self_owned_image_hash",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_hash.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_header_valid",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_header_valid),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_ready",
            final_output_summary
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_boundary_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.boundary_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_materialization_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.materialization_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_contract.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_execution_handoff_ready",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.execution_handoff_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_target",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_target.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_evidence_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_evidence_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_first_blocker",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_decision_code",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_decision_code.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_recommended_next_action",
            final_output_summary
                .as_ref()
                .map(|summary| summary.recommended_next_action.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.nsdb_replay_contract.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_nsdb_replay_ready",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.nsdb_replay_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.nsdb_replay_status.as_str()),
        ),
        crate::json_usize_field(
            "nsld_final_executable_output_nsdb_replay_checkpoint_count",
            final_output_summary
                .as_ref()
                .map(|summary| summary.nsdb_replay_checkpoint_count)
                .unwrap_or(0),
        ),
        crate::json_usize_field(
            "nsld_final_executable_output_nsdb_replayable_checkpoint_count",
            final_output_summary
                .as_ref()
                .map(|summary| summary.nsdb_replayable_checkpoint_count)
                .unwrap_or(0),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_command",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.nsdb_replay_command.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_next_action",
            final_output_summary
                .as_ref()
                .map(|summary| summary.nsdb_replay_next_action.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_next_command",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.nsdb_replay_next_command.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_first_blocker",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.nsdb_replay_first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_package_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.object_package_summary_contract.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_object_package_ready",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.object_package_summary_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_package_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.object_package_summary_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_package_next_action",
            final_output_summary
                .as_ref()
                .map(|summary| summary.object_package_summary_next_action.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_package_next_command",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.object_package_summary_next_command.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_transcript_contract.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_debugger_transcript_ready",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.debugger_transcript_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_transcript_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_next_action",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_transcript_next_action.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_next_command",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.nsdb_replay_next_command.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_first_blocker",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.debugger_transcript_first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_handoff_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_handoff_contract.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_path",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_path.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_debugger_cursor_ready",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.debugger_cursor_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_next_command",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.debugger_cursor_next_command.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_contract.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_source_protocol",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_source_protocol.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_path",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_path.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_debugger_cursor_lineage_ready",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.debugger_cursor_lineage_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_status.as_str()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_debugger_cursor_lineage_entry_count",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_entry_count),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_latest_hash",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.debugger_cursor_lineage_latest_hash.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_first_blocker",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.debugger_cursor_lineage_first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_next_action",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.debugger_cursor_lineage_next_action.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_next_command",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.debugger_cursor_lineage_next_command.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_repair_contract.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_path",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_repair_path.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_repair_status.as_str()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_entry_count",
            final_output_summary
                .as_ref()
                .map(|summary| summary.debugger_cursor_lineage_repair_entry_count),
        ),
        crate::json_optional_bool_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_latest_mutated",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.debugger_cursor_lineage_repair_latest_mutated),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_path",
            final_output_summary.as_ref().and_then(|summary| {
                summary
                    .debugger_cursor_lineage_repair_latest_archived_path
                    .as_deref()
            }),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_hash",
            final_output_summary.as_ref().and_then(|summary| {
                summary
                    .debugger_cursor_lineage_repair_latest_archived_hash
                    .as_deref()
            }),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_repair_latest_rebuilt_hash",
            final_output_summary.as_ref().and_then(|summary| {
                summary
                    .debugger_cursor_lineage_repair_latest_rebuilt_hash
                    .as_deref()
            }),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_path_present",
            final_output_summary
                .as_ref()
                .map(|summary| summary.path_present)
                .unwrap_or(false),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_nsld_owned",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.nsld_owned),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_object_valid",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.object_valid),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_path",
            final_output_summary
                .as_ref()
                .map(|summary| summary.object_path.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_family",
            final_output_summary
                .as_ref()
                .map(|summary| summary.object_family.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_magic_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.object_magic_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_magic",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.object_magic.as_deref()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_object_expected_size_bytes",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.object_expected_size_bytes),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_object_actual_size_bytes",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.object_actual_size_bytes),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_expected_hash",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.object_expected_hash.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_object_actual_hash",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.object_actual_hash.as_deref()),
        ),
        crate::json_string_array_field(
            "nsld_final_executable_output_object_issues",
            final_output_summary
                .as_ref()
                .map(|summary| summary.object_issues.as_slice())
                .unwrap_or(&[]),
        ),
        crate::json_usize_field(
            "nsld_final_executable_output_blocker_count",
            final_output_summary
                .as_ref()
                .map(|summary| summary.blockers.len())
                .unwrap_or(0),
        ),
        crate::json_string_array_field(
            "nsld_final_executable_output_blockers",
            final_output_summary
                .as_ref()
                .map(|summary| summary.blockers.as_slice())
                .unwrap_or(&[]),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_first_blocker",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
    ]
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => crate::json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => crate::json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}
