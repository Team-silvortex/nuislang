use super::{display_text::*, reports::*};

pub(crate) fn print_nsld_final_executable_writer_plan_report(
    report: &NsldFinalExecutableWriterPlanReport,
) {
    println!("Nsld final executable writer plan");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_kind: {}", report.writer_kind);
    println!("  writer_status: {}", report.writer_status);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  input_count: {}", report.input_count);
    for input in &report.inputs {
        println!(
            "  writer_input: order={} id={} kind={} required={} present={} hash={} path={}",
            input.order_index,
            input.input_id,
            input.input_kind,
            input.required,
            input.present,
            input.content_hash,
            input.path
        );
    }
    for step in &report.writer_steps {
        println!("  writer_step: {step}");
    }
    for blocker in &report.writer_blockers {
        println!("  writer_blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_writer_input_emit_report(
    report: &NsldFinalExecutableWriterInputEmitReport,
) {
    println!("Nsld final executable writer input emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_input_hash: {}", report.writer_input_hash);
    println!("  writer_kind: {}", report.writer_kind);
    println!("  writer_status: {}", report.writer_status);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  command_arg_count: {}", report.command_arg_count);
    for blocker in &report.writer_blockers {
        println!("  writer_blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_final_executable_writer_input_verify_report(
    report: &NsldFinalExecutableWriterInputVerifyReport,
) {
    println!("Nsld final executable writer input verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_writer_input_hash: {}",
        report.expected_writer_input_hash
    );
    println!(
        "  actual_writer_input_hash: {}",
        optional_string_text(report.actual_writer_input_hash.as_deref())
    );
    println!(
        "  expected_final_stage_plan_hash: {}",
        report.expected_final_stage_plan_hash
    );
    println!(
        "  actual_final_stage_plan_hash: {}",
        optional_string_text(report.actual_final_stage_plan_hash.as_deref())
    );
    println!("  expected_writer_kind: {}", report.expected_writer_kind);
    println!(
        "  actual_writer_kind: {}",
        optional_string_text(report.actual_writer_kind.as_deref())
    );
    println!(
        "  expected_writer_status: {}",
        report.expected_writer_status
    );
    println!(
        "  actual_writer_status: {}",
        optional_string_text(report.actual_writer_status.as_deref())
    );
    println!(
        "  expected_command_arg_count: {}",
        report.expected_command_arg_count
    );
    println!(
        "  actual_command_arg_count: {}",
        optional_usize_text(report.actual_command_arg_count)
    );
    for arg in &report.expected_command_args {
        println!("  expected_command_arg: {arg}");
    }
    for arg in &report.actual_command_args {
        println!("  actual_command_arg: {arg}");
    }
    for blocker in &report.expected_writer_blockers {
        println!("  expected_writer_blocker: {blocker}");
    }
    for blocker in &report.actual_writer_blockers {
        println!("  actual_writer_blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_host_dry_run_report(
    report: &NsldFinalExecutableHostDryRunReport,
) {
    println!("Nsld final executable host dry run");
    println!("  manifest: {}", report.manifest);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!("  writer_input_valid: {}", report.writer_input_valid);
    println!(
        "  writer_input_hash: {}",
        optional_string_text(report.writer_input_hash.as_deref())
    );
    println!("  driver: {}", report.driver);
    println!("  driver_available: {}", report.driver_available);
    println!(
        "  driver_resolved_path: {}",
        optional_string_text(report.driver_resolved_path.as_deref())
    );
    println!("  command_arg_count: {}", report.command_arg_count);
    println!("  environment_ready: {}", report.environment_ready);
    println!("  invocation_policy: {}", report.invocation_policy);
    println!(
        "  invocation_policy_reason: {}",
        report.invocation_policy_reason
    );
    println!(
        "  can_invoke_host_finalizer: {}",
        report.can_invoke_host_finalizer
    );
    for arg in &report.command_args {
        println!("  command_arg: {arg}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_host_invoke_plan_report(
    report: &NsldFinalExecutableHostInvokePlanReport,
) {
    println!("Nsld final executable host invoke plan");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!("  invocation_kind: {}", report.invocation_kind);
    println!("  invocation_policy: {}", report.invocation_policy);
    println!(
        "  invocation_policy_reason: {}",
        report.invocation_policy_reason
    );
    println!(
        "  requires_explicit_allow: {}",
        report.requires_explicit_allow
    );
    println!(
        "  explicit_allow_present: {}",
        report.explicit_allow_present
    );
    println!("  environment_ready: {}", report.environment_ready);
    println!("  driver_available: {}", report.driver_available);
    println!(
        "  driver_resolved_path: {}",
        optional_string_text(report.driver_resolved_path.as_deref())
    );
    println!(
        "  can_invoke_host_finalizer: {}",
        report.can_invoke_host_finalizer
    );
    println!("  would_invoke: {}", report.would_invoke);
    println!("  command_arg_count: {}", report.command_arg_count);
    for arg in &report.command_args {
        println!("  command_arg: {arg}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_host_invoke_plan_emit_report(
    report: &NsldFinalExecutableHostInvokePlanEmitReport,
) {
    println!("Nsld final executable host invoke plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  invoke_plan_hash: {}", report.invoke_plan_hash);
    println!("  invocation_policy: {}", report.invocation_policy);
    println!(
        "  requires_explicit_allow: {}",
        report.requires_explicit_allow
    );
    println!(
        "  explicit_allow_present: {}",
        report.explicit_allow_present
    );
    println!("  would_invoke: {}", report.would_invoke);
    println!("  blocker_count: {}", report.blocker_count);
}

pub(crate) fn print_nsld_final_executable_host_invoke_plan_verify_report(
    report: &NsldFinalExecutableHostInvokePlanVerifyReport,
) {
    println!("Nsld final executable host invoke plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_invoke_plan_hash: {}",
        report.expected_invoke_plan_hash
    );
    println!(
        "  actual_invoke_plan_hash: {}",
        optional_string_text(report.actual_invoke_plan_hash.as_deref())
    );
    println!(
        "  expected_invocation_policy: {}",
        report.expected_invocation_policy
    );
    println!(
        "  actual_invocation_policy: {}",
        optional_string_text(report.actual_invocation_policy.as_deref())
    );
    println!(
        "  expected_requires_explicit_allow: {}",
        report.expected_requires_explicit_allow
    );
    println!(
        "  actual_requires_explicit_allow: {}",
        optional_bool_text(report.actual_requires_explicit_allow)
    );
    println!(
        "  expected_explicit_allow_present: {}",
        report.expected_explicit_allow_present
    );
    println!(
        "  actual_explicit_allow_present: {}",
        optional_bool_text(report.actual_explicit_allow_present)
    );
    println!("  expected_would_invoke: {}", report.expected_would_invoke);
    println!(
        "  actual_would_invoke: {}",
        optional_bool_text(report.actual_would_invoke)
    );
    println!(
        "  expected_command_arg_count: {}",
        report.expected_command_arg_count
    );
    println!(
        "  actual_command_arg_count: {}",
        optional_usize_text(report.actual_command_arg_count)
    );
    for arg in &report.expected_command_args {
        println!("  expected_command_arg: {arg}");
    }
    for arg in &report.actual_command_args {
        println!("  actual_command_arg: {arg}");
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
