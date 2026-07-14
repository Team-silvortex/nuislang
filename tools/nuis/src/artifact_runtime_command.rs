use crate::{
    append_json_field_strings,
    artifact_doctor::{collect_artifact_output_diagnostics, probe_artifact_doctor},
    artifact_doctor_render::render_artifact_doctor_json,
    build_report_nsld_status::print_nsld_artifact_chain_status,
    build_report_render::append_runtime_session_json_fields,
    json_bool_field, json_field, json_optional_string_field, load_link_plan_for_output_dir,
    resolve_frontdoor_build_manifest_path,
    run_artifact::run_artifact_prelaunch_summary,
    runtime_host_yir, success_logs_enabled,
};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub(crate) fn resolve_run_artifact_binary_path(input: &Path) -> Result<PathBuf, String> {
    if input.is_dir() {
        let manifest_path = resolve_frontdoor_build_manifest_path(input)?;
        let report = nuisc::aot::verify_build_manifest(&manifest_path)?;
        let binary = Path::new(&report.output_dir).join(&report.artifact_binary_name);
        if binary.exists() {
            return Ok(binary);
        }
        return Err(format!(
            "output directory `{}` points to missing binary `{}`",
            input.display(),
            binary.display()
        ));
    }
    let file_name = input.file_name().and_then(|value| value.to_str());
    if file_name == Some("nuis.build.manifest.toml") {
        let report = nuisc::aot::verify_build_manifest(input)?;
        let binary = Path::new(&report.output_dir).join(&report.artifact_binary_name);
        if binary.exists() {
            return Ok(binary);
        }
        return Err(format!(
            "manifest `{}` points to missing binary `{}`",
            input.display(),
            binary.display()
        ));
    }
    if file_name == Some("nuis.compiled.artifact") {
        let artifact = nuisc::aot::parse_nuis_compiled_artifact(input)?;
        let binary = input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&artifact.binary_name);
        if binary.exists() {
            return Ok(binary);
        }
        return Err(format!(
            "artifact `{}` expects unpacked sibling binary `{}`",
            input.display(),
            binary.display()
        ));
    }
    if input.exists() {
        return Ok(input.to_path_buf());
    }
    Err(format!(
        "run-artifact expected an output directory, binary path, `nuis.compiled.artifact`, or `nuis.build.manifest.toml`; missing `{}`",
        input.display()
    ))
}

pub(crate) fn render_run_artifact_json(input: &Path) -> String {
    let doctor = probe_artifact_doctor(input);
    let resolved_binary = resolve_run_artifact_binary_path(input).ok();
    let manifest_verify = doctor
        .manifest_path
        .as_ref()
        .filter(|_| doctor.manifest_verified)
        .and_then(|path| nuisc::aot::verify_build_manifest(path).ok());
    let link_plan = doctor
        .output_dir
        .as_ref()
        .and_then(|output_dir| load_link_plan_for_output_dir(output_dir));
    let prelaunch =
        run_artifact_prelaunch_summary(doctor.output_dir.as_deref(), resolved_binary.as_deref());
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", "run_artifact"),
            json_field("input", &input.display().to_string()),
            json_field("source_kind", &doctor.source_kind),
            json_bool_field("ready_to_run", doctor.ready_to_run),
            json_field("recommended_next_step", &doctor.recommended_next_step),
            json_field("recommended_command", &doctor.recommended_command),
            json_field("recommended_reason", &doctor.recommended_reason),
            json_optional_string_field(
                "binary_path",
                resolved_binary
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_bool_field("binary_resolved", resolved_binary.is_some()),
            json_field("run_artifact_prelaunch_kind", &prelaunch.kind),
            json_field("run_artifact_prelaunch_status", &prelaunch.status),
            json_optional_string_field(
                "run_artifact_prelaunch_command",
                prelaunch.command.as_deref(),
            ),
            json_optional_string_field(
                "run_artifact_prelaunch_entrypoint_path",
                prelaunch.entrypoint_path.as_deref(),
            ),
            json_field("run_artifact_prelaunch_reason", &prelaunch.reason),
        ],
    );
    append_runtime_session_json_fields(&mut out, manifest_verify.as_ref());
    append_json_field_strings(
        &mut out,
        runtime_host_yir::runtime_host_yir_json_fields(
            doctor.artifact_path.as_deref(),
            doctor.artifact_verified,
        ),
    );
    crate::append_workflow_link_plan_json_fields(&mut out, link_plan.as_ref());
    out.push('}');
    out
}

pub(crate) fn handle_run_artifact(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        println!("{}", render_run_artifact_json(&input));
        return Ok(());
    }
    let binary = resolve_run_artifact_binary_path(&input)?;
    let mut command = Command::new(&binary);
    if cfg!(test) {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to run `{}`: {error}", binary.display()))?;
    if success_logs_enabled() {
        println!("run-artifact: {}", binary.display());
        println!(
            "  exit_status: {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".to_owned())
        );
        let doctor = probe_artifact_doctor(&input);
        let link_plan = doctor
            .output_dir
            .as_ref()
            .and_then(|output_dir| load_link_plan_for_output_dir(output_dir));
        let prelaunch =
            run_artifact_prelaunch_summary(doctor.output_dir.as_deref(), Some(binary.as_path()));
        println!("  prelaunch_kind: {}", prelaunch.kind);
        println!("  prelaunch_status: {}", prelaunch.status);
        println!(
            "  prelaunch_command: {}",
            prelaunch.command.as_deref().unwrap_or("<none>")
        );
        println!(
            "  prelaunch_entrypoint_path: {}",
            prelaunch.entrypoint_path.as_deref().unwrap_or("<none>")
        );
        println!("  prelaunch_reason: {}", prelaunch.reason);
        print_run_artifact_link_plan_status(link_plan.as_ref());
    }
    if status.success() {
        return Ok(());
    }
    Err(format!(
        "artifact binary `{}` exited with status {:?}",
        binary.display(),
        status.code()
    ))
}

fn print_run_artifact_link_plan_status(link_plan: Option<&nuisc::linker::LinkPlan>) {
    println!("  link_plan_available: {}", link_plan.is_some());
    if let Some(plan) = link_plan {
        println!("  link_plan_final_stage: {}", plan.final_stage.kind);
        println!("  link_plan_final_driver: {}", plan.final_stage.driver);
        println!(
            "  link_plan_final_link_mode: {}",
            plan.final_stage.link_mode
        );
        println!("  link_plan_final_output: {}", plan.final_stage.output_path);
        println!(
            "  link_plan_lowering_plan_index_path: {}",
            plan.lowering_plan_index_path.as_deref().unwrap_or("<none>")
        );
        println!(
            "  link_plan_lowering_plan_index_source: {}",
            plan.lowering_plan_index_source
        );
        print_nsld_artifact_chain_status(plan);
    } else {
        println!("  link_plan_final_stage: <unavailable>");
        println!("  link_plan_final_driver: <unavailable>");
        println!("  link_plan_final_link_mode: <unavailable>");
        println!("  link_plan_final_output: <unavailable>");
        println!("  link_plan_lowering_plan_index_path: <unavailable>");
        println!("  link_plan_lowering_plan_index_source: <unavailable>");
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

pub(crate) fn handle_artifact_doctor(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        println!("{}", render_artifact_doctor_json(&input));
        return Ok(());
    }
    let report = probe_artifact_doctor(&input);
    let diagnostics = collect_artifact_output_diagnostics(&input, &report);
    println!("artifact doctor: {}", report.input.display());
    println!("  source_kind: {}", report.source_kind);
    if let Some(path) = report.output_dir.as_ref() {
        println!("  output_dir: {}", path.display());
    }
    if let Some(path) = report.manifest_path.as_ref() {
        println!("  manifest: {}", path.display());
    }
    if let Some(path) = report.artifact_path.as_ref() {
        println!("  artifact: {}", path.display());
    }
    if let Some(path) = report.binary_path.as_ref() {
        println!("  binary: {}", path.display());
    }
    println!("  manifest_exists: {}", report.manifest_exists);
    println!("  artifact_exists: {}", report.artifact_exists);
    println!("  binary_exists: {}", report.binary_exists);
    println!("  manifest_verified: {}", report.manifest_verified);
    println!("  artifact_verified: {}", report.artifact_verified);
    if let Some(kind) = report.artifact_container_kind.as_deref() {
        println!("  artifact_container_kind: {}", kind);
    }
    if let Some(version) = report.artifact_container_version {
        println!("  artifact_container_version: {}", version);
    }
    if let Some(count) = report.artifact_section_count {
        println!("  artifact_section_count: {}", count);
    }
    if !report.artifact_section_names.is_empty() {
        println!(
            "  artifact_section_names: {}",
            report.artifact_section_names.join(", ")
        );
    }
    if let Some(valid) = report.artifact_section_table_valid {
        println!("  artifact_section_table_valid: {}", valid);
    }
    if let Some(count) = report.lowering_unit_count {
        println!("  lowering_unit_count: {}", count);
    }
    if !report.lowering_domain_families.is_empty() {
        println!(
            "  lowering_domain_families: {}",
            report.lowering_domain_families.join(", ")
        );
    }
    if !report.lowering_targets.is_empty() {
        println!("  lowering_targets: {}", report.lowering_targets.join(", "));
    }
    println!("  ready_to_run: {}", report.ready_to_run);
    let resolved_binary = report.binary_path.as_deref().filter(|path| path.exists());
    let artifact_closure =
        run_artifact_prelaunch_summary(report.output_dir.as_deref(), resolved_binary);
    println!("  artifact_closure_kind: {}", artifact_closure.kind);
    println!("  artifact_closure_status: {}", artifact_closure.status);
    println!(
        "  artifact_closure_command: {}",
        artifact_closure.command.as_deref().unwrap_or("<none>")
    );
    println!(
        "  artifact_closure_entrypoint_path: {}",
        artifact_closure
            .entrypoint_path
            .as_deref()
            .unwrap_or("<none>")
    );
    println!("  artifact_closure_reason: {}", artifact_closure.reason);
    println!(
        "  artifact_diagnostic_code: {}",
        diagnostics.artifact_diagnostic_code
    );
    println!("  self_check_ready: {}", diagnostics.self_check.ready);
    println!("  self_check_code: {}", diagnostics.self_check.code);
    if let Some(error) = report.manifest_verify_error.as_deref() {
        println!("  manifest_verify_error: {}", error);
    }
    if let Some(error) = report.artifact_verify_error.as_deref() {
        println!("  artifact_verify_error: {}", error);
    }
    if let Some(error) = diagnostics.self_check.error.as_deref() {
        println!("  self_check_error: {}", error);
    }
    println!("  recommended_next_step: {}", report.recommended_next_step);
    println!("  recommended_command: {}", report.recommended_command);
    println!("  recommended_reason: {}", report.recommended_reason);
    println!(
        "  project_checks_available: {}",
        diagnostics.project_checks.available()
    );
    println!("  project_checks_code: {}", diagnostics.project_checks.code);
    if let Some(snapshot) = diagnostics.project_checks.snapshot.as_ref() {
        println!("  project_checks_root: {}", snapshot.project_root.display());
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
        "  link_plan_final_output: {}",
        diagnostics
            .link_plan
            .as_ref()
            .map(|plan| plan.final_stage.output_path.as_str())
            .unwrap_or("<unavailable>")
    );
    println!(
        "  link_plan_lowering_plan_index_path: {}",
        diagnostics
            .link_plan
            .as_ref()
            .and_then(|plan| plan.lowering_plan_index_path.as_deref())
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
    println!(
        "  link_plan_domain_units: {}",
        diagnostics
            .link_plan
            .as_ref()
            .map(|plan| plan.domain_units.len())
            .unwrap_or(0)
    );
    if let Some(plan) = diagnostics.link_plan.plan.as_ref() {
        print_nsld_artifact_chain_status(plan);
        for unit in &plan.domain_units {
            let abi = unit.abi.as_deref().unwrap_or("<none>");
            let lowering = unit.selected_lowering_target.as_deref().unwrap_or("<none>");
            let backend = unit.backend_family.as_deref().unwrap_or("<none>");
            println!(
                "  link_plan_domain_unit: {} package={} role={} abi={} lowering={} backend={}",
                unit.domain_family, unit.package_id, unit.packaging_role, abi, lowering, backend
            );
        }
    }
    Ok(())
}
