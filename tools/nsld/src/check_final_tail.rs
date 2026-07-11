use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    final_stage::{
        nsld_verify_final_executable_launcher_dry_run_report,
        nsld_verify_final_executable_launcher_manifest_report,
        nsld_verify_final_executable_pipeline_report,
    },
};
use std::path::Path;

pub(crate) struct NsldCheckFinalTailSnapshot {
    pub(crate) final_executable_launcher_manifest_present: bool,
    pub(crate) final_executable_launcher_manifest_valid: Option<bool>,
    pub(crate) final_executable_launcher_manifest_hash: Option<String>,
    pub(crate) final_executable_launcher_manifest_ready: Option<bool>,
    pub(crate) final_executable_launcher_manifest_blocker_count: Option<usize>,
    pub(crate) final_executable_launcher_manifest_issues: Vec<String>,
    pub(crate) final_executable_launcher_dry_run_present: bool,
    pub(crate) final_executable_launcher_dry_run_valid: Option<bool>,
    pub(crate) final_executable_launcher_dry_run_hash: Option<String>,
    pub(crate) final_executable_launcher_dry_run_ready: Option<bool>,
    pub(crate) final_executable_launcher_dry_run_would_enter_lifecycle_hook: Option<bool>,
    pub(crate) final_executable_launcher_dry_run_blocker_count: Option<usize>,
    pub(crate) final_executable_launcher_dry_run_issues: Vec<String>,
    pub(crate) final_executable_pipeline_present: bool,
    pub(crate) final_executable_pipeline_valid: Option<bool>,
    pub(crate) final_executable_pipeline_hash: Option<String>,
    pub(crate) final_executable_pipeline_ready: Option<bool>,
    pub(crate) final_executable_pipeline_emitted: Option<bool>,
    pub(crate) final_executable_pipeline_required_stage_path_count: Option<usize>,
    pub(crate) final_executable_pipeline_required_stage_path_present_count: Option<usize>,
    pub(crate) final_executable_pipeline_missing_required_stage_paths: Vec<String>,
    pub(crate) final_executable_pipeline_blocker_count: Option<usize>,
    pub(crate) final_executable_pipeline_issues: Vec<String>,
}

pub(crate) fn nsld_check_final_tail_snapshot(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldCheckFinalTailSnapshot {
    let final_executable_launcher_manifest_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableLauncherManifest,
    );
    let final_executable_launcher_manifest_present =
        final_executable_launcher_manifest_path.exists();
    let final_executable_launcher_manifest_verify_report =
        final_executable_launcher_manifest_present
            .then(|| nsld_verify_final_executable_launcher_manifest_report(manifest, plan));
    let final_executable_launcher_manifest_valid = final_executable_launcher_manifest_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_launcher_manifest_hash = final_executable_launcher_manifest_verify_report
        .as_ref()
        .and_then(|report| report.actual_launcher_manifest_hash.clone());
    let final_executable_launcher_manifest_ready = final_executable_launcher_manifest_verify_report
        .as_ref()
        .and_then(|report| report.actual_ready);
    let final_executable_launcher_manifest_blocker_count =
        final_executable_launcher_manifest_verify_report
            .as_ref()
            .and_then(|report| report.actual_blocker_count);
    let final_executable_launcher_manifest_issues =
        final_executable_launcher_manifest_verify_report
            .as_ref()
            .map(|report| report.issues.clone())
            .unwrap_or_default();
    let final_executable_launcher_dry_run_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableLauncherDryRun,
    );
    let final_executable_launcher_dry_run_present = final_executable_launcher_dry_run_path.exists();
    let final_executable_launcher_dry_run_verify_report = final_executable_launcher_dry_run_present
        .then(|| nsld_verify_final_executable_launcher_dry_run_report(manifest, plan));
    let final_executable_launcher_dry_run_valid = final_executable_launcher_dry_run_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_launcher_dry_run_hash = final_executable_launcher_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_dry_run_hash.clone());
    let final_executable_launcher_dry_run_ready = final_executable_launcher_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_dry_run_ready);
    let final_executable_launcher_dry_run_would_enter_lifecycle_hook =
        final_executable_launcher_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_would_enter_lifecycle_hook);
    let final_executable_launcher_dry_run_blocker_count =
        final_executable_launcher_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_blocker_count);
    let final_executable_launcher_dry_run_issues = final_executable_launcher_dry_run_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let final_executable_pipeline_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutablePipeline,
    );
    let final_executable_pipeline_present = final_executable_pipeline_path.exists();
    let final_executable_pipeline_verify_report = final_executable_pipeline_present
        .then(|| nsld_verify_final_executable_pipeline_report(manifest, plan));
    let final_executable_pipeline_valid = final_executable_pipeline_verify_report
        .as_ref()
        .map(|report| report.valid);
    let final_executable_pipeline_hash = final_executable_pipeline_verify_report
        .as_ref()
        .and_then(|report| report.actual_pipeline_hash.clone());
    let final_executable_pipeline_ready = final_executable_pipeline_verify_report
        .as_ref()
        .and_then(|report| report.actual_valid);
    let final_executable_pipeline_emitted = final_executable_pipeline_verify_report
        .as_ref()
        .and_then(|report| report.actual_final_executable_emitted);
    let final_executable_pipeline_required_stage_path_count =
        final_executable_pipeline_verify_report
            .as_ref()
            .map(|report| report.expected_required_stage_path_count);
    let final_executable_pipeline_required_stage_path_present_count =
        final_executable_pipeline_verify_report
            .as_ref()
            .map(|report| report.expected_required_stage_path_present_count);
    let final_executable_pipeline_missing_required_stage_paths =
        final_executable_pipeline_verify_report
            .as_ref()
            .map(|report| report.expected_missing_required_stage_paths.clone())
            .unwrap_or_default();
    let final_executable_pipeline_blocker_count = final_executable_pipeline_verify_report
        .as_ref()
        .and_then(|report| report.actual_blocker_count);
    let final_executable_pipeline_issues = final_executable_pipeline_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();

    NsldCheckFinalTailSnapshot {
        final_executable_launcher_manifest_present,
        final_executable_launcher_manifest_valid,
        final_executable_launcher_manifest_hash,
        final_executable_launcher_manifest_ready,
        final_executable_launcher_manifest_blocker_count,
        final_executable_launcher_manifest_issues,
        final_executable_launcher_dry_run_present,
        final_executable_launcher_dry_run_valid,
        final_executable_launcher_dry_run_hash,
        final_executable_launcher_dry_run_ready,
        final_executable_launcher_dry_run_would_enter_lifecycle_hook,
        final_executable_launcher_dry_run_blocker_count,
        final_executable_launcher_dry_run_issues,
        final_executable_pipeline_present,
        final_executable_pipeline_valid,
        final_executable_pipeline_hash,
        final_executable_pipeline_ready,
        final_executable_pipeline_emitted,
        final_executable_pipeline_required_stage_path_count,
        final_executable_pipeline_required_stage_path_present_count,
        final_executable_pipeline_missing_required_stage_paths,
        final_executable_pipeline_blocker_count,
        final_executable_pipeline_issues,
    }
}

pub(crate) fn push_final_tail_snapshot_issues(
    issues: &mut Vec<String>,
    snapshot: &NsldCheckFinalTailSnapshot,
) {
    if snapshot.final_executable_launcher_manifest_valid == Some(false) {
        issues.push("final executable launcher manifest verification failed".to_owned());
        issues.extend(
            snapshot
                .final_executable_launcher_manifest_issues
                .iter()
                .cloned(),
        );
    }
    if snapshot.final_executable_launcher_dry_run_valid == Some(false) {
        issues.push("final executable launcher dry-run verification failed".to_owned());
        issues.extend(
            snapshot
                .final_executable_launcher_dry_run_issues
                .iter()
                .cloned(),
        );
    }
    if snapshot.final_executable_pipeline_valid == Some(false) {
        issues.push("final executable pipeline verification failed".to_owned());
        issues.extend(snapshot.final_executable_pipeline_issues.iter().cloned());
    }
}
