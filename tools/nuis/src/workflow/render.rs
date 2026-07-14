use super::frontdoor::{
    build_workflow_frontdoor_surface, recommended_single_source_workflow_command,
    single_source_workflow_next_step_label, single_source_workflow_source_profile,
    WorkflowRecommendation,
};
use super::*;

pub(crate) fn render_workflow_json(input: &Path) -> Result<String, String> {
    if nuisc::project::is_project_input(input) {
        let project = nuisc::project::load_project(input)?;
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
        let output_dir = default_build_output_dir(input);
        let artifact_report = probe_artifact_doctor(&output_dir);
        let diagnostics = collect_artifact_output_diagnostics(input, &artifact_report);
        let mut out = String::from("{");
        append_json_field_strings(
            &mut out,
            vec![
                json_field("source_kind", frontdoor.source_kind),
                json_field("input", &input.display().to_string()),
                json_field("project", &project.manifest.name),
                json_field("root", &project.root.display().to_string()),
                json_field("entry", &project.manifest.entry),
                json_field(
                    "default_build_output_dir",
                    &output_dir.display().to_string(),
                ),
                json_field(
                    "default_release_output_dir",
                    &default_release_check_output_dir(input)
                        .display()
                        .to_string(),
                ),
                json_field("artifact_workflow", artifact_workflow_brief()),
                json_field(
                    "artifact_doctor_command",
                    &artifact_doctor_command_for_output_dir(&output_dir),
                ),
                json_field(
                    "artifact_nsld_drive_dry_run_command",
                    &nsld_drive_dry_run_command_for_output_dir(&output_dir),
                ),
                json_field(
                    "artifact_nsld_drive_dry_run_json_command",
                    &nsld_drive_dry_run_json_command_for_output_dir(&output_dir),
                ),
                json_field(
                    "artifact_nsld_drive_apply_next_command",
                    &nsld_drive_apply_next_command_for_output_dir(&output_dir),
                ),
                json_field(
                    "artifact_nsld_drive_apply_next_json_command",
                    &nsld_drive_apply_next_json_command_for_output_dir(&output_dir),
                ),
                json_field(
                    "artifact_nsld_drive_apply_until_clean_command",
                    &nsld_drive_apply_until_clean_command_for_output_dir(&output_dir),
                ),
                json_field(
                    "artifact_nsld_drive_apply_until_clean_json_command",
                    &nsld_drive_apply_until_clean_json_command_for_output_dir(&output_dir),
                ),
                nsld_drive_command_set_json_field(
                    "artifact_nsld_drive_command_set",
                    Some(&nsld_drive_command_set_for_output_dir(&output_dir)),
                ),
                json_field(
                    "run_artifact_command",
                    &run_artifact_command_for_output_dir(&output_dir),
                ),
                json_bool_field("artifact_ready_to_run", artifact_report.ready_to_run),
                json_field(
                    "artifact_recommended_next_step",
                    &artifact_report.recommended_next_step,
                ),
            ],
        );
        append_artifact_output_diagnostic_json_fields(
            &mut out,
            &diagnostics,
            "artifact_self_check_ready",
            "artifact_self_check_code",
            "artifact_self_check_error",
            false,
        );
        append_workflow_link_plan_json_fields(&mut out, diagnostics.link_plan.plan.as_ref());
        append_json_field_strings(&mut out, workflow_compile_pipeline_json_fields(input));
        append_json_field_strings(
            &mut out,
            workflow_contract_json_fields(&frontdoor, true, true, include_galaxy_flow, true),
        );
        out.push('}');
        return Ok(out);
    }

    let frontdoor = build_workflow_frontdoor_surface(
        single_source_workflow_source_profile(),
        WorkflowRecommendation {
            label: single_source_workflow_next_step_label(),
            command: recommended_single_source_workflow_command(),
            reason: "single-file inputs usually want direct compile truth first, so `check` stays the best default front-door step",
        },
    );
    let output_dir = default_build_output_dir(input);
    let artifact_report = probe_artifact_doctor(&output_dir);
    let diagnostics = collect_artifact_output_diagnostics(input, &artifact_report);
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("source_kind", frontdoor.source_kind),
            json_field("input", &input.display().to_string()),
            json_field("single_source_compile_workflow", frontdoor.workflow_brief),
            json_field("single_source_compile_samples", frontdoor.workflow_samples),
            json_field(
                "default_build_output_dir",
                &output_dir.display().to_string(),
            ),
            json_field(
                "default_release_output_dir",
                &default_release_check_output_dir(input)
                    .display()
                    .to_string(),
            ),
            json_field("artifact_workflow", artifact_workflow_brief()),
            json_field(
                "artifact_doctor_command",
                &artifact_doctor_command_for_output_dir(&output_dir),
            ),
            json_field(
                "artifact_nsld_drive_dry_run_command",
                &nsld_drive_dry_run_command_for_output_dir(&output_dir),
            ),
            json_field(
                "artifact_nsld_drive_dry_run_json_command",
                &nsld_drive_dry_run_json_command_for_output_dir(&output_dir),
            ),
            json_field(
                "artifact_nsld_drive_apply_next_command",
                &nsld_drive_apply_next_command_for_output_dir(&output_dir),
            ),
            json_field(
                "artifact_nsld_drive_apply_next_json_command",
                &nsld_drive_apply_next_json_command_for_output_dir(&output_dir),
            ),
            json_field(
                "artifact_nsld_drive_apply_until_clean_command",
                &nsld_drive_apply_until_clean_command_for_output_dir(&output_dir),
            ),
            json_field(
                "artifact_nsld_drive_apply_until_clean_json_command",
                &nsld_drive_apply_until_clean_json_command_for_output_dir(&output_dir),
            ),
            nsld_drive_command_set_json_field(
                "artifact_nsld_drive_command_set",
                Some(&nsld_drive_command_set_for_output_dir(&output_dir)),
            ),
            json_field(
                "run_artifact_command",
                &run_artifact_command_for_output_dir(&output_dir),
            ),
            json_bool_field("artifact_ready_to_run", artifact_report.ready_to_run),
            json_field(
                "artifact_recommended_next_step",
                &artifact_report.recommended_next_step,
            ),
        ],
    );
    append_artifact_output_diagnostic_json_fields(
        &mut out,
        &diagnostics,
        "artifact_self_check_ready",
        "artifact_self_check_code",
        "artifact_self_check_error",
        false,
    );
    append_workflow_link_plan_json_fields(&mut out, diagnostics.link_plan.plan.as_ref());
    append_json_field_strings(&mut out, workflow_compile_pipeline_json_fields(input));
    append_json_field_strings(
        &mut out,
        workflow_contract_json_fields(&frontdoor, false, false, false, true),
    );
    out.push('}');
    Ok(out)
}
