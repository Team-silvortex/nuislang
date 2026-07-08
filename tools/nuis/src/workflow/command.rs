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
    Ok(())
}
