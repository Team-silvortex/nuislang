use super::{
    final_executable_paths::{
        nsld_final_executable_blocked_path, nsld_final_executable_host_invoke_plan_path,
        nsld_final_executable_image_dry_run_bytes_path, nsld_final_executable_image_dry_run_path,
        nsld_final_executable_layout_plan_path, nsld_final_executable_writer_input_path,
    },
    final_executable_writer::{final_executable_writer_blockers, final_executable_writer_steps},
    final_stage::nsld_final_stage_plan_report,
    reports::{NsldFinalExecutableEmitReport, NsldFinalExecutableWriterPlanReport},
};
use std::path::Path;

pub(crate) fn nsld_final_executable_readiness_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitReport {
    let final_stage = nsld_final_stage_plan_report(manifest, plan);
    let mut blockers = final_stage.blockers.clone();
    let writer_kind = if final_stage.host_wrapper_required {
        "host-assisted-final-executable"
    } else {
        "self-contained-final-executable"
    }
    .to_owned();
    let writer_blockers = final_executable_writer_blockers(&final_stage);
    let writer_status = if final_stage.ready && writer_blockers.is_empty() {
        "ready"
    } else {
        "blocked"
    }
    .to_owned();
    blockers.extend(writer_blockers.iter().cloned());
    let emitted = false;
    let can_emit_final_executable = blockers.is_empty();
    let mut notes = final_stage.notes.clone();
    notes.push("final-executable-emit-is-contract-only".to_owned());
    if final_stage.host_wrapper_required {
        notes.push("host-wrapper-remains-cffi-compatibility-domain".to_owned());
    }

    NsldFinalExecutableEmitReport {
        manifest: final_stage.manifest,
        output_path: final_stage.final_output_path,
        blocked_report_path: nsld_final_executable_blocked_path(plan)
            .display()
            .to_string(),
        emitted,
        can_emit_final_executable,
        final_stage_ready: final_stage.ready,
        final_stage_plan_hash: final_stage.plan_hash,
        final_stage_driver: final_stage.final_stage_driver,
        final_stage_link_mode: final_stage.final_stage_link_mode,
        host_wrapper_required: final_stage.host_wrapper_required,
        writer_kind,
        writer_status,
        writer_blockers,
        writer_input_path: nsld_final_executable_writer_input_path(plan)
            .display()
            .to_string(),
        writer_input_valid: None,
        writer_input_hash: None,
        writer_input_issues: Vec::new(),
        host_dry_run_environment_ready: None,
        host_dry_run_driver_available: None,
        host_dry_run_driver_resolved_path: None,
        host_dry_run_can_invoke: None,
        host_dry_run_invocation_policy: None,
        host_dry_run_invocation_policy_reason: None,
        host_dry_run_command_arg_count: 0,
        host_dry_run_command_args: Vec::new(),
        host_dry_run_blocker_count: 0,
        host_dry_run_blockers: Vec::new(),
        host_invoke_plan_path: nsld_final_executable_host_invoke_plan_path(plan)
            .display()
            .to_string(),
        host_invoke_plan_valid: None,
        host_invoke_plan_hash: None,
        host_invoke_plan_invocation_policy: None,
        host_invoke_plan_requires_explicit_allow: None,
        host_invoke_plan_explicit_allow_present: None,
        host_invoke_plan_would_invoke: None,
        host_invoke_plan_blocker_count: None,
        host_invoke_plan_issues: Vec::new(),
        layout_plan_path: nsld_final_executable_layout_plan_path(plan)
            .display()
            .to_string(),
        layout_plan_valid: None,
        layout_plan_hash: None,
        layout_plan_issues: Vec::new(),
        image_dry_run_path: nsld_final_executable_image_dry_run_path(plan)
            .display()
            .to_string(),
        image_dry_run_bytes_path: nsld_final_executable_image_dry_run_bytes_path(plan)
            .display()
            .to_string(),
        image_dry_run_valid: None,
        image_dry_run_hash: None,
        image_dry_run_size_bytes: None,
        image_dry_run_issues: Vec::new(),
        input_count: final_stage.input_count,
        blockers,
        notes,
    }
}

pub(crate) fn nsld_final_executable_writer_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableWriterPlanReport {
    let final_stage = nsld_final_stage_plan_report(manifest, plan);
    let writer_kind = if final_stage.host_wrapper_required {
        "host-assisted-final-executable"
    } else {
        "self-contained-final-executable"
    }
    .to_owned();
    let writer_blockers = final_executable_writer_blockers(&final_stage);
    let writer_status = if final_stage.ready && writer_blockers.is_empty() {
        "ready"
    } else {
        "blocked"
    }
    .to_owned();
    let writer_steps = final_executable_writer_steps(&final_stage);
    let mut notes = final_stage.notes.clone();
    notes.push("final-executable-writer-plan-is-non-mutating".to_owned());

    NsldFinalExecutableWriterPlanReport {
        manifest: final_stage.manifest,
        output_path: final_stage.final_output_path,
        writer_kind,
        writer_status,
        final_stage_plan_hash: final_stage.plan_hash,
        final_stage_driver: final_stage.final_stage_driver,
        final_stage_link_mode: final_stage.final_stage_link_mode,
        host_wrapper_required: final_stage.host_wrapper_required,
        input_count: final_stage.input_count,
        inputs: final_stage.inputs,
        writer_steps,
        writer_blockers,
        notes,
    }
}
