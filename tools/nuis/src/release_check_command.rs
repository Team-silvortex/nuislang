use std::path::PathBuf;

use crate::{
    append_json_field_strings,
    artifact_doctor::{probe_artifact_doctor, run_build_output_self_check},
    json_bool_field, json_field, json_optional_string_field,
    release_check_handoff::collect_device_sample_handoff_mirror,
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
    json: bool,
) -> Result<(), String> {
    let _quiet_success_logs = QuietSuccessLogs::enable_if(json);
    let emit_logs = !json && success_logs_enabled();
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
        let abi_checks =
            nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)?;
        let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
        let lowering_checks =
            nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
        if emit_logs {
            println!("release-check: abi");
        }
        for check in &abi_checks {
            if emit_logs {
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
        if emit_logs {
            println!("release-check: registry");
        }
        for check in &registry_checks {
            if emit_logs {
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
        if emit_logs {
            println!("release-check: lowering");
        }
        for check in &lowering_checks {
            if emit_logs {
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
    if emit_logs {
        println!("release-check: check");
    }
    nuisc::run(nuisc::CommandKind::Check {
        input: input.clone(),
    })?;
    if emit_logs {
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
    if emit_logs {
        println!("release-check: verify-build-manifest");
    }
    let manifest = output_dir.join("nuis.build.manifest.toml");
    nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
        manifest: manifest.clone(),
        json: false,
    })?;
    if emit_logs {
        println!("release-check: artifact-doctor");
    }
    run_build_output_self_check(&output_dir).map_err(|error| {
        format!("release-check aborted because built outputs failed self-check: {error}")
    })?;
    if emit_logs {
        print_payload_decoder_manifest_status(&output_dir);
        print_nsld_drive_status(&output_dir);
    }
    if json {
        println!("{}", render_release_check_summary_json(&input, &output_dir));
    } else if emit_logs {
        println!("release-check: ok");
        println!("  output_dir: {}", output_dir.display());
        println!("  manifest: {}", manifest.display());
    }
    Ok(())
}

struct QuietSuccessLogs {
    previous: Option<std::ffi::OsString>,
    active: bool,
}

impl QuietSuccessLogs {
    fn enable_if(active: bool) -> Self {
        if !active {
            return Self {
                previous: None,
                active,
            };
        }
        let previous = std::env::var_os("NUIS_TEST_QUIET_SUCCESS_LOGS");
        std::env::set_var("NUIS_TEST_QUIET_SUCCESS_LOGS", "1");
        Self { previous, active }
    }
}

impl Drop for QuietSuccessLogs {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Some(previous) = self.previous.as_ref() {
            std::env::set_var("NUIS_TEST_QUIET_SUCCESS_LOGS", previous);
        } else {
            std::env::remove_var("NUIS_TEST_QUIET_SUCCESS_LOGS");
        }
    }
}

pub(crate) fn render_release_check_summary_json(
    input: &std::path::Path,
    output_dir: &std::path::Path,
) -> String {
    let doctor = probe_artifact_doctor(output_dir);
    let manifest = output_dir.join("nuis.build.manifest.toml");
    let nsld_drive_commands = nsld_drive_command_set_for_output_dir(output_dir);
    let final_output = load_link_plan_for_output_dir(output_dir)
        .as_ref()
        .map(nsld_final_executable_output_boundary_summary);
    let device_sample_handoff = collect_device_sample_handoff_mirror(output_dir);
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", "release_check_summary"),
            json_field("input", &input.display().to_string()),
            json_field("output_dir", &output_dir.display().to_string()),
            json_field("manifest", &manifest.display().to_string()),
            json_bool_field("ready_to_run", doctor.ready_to_run),
            json_field("recommended_next_step", &doctor.recommended_next_step),
            json_field("recommended_command", &doctor.recommended_command),
            json_field("runtime_materialization_policy", "explicit-run-artifact"),
            json_bool_field("runs_run_artifact", false),
            json_bool_field("materializes_nsdb_handoff", false),
            json_field(
                "recommended_run_artifact_json_command",
                &format!("nuis run-artifact {} --json", output_dir.display()),
            ),
            json_field("nsld_drive_protocol", &nsld_drive_commands.protocol),
            json_field(
                "nsld_drive_recommended_first_json_command",
                &nsld_drive_commands.recommended_first_json_command,
            ),
            json_bool_field(
                "nsld_drive_apply_next_mutates_artifacts",
                nsld_drive_commands.apply_next_mutates_artifacts,
            ),
            json_bool_field(
                "final_executable_output_ready",
                final_output.as_ref().is_some_and(|summary| summary.ready),
            ),
            json_optional_string_field(
                "final_executable_output_boundary_status",
                final_output
                    .as_ref()
                    .map(|summary| summary.boundary_status.as_str()),
            ),
        ],
    );
    append_json_field_strings(
        &mut out,
        doctor
            .payload_decoder_manifest
            .json_fields_with_prefix("release_check_payload_decoder_manifest"),
    );
    append_json_field_strings(
        &mut out,
        device_sample_handoff.json_fields_with_prefix("release_check_device_sample_handoff"),
    );
    out.push('}');
    out
}

fn print_payload_decoder_manifest_status(output_dir: &std::path::Path) {
    let doctor = probe_artifact_doctor(output_dir);
    let manifest = &doctor.payload_decoder_manifest;
    let device_sample_handoff = collect_device_sample_handoff_mirror(output_dir);
    println!("release-check: payload-decoder-manifest");
    println!("  runtime_materialization_policy: explicit-run-artifact");
    println!("  runs_run_artifact: false");
    println!("  materializes_nsdb_handoff: false");
    println!(
        "  recommended_run_artifact_json_command: nuis run-artifact {} --json",
        output_dir.display()
    );
    println!("  available: {}", manifest.available);
    if let Some(path) = manifest.path.as_ref() {
        println!("  path: {}", path.display());
    }
    println!("  protocol: {}", manifest.protocol);
    println!("  schema: {}", manifest.schema);
    println!("  status: {}", manifest.status);
    println!("  record_count: {}", manifest.record_count);
    println!("  invalid_record_count: {}", manifest.invalid_record_count);
    println!("  first_diagnostic: {}", manifest.first_diagnostic);
    println!("release-check: device-sample-handoff");
    println!("  available: {}", device_sample_handoff.available);
    if let Some(path) = device_sample_handoff.path.as_ref() {
        println!("  path: {}", path.display());
    }
    println!("  protocol: {}", device_sample_handoff.protocol);
    println!("  status: {}", device_sample_handoff.status);
    println!("  record_count: {}", device_sample_handoff.record_count);
    println!(
        "  pending_validation_count: {}",
        device_sample_handoff.pending_validation_count
    );
    println!(
        "  first_provider_family: {}",
        device_sample_handoff.first_provider_family
    );
    println!(
        "  first_handoff_target: {}",
        device_sample_handoff.first_handoff_target
    );
    println!(
        "  first_validation_status: {}",
        device_sample_handoff.first_validation_status
    );
    println!(
        "  first_input_evidence: {}",
        device_sample_handoff.first_input_evidence
    );
    println!(
        "  first_next_action: {}",
        device_sample_handoff.first_next_action
    );
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
