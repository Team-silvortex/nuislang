use super::{
    json_fields::{
        json_bool_field, json_optional_bool_field, json_optional_string_field,
        json_optional_usize_field, json_string_array_field, json_string_field, json_usize_field,
    },
    reports::{
        NsldFinalExecutableLauncherDryRunEmitReport, NsldFinalExecutableLauncherDryRunReport,
        NsldFinalExecutableLauncherDryRunVerifyReport,
        NsldFinalExecutableLauncherManifestEmitReport, NsldFinalExecutableLauncherManifestReport,
        NsldFinalExecutableLauncherManifestVerifyReport, NsldFinalExecutablePipelineEmitReport,
        NsldFinalExecutablePipelineVerifyReport,
    },
};

pub(crate) fn nsld_final_executable_launcher_manifest_report_json(
    report: &NsldFinalExecutableLauncherManifestReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_launcher_manifest"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("launcher_manifest_path", &report.launcher_manifest_path),
        json_bool_field("ready", report.ready),
        json_string_field("launcher_kind", &report.launcher_kind),
        json_string_field("launcher_format", &report.launcher_format),
        json_string_field("host_envelope_family", &report.host_envelope_family),
        json_string_field("host_os", &report.host_os),
        json_string_field("host_arch", &report.host_arch),
        json_string_field("output_kind", &report.output_kind),
        json_string_field("output_validation_mode", &report.output_validation_mode),
        json_string_field("final_output_path", &report.final_output_path),
        json_bool_field("final_output_present", report.final_output_present),
        json_optional_usize_field("final_output_size_bytes", report.final_output_size_bytes),
        json_optional_string_field("final_output_hash", report.final_output_hash.as_deref()),
        json_string_field("nsb_path", &report.nsb_path),
        json_bool_field("nsb_present", report.nsb_present),
        json_optional_usize_field("nsb_size_bytes", report.nsb_size_bytes),
        json_optional_string_field("nsb_hash", report.nsb_hash.as_deref()),
        json_bool_field("image_header_required", report.image_header_required),
        json_bool_field("image_header_valid", report.image_header_valid),
        json_string_field("entry_lifecycle_hook", &report.entry_lifecycle_hook),
        json_string_field("scheduler_entry", &report.scheduler_entry),
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
        json_string_array_field("verification_steps", &report.verification_steps),
        json_usize_field("blocker_count", report.blockers.len()),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_launcher_manifest_emit_report_json(
    report: &NsldFinalExecutableLauncherManifestEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_launcher_manifest_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("launcher_manifest_hash", &report.launcher_manifest_hash),
        json_bool_field("ready", report.ready),
        json_usize_field("blocker_count", report.blocker_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_launcher_manifest_verify_report_json(
    report: &NsldFinalExecutableLauncherManifestVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_launcher_manifest_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_launcher_manifest_hash",
            &report.expected_launcher_manifest_hash,
        ),
        json_optional_string_field(
            "actual_launcher_manifest_hash",
            report.actual_launcher_manifest_hash.as_deref(),
        ),
        json_bool_field("expected_ready", report.expected_ready),
        json_optional_bool_field("actual_ready", report.actual_ready),
        json_string_field("expected_nsb_path", &report.expected_nsb_path),
        json_optional_string_field("actual_nsb_path", report.actual_nsb_path.as_deref()),
        json_optional_usize_field("expected_nsb_size_bytes", report.expected_nsb_size_bytes),
        json_optional_usize_field("actual_nsb_size_bytes", report.actual_nsb_size_bytes),
        json_optional_string_field("expected_nsb_hash", report.expected_nsb_hash.as_deref()),
        json_optional_string_field("actual_nsb_hash", report.actual_nsb_hash.as_deref()),
        json_string_field("expected_output_kind", &report.expected_output_kind),
        json_optional_string_field("actual_output_kind", report.actual_output_kind.as_deref()),
        json_string_field(
            "expected_output_validation_mode",
            &report.expected_output_validation_mode,
        ),
        json_optional_string_field(
            "actual_output_validation_mode",
            report.actual_output_validation_mode.as_deref(),
        ),
        json_string_field(
            "expected_final_output_path",
            &report.expected_final_output_path,
        ),
        json_optional_string_field(
            "actual_final_output_path",
            report.actual_final_output_path.as_deref(),
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
        json_bool_field(
            "expected_image_header_required",
            report.expected_image_header_required,
        ),
        json_optional_bool_field(
            "actual_image_header_required",
            report.actual_image_header_required,
        ),
        json_bool_field(
            "expected_image_header_valid",
            report.expected_image_header_valid,
        ),
        json_optional_bool_field(
            "actual_image_header_valid",
            report.actual_image_header_valid,
        ),
        json_string_field(
            "expected_entry_lifecycle_hook",
            &report.expected_entry_lifecycle_hook,
        ),
        json_optional_string_field(
            "actual_entry_lifecycle_hook",
            report.actual_entry_lifecycle_hook.as_deref(),
        ),
        json_string_field("expected_scheduler_entry", &report.expected_scheduler_entry),
        json_optional_string_field(
            "actual_scheduler_entry",
            report.actual_scheduler_entry.as_deref(),
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
        json_optional_usize_field(
            "expected_scheduler_metadata_offset",
            report.expected_scheduler_metadata_offset,
        ),
        json_optional_usize_field(
            "actual_scheduler_metadata_offset",
            report.actual_scheduler_metadata_offset,
        ),
        json_optional_string_field(
            "expected_scheduler_metadata_hash",
            report.expected_scheduler_metadata_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_scheduler_metadata_hash",
            report.actual_scheduler_metadata_hash.as_deref(),
        ),
        json_string_array_field(
            "expected_verification_steps",
            &report.expected_verification_steps,
        ),
        json_string_array_field(
            "actual_verification_steps",
            &report.actual_verification_steps,
        ),
        json_usize_field("expected_blocker_count", report.expected_blocker_count),
        json_optional_usize_field("actual_blocker_count", report.actual_blocker_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_launcher_dry_run_report_json(
    report: &NsldFinalExecutableLauncherDryRunReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_launcher_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("launcher_manifest_path", &report.launcher_manifest_path),
        json_bool_field("launcher_manifest_valid", report.launcher_manifest_valid),
        json_optional_string_field("final_output_path", report.final_output_path.as_deref()),
        json_bool_field("final_output_readable", report.final_output_readable),
        json_optional_string_field(
            "final_output_hash_expected",
            report.final_output_hash_expected.as_deref(),
        ),
        json_optional_string_field(
            "final_output_hash_actual",
            report.final_output_hash_actual.as_deref(),
        ),
        json_bool_field(
            "final_output_hash_matches",
            report.final_output_hash_matches,
        ),
        json_optional_string_field("nsb_path", report.nsb_path.as_deref()),
        json_bool_field("nsb_readable", report.nsb_readable),
        json_optional_string_field("nsb_hash_expected", report.nsb_hash_expected.as_deref()),
        json_optional_string_field("nsb_hash_actual", report.nsb_hash_actual.as_deref()),
        json_bool_field("nsb_hash_matches", report.nsb_hash_matches),
        json_optional_string_field("output_kind", report.output_kind.as_deref()),
        json_optional_string_field(
            "output_validation_mode",
            report.output_validation_mode.as_deref(),
        ),
        json_optional_bool_field("image_header_required", report.image_header_required),
        json_optional_bool_field("image_header_valid", report.image_header_valid),
        json_optional_string_field(
            "entry_lifecycle_hook",
            report.entry_lifecycle_hook.as_deref(),
        ),
        json_optional_string_field("scheduler_entry", report.scheduler_entry.as_deref()),
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
        json_bool_field("dry_run_ready", report.dry_run_ready),
        json_bool_field(
            "would_enter_lifecycle_hook",
            report.would_enter_lifecycle_hook,
        ),
        json_string_array_field("launch_steps", &report.launch_steps),
        json_usize_field("blocker_count", report.blockers.len()),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_launcher_dry_run_emit_report_json(
    report: &NsldFinalExecutableLauncherDryRunEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_launcher_dry_run_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("dry_run_hash", &report.dry_run_hash),
        json_bool_field("dry_run_ready", report.dry_run_ready),
        json_usize_field("blocker_count", report.blocker_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_launcher_dry_run_verify_report_json(
    report: &NsldFinalExecutableLauncherDryRunVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_launcher_dry_run_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_dry_run_hash", &report.expected_dry_run_hash),
        json_optional_string_field("actual_dry_run_hash", report.actual_dry_run_hash.as_deref()),
        json_bool_field("expected_dry_run_ready", report.expected_dry_run_ready),
        json_optional_bool_field("actual_dry_run_ready", report.actual_dry_run_ready),
        json_bool_field(
            "expected_would_enter_lifecycle_hook",
            report.expected_would_enter_lifecycle_hook,
        ),
        json_optional_bool_field(
            "actual_would_enter_lifecycle_hook",
            report.actual_would_enter_lifecycle_hook,
        ),
        json_optional_string_field(
            "expected_nsb_hash_actual",
            report.expected_nsb_hash_actual.as_deref(),
        ),
        json_optional_string_field(
            "actual_nsb_hash_actual",
            report.actual_nsb_hash_actual.as_deref(),
        ),
        json_string_array_field("expected_launch_steps", &report.expected_launch_steps),
        json_string_array_field("actual_launch_steps", &report.actual_launch_steps),
        json_usize_field("expected_blocker_count", report.expected_blocker_count),
        json_optional_usize_field("actual_blocker_count", report.actual_blocker_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

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
