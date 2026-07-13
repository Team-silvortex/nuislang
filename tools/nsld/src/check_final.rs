use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    check_final_tail::{
        nsld_check_final_tail_snapshot, push_final_tail_snapshot_issues, NsldCheckFinalTailSnapshot,
    },
    final_stage::{
        nsld_final_executable_output_report, nsld_final_stage_plan_report,
        nsld_verify_final_executable_emit_report,
        nsld_verify_final_executable_host_invoke_plan_report,
        nsld_verify_final_executable_image_dry_run_report,
        nsld_verify_final_executable_layout_plan_report,
        nsld_verify_final_executable_writer_input_report, nsld_verify_final_stage_plan_report,
    },
};
use std::path::Path;

pub(crate) struct NsldCheckFinalSnapshot {
    pub(crate) final_stage_plan_present: bool,
    pub(crate) final_stage_plan_valid: Option<bool>,
    pub(crate) final_stage_plan_ready: Option<bool>,
    pub(crate) final_stage_plan_hash: Option<String>,
    pub(crate) final_stage_plan_blocker_count: Option<usize>,
    pub(crate) final_stage_plan_issues: Vec<String>,
    pub(crate) final_executable_writer_input_present: bool,
    pub(crate) final_executable_writer_input_valid: Option<bool>,
    pub(crate) final_executable_writer_input_hash: Option<String>,
    pub(crate) final_executable_writer_input_command_arg_count: Option<usize>,
    pub(crate) final_executable_writer_input_issues: Vec<String>,
    pub(crate) final_executable_host_invoke_plan_present: bool,
    pub(crate) final_executable_host_invoke_plan_valid: Option<bool>,
    pub(crate) final_executable_host_invoke_plan_hash: Option<String>,
    pub(crate) final_executable_host_invoke_plan_invocation_policy: Option<String>,
    pub(crate) final_executable_host_invoke_plan_requires_explicit_allow: Option<bool>,
    pub(crate) final_executable_host_invoke_plan_explicit_allow_present: Option<bool>,
    pub(crate) final_executable_host_invoke_plan_would_invoke: Option<bool>,
    pub(crate) final_executable_host_invoke_plan_blocker_count: Option<usize>,
    pub(crate) final_executable_host_invoke_plan_issues: Vec<String>,
    pub(crate) final_executable_layout_plan_present: bool,
    pub(crate) final_executable_layout_plan_valid: Option<bool>,
    pub(crate) final_executable_layout_plan_hash: Option<String>,
    pub(crate) final_executable_layout_plan_payload_count: Option<usize>,
    pub(crate) final_executable_layout_plan_issues: Vec<String>,
    pub(crate) final_executable_image_dry_run_present: bool,
    pub(crate) final_executable_image_dry_run_valid: Option<bool>,
    pub(crate) final_executable_image_dry_run_hash: Option<String>,
    pub(crate) final_executable_image_dry_run_size_bytes: Option<usize>,
    pub(crate) final_executable_image_dry_run_issues: Vec<String>,
    pub(crate) final_executable_blocked_present: bool,
    pub(crate) final_executable_blocked_valid: Option<bool>,
    pub(crate) final_executable_blocked_emitted: Option<bool>,
    pub(crate) final_executable_blocked_plan_hash: Option<String>,
    pub(crate) final_executable_blocked_blocker_count: Option<usize>,
    pub(crate) final_executable_blocked_issues: Vec<String>,
    pub(crate) final_executable_output_path_present: bool,
    pub(crate) final_executable_output_kind: String,
    pub(crate) final_executable_output_validation_mode: String,
    pub(crate) final_executable_output_boundary_status: String,
    pub(crate) final_executable_output_materialization_status: String,
    pub(crate) final_executable_output_execution_handoff_status: String,
    pub(crate) final_executable_output_execution_handoff_target: String,
    pub(crate) final_executable_output_execution_handoff_evidence_status: String,
    pub(crate) final_executable_output_recommended_next_action: String,
    pub(crate) final_executable_output_nsld_owned: bool,
    pub(crate) final_executable_output_present: bool,
    pub(crate) final_executable_output_size_bytes: Option<usize>,
    pub(crate) final_executable_output_hash: Option<String>,
    pub(crate) final_executable_output_image_header_required: Option<bool>,
    pub(crate) final_executable_output_image_header_valid: Option<bool>,
    pub(crate) final_executable_output_image_magic: Option<String>,
    pub(crate) final_executable_output_image_version: Option<usize>,
    pub(crate) final_executable_output_image_layout_hash: Option<String>,
    pub(crate) final_executable_output_image_byte_map_hash: Option<String>,
    pub(crate) final_executable_output_runnable_candidate: Option<bool>,
    pub(crate) final_executable_output_blocker_count: Option<usize>,
    pub(crate) final_executable_output_blockers: Vec<String>,
    pub(crate) final_executable_output_issues: Vec<String>,
    pub(crate) tail: NsldCheckFinalTailSnapshot,
}

pub(crate) fn nsld_check_final_snapshot(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldCheckFinalSnapshot {
    let final_stage_plan_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::FinalStagePlan);
    let final_stage_plan_present = final_stage_plan_path.exists();
    let final_stage_plan_verify_report =
        final_stage_plan_present.then(|| nsld_verify_final_stage_plan_report(manifest, plan));
    let final_stage_plan_valid = final_stage_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_stage_plan_issues = final_stage_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let expected_final_stage_plan_report =
        final_stage_plan_present.then(|| nsld_final_stage_plan_report(manifest, plan));
    let final_stage_plan_ready = expected_final_stage_plan_report
        .as_ref()
        .map(|report| report.ready);
    let final_stage_plan_hash = expected_final_stage_plan_report
        .as_ref()
        .map(|report| report.plan_hash.clone());
    let final_stage_plan_blocker_count = expected_final_stage_plan_report
        .as_ref()
        .map(|report| report.blockers.len());
    let final_executable_writer_input_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableWriterInput,
    );
    let final_executable_writer_input_present = final_executable_writer_input_path.exists();
    let final_executable_writer_input_verify_report = final_executable_writer_input_present
        .then(|| nsld_verify_final_executable_writer_input_report(manifest, plan));
    let final_executable_writer_input_valid = final_executable_writer_input_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_writer_input_hash = final_executable_writer_input_verify_report
        .as_ref()
        .and_then(|report| report.actual_writer_input_hash.clone());
    let final_executable_writer_input_command_arg_count =
        final_executable_writer_input_verify_report
            .as_ref()
            .and_then(|report| report.actual_command_arg_count);
    let final_executable_writer_input_issues = final_executable_writer_input_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let final_executable_host_invoke_plan_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableHostInvokePlan,
    );
    let final_executable_host_invoke_plan_present = final_executable_host_invoke_plan_path.exists();
    let final_executable_host_invoke_plan_verify_report = final_executable_host_invoke_plan_present
        .then(|| nsld_verify_final_executable_host_invoke_plan_report(manifest, plan));
    let final_executable_host_invoke_plan_valid = final_executable_host_invoke_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_host_invoke_plan_hash = final_executable_host_invoke_plan_verify_report
        .as_ref()
        .and_then(|report| report.actual_invoke_plan_hash.clone());
    let final_executable_host_invoke_plan_invocation_policy =
        final_executable_host_invoke_plan_verify_report
            .as_ref()
            .and_then(|report| report.actual_invocation_policy.clone());
    let final_executable_host_invoke_plan_requires_explicit_allow =
        final_executable_host_invoke_plan_verify_report
            .as_ref()
            .and_then(|report| report.actual_requires_explicit_allow);
    let final_executable_host_invoke_plan_explicit_allow_present =
        final_executable_host_invoke_plan_verify_report
            .as_ref()
            .and_then(|report| report.actual_explicit_allow_present);
    let final_executable_host_invoke_plan_would_invoke =
        final_executable_host_invoke_plan_verify_report
            .as_ref()
            .and_then(|report| report.actual_would_invoke);
    let final_executable_host_invoke_plan_blocker_count =
        final_executable_host_invoke_plan_verify_report
            .as_ref()
            .and_then(|report| report.actual_blocker_count);
    let final_executable_host_invoke_plan_issues = final_executable_host_invoke_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let final_executable_layout_plan_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableLayoutPlan,
    );
    let final_executable_layout_plan_present = final_executable_layout_plan_path.exists();
    let final_executable_layout_plan_verify_report = final_executable_layout_plan_present
        .then(|| nsld_verify_final_executable_layout_plan_report(manifest, plan));
    let final_executable_layout_plan_valid = final_executable_layout_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_layout_plan_hash = final_executable_layout_plan_verify_report
        .as_ref()
        .and_then(|report| report.actual_layout_hash.clone());
    let final_executable_layout_plan_payload_count = final_executable_layout_plan_verify_report
        .as_ref()
        .and_then(|report| report.actual_payload_count);
    let final_executable_layout_plan_issues = final_executable_layout_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let final_executable_image_dry_run_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableImageDryRun,
    );
    let final_executable_image_dry_run_present = final_executable_image_dry_run_path.exists();
    let final_executable_image_dry_run_verify_report = final_executable_image_dry_run_present
        .then(|| nsld_verify_final_executable_image_dry_run_report(manifest, plan));
    let final_executable_image_dry_run_valid = final_executable_image_dry_run_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_image_dry_run_hash = final_executable_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_image_hash.clone());
    let final_executable_image_dry_run_size_bytes = final_executable_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_image_size_bytes);
    let final_executable_image_dry_run_issues = final_executable_image_dry_run_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let final_executable_blocked_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableBlocked,
    );
    let final_executable_blocked_present = final_executable_blocked_path.exists();
    let final_executable_blocked_verify_report = final_executable_blocked_present
        .then(|| nsld_verify_final_executable_emit_report(manifest, plan));
    let final_executable_blocked_valid = final_executable_blocked_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_blocked_emitted = final_executable_blocked_verify_report
        .as_ref()
        .and_then(|report| report.actual_emitted);
    let final_executable_blocked_plan_hash = final_executable_blocked_verify_report
        .as_ref()
        .and_then(|report| report.actual_final_stage_plan_hash.clone());
    let final_executable_blocked_blocker_count = final_executable_blocked_verify_report
        .as_ref()
        .and_then(|report| report.actual_blocker_count);
    let final_executable_blocked_issues = final_executable_blocked_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let final_executable_output_report = nsld_final_executable_output_report(manifest, plan);
    let final_executable_output_path_present = final_executable_output_report.path_present;
    let final_executable_output_kind = final_executable_output_report.output_kind.clone();
    let final_executable_output_validation_mode = final_executable_output_report
        .output_validation_mode
        .clone();
    let final_executable_output_boundary_status =
        final_executable_output_report.boundary_status.clone();
    let final_executable_output_materialization_status = final_executable_output_report
        .materialization_status
        .clone();
    let final_executable_output_execution_handoff_status = final_executable_output_report
        .execution_handoff_status
        .clone();
    let final_executable_output_execution_handoff_target = final_executable_output_report
        .execution_handoff_target
        .clone();
    let final_executable_output_execution_handoff_evidence_status = final_executable_output_report
        .execution_handoff_evidence_status
        .clone();
    let final_executable_output_recommended_next_action = final_executable_output_report
        .recommended_next_action
        .clone();
    let final_executable_output_nsld_owned = final_executable_output_report.nsld_owned_output;
    let final_executable_output_present = final_executable_output_report.present;
    let final_executable_output_size_bytes = final_executable_output_report.size_bytes;
    let final_executable_output_hash = final_executable_output_report.output_hash.clone();
    let final_executable_output_image_header_required =
        Some(final_executable_output_report.output_image_header_required);
    let final_executable_output_image_header_valid =
        Some(final_executable_output_report.output_image_header_valid);
    let final_executable_output_image_magic =
        final_executable_output_report.output_image_magic.clone();
    let final_executable_output_image_version = final_executable_output_report.output_image_version;
    let final_executable_output_image_layout_hash =
        final_executable_output_report.output_layout_hash.clone();
    let final_executable_output_image_byte_map_hash =
        final_executable_output_report.output_byte_map_hash.clone();
    let final_executable_output_runnable_candidate =
        Some(final_executable_output_report.runnable_candidate);
    let final_executable_output_blocker_count = Some(final_executable_output_report.blockers.len());
    let final_executable_output_blockers = final_executable_output_report.blockers.clone();
    let mut final_executable_output_issues = final_executable_output_report.issues.clone();
    if final_executable_output_report.present && !final_executable_output_report.runnable_candidate
    {
        final_executable_output_issues.extend(
            final_executable_output_report
                .blockers
                .iter()
                .map(|blocker| format!("final-executable-output:{blocker}")),
        );
    }
    let tail = nsld_check_final_tail_snapshot(manifest, plan);

    NsldCheckFinalSnapshot {
        final_stage_plan_present,
        final_stage_plan_valid,
        final_stage_plan_ready,
        final_stage_plan_hash,
        final_stage_plan_blocker_count,
        final_stage_plan_issues,
        final_executable_writer_input_present,
        final_executable_writer_input_valid,
        final_executable_writer_input_hash,
        final_executable_writer_input_command_arg_count,
        final_executable_writer_input_issues,
        final_executable_host_invoke_plan_present,
        final_executable_host_invoke_plan_valid,
        final_executable_host_invoke_plan_hash,
        final_executable_host_invoke_plan_invocation_policy,
        final_executable_host_invoke_plan_requires_explicit_allow,
        final_executable_host_invoke_plan_explicit_allow_present,
        final_executable_host_invoke_plan_would_invoke,
        final_executable_host_invoke_plan_blocker_count,
        final_executable_host_invoke_plan_issues,
        final_executable_layout_plan_present,
        final_executable_layout_plan_valid,
        final_executable_layout_plan_hash,
        final_executable_layout_plan_payload_count,
        final_executable_layout_plan_issues,
        final_executable_image_dry_run_present,
        final_executable_image_dry_run_valid,
        final_executable_image_dry_run_hash,
        final_executable_image_dry_run_size_bytes,
        final_executable_image_dry_run_issues,
        final_executable_blocked_present,
        final_executable_blocked_valid,
        final_executable_blocked_emitted,
        final_executable_blocked_plan_hash,
        final_executable_blocked_blocker_count,
        final_executable_blocked_issues,
        final_executable_output_path_present,
        final_executable_output_kind,
        final_executable_output_validation_mode,
        final_executable_output_boundary_status,
        final_executable_output_materialization_status,
        final_executable_output_execution_handoff_status,
        final_executable_output_execution_handoff_target,
        final_executable_output_execution_handoff_evidence_status,
        final_executable_output_recommended_next_action,
        final_executable_output_nsld_owned,
        final_executable_output_present,
        final_executable_output_size_bytes,
        final_executable_output_hash,
        final_executable_output_image_header_required,
        final_executable_output_image_header_valid,
        final_executable_output_image_magic,
        final_executable_output_image_version,
        final_executable_output_image_layout_hash,
        final_executable_output_image_byte_map_hash,
        final_executable_output_runnable_candidate,
        final_executable_output_blocker_count,
        final_executable_output_blockers,
        final_executable_output_issues,
        tail,
    }
}

pub(crate) fn push_final_snapshot_issues(
    issues: &mut Vec<String>,
    snapshot: &NsldCheckFinalSnapshot,
) {
    if snapshot.final_stage_plan_valid == Some(false) {
        issues.push("final-stage plan verification failed".to_owned());
        issues.extend(snapshot.final_stage_plan_issues.iter().cloned());
    }
    if snapshot.final_executable_writer_input_valid == Some(false) {
        issues.push("final executable writer input verification failed".to_owned());
        issues.extend(
            snapshot
                .final_executable_writer_input_issues
                .iter()
                .cloned(),
        );
    }
    if snapshot.final_executable_host_invoke_plan_valid == Some(false) {
        issues.push("final executable host invoke plan verification failed".to_owned());
        issues.extend(
            snapshot
                .final_executable_host_invoke_plan_issues
                .iter()
                .cloned(),
        );
    }
    if snapshot.final_executable_layout_plan_valid == Some(false) {
        issues.push("final executable layout plan verification failed".to_owned());
        issues.extend(snapshot.final_executable_layout_plan_issues.iter().cloned());
    }
    if snapshot.final_executable_image_dry_run_valid == Some(false) {
        issues.push("final executable image dry-run verification failed".to_owned());
        issues.extend(
            snapshot
                .final_executable_image_dry_run_issues
                .iter()
                .cloned(),
        );
    }
    if snapshot.final_executable_blocked_valid == Some(false) {
        issues.push("final executable blocked report verification failed".to_owned());
        issues.extend(snapshot.final_executable_blocked_issues.iter().cloned());
    }
    if snapshot.final_executable_output_present
        && snapshot.final_executable_output_runnable_candidate == Some(false)
    {
        issues.push("final executable output verification failed".to_owned());
        issues.extend(snapshot.final_executable_output_issues.iter().cloned());
    }
    push_final_tail_snapshot_issues(issues, &snapshot.tail);
}
