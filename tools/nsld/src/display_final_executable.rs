use super::{display_text::*, reports::*};

pub(crate) fn print_nsld_final_executable_emit_report(report: &NsldFinalExecutableEmitReport) {
    print_nsld_final_executable_report_with_title(report, "Nsld final executable emit");
}

pub(crate) fn print_nsld_final_executable_readiness_report(report: &NsldFinalExecutableEmitReport) {
    print_nsld_final_executable_report_with_title(report, "Nsld final executable readiness");
}

fn print_nsld_final_executable_report_with_title(
    report: &NsldFinalExecutableEmitReport,
    title: &str,
) {
    println!("{title}");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  blocked_report_path: {}", report.blocked_report_path);
    println!("  emitted: {}", report.emitted);
    println!(
        "  can_emit_final_executable: {}",
        report.can_emit_final_executable
    );
    println!("  final_stage_ready: {}", report.final_stage_ready);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  writer_kind: {}", report.writer_kind);
    println!("  writer_status: {}", report.writer_status);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!(
        "  writer_input_valid: {}",
        report
            .writer_input_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  writer_input_hash: {}",
        optional_string_text(report.writer_input_hash.as_deref())
    );
    println!("  input_count: {}", report.input_count);
    println!("  blocker_count: {}", report.blockers.len());
    for blocker in &report.writer_blockers {
        println!("  writer_blocker: {blocker}");
    }
    for issue in &report.writer_input_issues {
        println!("  writer_input_issue: {issue}");
    }
    println!(
        "  host_dry_run_environment_ready: {}",
        report
            .host_dry_run_environment_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  host_dry_run_driver_available: {}",
        report
            .host_dry_run_driver_available
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  host_dry_run_driver_resolved_path: {}",
        optional_string_text(report.host_dry_run_driver_resolved_path.as_deref())
    );
    println!(
        "  host_dry_run_can_invoke: {}",
        report
            .host_dry_run_can_invoke
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  host_dry_run_invocation_policy: {}",
        optional_string_text(report.host_dry_run_invocation_policy.as_deref())
    );
    println!(
        "  host_dry_run_invocation_policy_reason: {}",
        optional_string_text(report.host_dry_run_invocation_policy_reason.as_deref())
    );
    println!(
        "  host_dry_run_command_arg_count: {}",
        report.host_dry_run_command_arg_count
    );
    for arg in &report.host_dry_run_command_args {
        println!("  host_dry_run_command_arg: {arg}");
    }
    for blocker in &report.host_dry_run_blockers {
        println!("  host_dry_run_blocker: {blocker}");
    }
    println!(
        "  host_dry_run_blocker_count: {}",
        report.host_dry_run_blocker_count
    );
    println!("  host_invoke_plan_path: {}", report.host_invoke_plan_path);
    println!(
        "  host_invoke_plan_valid: {}",
        optional_bool_text(report.host_invoke_plan_valid)
    );
    println!(
        "  host_invoke_plan_hash: {}",
        optional_string_text(report.host_invoke_plan_hash.as_deref())
    );
    println!(
        "  host_invoke_plan_invocation_policy: {}",
        optional_string_text(report.host_invoke_plan_invocation_policy.as_deref())
    );
    println!(
        "  host_invoke_plan_requires_explicit_allow: {}",
        optional_bool_text(report.host_invoke_plan_requires_explicit_allow)
    );
    println!(
        "  host_invoke_plan_explicit_allow_present: {}",
        optional_bool_text(report.host_invoke_plan_explicit_allow_present)
    );
    println!(
        "  host_invoke_plan_would_invoke: {}",
        optional_bool_text(report.host_invoke_plan_would_invoke)
    );
    println!(
        "  host_invoke_plan_blocker_count: {}",
        optional_usize_text(report.host_invoke_plan_blocker_count)
    );
    for issue in &report.host_invoke_plan_issues {
        println!("  host_invoke_plan_issue: {issue}");
    }
    println!("  layout_plan_path: {}", report.layout_plan_path);
    println!(
        "  layout_plan_valid: {}",
        optional_bool_text(report.layout_plan_valid)
    );
    println!(
        "  layout_plan_hash: {}",
        optional_string_text(report.layout_plan_hash.as_deref())
    );
    for issue in &report.layout_plan_issues {
        println!("  layout_plan_issue: {issue}");
    }
    println!("  image_dry_run_path: {}", report.image_dry_run_path);
    println!(
        "  image_dry_run_bytes_path: {}",
        report.image_dry_run_bytes_path
    );
    println!(
        "  image_dry_run_valid: {}",
        optional_bool_text(report.image_dry_run_valid)
    );
    println!(
        "  image_dry_run_hash: {}",
        optional_string_text(report.image_dry_run_hash.as_deref())
    );
    println!(
        "  image_dry_run_size_bytes: {}",
        optional_usize_text(report.image_dry_run_size_bytes)
    );
    for issue in &report.image_dry_run_issues {
        println!("  image_dry_run_issue: {issue}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_emit_verify_report(
    report: &NsldFinalExecutableEmitVerifyReport,
) {
    println!("Nsld final executable emit verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_final_stage_plan_hash: {}",
        report.expected_final_stage_plan_hash
    );
    println!(
        "  actual_final_stage_plan_hash: {}",
        optional_string_text(report.actual_final_stage_plan_hash.as_deref())
    );
    println!("  expected_emitted: {}", report.expected_emitted);
    println!(
        "  actual_emitted: {}",
        report
            .actual_emitted
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  expected_writer_input_valid: {}",
        optional_bool_text(report.expected_writer_input_valid)
    );
    println!(
        "  actual_writer_input_valid: {}",
        optional_bool_text(report.actual_writer_input_valid)
    );
    println!(
        "  expected_writer_input_hash: {}",
        optional_string_text(report.expected_writer_input_hash.as_deref())
    );
    println!(
        "  actual_writer_input_hash: {}",
        optional_string_text(report.actual_writer_input_hash.as_deref())
    );
    for issue in &report.expected_writer_input_issues {
        println!("  expected_writer_input_issue: {issue}");
    }
    for issue in &report.actual_writer_input_issues {
        println!("  actual_writer_input_issue: {issue}");
    }
    println!(
        "  expected_host_dry_run_environment_ready: {}",
        optional_bool_text(report.expected_host_dry_run_environment_ready)
    );
    println!(
        "  actual_host_dry_run_environment_ready: {}",
        optional_bool_text(report.actual_host_dry_run_environment_ready)
    );
    println!(
        "  expected_host_dry_run_driver_available: {}",
        optional_bool_text(report.expected_host_dry_run_driver_available)
    );
    println!(
        "  actual_host_dry_run_driver_available: {}",
        optional_bool_text(report.actual_host_dry_run_driver_available)
    );
    println!(
        "  expected_host_dry_run_can_invoke: {}",
        optional_bool_text(report.expected_host_dry_run_can_invoke)
    );
    println!(
        "  actual_host_dry_run_can_invoke: {}",
        optional_bool_text(report.actual_host_dry_run_can_invoke)
    );
    println!(
        "  expected_host_dry_run_driver_resolved_path: {}",
        optional_string_text(report.expected_host_dry_run_driver_resolved_path.as_deref())
    );
    println!(
        "  actual_host_dry_run_driver_resolved_path: {}",
        optional_string_text(report.actual_host_dry_run_driver_resolved_path.as_deref())
    );
    println!(
        "  expected_host_dry_run_invocation_policy: {}",
        optional_string_text(report.expected_host_dry_run_invocation_policy.as_deref())
    );
    println!(
        "  actual_host_dry_run_invocation_policy: {}",
        optional_string_text(report.actual_host_dry_run_invocation_policy.as_deref())
    );
    println!(
        "  expected_host_dry_run_invocation_policy_reason: {}",
        optional_string_text(
            report
                .expected_host_dry_run_invocation_policy_reason
                .as_deref()
        )
    );
    println!(
        "  actual_host_dry_run_invocation_policy_reason: {}",
        optional_string_text(
            report
                .actual_host_dry_run_invocation_policy_reason
                .as_deref()
        )
    );
    println!(
        "  expected_host_dry_run_command_arg_count: {}",
        report.expected_host_dry_run_command_arg_count
    );
    println!(
        "  actual_host_dry_run_command_arg_count: {}",
        optional_usize_text(report.actual_host_dry_run_command_arg_count)
    );
    for arg in &report.expected_host_dry_run_command_args {
        println!("  expected_host_dry_run_command_arg: {arg}");
    }
    for arg in &report.actual_host_dry_run_command_args {
        println!("  actual_host_dry_run_command_arg: {arg}");
    }
    println!(
        "  expected_host_dry_run_blocker_count: {}",
        report.expected_host_dry_run_blocker_count
    );
    println!(
        "  actual_host_dry_run_blocker_count: {}",
        optional_usize_text(report.actual_host_dry_run_blocker_count)
    );
    for blocker in &report.expected_host_dry_run_blockers {
        println!("  expected_host_dry_run_blocker: {blocker}");
    }
    for blocker in &report.actual_host_dry_run_blockers {
        println!("  actual_host_dry_run_blocker: {blocker}");
    }
    println!(
        "  expected_host_invoke_plan_valid: {}",
        optional_bool_text(report.expected_host_invoke_plan_valid)
    );
    println!(
        "  actual_host_invoke_plan_valid: {}",
        optional_bool_text(report.actual_host_invoke_plan_valid)
    );
    println!(
        "  expected_host_invoke_plan_would_invoke: {}",
        optional_bool_text(report.expected_host_invoke_plan_would_invoke)
    );
    println!(
        "  actual_host_invoke_plan_would_invoke: {}",
        optional_bool_text(report.actual_host_invoke_plan_would_invoke)
    );
    println!(
        "  expected_host_invoke_plan_hash: {}",
        optional_string_text(report.expected_host_invoke_plan_hash.as_deref())
    );
    println!(
        "  actual_host_invoke_plan_hash: {}",
        optional_string_text(report.actual_host_invoke_plan_hash.as_deref())
    );
    println!(
        "  expected_host_invoke_plan_invocation_policy: {}",
        optional_string_text(
            report
                .expected_host_invoke_plan_invocation_policy
                .as_deref()
        )
    );
    println!(
        "  actual_host_invoke_plan_invocation_policy: {}",
        optional_string_text(report.actual_host_invoke_plan_invocation_policy.as_deref())
    );
    println!(
        "  expected_host_invoke_plan_requires_explicit_allow: {}",
        optional_bool_text(report.expected_host_invoke_plan_requires_explicit_allow)
    );
    println!(
        "  actual_host_invoke_plan_requires_explicit_allow: {}",
        optional_bool_text(report.actual_host_invoke_plan_requires_explicit_allow)
    );
    println!(
        "  expected_host_invoke_plan_explicit_allow_present: {}",
        optional_bool_text(report.expected_host_invoke_plan_explicit_allow_present)
    );
    println!(
        "  actual_host_invoke_plan_explicit_allow_present: {}",
        optional_bool_text(report.actual_host_invoke_plan_explicit_allow_present)
    );
    println!(
        "  expected_host_invoke_plan_blocker_count: {}",
        optional_usize_text(report.expected_host_invoke_plan_blocker_count)
    );
    println!(
        "  actual_host_invoke_plan_blocker_count: {}",
        optional_usize_text(report.actual_host_invoke_plan_blocker_count)
    );
    for issue in &report.expected_host_invoke_plan_issues {
        println!("  expected_host_invoke_plan_issue: {issue}");
    }
    for issue in &report.actual_host_invoke_plan_issues {
        println!("  actual_host_invoke_plan_issue: {issue}");
    }
    println!(
        "  expected_layout_plan_valid: {}",
        optional_bool_text(report.expected_layout_plan_valid)
    );
    println!(
        "  actual_layout_plan_valid: {}",
        optional_bool_text(report.actual_layout_plan_valid)
    );
    println!(
        "  expected_layout_plan_hash: {}",
        optional_string_text(report.expected_layout_plan_hash.as_deref())
    );
    println!(
        "  actual_layout_plan_hash: {}",
        optional_string_text(report.actual_layout_plan_hash.as_deref())
    );
    for issue in &report.expected_layout_plan_issues {
        println!("  expected_layout_plan_issue: {issue}");
    }
    for issue in &report.actual_layout_plan_issues {
        println!("  actual_layout_plan_issue: {issue}");
    }
    println!(
        "  expected_image_dry_run_valid: {}",
        optional_bool_text(report.expected_image_dry_run_valid)
    );
    println!(
        "  actual_image_dry_run_valid: {}",
        optional_bool_text(report.actual_image_dry_run_valid)
    );
    println!(
        "  expected_image_dry_run_hash: {}",
        optional_string_text(report.expected_image_dry_run_hash.as_deref())
    );
    println!(
        "  actual_image_dry_run_hash: {}",
        optional_string_text(report.actual_image_dry_run_hash.as_deref())
    );
    println!(
        "  expected_image_dry_run_size_bytes: {}",
        optional_usize_text(report.expected_image_dry_run_size_bytes)
    );
    println!(
        "  actual_image_dry_run_size_bytes: {}",
        optional_usize_text(report.actual_image_dry_run_size_bytes)
    );
    for issue in &report.expected_image_dry_run_issues {
        println!("  expected_image_dry_run_issue: {issue}");
    }
    for issue in &report.actual_image_dry_run_issues {
        println!("  actual_image_dry_run_issue: {issue}");
    }
    println!(
        "  expected_blocker_count: {}",
        report.expected_blocker_count
    );
    println!(
        "  actual_blocker_count: {}",
        optional_usize_text(report.actual_blocker_count)
    );
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

pub(crate) fn print_nsld_final_executable_output_report(report: &NsldFinalExecutableOutputReport) {
    println!("Nsld final executable output");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  present: {}", report.present);
    println!("  size_bytes: {}", optional_usize_text(report.size_bytes));
    println!(
        "  output_hash: {}",
        optional_string_text(report.output_hash.as_deref())
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
    println!("  runnable_candidate: {}", report.runnable_candidate);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
