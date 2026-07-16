use super::{
    display::optional_usize_text,
    reports::{NsldObjectEmitReport, NsldObjectEmitVerifyReport, NsldObjectOutputVerifyReport},
};

pub(crate) fn print_nsld_object_emit_report(report: &NsldObjectEmitReport) {
    println!("Nsld object emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!("  blocked_report_path: {}", report.blocked_report_path);
    println!(
        "  image_dry_run_report_path: {}",
        report.image_dry_run_report_path
    );
    println!("  image_dry_run_path: {}", report.image_dry_run_path);
    println!(
        "  image_dry_run_hash: {}",
        report.image_dry_run_hash.as_deref().unwrap_or("missing")
    );
    println!("  writer_target_id: {}", report.writer_target_id);
    println!("  writer_backend_kind: {}", report.writer_backend_kind);
    println!("  object_family: {}", report.object_family);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  emitted: {}", report.emitted);
    println!("  can_emit_object: {}", report.can_emit_object);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_emit_verify_report(report: &NsldObjectEmitVerifyReport) {
    println!("Nsld object emit verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_object_plan_hash: {}",
        report.expected_object_plan_hash
    );
    println!(
        "  expected_writer_backend_kind: {}",
        report.expected_writer_backend_kind
    );
    println!(
        "  expected_object_family: {}",
        report.expected_object_family
    );
    println!(
        "  actual_object_plan_hash: {}",
        report
            .actual_object_plan_hash
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
        "  expected_image_dry_run_hash: {}",
        report
            .expected_image_dry_run_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_image_dry_run_hash: {}",
        report
            .actual_image_dry_run_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  image_dry_run_report_valid: {}",
        report.image_dry_run_report_valid
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_object_output_verify_report(report: &NsldObjectOutputVerifyReport) {
    println!("Nsld object output verify");
    println!("  manifest: {}", report.manifest);
    println!("  object_output_path: {}", report.object_output_path);
    println!("  image_dry_run_path: {}", report.image_dry_run_path);
    println!("  valid: {}", report.valid);
    println!("  object_family: {}", report.object_family);
    println!("  object_magic_status: {}", report.object_magic_status);
    println!(
        "  object_magic: {}",
        report.object_magic.as_deref().unwrap_or("missing")
    );
    println!(
        "  expected_size_bytes: {}",
        optional_usize_text(report.expected_size_bytes)
    );
    println!(
        "  actual_size_bytes: {}",
        optional_usize_text(report.actual_size_bytes)
    );
    println!(
        "  expected_hash: {}",
        report.expected_hash.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_hash: {}",
        report.actual_hash.as_deref().unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
