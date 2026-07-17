use super::{display_text::*, reports::NsldFinalExecutableOutputReport};

pub(crate) fn print_nsld_final_executable_output_report(report: &NsldFinalExecutableOutputReport) {
    println!("Nsld final executable output");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  output_kind: {}", report.output_kind);
    println!(
        "  output_validation_mode: {}",
        report.output_validation_mode
    );
    println!("  boundary_status: {}", report.boundary_status);
    println!(
        "  materialization_status: {}",
        report.materialization_status
    );
    println!(
        "  execution_handoff_contract: {}",
        report.execution_handoff_contract
    );
    println!(
        "  execution_handoff_ready: {}",
        report.execution_handoff_ready
    );
    println!(
        "  execution_handoff_status: {}",
        report.execution_handoff_status
    );
    println!(
        "  execution_handoff_target: {}",
        report.execution_handoff_target
    );
    println!(
        "  execution_handoff_evidence_status: {}",
        report.execution_handoff_evidence_status
    );
    println!(
        "  execution_handoff_first_blocker: {}",
        optional_string_text(report.execution_handoff_first_blocker.as_deref())
    );
    println!(
        "  execution_handoff_decision_code: {}",
        report.execution_handoff_decision_code
    );
    println!(
        "  entrypoint_materialization_evidence_status: {}",
        report.entrypoint_materialization_evidence_status
    );
    println!(
        "  launcher_manifest_path: {}",
        report.launcher_manifest_path
    );
    println!(
        "  launcher_manifest_present: {}",
        report.launcher_manifest_present
    );
    println!(
        "  launcher_manifest_ready: {}",
        optional_bool_text(report.launcher_manifest_ready)
    );
    println!(
        "  launcher_manifest_blocker_count: {}",
        optional_usize_text(report.launcher_manifest_blocker_count)
    );
    println!("  launcher_dry_run_path: {}", report.launcher_dry_run_path);
    println!(
        "  launcher_dry_run_present: {}",
        report.launcher_dry_run_present
    );
    println!(
        "  launcher_dry_run_ready: {}",
        optional_bool_text(report.launcher_dry_run_ready)
    );
    println!(
        "  launcher_dry_run_would_enter_lifecycle_hook: {}",
        optional_bool_text(report.launcher_dry_run_would_enter_lifecycle_hook)
    );
    println!(
        "  launcher_dry_run_blocker_count: {}",
        optional_usize_text(report.launcher_dry_run_blocker_count)
    );
    println!(
        "  container_loader_status: {}",
        report.container_loader_status
    );
    println!(
        "  container_loader_payload_scan_kind: {}",
        report.container_loader_payload_scan_kind
    );
    println!(
        "  container_loader_parsed: {}",
        report.container_loader_parsed
    );
    println!(
        "  container_loader_readiness: {}",
        optional_string_text(report.container_loader_readiness.as_deref())
    );
    println!(
        "  container_loader_ready: {}",
        optional_bool_text(report.container_loader_ready)
    );
    println!(
        "  container_loader_handoff_status: {}",
        report.container_loader_handoff_status
    );
    println!(
        "  container_loader_handoff_ready: {}",
        report.container_loader_handoff_ready
    );
    println!(
        "  container_loader_handoff_first_blocker: {}",
        optional_string_text(report.container_loader_handoff_first_blocker.as_deref())
    );
    println!(
        "  container_loader_entry_symbol: {}",
        optional_string_text(report.container_loader_entry_symbol.as_deref())
    );
    println!(
        "  container_loader_entry_kind: {}",
        optional_string_text(report.container_loader_entry_kind.as_deref())
    );
    println!(
        "  container_loader_entry_section_id: {}",
        optional_string_text(report.container_loader_entry_section_id.as_deref())
    );
    println!(
        "  container_loader_symbol_count: {}",
        optional_usize_text(report.container_loader_symbol_count)
    );
    println!(
        "  first_payload_execution_status: {}",
        report.first_payload_execution_status
    );
    println!(
        "  first_payload_execution_ready: {}",
        report.first_payload_execution_ready
    );
    println!(
        "  first_payload_execution_target: {}",
        report.first_payload_execution_target
    );
    println!(
        "  first_payload_execution_entry_symbol: {}",
        optional_string_text(report.first_payload_execution_entry_symbol.as_deref())
    );
    println!(
        "  first_payload_execution_entry_kind: {}",
        optional_string_text(report.first_payload_execution_entry_kind.as_deref())
    );
    println!(
        "  first_payload_execution_entry_section_id: {}",
        optional_string_text(report.first_payload_execution_entry_section_id.as_deref())
    );
    println!(
        "  first_payload_execution_first_blocker: {}",
        optional_string_text(report.first_payload_execution_first_blocker.as_deref())
    );
    println!(
        "  final_output_nsdb_handoff_protocol: {}",
        report.final_output_nsdb_handoff_protocol
    );
    println!(
        "  final_output_nsdb_handoff_persisted: {}",
        report.final_output_nsdb_handoff_persisted
    );
    println!(
        "  final_output_nsdb_handoff_path: {}",
        report.final_output_nsdb_handoff_path
    );
    println!(
        "  final_output_nsdb_handoff_record_count: {}",
        report.final_output_nsdb_handoff_record_count
    );
    println!(
        "  final_output_nsdb_handoff_ready_record_count: {}",
        report.final_output_nsdb_handoff_ready_record_count
    );
    println!(
        "  final_output_nsdb_handoff_first_trace_id: {}",
        optional_string_text(report.final_output_nsdb_handoff_first_trace_id.as_deref())
    );
    println!(
        "  final_output_nsdb_handoff_error: {}",
        optional_string_text(report.final_output_nsdb_handoff_error.as_deref())
    );
    println!(
        "  final_output_nsdb_replay_contract: {}",
        report.final_output_nsdb_replay_contract
    );
    println!(
        "  final_output_nsdb_replay_ready: {}",
        report.final_output_nsdb_replay_ready
    );
    println!(
        "  final_output_nsdb_replay_status: {}",
        report.final_output_nsdb_replay_status
    );
    println!(
        "  final_output_nsdb_replay_command: {}",
        optional_string_text(report.final_output_nsdb_replay_command.as_deref())
    );
    println!(
        "  final_output_nsdb_replay_next_action: {}",
        report.final_output_nsdb_replay_next_action
    );
    println!(
        "  final_output_nsdb_replay_next_command: {}",
        optional_string_text(report.final_output_nsdb_replay_next_command.as_deref())
    );
    println!(
        "  final_output_nsdb_replay_checkpoint_count: {}",
        report.final_output_nsdb_replay_checkpoint_count
    );
    println!(
        "  final_output_nsdb_replayable_checkpoint_count: {}",
        report.final_output_nsdb_replayable_checkpoint_count
    );
    println!(
        "  final_output_nsdb_replay_first_blocker: {}",
        optional_string_text(report.final_output_nsdb_replay_first_blocker.as_deref())
    );
    println!(
        "  device_provider_sample_manifest_available: {}",
        report.device_provider_sample_manifest_available
    );
    println!(
        "  device_provider_sample_manifest_status: {}",
        report.device_provider_sample_manifest_status
    );
    println!(
        "  device_provider_sample_manifest_record_count: {}",
        report.device_provider_sample_manifest_record_count
    );
    println!(
        "  device_provider_sample_manifest_ready_record_count: {}",
        report.device_provider_sample_manifest_ready_record_count
    );
    println!(
        "  device_provider_sample_manifest_pending_record_count: {}",
        report.device_provider_sample_manifest_pending_record_count
    );
    println!(
        "  device_provider_sample_manifest_blocked_record_count: {}",
        report.device_provider_sample_manifest_blocked_record_count
    );
    println!(
        "  device_provider_sample_manifest_first_provider_family: {}",
        optional_string_text(
            report
                .device_provider_sample_manifest_first_provider_family
                .as_deref()
        )
    );
    println!(
        "  device_provider_sample_manifest_first_materialization_status: {}",
        optional_string_text(
            report
                .device_provider_sample_manifest_first_materialization_status
                .as_deref()
        )
    );
    println!(
        "  device_provider_sample_manifest_first_blocker: {}",
        optional_string_text(
            report
                .device_provider_sample_manifest_first_blocker
                .as_deref()
        )
    );
    println!(
        "  payload_execution_trace_protocol: {}",
        payload_execution_trace_protocol()
    );
    println!(
        "  payload_execution_trace_available: {}",
        payload_execution_trace_available(report)
    );
    println!(
        "  payload_execution_trace_record_count: {}",
        payload_execution_trace_record_count(report)
    );
    println!(
        "  payload_execution_trace_ready_record_count: {}",
        payload_execution_trace_ready_record_count(report)
    );
    if payload_execution_trace_available(report) {
        println!(
            "  payload_execution_trace_record: {} container-loader-handoff {}",
            payload_execution_trace_id(report),
            report.first_payload_execution_status
        );
    }
    println!(
        "  recommended_next_action: {}",
        report.recommended_next_action
    );
    println!("  path_present: {}", report.path_present);
    println!("  nsld_owned_output: {}", report.nsld_owned_output);
    println!("  present: {}", report.present);
    println!("  size_bytes: {}", optional_usize_text(report.size_bytes));
    println!(
        "  output_hash: {}",
        optional_string_text(report.output_hash.as_deref())
    );
    println!(
        "  output_image_header_required: {}",
        report.output_image_header_required
    );
    println!(
        "  output_image_header_valid: {}",
        report.output_image_header_valid
    );
    println!(
        "  output_image_magic: {}",
        optional_string_text(report.output_image_magic.as_deref())
    );
    println!(
        "  output_image_version: {}",
        optional_usize_text(report.output_image_version)
    );
    println!(
        "  output_image_header_size: {}",
        optional_usize_text(report.output_image_header_size)
    );
    println!(
        "  output_payload_byte_offset: {}",
        optional_usize_text(report.output_payload_byte_offset)
    );
    println!(
        "  output_payload_byte_span: {}",
        optional_usize_text(report.output_payload_byte_span)
    );
    println!(
        "  output_layout_hash: {}",
        optional_string_text(report.output_layout_hash.as_deref())
    );
    println!(
        "  output_byte_map_hash: {}",
        optional_string_text(report.output_byte_map_hash.as_deref())
    );
    println!(
        "  scheduler_metadata_payload_id: {}",
        optional_string_text(report.scheduler_metadata_payload_id.as_deref())
    );
    println!(
        "  scheduler_metadata_present: {}",
        optional_bool_text(report.scheduler_metadata_present)
    );
    println!(
        "  scheduler_metadata_offset: {}",
        optional_usize_text(report.scheduler_metadata_offset)
    );
    println!(
        "  scheduler_metadata_hash: {}",
        optional_string_text(report.scheduler_metadata_hash.as_deref())
    );
    println!(
        "  expected_image_size_bytes: {}",
        optional_usize_text(report.expected_image_size_bytes)
    );
    println!(
        "  expected_image_hash: {}",
        optional_string_text(report.expected_image_hash.as_deref())
    );
    println!(
        "  matches_expected_image: {}",
        report.matches_expected_image
    );
    println!(
        "  expected_image_resolver_status: {}",
        optional_string_text(report.expected_image_resolver_status.as_deref())
    );
    println!(
        "  expected_image_patch_application_status: {}",
        optional_string_text(report.expected_image_patch_application_status.as_deref())
    );
    println!(
        "  expected_image_patch_byte_audit_status: {}",
        optional_string_text(report.expected_image_patch_byte_audit_status.as_deref())
    );
    println!(
        "  expected_image_patch_byte_audit_hash: {}",
        optional_string_text(report.expected_image_patch_byte_audit_hash.as_deref())
    );
    println!(
        "  matches_verified_patched_image: {}",
        report.matches_verified_patched_image
    );
    println!(
        "  final_stage_plan_valid: {}",
        report.final_stage_plan_valid
    );
    println!(
        "  final_stage_plan_hash: {}",
        optional_string_text(report.final_stage_plan_hash.as_deref())
    );
    println!(
        "  final_executable_emit_valid: {}",
        report.final_executable_emit_valid
    );
    println!(
        "  final_executable_emitted: {}",
        optional_bool_text(report.final_executable_emitted)
    );
    println!(
        "  final_executable_blocker_count: {}",
        optional_usize_text(report.final_executable_blocker_count)
    );
    println!("  object_output_valid: {}", report.object_output_valid);
    println!("  object_output_path: {}", report.object_output_path);
    println!("  object_output_family: {}", report.object_output_family);
    println!(
        "  object_output_magic_status: {}",
        report.object_output_magic_status
    );
    println!(
        "  object_output_magic: {}",
        optional_string_text(report.object_output_magic.as_deref())
    );
    println!(
        "  object_output_expected_size_bytes: {}",
        optional_usize_text(report.object_output_expected_size_bytes)
    );
    println!(
        "  object_output_actual_size_bytes: {}",
        optional_usize_text(report.object_output_actual_size_bytes)
    );
    println!(
        "  object_output_expected_hash: {}",
        optional_string_text(report.object_output_expected_hash.as_deref())
    );
    println!(
        "  object_output_actual_hash: {}",
        optional_string_text(report.object_output_actual_hash.as_deref())
    );
    println!(
        "  object_package_summary_contract: {}",
        report.object_package_summary_contract
    );
    println!(
        "  object_package_summary_status: {}",
        report.object_package_summary_status
    );
    println!(
        "  object_package_summary_ready: {}",
        report.object_package_summary_ready
    );
    println!(
        "  object_package_summary_replay_status: {}",
        report.object_package_summary_replay_status
    );
    println!(
        "  object_package_summary_replay_ready: {}",
        report.object_package_summary_replay_ready
    );
    println!(
        "  object_package_summary_next_action: {}",
        report.object_package_summary_next_action
    );
    println!(
        "  object_package_summary_next_command: {}",
        optional_string_text(report.object_package_summary_next_command.as_deref())
    );
    for issue in &report.object_output_issues {
        println!("  object_output_issue: {issue}");
    }
    println!("  runnable_candidate: {}", report.runnable_candidate);
    println!(
        "  backend_artifact_candidate_count: {}",
        report.backend_artifact_candidate_count
    );
    println!(
        "  backend_artifact_ready_count: {}",
        report.backend_artifact_ready_count
    );
    println!(
        "  backend_artifact_selection_status: {}",
        report.backend_artifact_selection_status
    );
    println!(
        "  backend_artifact_first_unready: {}",
        optional_string_text(report.backend_artifact_first_unready.as_deref())
    );
    println!(
        "  backend_artifact_selected_candidate: {}",
        optional_string_text(report.backend_artifact_selected_candidate.as_deref())
    );
    println!(
        "  backend_artifact_selection_reason: {}",
        report.backend_artifact_selection_reason
    );
    println!(
        "  backend_artifact_assembly_status: {}",
        report.backend_artifact_assembly_status
    );
    println!(
        "  backend_artifact_selected_payload_path: {}",
        optional_string_text(report.backend_artifact_selected_payload_path.as_deref())
    );
    println!(
        "  backend_artifact_selected_payload_consumed: {}",
        report.backend_artifact_selected_payload_consumed
    );
    println!(
        "  backend_artifact_assembly_first_blocker: {}",
        optional_string_text(report.backend_artifact_assembly_first_blocker.as_deref())
    );
    for candidate in &report.backend_artifact_ordered_candidates {
        println!("  backend_artifact_ordered_candidate: {candidate}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn payload_execution_trace_protocol() -> &'static str {
    "nsdb-yir-payload-execution-trace-v1"
}

fn payload_execution_trace_available(report: &NsldFinalExecutableOutputReport) -> bool {
    report.first_payload_execution_target == "container-loader"
}

fn payload_execution_trace_record_count(report: &NsldFinalExecutableOutputReport) -> usize {
    usize::from(payload_execution_trace_available(report))
}

fn payload_execution_trace_ready_record_count(report: &NsldFinalExecutableOutputReport) -> usize {
    usize::from(payload_execution_trace_available(report) && report.first_payload_execution_ready)
}

fn payload_execution_trace_id(report: &NsldFinalExecutableOutputReport) -> String {
    let symbol = report
        .first_payload_execution_entry_symbol
        .as_deref()
        .unwrap_or("unknown-symbol");
    format!(
        "payload-trace:{}:{}",
        report.first_payload_execution_target, symbol
    )
}
