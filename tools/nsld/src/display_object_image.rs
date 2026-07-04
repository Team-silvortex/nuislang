use super::{display::optional_usize_text, reports::*};

pub(crate) fn print_nsld_object_image_dry_run_report(report: &NsldObjectImageDryRunReport) {
    println!("Nsld object image dry run");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  writer_target_id: {}", report.writer_target_id);
    println!("  writer_backend_kind: {}", report.writer_backend_kind);
    println!("  object_family: {}", report.object_family);
    println!("  backend_kind: {}", report.backend_kind);
    println!("  backend_family: {}", report.backend_family);
    println!("  backend_status: {}", report.backend_status);
    println!("  object_format: {}", report.object_format);
    println!("  file_layout_hash: {}", report.file_layout_hash);
    println!("  record_count: {}", report.record_count);
    println!("  total_file_size_bytes: {}", report.total_file_size_bytes);
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        report.image_hash.as_deref().unwrap_or("missing")
    );
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_image_dry_run_emit_report(
    report: &NsldObjectImageDryRunEmitReport,
) {
    println!("Nsld object image dry run emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  image_emitted: {}", report.image_emitted);
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        report.image_hash.as_deref().unwrap_or("missing")
    );
}

pub(crate) fn print_nsld_object_image_dry_run_verify_report(
    report: &NsldObjectImageDryRunVerifyReport,
) {
    println!("Nsld object image dry run verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  image_path: {}", report.image_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_writer_backend_kind: {}",
        report.expected_writer_backend_kind
    );
    println!(
        "  expected_object_family: {}",
        report.expected_object_family
    );
    println!(
        "  expected_file_layout_hash: {}",
        report.expected_file_layout_hash
    );
    println!(
        "  expected_backend_family: {}",
        report.expected_backend_family
    );
    println!(
        "  expected_backend_status: {}",
        report.expected_backend_status
    );
    println!(
        "  actual_file_layout_hash: {}",
        report
            .actual_file_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_writer_backend_kind: {}",
        report
            .actual_writer_backend_kind
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_object_family: {}",
        report.actual_object_family.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_backend_family: {}",
        report.actual_backend_family.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_backend_status: {}",
        report.actual_backend_status.as_deref().unwrap_or("missing")
    );
    println!("  expected_image_ready: {}", report.expected_image_ready);
    println!(
        "  actual_image_ready: {}",
        report
            .actual_image_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  actual_image_file_size_bytes: {}",
        optional_usize_text(report.actual_image_file_size_bytes)
    );
    println!(
        "  actual_image_file_hash: {}",
        report
            .actual_image_file_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
