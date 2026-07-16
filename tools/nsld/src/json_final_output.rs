use super::{json_fields::*, reports::NsldFinalExecutableOutputReport};

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
        json_string_field(
            "entrypoint_materialization_evidence_status",
            &report.entrypoint_materialization_evidence_status,
        ),
        json_string_field("launcher_manifest_path", &report.launcher_manifest_path),
        json_bool_field(
            "launcher_manifest_present",
            report.launcher_manifest_present,
        ),
        json_optional_bool_field("launcher_manifest_ready", report.launcher_manifest_ready),
        json_optional_usize_field(
            "launcher_manifest_blocker_count",
            report.launcher_manifest_blocker_count,
        ),
        json_string_field("launcher_dry_run_path", &report.launcher_dry_run_path),
        json_bool_field("launcher_dry_run_present", report.launcher_dry_run_present),
        json_optional_bool_field("launcher_dry_run_ready", report.launcher_dry_run_ready),
        json_optional_bool_field(
            "launcher_dry_run_would_enter_lifecycle_hook",
            report.launcher_dry_run_would_enter_lifecycle_hook,
        ),
        json_optional_usize_field(
            "launcher_dry_run_blocker_count",
            report.launcher_dry_run_blocker_count,
        ),
        json_string_field("container_loader_status", &report.container_loader_status),
        json_string_field(
            "container_loader_payload_scan_kind",
            &report.container_loader_payload_scan_kind,
        ),
        json_bool_field("container_loader_parsed", report.container_loader_parsed),
        json_optional_string_field(
            "container_loader_readiness",
            report.container_loader_readiness.as_deref(),
        ),
        json_optional_bool_field("container_loader_ready", report.container_loader_ready),
        json_string_field(
            "container_loader_handoff_status",
            &report.container_loader_handoff_status,
        ),
        json_bool_field(
            "container_loader_handoff_ready",
            report.container_loader_handoff_ready,
        ),
        json_optional_string_field(
            "container_loader_handoff_first_blocker",
            report.container_loader_handoff_first_blocker.as_deref(),
        ),
        json_optional_string_field(
            "container_loader_entry_symbol",
            report.container_loader_entry_symbol.as_deref(),
        ),
        json_optional_string_field(
            "container_loader_entry_kind",
            report.container_loader_entry_kind.as_deref(),
        ),
        json_optional_string_field(
            "container_loader_entry_section_id",
            report.container_loader_entry_section_id.as_deref(),
        ),
        json_optional_usize_field(
            "container_loader_symbol_count",
            report.container_loader_symbol_count,
        ),
        json_string_field(
            "first_payload_execution_status",
            &report.first_payload_execution_status,
        ),
        json_bool_field(
            "first_payload_execution_ready",
            report.first_payload_execution_ready,
        ),
        json_string_field(
            "first_payload_execution_target",
            &report.first_payload_execution_target,
        ),
        json_optional_string_field(
            "first_payload_execution_entry_symbol",
            report.first_payload_execution_entry_symbol.as_deref(),
        ),
        json_optional_string_field(
            "first_payload_execution_entry_kind",
            report.first_payload_execution_entry_kind.as_deref(),
        ),
        json_optional_string_field(
            "first_payload_execution_entry_section_id",
            report.first_payload_execution_entry_section_id.as_deref(),
        ),
        json_optional_string_field(
            "first_payload_execution_first_blocker",
            report.first_payload_execution_first_blocker.as_deref(),
        ),
        json_string_field(
            "final_output_nsdb_handoff_protocol",
            &report.final_output_nsdb_handoff_protocol,
        ),
        json_bool_field(
            "final_output_nsdb_handoff_persisted",
            report.final_output_nsdb_handoff_persisted,
        ),
        json_string_field(
            "final_output_nsdb_handoff_path",
            &report.final_output_nsdb_handoff_path,
        ),
        json_usize_field(
            "final_output_nsdb_handoff_record_count",
            report.final_output_nsdb_handoff_record_count,
        ),
        json_usize_field(
            "final_output_nsdb_handoff_ready_record_count",
            report.final_output_nsdb_handoff_ready_record_count,
        ),
        json_optional_string_field(
            "final_output_nsdb_handoff_first_trace_id",
            report.final_output_nsdb_handoff_first_trace_id.as_deref(),
        ),
        json_optional_string_field(
            "final_output_nsdb_handoff_error",
            report.final_output_nsdb_handoff_error.as_deref(),
        ),
        json_string_field(
            "final_output_nsdb_replay_contract",
            &report.final_output_nsdb_replay_contract,
        ),
        json_bool_field(
            "final_output_nsdb_replay_ready",
            report.final_output_nsdb_replay_ready,
        ),
        json_string_field(
            "final_output_nsdb_replay_status",
            &report.final_output_nsdb_replay_status,
        ),
        json_optional_string_field(
            "final_output_nsdb_replay_command",
            report.final_output_nsdb_replay_command.as_deref(),
        ),
        json_string_field(
            "final_output_nsdb_replay_next_action",
            &report.final_output_nsdb_replay_next_action,
        ),
        json_optional_string_field(
            "final_output_nsdb_replay_next_command",
            report.final_output_nsdb_replay_next_command.as_deref(),
        ),
        json_usize_field(
            "final_output_nsdb_replay_checkpoint_count",
            report.final_output_nsdb_replay_checkpoint_count,
        ),
        json_usize_field(
            "final_output_nsdb_replayable_checkpoint_count",
            report.final_output_nsdb_replayable_checkpoint_count,
        ),
        json_optional_string_field(
            "final_output_nsdb_replay_first_blocker",
            report.final_output_nsdb_replay_first_blocker.as_deref(),
        ),
        json_bool_field(
            "device_provider_sample_manifest_available",
            report.device_provider_sample_manifest_available,
        ),
        json_string_field(
            "device_provider_sample_manifest_path",
            &report.device_provider_sample_manifest_path,
        ),
        json_string_field(
            "device_provider_sample_manifest_status",
            &report.device_provider_sample_manifest_status,
        ),
        json_usize_field(
            "device_provider_sample_manifest_record_count",
            report.device_provider_sample_manifest_record_count,
        ),
        json_usize_field(
            "device_provider_sample_manifest_ready_record_count",
            report.device_provider_sample_manifest_ready_record_count,
        ),
        json_usize_field(
            "device_provider_sample_manifest_pending_record_count",
            report.device_provider_sample_manifest_pending_record_count,
        ),
        json_optional_string_field(
            "device_provider_sample_manifest_first_provider_family",
            report
                .device_provider_sample_manifest_first_provider_family
                .as_deref(),
        ),
        json_optional_string_field(
            "device_provider_sample_manifest_first_materialization_status",
            report
                .device_provider_sample_manifest_first_materialization_status
                .as_deref(),
        ),
        json_optional_string_field(
            "device_provider_sample_manifest_first_blocker",
            report
                .device_provider_sample_manifest_first_blocker
                .as_deref(),
        ),
        json_string_field(
            "payload_execution_trace_protocol",
            payload_execution_trace_protocol(),
        ),
        json_bool_field(
            "payload_execution_trace_available",
            payload_execution_trace_available(report),
        ),
        json_usize_field(
            "payload_execution_trace_record_count",
            payload_execution_trace_record_count(report),
        ),
        json_usize_field(
            "payload_execution_trace_ready_record_count",
            payload_execution_trace_ready_record_count(report),
        ),
        payload_execution_trace_records_json(report),
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
        json_optional_string_field(
            "expected_image_resolver_status",
            report.expected_image_resolver_status.as_deref(),
        ),
        json_optional_string_field(
            "expected_image_patch_application_status",
            report.expected_image_patch_application_status.as_deref(),
        ),
        json_optional_string_field(
            "expected_image_patch_byte_audit_status",
            report.expected_image_patch_byte_audit_status.as_deref(),
        ),
        json_optional_string_field(
            "expected_image_patch_byte_audit_hash",
            report.expected_image_patch_byte_audit_hash.as_deref(),
        ),
        json_bool_field(
            "matches_verified_patched_image",
            report.matches_verified_patched_image,
        ),
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
        json_bool_field("object_output_valid", report.object_output_valid),
        json_string_field("object_output_path", &report.object_output_path),
        json_string_field("object_output_family", &report.object_output_family),
        json_string_field(
            "object_output_magic_status",
            &report.object_output_magic_status,
        ),
        json_optional_string_field("object_output_magic", report.object_output_magic.as_deref()),
        json_optional_usize_field(
            "object_output_expected_size_bytes",
            report.object_output_expected_size_bytes,
        ),
        json_optional_usize_field(
            "object_output_actual_size_bytes",
            report.object_output_actual_size_bytes,
        ),
        json_optional_string_field(
            "object_output_expected_hash",
            report.object_output_expected_hash.as_deref(),
        ),
        json_optional_string_field(
            "object_output_actual_hash",
            report.object_output_actual_hash.as_deref(),
        ),
        json_string_array_field("object_output_issues", &report.object_output_issues),
        json_bool_field("runnable_candidate", report.runnable_candidate),
        json_usize_field(
            "backend_artifact_candidate_count",
            report.backend_artifact_candidate_count,
        ),
        json_usize_field(
            "backend_artifact_ready_count",
            report.backend_artifact_ready_count,
        ),
        json_string_field(
            "backend_artifact_selection_status",
            &report.backend_artifact_selection_status,
        ),
        json_string_array_field(
            "backend_artifact_ordered_candidates",
            &report.backend_artifact_ordered_candidates,
        ),
        json_optional_string_field(
            "backend_artifact_selected_candidate",
            report.backend_artifact_selected_candidate.as_deref(),
        ),
        json_string_field(
            "backend_artifact_selection_reason",
            &report.backend_artifact_selection_reason,
        ),
        json_string_field(
            "backend_artifact_assembly_status",
            &report.backend_artifact_assembly_status,
        ),
        json_optional_string_field(
            "backend_artifact_selected_payload_path",
            report.backend_artifact_selected_payload_path.as_deref(),
        ),
        json_bool_field(
            "backend_artifact_selected_payload_consumed",
            report.backend_artifact_selected_payload_consumed,
        ),
        json_optional_string_field(
            "backend_artifact_assembly_first_blocker",
            report.backend_artifact_assembly_first_blocker.as_deref(),
        ),
        json_optional_string_field(
            "backend_artifact_first_unready",
            report.backend_artifact_first_unready.as_deref(),
        ),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
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

fn payload_execution_trace_records_json(report: &NsldFinalExecutableOutputReport) -> String {
    if !payload_execution_trace_available(report) {
        return "\"payload_execution_trace_records\":[]".to_owned();
    }

    let symbol = report
        .first_payload_execution_entry_symbol
        .as_deref()
        .unwrap_or("unknown-symbol");
    let trace_id = format!(
        "payload-trace:{}:{}",
        report.first_payload_execution_target, symbol
    );
    let next_action = if report.first_payload_execution_ready {
        "handoff-payload-trace-to-nsdb"
    } else {
        "resolve-payload-execution-blocker"
    };
    let fields = [
        json_string_field("trace_id", &trace_id),
        json_string_field("status", &report.first_payload_execution_status),
        json_string_field("execution_phase", "container-loader-handoff"),
        json_string_field("target", &report.first_payload_execution_target),
        json_optional_string_field(
            "entry_symbol",
            report.first_payload_execution_entry_symbol.as_deref(),
        ),
        json_optional_string_field(
            "entry_kind",
            report.first_payload_execution_entry_kind.as_deref(),
        ),
        json_optional_string_field(
            "entry_section_id",
            report.first_payload_execution_entry_section_id.as_deref(),
        ),
        json_optional_string_field(
            "first_blocker",
            report.first_payload_execution_first_blocker.as_deref(),
        ),
        json_string_field("next_action", next_action),
    ];

    format!(
        "\"payload_execution_trace_records\":[{{{}}}]",
        fields.join(",")
    )
}
