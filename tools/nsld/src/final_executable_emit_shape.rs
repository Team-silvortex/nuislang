use super::{
    final_executable_summary::nsld_final_executable_readiness_report,
    final_stage::{
        nsld_final_executable_host_dry_run_report,
        nsld_verify_final_executable_host_invoke_plan_report,
        nsld_verify_final_executable_image_dry_run_report,
        nsld_verify_final_executable_layout_plan_report,
        nsld_verify_final_executable_writer_input_report,
    },
    reports::NsldFinalExecutableEmitReport,
};
use std::path::Path;

pub(crate) fn nsld_final_executable_emit_report_shape(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitReport {
    let mut report = nsld_final_executable_readiness_report(manifest, plan);
    let writer_input = nsld_verify_final_executable_writer_input_report(manifest, plan);
    let host_dry_run = nsld_final_executable_host_dry_run_report(manifest, plan);
    let host_invoke_plan = nsld_verify_final_executable_host_invoke_plan_report(manifest, plan);
    let layout_plan = nsld_verify_final_executable_layout_plan_report(manifest, plan);
    let image_dry_run = nsld_verify_final_executable_image_dry_run_report(manifest, plan);
    report.writer_input_path = writer_input.input_path;
    report.writer_input_valid = Some(writer_input.valid);
    report.writer_input_hash = writer_input.actual_writer_input_hash;
    report.writer_input_issues = writer_input.issues;
    report.host_dry_run_environment_ready = Some(host_dry_run.environment_ready);
    report.host_dry_run_driver_available = Some(host_dry_run.driver_available);
    report.host_dry_run_driver_resolved_path = host_dry_run.driver_resolved_path;
    report.host_dry_run_can_invoke = Some(host_dry_run.can_invoke_host_finalizer);
    report.host_dry_run_invocation_policy = Some(host_dry_run.invocation_policy);
    report.host_dry_run_invocation_policy_reason = Some(host_dry_run.invocation_policy_reason);
    report.host_dry_run_command_arg_count = host_dry_run.command_args.len();
    report.host_dry_run_command_args = host_dry_run.command_args;
    report.host_dry_run_blocker_count = host_dry_run.blockers.len();
    report.host_dry_run_blockers = host_dry_run.blockers;
    report.host_invoke_plan_path = host_invoke_plan.input_path;
    report.host_invoke_plan_valid = Some(host_invoke_plan.valid);
    report.host_invoke_plan_hash = host_invoke_plan.actual_invoke_plan_hash;
    report.host_invoke_plan_invocation_policy = host_invoke_plan.actual_invocation_policy;
    report.host_invoke_plan_requires_explicit_allow = Some(
        host_invoke_plan
            .actual_requires_explicit_allow
            .unwrap_or(false),
    );
    report.host_invoke_plan_explicit_allow_present = Some(
        host_invoke_plan
            .actual_explicit_allow_present
            .unwrap_or(false),
    );
    report.host_invoke_plan_would_invoke =
        Some(host_invoke_plan.actual_would_invoke.unwrap_or(false));
    report.host_invoke_plan_blocker_count =
        Some(host_invoke_plan.actual_blocker_count.unwrap_or(0));
    report.host_invoke_plan_issues = host_invoke_plan.issues;
    report.layout_plan_path = layout_plan.input_path;
    report.layout_plan_valid = Some(layout_plan.valid);
    report.layout_plan_hash = layout_plan.actual_layout_hash;
    report.layout_plan_issues = layout_plan.issues;
    report.image_dry_run_path = image_dry_run.input_path;
    report.image_dry_run_bytes_path = image_dry_run.image_path;
    report.image_dry_run_valid = Some(image_dry_run.valid);
    report.image_dry_run_hash = image_dry_run.actual_image_hash;
    report.image_dry_run_size_bytes = image_dry_run.actual_image_size_bytes;
    report.image_dry_run_issues = image_dry_run.issues;
    if !writer_input.valid {
        report
            .blockers
            .push("final-executable-writer-input:invalid".to_owned());
        report.blockers.extend(
            report
                .writer_input_issues
                .iter()
                .map(|issue| format!("final-executable-writer-input:{issue}")),
        );
        report.can_emit_final_executable = false;
    }
    if !host_dry_run.environment_ready && report.host_wrapper_required {
        report
            .blockers
            .push("host-finalizer-environment:not-ready".to_owned());
        report.blockers.extend(
            report
                .host_dry_run_blockers
                .iter()
                .map(|blocker| format!("host-finalizer-dry-run:{blocker}")),
        );
        report.can_emit_final_executable = false;
    }
    if report.host_wrapper_required && !host_invoke_plan.valid {
        report
            .blockers
            .push("host-finalizer-invoke-plan:invalid".to_owned());
        report.blockers.extend(
            report
                .host_invoke_plan_issues
                .iter()
                .map(|issue| format!("host-finalizer-invoke-plan:{issue}")),
        );
        report.can_emit_final_executable = false;
    }
    if report.host_wrapper_required && host_invoke_plan.actual_would_invoke != Some(true) {
        report
            .blockers
            .push("host-finalizer-invoke-plan:not-allowed".to_owned());
        report.can_emit_final_executable = false;
    }
    if !layout_plan.valid {
        report
            .blockers
            .push("final-executable-layout-plan:invalid".to_owned());
        report.blockers.extend(
            report
                .layout_plan_issues
                .iter()
                .map(|issue| format!("final-executable-layout-plan:{issue}")),
        );
        report.can_emit_final_executable = false;
    }
    if !image_dry_run.valid {
        report
            .blockers
            .push("final-executable-image-dry-run:invalid".to_owned());
        report.blockers.extend(
            report
                .image_dry_run_issues
                .iter()
                .map(|issue| format!("final-executable-image-dry-run:{issue}")),
        );
        report.can_emit_final_executable = false;
    }
    report.emitted = report.can_emit_final_executable;
    report
}
