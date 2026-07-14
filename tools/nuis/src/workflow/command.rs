use super::frontdoor::{
    build_workflow_frontdoor_surface, recommended_single_source_workflow_command,
    single_source_workflow_next_step_label, single_source_workflow_source_profile,
    WorkflowRecommendation,
};
use super::*;
pub(crate) fn default_build_output_dir(input: &Path) -> PathBuf {
    PathBuf::from(format!(
        "target/nuis-build/{}",
        sanitize_workflow_path_label(
            input
                .file_stem()
                .or_else(|| input.file_name())
                .and_then(|item| item.to_str())
                .unwrap_or("input")
        )
    ))
}

pub(crate) fn default_release_check_output_dir(input: &Path) -> PathBuf {
    PathBuf::from(format!(
        "target/nuis-release-check/{}",
        sanitize_workflow_path_label(
            input
                .file_stem()
                .or_else(|| input.file_name())
                .and_then(|item| item.to_str())
                .unwrap_or("input")
        )
    ))
}

pub(crate) fn handle_workflow(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
        let galaxy_manifest_path = project.root.join("galaxy.toml");
        let declared_tests = project
            .manifest
            .tests
            .iter()
            .map(|relative| project.root.join(relative))
            .collect::<Vec<_>>();
        let missing_tests = declared_tests
            .iter()
            .filter(|path| !path.exists())
            .cloned()
            .collect::<Vec<_>>();
        let galaxy_check = if galaxy_manifest_path.exists() {
            Some(galaxy::check(&project.root))
        } else {
            None
        };
        let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
        let galaxy_doctor = galaxy::doctor_project(&project.root)?;
        let hidden_manual_only_library_modules =
            hidden_manual_only_library_modules_for_project(&project);
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
            !hidden_manual_only_library_modules.is_empty(),
        );
        let include_galaxy_flow =
            galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
        let output_dir = default_build_output_dir(&input);
        let artifact_report = probe_artifact_doctor(&output_dir);
        let diagnostics = collect_artifact_output_diagnostics(&input, &artifact_report);
        if json {
            println!("{}", render_workflow_json(&input)?);
            return Ok(());
        }

        println!("workflow: project");
        println!("  input: {}", input.display());
        println!("  project: {}", project.manifest.name);
        println!("  root: {}", project.root.display());
        println!("  entry: {}", project.manifest.entry);
        print_workflow_frontdoor_surface(&frontdoor);
        println!(
            "  recommended_next_step: {}",
            frontdoor.recommended_next_step
        );
        println!("  recommended_command: {}", frontdoor.recommended_command);
        println!("  recommended_reason: {}", frontdoor.recommended_reason);
        print_project_management_hints(include_galaxy_flow);
        println!("  debug_workflow: {}", debug_workflow_brief());
        print_scheduler_sample_field("debug_samples", debug_workflow_samples_brief());
        println!("  default_build_output_dir: {}", output_dir.display());
        println!(
            "  default_release_output_dir: {}",
            default_release_check_output_dir(&input).display()
        );
        println!("  artifact_workflow: {}", artifact_workflow_brief());
        println!(
            "  artifact_doctor_command: {}",
            artifact_doctor_command_for_output_dir(&output_dir)
        );
        println!(
            "  run_artifact_command: {}",
            run_artifact_command_for_output_dir(&output_dir)
        );
        println!("  artifact_ready_to_run: {}", artifact_report.ready_to_run);
        println!(
            "  artifact_diagnostic_code: {}",
            diagnostics.artifact_diagnostic_code
        );
        println!(
            "  artifact_self_check_ready: {}",
            diagnostics.self_check.ready
        );
        println!(
            "  artifact_self_check_code: {}",
            diagnostics.self_check.code
        );
        if let Some(error) = diagnostics.self_check.error.as_deref() {
            println!("  artifact_self_check_error: {}", error);
        }
        println!(
            "  artifact_recommended_next_step: {}",
            artifact_report.recommended_next_step
        );
        println!(
            "  project_checks_available: {}",
            diagnostics.project_checks.available()
        );
        if let Some(snapshot) = diagnostics.project_checks.snapshot.as_ref() {
            println!(
                "  abi_checks_ok: {} ({})",
                snapshot.abi_checks.iter().all(|check| check.ok),
                snapshot.abi_checks.len()
            );
            println!(
                "  registry_checks_ok: {} ({})",
                snapshot.registry_checks.iter().all(|check| check.ok),
                snapshot.registry_checks.len()
            );
            println!(
                "  lowering_checks_ok: {} ({})",
                snapshot.lowering_checks.iter().all(|check| check.ok),
                snapshot.lowering_checks.len()
            );
        }
        println!("  project_checks_code: {}", diagnostics.project_checks.code);
        println!(
            "  link_plan_available: {}",
            diagnostics.link_plan.plan.is_some()
        );
        println!(
            "  link_plan_final_stage: {}",
            diagnostics
                .link_plan
                .as_ref()
                .map(|plan| plan.final_stage.kind.as_str())
                .unwrap_or("<unavailable>")
        );
        println!(
            "  link_plan_final_driver: {}",
            diagnostics
                .link_plan
                .as_ref()
                .map(|plan| plan.final_stage.driver.as_str())
                .unwrap_or("<unavailable>")
        );
        println!(
            "  link_plan_final_link_mode: {}",
            diagnostics
                .link_plan
                .as_ref()
                .map(|plan| plan.final_stage.link_mode.as_str())
                .unwrap_or("<unavailable>")
        );
        println!(
            "  link_plan_lowering_plan_index_source: {}",
            diagnostics
                .link_plan
                .as_ref()
                .map(|plan| plan.lowering_plan_index_source.as_str())
                .unwrap_or("<unavailable>")
        );
        print_nsld_prepared_artifact_chain(diagnostics.link_plan.as_ref());
        return Ok(());
    }

    if json {
        println!("{}", render_workflow_json(&input)?);
        return Ok(());
    }

    let frontdoor = build_workflow_frontdoor_surface(
        single_source_workflow_source_profile(),
        WorkflowRecommendation {
            label: single_source_workflow_next_step_label(),
            command: recommended_single_source_workflow_command(),
            reason: "single-file inputs usually want direct compile truth first, so `check` stays the best default front-door step",
        },
    );
    let output_dir = default_build_output_dir(&input);
    let artifact_report = probe_artifact_doctor(&output_dir);
    let diagnostics = collect_artifact_output_diagnostics(&input, &artifact_report);
    println!("workflow: single-file");
    println!("  input: {}", input.display());
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!(
        "  single_source_compile_workflow: {}",
        frontdoor.workflow_brief
    );
    print_scheduler_sample_field("single_source_compile_samples", frontdoor.workflow_samples);
    println!("  debug_workflow: {}", debug_workflow_brief());
    print_scheduler_sample_field("debug_samples", debug_workflow_samples_brief());
    println!("  default_build_output_dir: {}", output_dir.display());
    println!(
        "  default_release_output_dir: {}",
        default_release_check_output_dir(&input).display()
    );
    println!("  artifact_workflow: {}", artifact_workflow_brief());
    println!(
        "  artifact_doctor_command: {}",
        artifact_doctor_command_for_output_dir(&output_dir)
    );
    println!(
        "  run_artifact_command: {}",
        run_artifact_command_for_output_dir(&output_dir)
    );
    println!("  artifact_ready_to_run: {}", artifact_report.ready_to_run);
    println!(
        "  artifact_diagnostic_code: {}",
        diagnostics.artifact_diagnostic_code
    );
    println!(
        "  artifact_self_check_ready: {}",
        diagnostics.self_check.ready
    );
    println!(
        "  artifact_self_check_code: {}",
        diagnostics.self_check.code
    );
    if let Some(error) = diagnostics.self_check.error.as_deref() {
        println!("  artifact_self_check_error: {}", error);
    }
    println!(
        "  artifact_recommended_next_step: {}",
        artifact_report.recommended_next_step
    );
    println!("  project_checks_code: unavailable");
    println!(
        "  link_plan_available: {}",
        diagnostics.link_plan.plan.is_some()
    );
    println!(
        "  link_plan_final_stage: {}",
        diagnostics
            .link_plan
            .as_ref()
            .map(|plan| plan.final_stage.kind.as_str())
            .unwrap_or("<unavailable>")
    );
    println!(
        "  link_plan_final_driver: {}",
        diagnostics
            .link_plan
            .as_ref()
            .map(|plan| plan.final_stage.driver.as_str())
            .unwrap_or("<unavailable>")
    );
    println!(
        "  link_plan_final_link_mode: {}",
        diagnostics
            .link_plan
            .as_ref()
            .map(|plan| plan.final_stage.link_mode.as_str())
            .unwrap_or("<unavailable>")
    );
    println!(
        "  link_plan_lowering_plan_index_source: {}",
        diagnostics
            .link_plan
            .as_ref()
            .map(|plan| plan.lowering_plan_index_source.as_str())
            .unwrap_or("<unavailable>")
    );
    print_nsld_prepared_artifact_chain(diagnostics.link_plan.as_ref());
    Ok(())
}

fn print_nsld_prepared_artifact_chain(link_plan: Option<&nuisc::linker::LinkPlan>) {
    if let Some(plan) = link_plan {
        let output_dir = std::path::Path::new(&plan.output_dir);
        let nsld_chain = nsld_prepared_artifact_chain_summary(output_dir);
        let nsld_tail = nsld_final_executable_tail_summary(output_dir);
        let nsld_final_output =
            crate::workflow::nsld_final_executable_output_boundary_summary(plan);
        let nsld_next = crate::workflow::nsld_next_action_summary(
            Some(&nsld_chain),
            Some(&nsld_tail),
            Some(&nsld_final_output),
        );
        let nsld_chain_next = crate::workflow::nsld_artifact_chain_next_action_mirror(
            Some(&nsld_chain),
            Some(&nsld_tail),
        );
        let nsld_drive_recommendation = crate::workflow::nsld_drive_recommendation_for_output_dir(
            Some(output_dir),
            &nsld_chain_next,
            Some(&nsld_final_output),
        );
        let workflow_prelaunch = workflow_run_artifact_prelaunch_summary(output_dir);
        println!("  nsld_prepare_command: {}", nsld_chain.prepare_command);
        println!(
            "  nsld_drive_dry_run_command: {}",
            crate::workflow::nsld_drive_dry_run_command_for_output_dir(output_dir)
        );
        println!(
            "  nsld_drive_dry_run_json_command: {}",
            crate::workflow::nsld_drive_dry_run_json_command_for_output_dir(output_dir)
        );
        println!(
            "  nsld_drive_apply_next_command: {}",
            crate::workflow::nsld_drive_apply_next_command_for_output_dir(output_dir)
        );
        println!(
            "  nsld_drive_apply_next_json_command: {}",
            crate::workflow::nsld_drive_apply_next_json_command_for_output_dir(output_dir)
        );
        println!(
            "  nsld_drive_apply_until_clean_command: {}",
            crate::workflow::nsld_drive_apply_until_clean_command_for_output_dir(output_dir)
        );
        println!(
            "  nsld_drive_apply_until_clean_json_command: {}",
            crate::workflow::nsld_drive_apply_until_clean_json_command_for_output_dir(output_dir)
        );
        println!(
            "  nsld_drive_recommended_available: {}",
            nsld_drive_recommendation.available
        );
        println!(
            "  nsld_drive_recommended_mode: {}",
            nsld_drive_recommendation.mode
        );
        println!(
            "  nsld_drive_recommended_command: {}",
            nsld_drive_recommendation
                .command
                .as_deref()
                .unwrap_or("<none>")
        );
        println!(
            "  nsld_drive_recommended_mutates_artifacts: {}",
            nsld_drive_recommendation.mutates_artifacts
        );
        println!(
            "  nsld_drive_recommended_reason: {}",
            nsld_drive_recommendation.reason
        );
        println!(
            "  workflow_run_artifact_prelaunch_kind: {}",
            workflow_prelaunch.kind
        );
        println!(
            "  workflow_run_artifact_prelaunch_status: {}",
            workflow_prelaunch.status
        );
        println!(
            "  workflow_run_artifact_prelaunch_evidence_status: {}",
            workflow_prelaunch.evidence_status
        );
        println!(
            "  workflow_run_artifact_prelaunch_command: {}",
            workflow_prelaunch.command.as_deref().unwrap_or("<none>")
        );
        println!(
            "  workflow_run_artifact_prelaunch_reason: {}",
            workflow_prelaunch.reason
        );
        println!("  nsld_prepared_artifact_chain_ready: {}", nsld_chain.ready);
        println!(
            "  nsld_prepared_artifact_stages: {}/{}",
            nsld_chain.present_count, nsld_chain.stage_count
        );
        println!(
            "  nsld_prepared_artifact_next_missing_stage: {}",
            nsld_chain.next_missing_stage.as_deref().unwrap_or("<none>")
        );
        println!("  nsld_next_action_source: {}", nsld_next.source);
        println!("  nsld_next_action: {}", nsld_next.action);
        println!(
            "  nsld_next_action_command: {}",
            nsld_next.command.as_deref().unwrap_or("<none>")
        );
        println!("  nsld_next_action_reason: {}", nsld_next.reason);
        println!(
            "  nsld_artifact_chain_next_action_available: {}",
            nsld_chain_next.available
        );
        println!(
            "  nsld_artifact_chain_next_action_source: {}",
            nsld_chain_next.source.as_deref().unwrap_or("<none>")
        );
        println!(
            "  nsld_artifact_chain_next_action_command_id: {}",
            nsld_chain_next.command_id.as_deref().unwrap_or("<none>")
        );
        println!(
            "  nsld_artifact_chain_next_action_command: {}",
            nsld_chain_next.command.as_deref().unwrap_or("<none>")
        );
        println!(
            "  nsld_artifact_chain_next_action_command_resolved: {}",
            nsld_chain_next
                .command_resolved
                .as_deref()
                .unwrap_or("<none>")
        );
        println!(
            "  nsld_artifact_chain_next_action_reason: {}",
            nsld_chain_next.reason.as_deref().unwrap_or("<none>")
        );
        println!(
            "  nsld_final_executable_pipeline_command: {}",
            nsld_tail.pipeline_command
        );
        println!("  nsld_final_executable_tail_ready: {}", nsld_tail.ready);
        println!(
            "  nsld_final_executable_tail_stages: {}/{}",
            nsld_tail.present_count, nsld_tail.stage_count
        );
        println!(
            "  nsld_final_executable_tail_next_missing_stage: {}",
            nsld_tail.next_missing_stage.as_deref().unwrap_or("<none>")
        );
        println!(
            "  nsld_final_executable_pipeline_valid: {}",
            nsld_tail
                .pipeline_valid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_final_executable_emitted: {}",
            nsld_tail
                .final_executable_emitted
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_launcher_manifest_ready: {}",
            nsld_tail
                .launcher_manifest_ready
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_launcher_dry_run_ready: {}",
            nsld_tail
                .launcher_dry_run_ready
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_would_enter_lifecycle_hook: {}",
            nsld_tail
                .would_enter_lifecycle_hook
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_blocker_count: {}",
            nsld_tail
                .blocker_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_first_blocker: {}",
            nsld_tail.first_blocker.as_deref().unwrap_or("<none>")
        );
        println!(
            "  nsld_final_executable_pipeline_execution_handoff_contract: {}",
            nsld_tail
                .execution_handoff_contract
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_execution_handoff_ready: {}",
            nsld_tail
                .execution_handoff_ready
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_execution_handoff_status: {}",
            nsld_tail
                .execution_handoff_status
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_execution_handoff_target: {}",
            nsld_tail
                .execution_handoff_target
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_execution_handoff_evidence_status: {}",
            nsld_tail
                .execution_handoff_evidence_status
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_execution_handoff_first_blocker: {}",
            nsld_tail
                .execution_handoff_first_blocker
                .as_deref()
                .unwrap_or("<none>")
        );
        println!(
            "  nsld_final_executable_pipeline_execution_handoff_decision_code: {}",
            nsld_tail
                .execution_handoff_decision_code
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_kind: {}",
            nsld_tail
                .entrypoint_materialization_kind
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_path: {}",
            nsld_tail
                .entrypoint_materialization_path
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_ready: {}",
            nsld_tail
                .entrypoint_materialization_ready
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_first_blocker: {}",
            nsld_tail
                .entrypoint_materialization_first_blocker
                .as_deref()
                .unwrap_or("<none>")
        );
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_present: {}",
            nsld_tail
                .entrypoint_materialization_present
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_hash: {}",
            nsld_tail
                .entrypoint_materialization_hash
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_runner_command: {}",
            nsld_tail
                .entrypoint_materialization_runner_command
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_scheduler_metadata_payload_id: {}",
            nsld_tail
                .scheduler_metadata_payload_id
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_scheduler_metadata_present: {}",
            nsld_tail
                .scheduler_metadata_present
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_scheduler_metadata_hash: {}",
            nsld_tail
                .scheduler_metadata_hash
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_final_executable_pipeline_required_stage_paths: {}/{}",
            nsld_tail
                .required_stage_path_present_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned()),
            nsld_tail
                .required_stage_path_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_pipeline_first_missing_required_stage_path: {}",
            nsld_tail
                .first_missing_required_stage_path
                .as_deref()
                .unwrap_or("<none>")
        );
        println!(
            "  nsld_self_owned_image_ready: {}",
            nsld_tail
                .self_owned_image_ready
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_self_owned_image_status: {}",
            nsld_tail.self_owned_image_status
        );
        println!(
            "  nsld_entrypoint_materialization_status: {}",
            nsld_tail.entrypoint_materialization_status
        );
        println!(
            "  nsld_self_owned_image_path: {}",
            nsld_tail
                .self_owned_image_path
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_self_owned_image_present: {}",
            nsld_tail
                .self_owned_image_present
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_self_owned_image_hash: {}",
            nsld_tail
                .self_owned_image_hash
                .as_deref()
                .unwrap_or("<unknown>")
        );
        println!(
            "  nsld_self_owned_image_header_valid: {}",
            nsld_tail
                .self_owned_image_header_valid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_output_ready: {}",
            nsld_final_output.ready
        );
        println!(
            "  nsld_final_executable_output_boundary_status: {}",
            nsld_final_output.boundary_status
        );
        println!(
            "  nsld_final_executable_output_materialization_status: {}",
            nsld_final_output.materialization_status
        );
        println!(
            "  nsld_final_executable_output_execution_handoff_contract: {}",
            nsld_final_output.execution_handoff_contract
        );
        println!(
            "  nsld_final_executable_output_execution_handoff_ready: {}",
            nsld_final_output.execution_handoff_ready
        );
        println!(
            "  nsld_final_executable_output_execution_handoff_status: {}",
            nsld_final_output.execution_handoff_status
        );
        println!(
            "  nsld_final_executable_output_execution_handoff_target: {}",
            nsld_final_output.execution_handoff_target
        );
        println!(
            "  nsld_final_executable_output_execution_handoff_evidence_status: {}",
            nsld_final_output.execution_handoff_evidence_status
        );
        println!(
            "  nsld_final_executable_output_execution_handoff_first_blocker: {}",
            nsld_final_output
                .execution_handoff_first_blocker
                .as_deref()
                .unwrap_or("<none>")
        );
        println!(
            "  nsld_final_executable_output_execution_handoff_decision_code: {}",
            nsld_final_output.execution_handoff_decision_code
        );
        println!(
            "  nsld_final_executable_output_recommended_next_action: {}",
            nsld_final_output.recommended_next_action
        );
        println!(
            "  nsld_final_executable_output_path_present: {}",
            nsld_final_output.path_present
        );
        println!(
            "  nsld_final_executable_output_nsld_owned: {}",
            nsld_final_output
                .nsld_owned
                .map(|owned| owned.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned())
        );
        println!(
            "  nsld_final_executable_output_blocker_count: {}",
            nsld_final_output.blockers.len()
        );
        println!(
            "  nsld_final_executable_output_first_blocker: {}",
            nsld_final_output
                .first_blocker
                .as_deref()
                .unwrap_or("<none>")
        );
        for blocker in &nsld_final_output.blockers {
            println!("  nsld_final_executable_output_blocker: {blocker}");
        }
    } else {
        println!("  nsld_prepare_command: <unavailable>");
        println!("  nsld_drive_dry_run_command: <unavailable>");
        println!("  nsld_drive_dry_run_json_command: <unavailable>");
        println!("  nsld_drive_apply_next_command: <unavailable>");
        println!("  nsld_drive_apply_next_json_command: <unavailable>");
        println!("  nsld_drive_apply_until_clean_command: <unavailable>");
        println!("  nsld_drive_apply_until_clean_json_command: <unavailable>");
        println!("  nsld_drive_recommended_available: false");
        println!("  nsld_drive_recommended_mode: unavailable");
        println!("  nsld_drive_recommended_command: <unavailable>");
        println!("  nsld_drive_recommended_mutates_artifacts: false");
        println!("  nsld_drive_recommended_reason: link plan is unavailable");
        println!("  workflow_run_artifact_prelaunch_kind: <unavailable>");
        println!("  workflow_run_artifact_prelaunch_status: <unavailable>");
        println!("  workflow_run_artifact_prelaunch_evidence_status: <unavailable>");
        println!("  workflow_run_artifact_prelaunch_command: <none>");
        println!("  workflow_run_artifact_prelaunch_reason: link plan is unavailable");
        println!("  nsld_prepared_artifact_chain_ready: false");
        println!("  nsld_prepared_artifact_stages: 0/0");
        println!("  nsld_prepared_artifact_next_missing_stage: <unavailable>");
        println!("  nsld_next_action_source: nuis-summary");
        println!("  nsld_next_action: unavailable");
        println!("  nsld_next_action_command: <unavailable>");
        println!("  nsld_next_action_reason: link plan is unavailable");
        println!("  nsld_artifact_chain_next_action_available: false");
        println!("  nsld_artifact_chain_next_action_source: <unavailable>");
        println!("  nsld_artifact_chain_next_action_command_id: <unavailable>");
        println!("  nsld_artifact_chain_next_action_command: <unavailable>");
        println!("  nsld_artifact_chain_next_action_command_resolved: <unavailable>");
        println!("  nsld_artifact_chain_next_action_reason: <unavailable>");
        println!("  nsld_final_executable_pipeline_command: <unavailable>");
        println!("  nsld_final_executable_tail_ready: false");
        println!("  nsld_final_executable_tail_stages: 0/0");
        println!("  nsld_final_executable_tail_next_missing_stage: <unavailable>");
        println!("  nsld_final_executable_pipeline_valid: <unknown>");
        println!("  nsld_final_executable_pipeline_final_executable_emitted: <unknown>");
        println!("  nsld_final_executable_pipeline_launcher_manifest_ready: <unknown>");
        println!("  nsld_final_executable_pipeline_launcher_dry_run_ready: <unknown>");
        println!("  nsld_final_executable_pipeline_would_enter_lifecycle_hook: <unknown>");
        println!("  nsld_final_executable_pipeline_blocker_count: <unknown>");
        println!("  nsld_final_executable_pipeline_first_blocker: <none>");
        println!("  nsld_final_executable_pipeline_execution_handoff_contract: <unknown>");
        println!("  nsld_final_executable_pipeline_execution_handoff_ready: <unknown>");
        println!("  nsld_final_executable_pipeline_execution_handoff_status: <unknown>");
        println!("  nsld_final_executable_pipeline_execution_handoff_target: <unknown>");
        println!("  nsld_final_executable_pipeline_execution_handoff_evidence_status: <unknown>");
        println!("  nsld_final_executable_pipeline_execution_handoff_first_blocker: <none>");
        println!("  nsld_final_executable_pipeline_execution_handoff_decision_code: <unknown>");
        println!("  nsld_final_executable_pipeline_entrypoint_materialization_kind: <unknown>");
        println!("  nsld_final_executable_pipeline_entrypoint_materialization_path: <unknown>");
        println!("  nsld_final_executable_pipeline_entrypoint_materialization_ready: <unknown>");
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_first_blocker: <none>"
        );
        println!("  nsld_final_executable_pipeline_entrypoint_materialization_present: <unknown>");
        println!("  nsld_final_executable_pipeline_entrypoint_materialization_hash: <unknown>");
        println!(
            "  nsld_final_executable_pipeline_entrypoint_materialization_runner_command: <unknown>"
        );
        println!("  nsld_final_executable_pipeline_scheduler_metadata_payload_id: <unknown>");
        println!("  nsld_final_executable_pipeline_scheduler_metadata_present: <unknown>");
        println!("  nsld_final_executable_pipeline_scheduler_metadata_hash: <unknown>");
        println!("  nsld_final_executable_pipeline_required_stage_paths: <unknown>/<unknown>");
        println!("  nsld_final_executable_pipeline_first_missing_required_stage_path: <none>");
        println!("  nsld_self_owned_image_ready: <unavailable>");
        println!("  nsld_self_owned_image_status: <unavailable>");
        println!("  nsld_entrypoint_materialization_status: <unavailable>");
        println!("  nsld_self_owned_image_path: <unavailable>");
        println!("  nsld_self_owned_image_present: <unavailable>");
        println!("  nsld_self_owned_image_hash: <unavailable>");
        println!("  nsld_self_owned_image_header_valid: <unavailable>");
        println!("  nsld_final_executable_output_ready: <unavailable>");
        println!("  nsld_final_executable_output_boundary_status: <unavailable>");
        println!("  nsld_final_executable_output_materialization_status: <unavailable>");
        println!("  nsld_final_executable_output_execution_handoff_contract: <unavailable>");
        println!("  nsld_final_executable_output_execution_handoff_ready: <unavailable>");
        println!("  nsld_final_executable_output_execution_handoff_status: <unavailable>");
        println!("  nsld_final_executable_output_execution_handoff_target: <unavailable>");
        println!("  nsld_final_executable_output_execution_handoff_evidence_status: <unavailable>");
        println!("  nsld_final_executable_output_execution_handoff_first_blocker: <none>");
        println!("  nsld_final_executable_output_execution_handoff_decision_code: <unavailable>");
        println!("  nsld_final_executable_output_recommended_next_action: <unavailable>");
        println!("  nsld_final_executable_output_path_present: <unavailable>");
        println!("  nsld_final_executable_output_nsld_owned: <unavailable>");
        println!("  nsld_final_executable_output_blocker_count: <unavailable>");
        println!("  nsld_final_executable_output_first_blocker: <none>");
    }
}

fn workflow_run_artifact_prelaunch_summary(
    output_dir: &std::path::Path,
) -> crate::run_artifact::RunArtifactPrelaunchSummary {
    let doctor = crate::artifact_doctor::probe_artifact_doctor(output_dir);
    let resolved_binary = doctor.binary_path.filter(|path| path.exists());
    crate::run_artifact::run_artifact_prelaunch_summary(
        Some(output_dir),
        resolved_binary.as_ref().map(std::path::PathBuf::as_path),
    )
}
