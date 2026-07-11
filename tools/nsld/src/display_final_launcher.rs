use super::{
    display_text::{optional_string_text, optional_usize_text},
    reports::{
        NsldFinalExecutableLauncherDryRunReport, NsldFinalExecutableLauncherManifestEmitReport,
        NsldFinalExecutableLauncherManifestReport, NsldFinalExecutableLauncherManifestVerifyReport,
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
