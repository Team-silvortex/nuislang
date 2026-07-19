use super::*;

pub(super) fn nsld_final_output_json_fields(
    nsld_final_output: Option<&NsldFinalExecutableOutputBoundarySummary>,
) -> Vec<String> {
    vec![
        json_bool_field(
            "nsld_final_executable_output_ready",
            nsld_final_output.is_some_and(|summary| summary.ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_boundary_status",
            nsld_final_output.map(|summary| summary.boundary_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_materialization_status",
            nsld_final_output.map(|summary| summary.materialization_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_contract",
            nsld_final_output.map(|summary| summary.execution_handoff_contract.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_execution_handoff_ready",
            nsld_final_output.is_some_and(|summary| summary.execution_handoff_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_status",
            nsld_final_output.map(|summary| summary.execution_handoff_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_target",
            nsld_final_output.map(|summary| summary.execution_handoff_target.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_evidence_status",
            nsld_final_output.map(|summary| summary.execution_handoff_evidence_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_first_blocker",
            nsld_final_output
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_decision_code",
            nsld_final_output.map(|summary| summary.execution_handoff_decision_code.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_entrypoint_materialization_evidence_status",
            nsld_final_output
                .map(|summary| summary.entrypoint_materialization_evidence_status.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_launcher_manifest_present",
            nsld_final_output.is_some_and(|summary| summary.launcher_manifest_present),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_launcher_manifest_ready",
            nsld_final_output.and_then(|summary| summary.launcher_manifest_ready),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_launcher_manifest_blocker_count",
            nsld_final_output.and_then(|summary| summary.launcher_manifest_blocker_count),
        ),
        json_bool_field(
            "nsld_final_executable_output_launcher_dry_run_present",
            nsld_final_output.is_some_and(|summary| summary.launcher_dry_run_present),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_launcher_dry_run_ready",
            nsld_final_output.and_then(|summary| summary.launcher_dry_run_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_launcher_dry_run_would_enter_lifecycle_hook",
            nsld_final_output
                .and_then(|summary| summary.launcher_dry_run_would_enter_lifecycle_hook),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_launcher_dry_run_blocker_count",
            nsld_final_output.and_then(|summary| summary.launcher_dry_run_blocker_count),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_payload_execution_trace_protocol",
            nsld_final_output.map(|summary| summary.payload_execution_trace_protocol.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_payload_execution_trace_available",
            nsld_final_output.is_some_and(|summary| summary.payload_execution_trace_available),
        ),
        json_usize_field(
            "nsld_final_executable_output_payload_execution_trace_record_count",
            nsld_final_output
                .map(|summary| summary.payload_execution_trace_record_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_output_payload_execution_trace_ready_record_count",
            nsld_final_output
                .map(|summary| summary.payload_execution_trace_ready_record_count)
                .unwrap_or(0),
        ),
        json_bool_field(
            "nsld_final_executable_output_device_provider_sample_manifest_available",
            nsld_final_output
                .is_some_and(|summary| summary.device_provider_sample_manifest_available),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_device_provider_sample_manifest_status",
            nsld_final_output
                .map(|summary| summary.device_provider_sample_manifest_status.as_str()),
        ),
        json_usize_field(
            "nsld_final_executable_output_device_provider_sample_manifest_record_count",
            nsld_final_output
                .map(|summary| summary.device_provider_sample_manifest_record_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_output_device_provider_sample_manifest_pending_record_count",
            nsld_final_output
                .map(|summary| summary.device_provider_sample_manifest_pending_record_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_output_device_provider_sample_manifest_blocked_record_count",
            nsld_final_output
                .map(|summary| summary.device_provider_sample_manifest_blocked_record_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_device_provider_sample_manifest_first_provider_family",
            nsld_final_output.map(|summary| {
                summary
                    .device_provider_sample_manifest_first_provider_family
                    .as_str()
            }),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_device_provider_sample_manifest_first_materialization_status",
            nsld_final_output.map(|summary| {
                summary
                    .device_provider_sample_manifest_first_materialization_status
                    .as_str()
            }),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_contract",
            nsld_final_output.map(|summary| summary.nsdb_replay_contract.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_nsdb_replay_ready",
            nsld_final_output.is_some_and(|summary| summary.nsdb_replay_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_status",
            nsld_final_output.map(|summary| summary.nsdb_replay_status.as_str()),
        ),
        json_usize_field(
            "nsld_final_executable_output_nsdb_replay_checkpoint_count",
            nsld_final_output
                .map(|summary| summary.nsdb_replay_checkpoint_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_output_nsdb_replayable_checkpoint_count",
            nsld_final_output
                .map(|summary| summary.nsdb_replayable_checkpoint_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_command",
            nsld_final_output.and_then(|summary| summary.nsdb_replay_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_next_action",
            nsld_final_output.map(|summary| summary.nsdb_replay_next_action.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_next_command",
            nsld_final_output.and_then(|summary| summary.nsdb_replay_next_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_first_blocker",
            nsld_final_output.and_then(|summary| summary.nsdb_replay_first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_package_contract",
            nsld_final_output.map(|summary| summary.object_package_summary_contract.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_object_package_ready",
            nsld_final_output.is_some_and(|summary| summary.object_package_summary_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_package_status",
            nsld_final_output.map(|summary| summary.object_package_summary_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_package_next_action",
            nsld_final_output.map(|summary| summary.object_package_summary_next_action.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_package_next_command",
            nsld_final_output.and_then(|summary| {
                summary.object_package_summary_next_command.as_deref()
            }),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_contract",
            nsld_final_output.map(|summary| summary.debugger_transcript_contract.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_debugger_transcript_ready",
            nsld_final_output.is_some_and(|summary| summary.debugger_transcript_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_status",
            nsld_final_output.map(|summary| summary.debugger_transcript_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_next_action",
            nsld_final_output.map(|summary| summary.debugger_transcript_next_action.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_next_command",
            nsld_final_output.and_then(|summary| summary.nsdb_replay_next_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_transcript_first_blocker",
            nsld_final_output.and_then(|summary| {
                summary.debugger_transcript_first_blocker.as_deref()
            }),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_handoff_contract",
            nsld_final_output.map(|summary| summary.debugger_cursor_handoff_contract.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_path",
            nsld_final_output.map(|summary| summary.debugger_cursor_path.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_debugger_cursor_ready",
            nsld_final_output.is_some_and(|summary| summary.debugger_cursor_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_status",
            nsld_final_output.map(|summary| summary.debugger_cursor_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_next_command",
            nsld_final_output.and_then(|summary| summary.debugger_cursor_next_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_contract",
            nsld_final_output.map(|summary| summary.debugger_cursor_lineage_contract.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_source_protocol",
            nsld_final_output
                .map(|summary| summary.debugger_cursor_lineage_source_protocol.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_path",
            nsld_final_output.map(|summary| summary.debugger_cursor_lineage_path.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_debugger_cursor_lineage_ready",
            nsld_final_output.is_some_and(|summary| summary.debugger_cursor_lineage_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_status",
            nsld_final_output.map(|summary| summary.debugger_cursor_lineage_status.as_str()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_debugger_cursor_lineage_entry_count",
            nsld_final_output.map(|summary| summary.debugger_cursor_lineage_entry_count),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_latest_hash",
            nsld_final_output
                .and_then(|summary| summary.debugger_cursor_lineage_latest_hash.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_first_blocker",
            nsld_final_output.and_then(|summary| {
                summary.debugger_cursor_lineage_first_blocker.as_deref()
            }),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_next_action",
            nsld_final_output
                .and_then(|summary| summary.debugger_cursor_lineage_next_action.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_debugger_cursor_lineage_next_command",
            nsld_final_output
                .and_then(|summary| summary.debugger_cursor_lineage_next_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_recommended_next_action",
            nsld_final_output.map(|summary| summary.recommended_next_action.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_path_present",
            nsld_final_output.is_some_and(|summary| summary.path_present),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_nsld_owned",
            nsld_final_output.and_then(|summary| summary.nsld_owned),
        ),
        json_bool_field(
            "nsld_final_executable_output_object_valid",
            nsld_final_output.is_some_and(|summary| summary.object_valid),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_path",
            nsld_final_output.map(|summary| summary.object_path.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_family",
            nsld_final_output.map(|summary| summary.object_family.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_magic_status",
            nsld_final_output.map(|summary| summary.object_magic_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_magic",
            nsld_final_output.and_then(|summary| summary.object_magic.as_deref()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_object_expected_size_bytes",
            nsld_final_output.and_then(|summary| summary.object_expected_size_bytes),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_object_actual_size_bytes",
            nsld_final_output.and_then(|summary| summary.object_actual_size_bytes),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_expected_hash",
            nsld_final_output.and_then(|summary| summary.object_expected_hash.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_object_actual_hash",
            nsld_final_output.and_then(|summary| summary.object_actual_hash.as_deref()),
        ),
        json_string_array_field(
            "nsld_final_executable_output_object_issues",
            nsld_final_output
                .map(|summary| summary.object_issues.as_slice())
                .unwrap_or(&[]),
        ),
        json_usize_field(
            "nsld_final_executable_output_blocker_count",
            nsld_final_output
                .map(|summary| summary.blockers.len())
                .unwrap_or(0),
        ),
        json_string_array_field(
            "nsld_final_executable_output_blockers",
            nsld_final_output
                .map(|summary| summary.blockers.as_slice())
                .unwrap_or(&[]),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_first_blocker",
            nsld_final_output.and_then(|summary| summary.first_blocker.as_deref()),
        ),
    ]
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}
