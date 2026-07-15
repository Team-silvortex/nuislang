use std::path::PathBuf;

use crate::{
    artifact_doctor::run_build_output_self_check,
    success_logs_enabled,
    workflow::{
        load_link_plan_for_output_dir, nsld_drive_command_set_for_output_dir,
        nsld_final_executable_output_boundary_summary,
    },
};

pub(crate) fn handle_release_check(
    input: PathBuf,
    output_dir: PathBuf,
    cpu_abi: Option<String>,
    target: Option<String>,
) -> Result<(), String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
        let abi_checks =
            nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)?;
        let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
        let lowering_checks =
            nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
        if success_logs_enabled() {
            println!("release-check: abi");
        }
        for check in &abi_checks {
            if success_logs_enabled() {
                let mut rendered = String::new();
                nuisc::project::write_project_abi_selection_check_lines(&mut rendered, check)
                    .expect("writing project abi selection check lines should not fail");
                for line in rendered.lines() {
                    println!("  {}", line);
                }
            }
        }
        if abi_checks.iter().any(|check| !check.ok) {
            return Err(
                "release-check aborted because one or more project domains failed ABI selection validation"
                    .to_owned(),
            );
        }
        if success_logs_enabled() {
            println!("release-check: registry");
        }
        for check in &registry_checks {
            if success_logs_enabled() {
                let mut rendered = String::new();
                nuisc::registry::write_project_domain_registry_check_lines(&mut rendered, check)
                    .expect("writing project domain registry check lines should not fail");
                for line in rendered.lines() {
                    println!("  {}", line);
                }
            }
        }
        if registry_checks.iter().any(|check| !check.ok) {
            return Err(
                "release-check aborted because one or more project domains failed registry validation"
                    .to_owned(),
            );
        }
        if success_logs_enabled() {
            println!("release-check: lowering");
        }
        for check in &lowering_checks {
            if success_logs_enabled() {
                for line in nuisc::project::render_project_lowering_selection_lines(check) {
                    println!("  {}", line);
                }
            }
        }
        if lowering_checks.iter().any(|check| !check.ok) {
            return Err(
                "release-check aborted because one or more project domains failed lowering selection validation"
                    .to_owned(),
            );
        }
    }
    if success_logs_enabled() {
        println!("release-check: check");
    }
    nuisc::run(nuisc::CommandKind::Check {
        input: input.clone(),
    })?;
    if success_logs_enabled() {
        println!("release-check: build");
    }
    nuisc::run(nuisc::CommandKind::Compile {
        input: input.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi,
        target,
        packaging_mode: None,
    })?;
    if success_logs_enabled() {
        println!("release-check: verify-build-manifest");
    }
    let manifest = output_dir.join("nuis.build.manifest.toml");
    nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
        manifest: manifest.clone(),
        json: false,
    })?;
    if success_logs_enabled() {
        println!("release-check: artifact-doctor");
    }
    run_build_output_self_check(&output_dir).map_err(|error| {
        format!("release-check aborted because built outputs failed self-check: {error}")
    })?;
    if success_logs_enabled() {
        print_nsld_drive_status(&output_dir);
    }
    if success_logs_enabled() {
        println!("release-check: ok");
        println!("  output_dir: {}", output_dir.display());
        println!("  manifest: {}", manifest.display());
    }
    Ok(())
}

fn print_nsld_drive_status(output_dir: &std::path::Path) {
    let nsld_drive_commands = nsld_drive_command_set_for_output_dir(output_dir);
    let final_output = load_link_plan_for_output_dir(output_dir)
        .as_ref()
        .map(nsld_final_executable_output_boundary_summary);
    println!("release-check: nsld-drive");
    println!("  protocol: {}", nsld_drive_commands.protocol);
    println!(
        "  recommended_first_json_command: {}",
        nsld_drive_commands.recommended_first_json_command
    );
    println!("  dry_run_command: {}", nsld_drive_commands.dry_run_command);
    println!(
        "  dry_run_json_command: {}",
        nsld_drive_commands.dry_run_json_command
    );
    println!(
        "  dry_run_mutates_artifacts: {}",
        nsld_drive_commands.dry_run_mutates_artifacts
    );
    println!(
        "  recommended_command: {}",
        nsld_drive_commands.apply_next_command
    );
    println!(
        "  recommended_json_command: {}",
        nsld_drive_commands.apply_next_json_command
    );
    println!(
        "  apply_next_command: {}",
        nsld_drive_commands.apply_next_command
    );
    println!(
        "  apply_next_json_command: {}",
        nsld_drive_commands.apply_next_json_command
    );
    println!(
        "  apply_next_mutates_artifacts: {}",
        nsld_drive_commands.apply_next_mutates_artifacts
    );
    println!(
        "  until_clean_command: {}",
        nsld_drive_commands.apply_until_clean_command
    );
    println!(
        "  until_clean_json_command: {}",
        nsld_drive_commands.apply_until_clean_json_command
    );
    println!(
        "  apply_until_clean_mutates_artifacts: {}",
        nsld_drive_commands.apply_until_clean_mutates_artifacts
    );
    println!(
        "  final_executable_output_ready: {}",
        final_output.as_ref().is_some_and(|summary| summary.ready)
    );
    println!(
        "  final_executable_output_boundary_status: {}",
        final_output
            .as_ref()
            .map(|summary| summary.boundary_status.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_materialization_status: {}",
        final_output
            .as_ref()
            .map(|summary| summary.materialization_status.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_execution_handoff_contract: {}",
        final_output
            .as_ref()
            .map(|summary| summary.execution_handoff_contract.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_execution_handoff_ready: {}",
        final_output
            .as_ref()
            .map(|summary| summary.execution_handoff_ready.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  final_executable_output_execution_handoff_status: {}",
        final_output
            .as_ref()
            .map(|summary| summary.execution_handoff_status.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_execution_handoff_target: {}",
        final_output
            .as_ref()
            .map(|summary| summary.execution_handoff_target.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_execution_handoff_evidence_status: {}",
        final_output
            .as_ref()
            .map(|summary| summary.execution_handoff_evidence_status.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_execution_handoff_first_blocker: {}",
        final_output
            .as_ref()
            .and_then(|summary| summary.execution_handoff_first_blocker.as_deref())
            .unwrap_or("<none>")
    );
    println!(
        "  final_executable_output_execution_handoff_decision_code: {}",
        final_output
            .as_ref()
            .map(|summary| summary.execution_handoff_decision_code.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_recommended_next_action: {}",
        final_output
            .as_ref()
            .map(|summary| summary.recommended_next_action.as_str())
            .unwrap_or("<unknown>")
    );
    println!(
        "  final_executable_output_path_present: {}",
        final_output
            .as_ref()
            .is_some_and(|summary| summary.path_present)
    );
    println!(
        "  final_executable_output_nsld_owned: {}",
        final_output
            .as_ref()
            .and_then(|summary| summary.nsld_owned)
            .map(|owned: bool| owned.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  final_executable_output_blocker_count: {}",
        final_output
            .as_ref()
            .map(|summary| summary.blockers.len())
            .unwrap_or(0)
    );
    println!(
        "  final_executable_output_first_blocker: {}",
        final_output
            .as_ref()
            .and_then(|summary| summary.first_blocker.as_deref())
            .unwrap_or("<none>")
    );
    if let Some(summary) = final_output.as_ref() {
        for blocker in &summary.blockers {
            println!("  final_executable_output_blocker: {blocker}");
        }
    }
    println!("  mode: apply-next");
    println!(
        "  note: nsld drive is reported as the linker artifact-chain closure step; release-check does not auto-apply or mutate artifacts yet"
    );
}
