use super::{
    final_executable_paths::{
        nsld_final_executable_blocked_path, nsld_final_executable_host_invoke_plan_path,
        nsld_final_executable_image_dry_run_path, nsld_final_executable_layout_plan_path,
        nsld_final_executable_writer_input_path, nsld_final_stage_plan_path,
    },
    final_executable_pipeline_entrypoint::{
        entrypoint_materialization_evidence, nsld_pipeline_entrypoint_materialization_plan,
        nsld_pipeline_entrypoint_materialization_status,
        render_host_entrypoint_runner_command_parts,
    },
    final_executable_pipeline_paths::{
        final_executable_pipeline_required_stage_paths, missing_paths,
        NsldFinalExecutablePipelineRequiredPaths,
    },
    final_executable_pipeline_status::nsld_pipeline_self_owned_image_status,
    reports::NsldFinalExecutablePipelineEmitReport,
};
use std::path::Path;

pub(crate) fn nsld_final_executable_pipeline_snapshot(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutablePipelineEmitReport {
    let final_executable =
        super::final_stage::nsld_verify_final_executable_emit_report(manifest, plan);
    let launcher_manifest =
        super::final_stage::nsld_verify_final_executable_launcher_manifest_report(manifest, plan);
    let launcher_dry_run =
        super::final_stage::nsld_verify_final_executable_launcher_dry_run_report(manifest, plan);
    let mut blockers = final_executable.expected_blockers.clone();
    if launcher_manifest.actual_ready != Some(true) {
        blockers.push("final-executable-launcher-manifest:not-ready".to_owned());
    }
    if launcher_dry_run.actual_dry_run_ready != Some(true) {
        blockers.push("final-executable-launcher-dry-run:not-ready".to_owned());
    }
    let self_owned_image_status = nsld_pipeline_self_owned_image_status(
        launcher_manifest.actual_ready == Some(true),
        launcher_manifest.actual_nsb_path.as_deref().unwrap_or(""),
        launcher_manifest.actual_nsb_size_bytes.is_some(),
        launcher_manifest.actual_nsb_hash.as_deref(),
        launcher_manifest.actual_image_header_valid == Some(true),
    )
    .to_owned();
    let entrypoint_materialization_status = nsld_pipeline_entrypoint_materialization_status(
        self_owned_image_status.as_str(),
        launcher_dry_run.actual_dry_run_ready == Some(true),
        launcher_dry_run.actual_would_enter_lifecycle_hook == Some(true),
    )
    .to_owned();
    let entrypoint_materialization = nsld_pipeline_entrypoint_materialization_plan(
        plan,
        entrypoint_materialization_status.as_str(),
        launcher_manifest.actual_execution_handoff_ready == Some(true),
        launcher_manifest
            .actual_execution_handoff_target
            .as_deref()
            .unwrap_or(""),
        launcher_manifest
            .actual_execution_handoff_first_blocker
            .as_deref(),
        &blockers,
    );
    let (entrypoint_materialization_present, entrypoint_materialization_hash) =
        entrypoint_materialization_evidence(entrypoint_materialization.path.as_deref());
    let entrypoint_materialization_runner_command =
        if let (true, Some(nsb_path), Some(scheduler_entry), Some(lifecycle_hook)) = (
            entrypoint_materialization.ready,
            launcher_manifest.actual_nsb_path.as_deref(),
            launcher_manifest.actual_scheduler_entry.as_deref(),
            launcher_manifest.actual_entry_lifecycle_hook.as_deref(),
        ) {
            Some(render_host_entrypoint_runner_command_parts(
                manifest,
                &plan.output_dir,
                nsb_path,
                scheduler_entry,
                lifecycle_hook,
            ))
        } else {
            None
        };
    let final_stage_plan_path = nsld_final_stage_plan_path(plan).display().to_string();
    let writer_input_path = nsld_final_executable_writer_input_path(plan)
        .display()
        .to_string();
    let host_invoke_plan_path = nsld_final_executable_host_invoke_plan_path(plan)
        .display()
        .to_string();
    let layout_plan_path = nsld_final_executable_layout_plan_path(plan)
        .display()
        .to_string();
    let image_dry_run_path = nsld_final_executable_image_dry_run_path(plan)
        .display()
        .to_string();
    let final_executable_blocked_path = nsld_final_executable_blocked_path(plan)
        .display()
        .to_string();
    let required_stage_paths =
        final_executable_pipeline_required_stage_paths(NsldFinalExecutablePipelineRequiredPaths {
            final_executable_emitted: final_executable.expected_emitted,
            final_stage_plan_path: &final_stage_plan_path,
            final_output_path: &plan.final_stage.output_path,
            writer_input_path: &writer_input_path,
            host_invoke_plan_path: &host_invoke_plan_path,
            layout_plan_path: &layout_plan_path,
            image_dry_run_path: &image_dry_run_path,
            final_executable_blocked_path: &final_executable_blocked_path,
            launcher_manifest_path: &launcher_manifest.input_path,
            launcher_dry_run_path: &launcher_dry_run.input_path,
            entrypoint_materialization_path: entrypoint_materialization.path.as_deref(),
        });
    let missing_required_stage_paths = missing_paths(&required_stage_paths);
    blockers.extend(
        missing_required_stage_paths
            .iter()
            .map(|path| format!("required-stage-path-missing:{path}")),
    );
    let issues = blockers
        .iter()
        .map(|blocker| format!("pipeline:{blocker}"))
        .collect::<Vec<_>>();

    NsldFinalExecutablePipelineEmitReport {
        manifest: manifest.display().to_string(),
        valid: blockers.is_empty(),
        final_stage_plan_path: nsld_final_stage_plan_path(plan).display().to_string(),
        final_output_path: plan.final_stage.output_path.clone(),
        writer_input_path: nsld_final_executable_writer_input_path(plan)
            .display()
            .to_string(),
        host_invoke_plan_path: nsld_final_executable_host_invoke_plan_path(plan)
            .display()
            .to_string(),
        layout_plan_path: nsld_final_executable_layout_plan_path(plan)
            .display()
            .to_string(),
        image_dry_run_path: nsld_final_executable_image_dry_run_path(plan)
            .display()
            .to_string(),
        final_executable_blocked_path: nsld_final_executable_blocked_path(plan)
            .display()
            .to_string(),
        launcher_manifest_path: launcher_manifest.input_path,
        launcher_dry_run_path: launcher_dry_run.input_path,
        final_executable_emitted: final_executable.expected_emitted,
        launcher_manifest_ready: launcher_manifest.actual_ready == Some(true),
        launcher_dry_run_ready: launcher_dry_run.actual_dry_run_ready == Some(true),
        would_enter_lifecycle_hook: launcher_dry_run.actual_would_enter_lifecycle_hook
            == Some(true),
        self_owned_image_status,
        entrypoint_materialization_status,
        entrypoint_materialization_kind: entrypoint_materialization.kind,
        entrypoint_materialization_path: entrypoint_materialization.path,
        entrypoint_materialization_ready: entrypoint_materialization.ready,
        entrypoint_materialization_first_blocker: entrypoint_materialization.first_blocker,
        entrypoint_materialization_present,
        entrypoint_materialization_hash,
        entrypoint_materialization_runner_command,
        execution_handoff_contract: launcher_manifest
            .actual_execution_handoff_contract
            .clone()
            .unwrap_or_default(),
        execution_handoff_ready: launcher_manifest.actual_execution_handoff_ready == Some(true),
        execution_handoff_status: launcher_manifest
            .actual_execution_handoff_status
            .clone()
            .unwrap_or_default(),
        execution_handoff_target: launcher_manifest
            .actual_execution_handoff_target
            .clone()
            .unwrap_or_default(),
        execution_handoff_evidence_status: launcher_manifest
            .actual_execution_handoff_evidence_status
            .clone()
            .unwrap_or_default(),
        execution_handoff_first_blocker: launcher_manifest
            .actual_execution_handoff_first_blocker
            .clone(),
        execution_handoff_decision_code: launcher_manifest
            .actual_execution_handoff_decision_code
            .clone()
            .unwrap_or_default(),
        scheduler_metadata_payload_id: launcher_manifest
            .actual_scheduler_metadata_payload_id
            .clone(),
        scheduler_metadata_present: launcher_manifest.actual_scheduler_metadata_present,
        scheduler_metadata_hash: launcher_manifest.actual_scheduler_metadata_hash.clone(),
        required_stage_path_count: required_stage_paths.len(),
        required_stage_path_present_count: required_stage_paths.len()
            - missing_required_stage_paths.len(),
        missing_required_stage_paths,
        blocker_count: blockers.len(),
        blockers,
        issues,
    }
}
