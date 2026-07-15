use super::{
    display_text::{optional_bool_text, optional_string_text, optional_usize_text},
    reports::{
        NsldFinalExecutableLauncherDryRunEmitReport, NsldFinalExecutableLauncherDryRunReport,
        NsldFinalExecutableLauncherDryRunVerifyReport,
        NsldFinalExecutableLauncherManifestEmitReport, NsldFinalExecutableLauncherManifestReport,
        NsldFinalExecutableLauncherManifestVerifyReport,
    },
};

pub(crate) fn print_nsld_final_executable_launcher_manifest_report(
    report: &NsldFinalExecutableLauncherManifestReport,
) {
    println!("Nsld final executable launcher manifest");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!(
        "  launcher_manifest_path: {}",
        report.launcher_manifest_path
    );
    println!("  ready: {}", report.ready);
    println!("  launcher_kind: {}", report.launcher_kind);
    println!("  launcher_format: {}", report.launcher_format);
    println!("  host_envelope_family: {}", report.host_envelope_family);
    println!("  host_os: {}", report.host_os);
    println!("  host_arch: {}", report.host_arch);
    println!("  output_kind: {}", report.output_kind);
    println!(
        "  output_validation_mode: {}",
        report.output_validation_mode
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
    println!("  final_output_path: {}", report.final_output_path);
    println!("  final_output_present: {}", report.final_output_present);
    println!(
        "  final_output_size_bytes: {}",
        optional_usize_text(report.final_output_size_bytes)
    );
    println!(
        "  final_output_hash: {}",
        optional_string_text(report.final_output_hash.as_deref())
    );
    println!("  nsb_path: {}", report.nsb_path);
    println!("  nsb_present: {}", report.nsb_present);
    println!(
        "  nsb_size_bytes: {}",
        optional_usize_text(report.nsb_size_bytes)
    );
    println!(
        "  nsb_hash: {}",
        optional_string_text(report.nsb_hash.as_deref())
    );
    println!("  image_header_required: {}", report.image_header_required);
    println!("  image_header_valid: {}", report.image_header_valid);
    println!("  entry_lifecycle_hook: {}", report.entry_lifecycle_hook);
    println!("  scheduler_entry: {}", report.scheduler_entry);
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
    for step in &report.verification_steps {
        println!("  verification_step: {step}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_launcher_manifest_emit_report(
    report: &NsldFinalExecutableLauncherManifestEmitReport,
) {
    println!("Nsld final executable launcher manifest emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!(
        "  launcher_manifest_hash: {}",
        report.launcher_manifest_hash
    );
    println!("  ready: {}", report.ready);
    println!("  blocker_count: {}", report.blocker_count);
}

pub(crate) fn print_nsld_final_executable_launcher_manifest_verify_report(
    report: &NsldFinalExecutableLauncherManifestVerifyReport,
) {
    println!("Nsld final executable launcher manifest verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_launcher_manifest_hash: {}",
        report.expected_launcher_manifest_hash
    );
    println!(
        "  actual_launcher_manifest_hash: {}",
        optional_string_text(report.actual_launcher_manifest_hash.as_deref())
    );
    println!("  expected_ready: {}", report.expected_ready);
    println!(
        "  actual_ready: {}",
        report
            .actual_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!("  expected_nsb_path: {}", report.expected_nsb_path);
    println!(
        "  actual_nsb_path: {}",
        optional_string_text(report.actual_nsb_path.as_deref())
    );
    println!(
        "  expected_nsb_hash: {}",
        optional_string_text(report.expected_nsb_hash.as_deref())
    );
    println!(
        "  actual_nsb_hash: {}",
        optional_string_text(report.actual_nsb_hash.as_deref())
    );
    println!("  expected_output_kind: {}", report.expected_output_kind);
    println!(
        "  actual_output_kind: {}",
        optional_string_text(report.actual_output_kind.as_deref())
    );
    println!(
        "  expected_output_validation_mode: {}",
        report.expected_output_validation_mode
    );
    println!(
        "  actual_output_validation_mode: {}",
        optional_string_text(report.actual_output_validation_mode.as_deref())
    );
    println!(
        "  expected_final_output_path: {}",
        report.expected_final_output_path
    );
    println!(
        "  actual_final_output_path: {}",
        optional_string_text(report.actual_final_output_path.as_deref())
    );
    println!(
        "  expected_final_output_hash: {}",
        optional_string_text(report.expected_final_output_hash.as_deref())
    );
    println!(
        "  actual_final_output_hash: {}",
        optional_string_text(report.actual_final_output_hash.as_deref())
    );
    println!(
        "  expected_image_header_required: {}",
        report.expected_image_header_required
    );
    println!(
        "  actual_image_header_required: {}",
        optional_bool_text(report.actual_image_header_required)
    );
    println!(
        "  expected_entry_lifecycle_hook: {}",
        report.expected_entry_lifecycle_hook
    );
    println!(
        "  actual_entry_lifecycle_hook: {}",
        optional_string_text(report.actual_entry_lifecycle_hook.as_deref())
    );
    println!(
        "  expected_scheduler_entry: {}",
        report.expected_scheduler_entry
    );
    println!(
        "  actual_scheduler_entry: {}",
        optional_string_text(report.actual_scheduler_entry.as_deref())
    );
    println!(
        "  expected_scheduler_metadata_payload_id: {}",
        optional_string_text(report.expected_scheduler_metadata_payload_id.as_deref())
    );
    println!(
        "  actual_scheduler_metadata_payload_id: {}",
        optional_string_text(report.actual_scheduler_metadata_payload_id.as_deref())
    );
    println!(
        "  expected_scheduler_metadata_present: {}",
        optional_bool_text(report.expected_scheduler_metadata_present)
    );
    println!(
        "  actual_scheduler_metadata_present: {}",
        optional_bool_text(report.actual_scheduler_metadata_present)
    );
    println!(
        "  expected_scheduler_metadata_offset: {}",
        optional_usize_text(report.expected_scheduler_metadata_offset)
    );
    println!(
        "  actual_scheduler_metadata_offset: {}",
        optional_usize_text(report.actual_scheduler_metadata_offset)
    );
    println!(
        "  expected_scheduler_metadata_hash: {}",
        optional_string_text(report.expected_scheduler_metadata_hash.as_deref())
    );
    println!(
        "  actual_scheduler_metadata_hash: {}",
        optional_string_text(report.actual_scheduler_metadata_hash.as_deref())
    );
    for step in &report.expected_verification_steps {
        println!("  expected_verification_step: {step}");
    }
    for step in &report.actual_verification_steps {
        println!("  actual_verification_step: {step}");
    }
    for blocker in &report.expected_blockers {
        println!("  expected_blocker: {blocker}");
    }
    for blocker in &report.actual_blockers {
        println!("  actual_blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_launcher_dry_run_report(
    report: &NsldFinalExecutableLauncherDryRunReport,
) {
    println!("Nsld final executable launcher dry-run");
    println!("  manifest: {}", report.manifest);
    println!(
        "  launcher_manifest_path: {}",
        report.launcher_manifest_path
    );
    println!(
        "  launcher_manifest_valid: {}",
        report.launcher_manifest_valid
    );
    println!(
        "  final_output_path: {}",
        optional_string_text(report.final_output_path.as_deref())
    );
    println!("  final_output_readable: {}", report.final_output_readable);
    println!(
        "  final_output_hash_expected: {}",
        optional_string_text(report.final_output_hash_expected.as_deref())
    );
    println!(
        "  final_output_hash_actual: {}",
        optional_string_text(report.final_output_hash_actual.as_deref())
    );
    println!(
        "  final_output_hash_matches: {}",
        report.final_output_hash_matches
    );
    println!(
        "  nsb_path: {}",
        optional_string_text(report.nsb_path.as_deref())
    );
    println!("  nsb_readable: {}", report.nsb_readable);
    println!(
        "  nsb_hash_expected: {}",
        optional_string_text(report.nsb_hash_expected.as_deref())
    );
    println!(
        "  nsb_hash_actual: {}",
        optional_string_text(report.nsb_hash_actual.as_deref())
    );
    println!("  nsb_hash_matches: {}", report.nsb_hash_matches);
    println!(
        "  output_kind: {}",
        optional_string_text(report.output_kind.as_deref())
    );
    println!(
        "  output_validation_mode: {}",
        optional_string_text(report.output_validation_mode.as_deref())
    );
    println!(
        "  execution_handoff_contract: {}",
        optional_string_text(report.execution_handoff_contract.as_deref())
    );
    println!(
        "  execution_handoff_ready: {}",
        optional_bool_text(report.execution_handoff_ready)
    );
    println!(
        "  execution_handoff_status: {}",
        optional_string_text(report.execution_handoff_status.as_deref())
    );
    println!(
        "  execution_handoff_target: {}",
        optional_string_text(report.execution_handoff_target.as_deref())
    );
    println!(
        "  execution_handoff_evidence_status: {}",
        optional_string_text(report.execution_handoff_evidence_status.as_deref())
    );
    println!(
        "  execution_handoff_first_blocker: {}",
        optional_string_text(report.execution_handoff_first_blocker.as_deref())
    );
    println!(
        "  execution_handoff_decision_code: {}",
        optional_string_text(report.execution_handoff_decision_code.as_deref())
    );
    println!(
        "  image_header_required: {}",
        optional_bool_text(report.image_header_required)
    );
    println!(
        "  image_header_valid: {}",
        report
            .image_header_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  entry_lifecycle_hook: {}",
        optional_string_text(report.entry_lifecycle_hook.as_deref())
    );
    println!(
        "  scheduler_entry: {}",
        optional_string_text(report.scheduler_entry.as_deref())
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
    println!("  dry_run_ready: {}", report.dry_run_ready);
    println!(
        "  would_enter_lifecycle_hook: {}",
        report.would_enter_lifecycle_hook
    );
    for step in &report.launch_steps {
        println!("  launch_step: {step}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_launcher_dry_run_emit_report(
    report: &NsldFinalExecutableLauncherDryRunEmitReport,
) {
    println!("Nsld final executable launcher dry-run emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  dry_run_hash: {}", report.dry_run_hash);
    println!("  dry_run_ready: {}", report.dry_run_ready);
    println!("  blocker_count: {}", report.blocker_count);
}

pub(crate) fn print_nsld_final_executable_launcher_dry_run_verify_report(
    report: &NsldFinalExecutableLauncherDryRunVerifyReport,
) {
    println!("Nsld final executable launcher dry-run verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_dry_run_hash: {}", report.expected_dry_run_hash);
    println!(
        "  actual_dry_run_hash: {}",
        optional_string_text(report.actual_dry_run_hash.as_deref())
    );
    println!(
        "  expected_dry_run_ready: {}",
        report.expected_dry_run_ready
    );
    println!(
        "  actual_dry_run_ready: {}",
        report
            .actual_dry_run_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  expected_would_enter_lifecycle_hook: {}",
        report.expected_would_enter_lifecycle_hook
    );
    println!(
        "  actual_would_enter_lifecycle_hook: {}",
        report
            .actual_would_enter_lifecycle_hook
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    for step in &report.expected_launch_steps {
        println!("  expected_launch_step: {step}");
    }
    for step in &report.actual_launch_steps {
        println!("  actual_launch_step: {step}");
    }
    for blocker in &report.expected_blockers {
        println!("  expected_blocker: {blocker}");
    }
    for blocker in &report.actual_blockers {
        println!("  actual_blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
