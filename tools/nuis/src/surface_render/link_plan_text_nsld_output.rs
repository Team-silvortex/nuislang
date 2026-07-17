use std::fmt;

pub(super) fn write_unavailable_nsld_final_output_text_fields<W: fmt::Write>(
    out: &mut W,
) -> fmt::Result {
    writeln!(out, "  nsld_final_executable_output_ready: <unavailable>")?;
    writeln!(
        out,
        "  nsld_final_executable_output_boundary_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_materialization_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_contract: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_ready: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_target: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_evidence_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_first_blocker: <none>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_decision_code: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_protocol: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_available: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_record_count: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_ready_record_count: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_contract: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_ready: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_checkpoints: <unavailable>/<unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_next_action: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_next_command: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_first_blocker: <none>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_recommended_next_action: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_path_present: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsld_owned: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_valid: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_path: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_identity: family=<unavailable> magic_status=<unavailable> magic=<unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_size_bytes: <unavailable>/<unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_hashes: expected=<unavailable> actual=<unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_blocker_count: <unavailable>"
    )?;
    writeln!(out, "  nsld_final_executable_output_first_blocker: <none>")
}

pub(super) fn write_nsld_final_output_text_fields<W: fmt::Write>(
    out: &mut W,
    final_output: &crate::workflow::NsldFinalExecutableOutputBoundarySummary,
) -> fmt::Result {
    writeln!(
        out,
        "  nsld_final_executable_output_ready: {}",
        crate::yes_no(final_output.ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_boundary_status: {}",
        final_output.boundary_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_materialization_status: {}",
        final_output.materialization_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_contract: {}",
        final_output.execution_handoff_contract
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_ready: {}",
        crate::yes_no(final_output.execution_handoff_ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_status: {}",
        final_output.execution_handoff_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_target: {}",
        final_output.execution_handoff_target
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_evidence_status: {}",
        final_output.execution_handoff_evidence_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_first_blocker: {}",
        final_output
            .execution_handoff_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_decision_code: {}",
        final_output.execution_handoff_decision_code
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_protocol: {}",
        final_output.payload_execution_trace_protocol
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_available: {}",
        crate::yes_no(final_output.payload_execution_trace_available)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_record_count: {}",
        final_output.payload_execution_trace_record_count
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_payload_execution_trace_ready_record_count: {}",
        final_output.payload_execution_trace_ready_record_count
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_contract: {}",
        final_output.nsdb_replay_contract
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_ready: {}",
        crate::yes_no(final_output.nsdb_replay_ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_status: {}",
        final_output.nsdb_replay_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_checkpoints: {}/{}",
        final_output.nsdb_replayable_checkpoint_count, final_output.nsdb_replay_checkpoint_count
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_next_action: {}",
        final_output.nsdb_replay_next_action
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_next_command: {}",
        final_output
            .nsdb_replay_next_command
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsdb_replay_first_blocker: {}",
        final_output
            .nsdb_replay_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_package_contract: {}",
        final_output.object_package_summary_contract
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_package_ready: {}",
        crate::yes_no(final_output.object_package_summary_ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_package_status: {}",
        final_output.object_package_summary_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_package_next_action: {}",
        final_output.object_package_summary_next_action
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_debugger_transcript_contract: {}",
        final_output.debugger_transcript_contract
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_debugger_transcript_ready: {}",
        crate::yes_no(final_output.debugger_transcript_ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_debugger_transcript_status: {}",
        final_output.debugger_transcript_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_debugger_transcript_next_action: {}",
        final_output.debugger_transcript_next_action
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_debugger_transcript_first_blocker: {}",
        final_output
            .debugger_transcript_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_recommended_next_action: {}",
        final_output.recommended_next_action
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_path_present: {}",
        crate::yes_no(final_output.path_present)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsld_owned: {}",
        final_output
            .nsld_owned
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_valid: {}",
        crate::yes_no(final_output.object_valid)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_path: {}",
        final_output.object_path
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_identity: family={} magic_status={} magic={}",
        final_output.object_family,
        final_output.object_magic_status,
        final_output
            .object_magic
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_size_bytes: {}/{}",
        optional_usize_text(final_output.object_actual_size_bytes),
        optional_usize_text(final_output.object_expected_size_bytes)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_object_hashes: expected={} actual={}",
        final_output
            .object_expected_hash
            .as_deref()
            .unwrap_or("<unavailable>"),
        final_output
            .object_actual_hash
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_blocker_count: {}",
        final_output.blockers.len()
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_first_blocker: {}",
        final_output.first_blocker.as_deref().unwrap_or("<none>")
    )?;
    for blocker in &final_output.blockers {
        writeln!(out, "  nsld_final_executable_output_blocker: {blocker}")?;
    }
    for issue in &final_output.object_issues {
        writeln!(out, "  nsld_final_executable_output_object_issue: {issue}")?;
    }
    Ok(())
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|size| size.to_string())
        .unwrap_or_else(|| "<unavailable>".to_owned())
}
