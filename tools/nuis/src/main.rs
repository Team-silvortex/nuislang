mod artifact_doctor;
mod artifact_doctor_render;
mod artifact_materialization;
mod build_report_command;
mod build_report_nsld_status;
mod build_report_render;
mod build_report_runtime;
mod cli;
mod dev_tensor;
mod dev_tensor_data;
mod dev_tensor_drift;
mod dev_tensor_hierarchy;
mod dev_tensor_render;
mod dev_tensor_status;
mod galaxy;
mod json_helpers;
mod json_surface;
mod language_runner;
mod project_imports;
mod public_surface;
mod run_artifact;
mod runtime_host_yir;
mod scheduler_surface;
mod surface_render;
mod workflow;

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use artifact_doctor::{
    collect_artifact_output_diagnostics, probe_artifact_doctor, run_build_output_self_check,
};
use artifact_doctor_render::{
    append_artifact_output_diagnostic_json_fields, render_artifact_doctor_json,
};
use artifact_materialization::{handle_materialize_artifact, handle_unpack_artifact_support};
use build_report_command::handle_build_report;
use build_report_nsld_status::print_nsld_artifact_chain_status;
use build_report_render::append_runtime_session_json_fields;
#[cfg(test)]
pub(crate) use build_report_render::render_build_report_json;
pub(crate) use json_helpers::*;
use json_surface::workflow_contract_json_fields;
#[cfg(test)]
pub(crate) use language_runner::{
    benchmark_run_report_json, collect_language_benchmark_run_report, resolve_runner_clock_domain,
    run_language_benchmarks_for_source_file, run_language_tests_for_source_file,
    wait_for_test_child, RawTestOutcome,
};
use language_runner::{handle_bench, handle_test};
#[cfg(test)]
pub(crate) use project_imports::{
    apply_suggested_project_imports, render_project_imports_apply_json, render_project_imports_json,
};
use project_imports::{handle_project_imports, hidden_manual_only_library_modules_for_project};
pub(crate) use public_surface::{
    describe_public_surface, describe_public_surface_modules, public_surface_json,
    public_surface_records, PublicSurfaceModuleRecord,
};
use run_artifact::run_artifact_prelaunch_summary;
#[cfg(test)]
pub(crate) use scheduler_surface::project_workflow_json_fields;
pub(crate) use scheduler_surface::{
    append_project_workflow_json_fields, project_plan_domains_json, scheduler_view_domain_record,
    scheduler_view_domain_record_json,
};
use surface_render::append_json_field_strings;
pub(crate) use workflow::{
    append_json_object_fields, append_workflow_link_plan_json_fields, debug_workflow_brief,
    debug_workflow_samples_brief, default_build_output_dir, handle_workflow,
    json_object_array_field, load_link_plan_for_output_dir, nsld_drive_command_set_for_output_dir,
    nsld_final_executable_output_boundary_summary, print_workflow_frontdoor_surface,
    project_abi_checks_json, project_domain_registry_checks_json, project_frontdoor_surface,
    project_lowering_checks_json, single_source_frontdoor_surface, toolchain_frontdoor_surface,
    workflow_frontdoor_json_object_field, WorkflowFrontdoorSurface,
};
#[cfg(test)]
pub(crate) use workflow::{
    artifact_doctor_command_for_output_dir, artifact_workflow_brief,
    build_workflow_frontdoor_surface, project_compile_workflow_source_profile,
    recommend_project_workflow_step, render_workflow_json, run_artifact_command_for_output_dir,
    single_source_workflow_source_profile, WorkflowRecommendation,
};
#[cfg(test)]
pub(crate) use workflow::{
    release_check_nsld_drive_command_for_output_dir,
    release_check_nsld_drive_dry_run_command_for_output_dir,
    release_check_nsld_drive_dry_run_json_command_for_output_dir,
    release_check_nsld_drive_json_command_for_output_dir,
    release_check_nsld_drive_until_clean_command_for_output_dir,
    release_check_nsld_drive_until_clean_json_command_for_output_dir,
};

fn main() {
    let result = thread::Builder::new()
        .name("nuis-main".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .map_err(|error| format!("failed to start nuis main thread: {error}"))
        .and_then(|handle| match handle.join() {
            Ok(result) => result,
            Err(_) => Err("nuis main thread panicked".to_owned()),
        });
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match cli::parse_args(std::env::args().skip(1))? {
        cli::CommandKind::Help => {
            print_help();
        }
        cli::CommandKind::Status => {
            let index = nuisc::registry::load_index(std::path::Path::new("nustar-packages"))?;
            let engine = nuisc::engine::default_engine();
            let frontdoor = toolchain_frontdoor_surface();
            println!("nuis toolchain frontdoor");
            print_workflow_frontdoor_surface(&frontdoor);
            println!(
                "  recommended_next_step: {}",
                frontdoor.recommended_next_step
            );
            println!("  recommended_command: {}", frontdoor.recommended_command);
            println!("  recommended_reason: {}", frontdoor.recommended_reason);
            println!("  tool: nuis");
            println!("  compiler_core: nuisc");
            println!("  resident_control: nuis-rc");
            println!("  profile: {}", engine.profile);
            println!("  yir: {}", engine.version);
            println!("  indexed_nustar: {}", index.len());
            println!("  nustar_loading: lazy");
            println!("  external_projects: yalivia, vulpoya");
            let dev_tensor = dev_tensor::dev_tensor_summary();
            let dev_tensor_drift = dev_tensor::dev_tensor_drift_summary();
            println!("  dev_tensor_model: architecture-module-function-progress-tensor");
            println!("  dev_tensor_cells: {}", dev_tensor.cell_count);
            println!(
                "  dev_tensor_bootstrap_critical_cells: {}",
                dev_tensor.bootstrap_critical_count
            );
            println!(
                "  dev_tensor_bootstrap_critical_average_progress: {}",
                dev_tensor.bootstrap_critical_average_progress
            );
            println!(
                "  dev_tensor_weakest_bootstrap_architecture: {}",
                dev_tensor.weakest_bootstrap_architecture
            );
            println!(
                "  dev_tensor_weakest_bootstrap_module: {}",
                dev_tensor.weakest_bootstrap_module
            );
            println!(
                "  dev_tensor_weakest_bootstrap_function: {}",
                dev_tensor.weakest_bootstrap_function
            );
            println!(
                "  dev_tensor_coverage_status: {}",
                dev_tensor.coverage_status
            );
            println!(
                "  dev_tensor_coverage: {}/{}",
                dev_tensor.coverage_covered_count, dev_tensor.coverage_expected_count
            );
            println!(
                "  dev_tensor_coverage_missing: {}",
                dev_tensor.coverage_missing_count
            );
            println!(
                "  dev_tensor_coverage_orphaned: {}",
                dev_tensor.coverage_orphaned_count
            );
            println!(
                "  dev_tensor_coverage_stale: {}",
                dev_tensor.coverage_stale_count
            );
            println!("  dev_tensor_drift_status: {}", dev_tensor_drift.status);
            println!(
                "  dev_tensor_drift_checks: {}/{}",
                dev_tensor_drift.passed_count, dev_tensor_drift.check_count
            );
            println!(
                "  dev_tensor_drift_first_failed_check: {}",
                dev_tensor_drift.first_failed_check.unwrap_or("<none>")
            );
        }
        cli::CommandKind::DevTensor { json } => handle_dev_tensor(json),
        cli::CommandKind::Registry { json } => {
            nuisc::run(nuisc::CommandKind::Registry { json })?;
        }
        cli::CommandKind::Fmt { input } => {
            nuisc::run(nuisc::CommandKind::Fmt { input })?;
        }
        cli::CommandKind::Bindings { input } => {
            nuisc::run(nuisc::CommandKind::Bindings { input })?;
        }
        cli::CommandKind::PackNustar { package_id, output } => {
            nuisc::run(nuisc::CommandKind::PackNustar { package_id, output })?;
        }
        cli::CommandKind::InspectNustar { input } => {
            nuisc::run(nuisc::CommandKind::InspectNustar { input })?;
        }
        cli::CommandKind::LoaderContract { package_id } => {
            nuisc::run(nuisc::CommandKind::LoaderContract { package_id })?;
        }
        cli::CommandKind::InspectArtifact { input, json } => {
            nuisc::run(nuisc::CommandKind::InspectArtifact { input, json })?;
        }
        cli::CommandKind::VerifyArtifact { input, json } => {
            nuisc::run(nuisc::CommandKind::VerifyArtifact { input, json })?;
        }
        cli::CommandKind::UnpackArtifactSupport {
            input,
            output_dir,
            json,
        } => handle_unpack_artifact_support(input, output_dir, json)?,
        cli::CommandKind::MaterializeArtifact {
            input,
            output_dir,
            json,
        } => handle_materialize_artifact(input, output_dir, json)?,
        cli::CommandKind::ArtifactDoctor { input, json } => handle_artifact_doctor(input, json)?,
        cli::CommandKind::BuildReport { input, json } => handle_build_report(input, json)?,
        cli::CommandKind::VerifyBuildManifest { manifest } => {
            nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
                manifest: resolve_frontdoor_build_manifest_path(&manifest)?,
                json: false,
            })?;
        }
        cli::CommandKind::CacheStatus {
            input,
            all,
            verbose_cache,
            json,
        } => {
            nuisc::run(nuisc::CommandKind::CacheStatus {
                input,
                all,
                verbose_cache,
                json,
            })?;
        }
        cli::CommandKind::CleanCache { input, all, json } => {
            nuisc::run(nuisc::CommandKind::CleanCache { input, all, json })?;
        }
        cli::CommandKind::PruneCache {
            input,
            all,
            keep,
            json,
        } => {
            nuisc::run(nuisc::CommandKind::PruneCache {
                input,
                all,
                keep,
                json,
            })?;
        }
        cli::CommandKind::ReleaseCheck {
            input,
            output_dir,
            cpu_abi,
            target,
        } => handle_release_check(input, output_dir, cpu_abi, target)?,
        cli::CommandKind::Check { input } => handle_check(input)?,
        cli::CommandKind::Test {
            input,
            list,
            ignored_only,
            include_ignored,
            exact,
            filter,
        } => handle_test(input, list, ignored_only, include_ignored, exact, filter)?,
        cli::CommandKind::Bench {
            input,
            list,
            json,
            exact,
            filter,
        } => handle_bench(input, list, json, exact, filter)?,
        cli::CommandKind::Build {
            input,
            output_dir,
            verbose_cache,
            cpu_abi,
            target,
        } => handle_build(input, output_dir, verbose_cache, cpu_abi, target)?,
        cli::CommandKind::RunArtifact { input, json } => handle_run_artifact(input, json)?,
        cli::CommandKind::DumpAst { input } => handle_dump_ast(input)?,
        cli::CommandKind::DumpNir { input } => handle_dump_nir(input)?,
        cli::CommandKind::DumpYir { input } => handle_dump_yir(input)?,
        cli::CommandKind::Workflow { input, json } => handle_workflow(input, json)?,
        cli::CommandKind::SchedulerView { input, json } => handle_scheduler_view(input, json)?,
        cli::CommandKind::Rc { args } => {
            run_nuis_rc(&args)?;
        }
        cli::CommandKind::ProjectStatus { input, json } => handle_project_status(input, json)?,
        cli::CommandKind::ProjectDoctor { input, json } => handle_project_doctor(input, json)?,
        cli::CommandKind::ProjectImports {
            input,
            json,
            apply_suggested,
        } => handle_project_imports(input, json, apply_suggested)?,
        cli::CommandKind::ProjectLockAbi { input } => handle_project_lock_abi(input)?,
        cli::CommandKind::Galaxy(command) => handle_galaxy(command)?,
    }

    Ok(())
}

fn handle_release_check(
    input: std::path::PathBuf,
    output_dir: std::path::PathBuf,
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
        let nsld_drive_commands = nsld_drive_command_set_for_output_dir(&output_dir);
        let final_output = load_link_plan_for_output_dir(&output_dir)
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
    if success_logs_enabled() {
        println!("release-check: ok");
        println!("  output_dir: {}", output_dir.display());
        println!("  manifest: {}", manifest.display());
    }
    Ok(())
}

fn handle_check(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::Check { input })?;
    Ok(())
}

fn sanitize_workflow_path_label(label: &str) -> String {
    let mut out = String::new();
    let mut previous_was_sep = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_sep = false;
        } else if !previous_was_sep {
            out.push('-');
            previous_was_sep = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "input".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn handle_build(
    input: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    verbose_cache: bool,
    cpu_abi: Option<String>,
    target: Option<String>,
) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::Compile {
        input,
        output_dir: output_dir.clone(),
        verbose_cache,
        cpu_abi,
        target,
    })?;
    let doctor = run_build_output_self_check(&output_dir)?;
    if success_logs_enabled() {
        println!("build: self-check");
        println!("  ready_to_run: {}", doctor.ready_to_run);
        println!("  recommended_next_step: {}", doctor.recommended_next_step);
        println!("  recommended_command: {}", doctor.recommended_command);
    }
    Ok(())
}

pub(crate) fn resolve_frontdoor_build_manifest_path(input: &Path) -> Result<PathBuf, String> {
    if input.file_name().and_then(|value| value.to_str()) == Some("nuis.build.manifest.toml") {
        return Ok(input.to_path_buf());
    }
    if input.is_dir() {
        let manifest_path = input.join("nuis.build.manifest.toml");
        if manifest_path.is_file() {
            return Ok(manifest_path);
        }
        return Err(format!(
            "`{}` does not contain `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    Err(format!(
        "expected an output directory or `nuis.build.manifest.toml`, got `{}`",
        input.display()
    ))
}

fn resolve_run_artifact_binary_path(input: &Path) -> Result<PathBuf, String> {
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

pub(crate) fn success_logs_enabled() -> bool {
    std::env::var_os("NUIS_TEST_QUIET_SUCCESS_LOGS").is_none()
}

fn handle_dev_tensor(json: bool) {
    if json {
        println!("{}", dev_tensor::render_dev_tensor_json());
        return;
    }
    for line in dev_tensor::render_dev_tensor_text() {
        println!("{line}");
    }
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
    append_workflow_link_plan_json_fields(&mut out, link_plan.as_ref());
    out.push('}');
    out
}

fn handle_run_artifact(input: PathBuf, json: bool) -> Result<(), String> {
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

fn handle_artifact_doctor(input: PathBuf, json: bool) -> Result<(), String> {
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

fn handle_dump_ast(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::DumpAst { input })?;
    Ok(())
}

fn handle_dump_nir(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::DumpNir { input })?;
    Ok(())
}

fn handle_dump_yir(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::DumpYir { input })?;
    Ok(())
}

fn handle_scheduler_view(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_scheduler_view_json(input);
    }
    println!("scheduler view: {}", input.display());
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
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
        let galaxy_manifest_path = project.root.join("galaxy.toml");
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
        println!("  source_kind: project");
        println!("  project: {}", project.manifest.name);
        print_workflow_frontdoor_surface(&frontdoor);
        println!(
            "  recommended_next_step: {}",
            frontdoor.recommended_next_step
        );
        println!("  recommended_command: {}", frontdoor.recommended_command);
        println!("  recommended_reason: {}", frontdoor.recommended_reason);
        println!(
            "  project_plan: {}",
            nuisc::project::describe_project_compilation_plan(&plan)
        );
        println!(
            "  synthetic_input: {} ({})",
            plan.synthetic_input.path.display(),
            plan.synthetic_input.kind
        );
        println!("  output_intents: {}", plan.output_intents.len());
        println!(
            "  output_intent_categories: {}",
            nuisc::project::describe_project_output_intent_categories(&plan)
        );
        println!(
            "  abi_mode: {}",
            if plan.abi_resolution.explicit {
                "explicit"
            } else {
                "auto-recommended"
            }
        );
        println!(
            "  resolved_domains: {}",
            plan.abi_resolution.requirements.len()
        );
        for item in nuisc::project::project_abi_selection_views(&plan.abi_resolution) {
            let domain = item.domain.clone();
            println!("  domain: {}", item.domain);
            for line in nuisc::project::render_project_abi_selection_view_lines(&item) {
                if let Some(detail) = line.strip_prefix("abi: ") {
                    println!(
                        "    abi: {}",
                        detail.split_once('=').map(|(_, abi)| abi).unwrap_or(detail)
                    );
                } else {
                    println!("    {}", line.trim_start());
                }
            }
            print_project_scheduler_contract_view(&domain)?;
        }
        return Ok(());
    }

    let artifacts = nuisc::pipeline::compile_source_path(&input)?;
    let manifests = nuisc::registry::load_required_manifests(
        std::path::Path::new("nustar-packages"),
        &artifacts.yir,
    )?;
    let frontdoor = single_source_frontdoor_surface();
    println!("  source_kind: single-file");
    println!("  ast_domain: {}", artifacts.ast.domain);
    println!("  ast_unit: {}", artifacts.ast.unit);
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!("  resolved_domains: {}", manifests.len());
    for manifest in manifests {
        println!("  domain: {}", manifest.domain_family);
        println!("    package: {}", manifest.package_id);
        print_project_scheduler_contract_view(&manifest.domain_family)?;
    }
    Ok(())
}

fn handle_scheduler_view_json(input: std::path::PathBuf) -> Result<(), String> {
    println!("{}", render_scheduler_view_json(&input)?);
    Ok(())
}

pub(crate) fn render_scheduler_view_json(input: &Path) -> Result<String, String> {
    surface_render::render_scheduler_view_json(input)
}

fn handle_project_status(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_project_status_json(input);
    }
    let mut rendered = String::new();
    surface_render::write_project_status_text_summary(&mut rendered, &input)?;
    print!("{rendered}");
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    for item in nuisc::project::project_abi_selection_views(&plan.abi_resolution) {
        let domain = item.domain.clone();
        for line in nuisc::project::render_project_abi_selection_view_lines(&item) {
            if let Some(detail) = line.strip_prefix("abi: ") {
                println!("  abi: {}", detail);
            } else {
                println!("    {}", line.trim_start());
            }
        }
        print_project_scheduler_contract_view(&domain)?;
    }
    Ok(())
}

fn handle_project_status_json(input: std::path::PathBuf) -> Result<(), String> {
    println!("{}", render_project_status_json(&input)?);
    Ok(())
}

pub(crate) fn render_project_status_json(input: &Path) -> Result<String, String> {
    surface_render::render_project_status_json(input)
}

fn handle_project_doctor(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_project_doctor_json(input);
    }
    let mut rendered = String::new();
    surface_render::write_project_doctor_text_summary(&mut rendered, &input)?;
    print!("{rendered}");
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let nova_profile = galaxy::inspect_ns_nova_profile(&project.root)?;
    let abi_checks =
        nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)?;
    let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
    let lowering_checks =
        nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
    for check in &abi_checks {
        let mut rendered = String::new();
        nuisc::project::write_project_abi_selection_check_lines(&mut rendered, check)
            .expect("writing project abi selection check lines should not fail");
        for line in rendered.lines() {
            println!("  {}", line);
        }
    }
    for check in &registry_checks {
        let mut rendered = String::new();
        nuisc::registry::write_project_domain_registry_check_lines(&mut rendered, check)
            .expect("writing project domain registry check lines should not fail");
        for line in rendered.lines() {
            println!("  {}", line);
        }
    }
    for check in &lowering_checks {
        for line in nuisc::project::render_project_lowering_selection_lines(check) {
            println!("  {}", line);
        }
    }
    for item in &plan.abi_resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
        print_project_scheduler_contract_view(&item.domain)?;
    }
    if let Some(profile) = nova_profile.as_ref() {
        println!(
            "  ns_nova_stdlib_schema: {}",
            profile.stdlib_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_stdlib_manifest_ref: {}",
            profile.stdlib_manifest.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_stdlib_declared_sources: {}",
            profile.stdlib_sources.len()
        );
        println!(
            "  ns_nova_family_schema: {}",
            profile.family_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_family_layers: {}",
            if profile.family_layers.is_empty() {
                "<none>".to_owned()
            } else {
                profile.family_layers.join(", ")
            }
        );
        println!(
            "  ns_nova_render_schema: {}",
            profile.render_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_render_units: owner={} bridge={} surface={}",
            profile.render_owner_unit.as_deref().unwrap_or("<none>"),
            profile.render_bridge_unit.as_deref().unwrap_or("<none>"),
            profile.render_surface_unit.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_selection_schema: {}",
            profile.selection_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_selection_units: owner={} bridge={} render={}",
            profile.selection_owner_unit.as_deref().unwrap_or("<none>"),
            profile.selection_bridge_unit.as_deref().unwrap_or("<none>"),
            profile.selection_render_unit.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_selection_controls: {}",
            if profile.selection_controls.is_empty() {
                "<none>".to_owned()
            } else {
                profile.selection_controls.join(", ")
            }
        );
    }

    Ok(())
}

fn handle_project_doctor_json(input: std::path::PathBuf) -> Result<(), String> {
    println!("{}", render_project_doctor_json(&input)?);
    Ok(())
}

pub(crate) fn render_project_doctor_json(input: &Path) -> Result<String, String> {
    surface_render::render_project_doctor_json(input)
}

fn print_domain_contract_completeness(contract: &nuisc::registry::NustarDomainContract) {
    println!("contract_status: {}", contract.contract_status);
    print_scheduler_sample_field(
        "required_contract_groups",
        &contract.required_contract_groups.join("; "),
    );
    if contract.missing_contract_groups.is_empty() {
        println!("missing_contract_groups: <none>");
    } else {
        print_scheduler_sample_field(
            "missing_contract_groups",
            &contract.missing_contract_groups.join("; "),
        );
    }
}

fn print_domain_contract_group(contract: &nuisc::registry::NustarDomainContract, group: &str) {
    println!("    {}:", group);
    match group {
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY => {
            println!("      package: {}", contract.package_id);
            println!("      contract_schema: {}", contract.contract_schema);
            println!("      frontend: {}", contract.frontend);
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER => {
            println!("      loader_abi: {}", contract.loader_abi);
            println!("      loader_entry: {}", contract.loader_entry);
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_ABI => {
            println!("      machine_abi_policy: {}", contract.machine_abi_policy);
            if !contract.abi_profiles.is_empty() {
                print_scheduler_sample_field(
                    "      abi_profiles",
                    &contract.abi_profiles.join("; "),
                );
            }
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE => {
            if !contract.host_ffi_surface.is_empty() {
                print_scheduler_sample_field(
                    "      host_ffi_surface",
                    &contract.host_ffi_surface.join("; "),
                );
                print_scheduler_sample_field(
                    "      host_ffi_abis",
                    &contract.host_ffi_abis.join("; "),
                );
            }
            if let Some(host_ffi_bridge) = contract.host_ffi_bridge.as_deref() {
                println!("      host_ffi_bridge: {}", host_ffi_bridge);
            }
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME => {
            if !contract.capability.support_surface.is_empty() {
                print_scheduler_sample_field(
                    "      support_surface",
                    &contract.capability.support_surface.join("; "),
                );
            }
            if !contract.capability.support_profile_slots.is_empty() {
                print_scheduler_sample_field(
                    "      support_profile_slots",
                    &contract.capability.support_profile_slots.join("; "),
                );
            }
            if !contract.capability.capability_tags.is_empty() {
                print_scheduler_sample_field(
                    "      capability_tags",
                    &contract.capability.capability_tags.join("; "),
                );
            }
            if !contract.capability.default_lanes.is_empty() {
                print_scheduler_sample_field(
                    "      default_lanes",
                    &contract.capability.default_lanes.join("; "),
                );
            }
            println!(
                "      scheduler_clock: {}",
                contract.scheduler.clock.brief()
            );
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER => {
            println!(
                "      scheduler_contract_stack: {}",
                contract.scheduler.contract_stack
            );
            println!(
                "      scheduler_result_roles: {}",
                contract.scheduler.result_roles
            );
            println!(
                "      scheduler_summary_api: {}",
                contract.scheduler.summary_api
            );
            println!(
                "      scheduler_observer_classes: {}",
                contract.scheduler.observer_classes
            );
            if let Some(navigation) = contract.scheduler.sample_navigation.as_deref() {
                println!("      scheduler_sample_navigation: {}", navigation);
            }
            if let Some(samples) = contract.scheduler.result_samples.as_deref() {
                print_scheduler_sample_field("      scheduler_result_samples", samples);
            }
            if let Some(samples) = contract.scheduler.transport_samples.as_deref() {
                print_scheduler_sample_field("      scheduler_transport_samples", samples);
            }
            if let Some(samples) = contract.scheduler.summary_samples.as_deref() {
                print_scheduler_sample_field("      scheduler_summary_samples", samples);
            }
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET => {
            if let Some(navigation) = contract.std_net.sample_navigation.as_deref() {
                println!("      std_net_navigation: {}", navigation);
            }
            if let Some(samples) = contract.std_net.recipe_samples.as_deref() {
                print_scheduler_sample_field("      std_net_samples", samples);
            }
        }
        _ => {
            println!("      <unrecognized contract group>");
        }
    }
}

fn print_project_scheduler_contract_view(domain: &str) -> Result<(), String> {
    let registration = nuisc::registry::load_domain_registration_for_domain(
        std::path::Path::new("nustar-packages"),
        domain,
    )?;
    let contract = registration.contract;
    println!("    registration:");
    println!("      manifest_path: {}", registration.manifest_path);
    println!("      entry_crate: {}", registration.entry_crate);
    println!("      ast_entry: {}", registration.ast_entry);
    println!("      nir_entry: {}", registration.nir_entry);
    println!(
        "      yir_lowering_entry: {}",
        registration.yir_lowering_entry
    );
    println!(
        "      part_verify_entry: {}",
        registration.part_verify_entry
    );
    if !registration.ast_surface.is_empty() {
        print_scheduler_sample_field("      ast_surface", &registration.ast_surface.join("; "));
    }
    if !registration.nir_surface.is_empty() {
        print_scheduler_sample_field("      nir_surface", &registration.nir_surface.join("; "));
    }
    if !registration.yir_lowering.is_empty() {
        print_scheduler_sample_field("      yir_lowering", &registration.yir_lowering.join("; "));
    }
    if !registration.part_verify.is_empty() {
        print_scheduler_sample_field("      part_verify", &registration.part_verify.join("; "));
    }
    if !registration.resource_families.is_empty() {
        print_scheduler_sample_field(
            "      resource_families",
            &registration.resource_families.join("; "),
        );
    }
    if !registration.unit_types.is_empty() {
        print_scheduler_sample_field("      unit_types", &registration.unit_types.join("; "));
    }
    if !registration.lowering_targets.is_empty() {
        print_scheduler_sample_field(
            "      lowering_targets",
            &registration.lowering_targets.join("; "),
        );
    }
    if !registration.ops.is_empty() {
        print_scheduler_sample_field("      ops", &registration.ops.join("; "));
    }
    print_domain_contract_completeness(&contract);
    print_scheduler_sample_field("contract_groups", &contract.contract_groups.join("; "));
    if !contract.extension_groups.is_empty() {
        print_scheduler_sample_field("extension_groups", &contract.extension_groups.join("; "));
    }
    for group in &contract.contract_groups {
        print_domain_contract_group(&contract, group);
    }
    for group in &contract.extension_groups {
        print_domain_contract_group(&contract, group);
    }
    Ok(())
}

fn print_scheduler_sample_field(label: &str, value: &str) {
    if value.contains("; ") {
        println!("    {}:", label);
        for segment in value.split("; ") {
            println!("      - {}", segment);
        }
    } else {
        println!("    {}: {}", label, value);
    }
}

fn print_project_management_hints(include_galaxy_flow: bool) {
    println!(
        "  project_compile_workflow: {}",
        nuisc::project_compile_workflow_brief()
    );
    print_scheduler_sample_field(
        "project_compile_samples",
        nuisc::project_compile_samples_brief(),
    );
    print_scheduler_sample_field(
        "project_test_workflow",
        nuisc::project_test_workflow_brief(),
    );
    if include_galaxy_flow {
        print_scheduler_sample_field(
            "project_galaxy_workflow",
            nuisc::project_galaxy_workflow_brief(),
        );
    }
}

fn handle_project_lock_abi(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let manifest_source = fs::read_to_string(&project.manifest_path).map_err(|error| {
        format!(
            "failed to read `{}`: {error}",
            project.manifest_path.display()
        )
    })?;
    let updated = upsert_abi_block(&manifest_source, &plan.abi_resolution.requirements);
    if updated == manifest_source {
        println!(
            "project abi already locked: {}",
            project.manifest_path.display()
        );
    } else {
        fs::write(&project.manifest_path, updated).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                project.manifest_path.display()
            )
        })?;
        println!("locked project abi: {}", project.manifest_path.display());
    }
    println!(
        "project_plan: {}",
        nuisc::project::describe_project_compilation_plan(&plan)
    );
    println!(
        "  mode: {}",
        if plan.abi_resolution.explicit {
            "explicit (normalized)"
        } else {
            "auto -> explicit"
        }
    );
    for item in plan.abi_resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
    }
    Ok(())
}

fn handle_galaxy(command: cli::GalaxyCommand) -> Result<(), String> {
    match command {
        cli::GalaxyCommand::Init { input, framework } => {
            let manifest_path = galaxy::init(&input, framework.as_deref())?;
            println!("initialized galaxy package");
            println!("  manifest: {}", manifest_path.display());
            if let Some(framework) = framework {
                println!("  framework: {}", framework);
            }
            println!("  local_index: {}", galaxy::local_index_root().display());
        }
        cli::GalaxyCommand::Check { input } => {
            let checked = galaxy::check(&input)?;
            println!("checked galaxy package: {}", checked.manifest.name);
            println!("  root: {}", checked.root.display());
            println!("  manifest: {}", checked.manifest_path.display());
            println!("  project_plan: {}", checked.project_plan_summary);
            println!("  version: {}", checked.manifest.version);
            println!("  package_kind: {}", checked.manifest.package_kind);
            if let Some(framework) = &checked.manifest.framework {
                println!("  framework: {}", framework);
            }
            println!("  project: {}", checked.manifest.project);
            println!("  include_files: {}", checked.include_files.len());
            println!("  local_index: {}", galaxy::local_index_root().display());
            for (domain, abi) in checked.abi_entries {
                println!("  abi: {}={}", domain, abi);
            }
        }
        cli::GalaxyCommand::Pack { input, output } => {
            let bundle = galaxy::pack(&input, &output)?;
            println!("packed galaxy bundle");
            println!("  bundle: {}", bundle.display());
            println!("  local_index: {}", galaxy::local_index_root().display());
            println!(
                "  local_packages: {}",
                galaxy::local_packages_root().display()
            );
        }
        cli::GalaxyCommand::Inspect { input } => {
            let inspected = galaxy::inspect_bundle(&input)?;
            println!("inspected galaxy bundle: {}", input.display());
            println!("  name: {}", inspected.manifest.name);
            println!("  version: {}", inspected.manifest.version);
            println!("  package_kind: {}", inspected.manifest.package_kind);
            if let Some(framework) = &inspected.manifest.framework {
                println!("  framework: {}", framework);
            }
            println!("  project: {}", inspected.manifest.project);
            println!("  summary: {}", inspected.manifest.summary);
            println!("  entries: {}", inspected.entries.len());
            for entry in inspected.entries {
                println!("  file: {} ({} bytes)", entry.path, entry.bytes);
            }
        }
        cli::GalaxyCommand::PublishLocal { input, output } => {
            let bundle = galaxy::publish_local(&input, output.as_deref())?;
            println!("published galaxy bundle locally");
            println!("  bundle: {}", bundle.display());
            println!("  local_index: {}", galaxy::local_index_root().display());
            println!(
                "  local_packages: {}",
                galaxy::local_packages_root().display()
            );
        }
        cli::GalaxyCommand::List => {
            let entries = galaxy::list_local()?;
            if entries.is_empty() {
                println!("no local galaxy packages");
            } else {
                for entry in entries {
                    println!("package: {}", entry.name);
                    println!("  version: {}", entry.version);
                    println!("  bundle: {}", entry.package);
                    println!("  project: {}", entry.project);
                    if let Some(bytes) = entry.bundle_bytes {
                        println!("  bundle_bytes: {}", bytes);
                    }
                    if let Some(hash) = &entry.bundle_fnv1a64 {
                        println!("  bundle_fnv1a64: {}", hash);
                    }
                    if !entry.abi.is_empty() {
                        println!("  abi: {}", entry.abi.join(", "));
                    }
                }
            }
        }
        cli::GalaxyCommand::InstallLocal {
            name,
            version,
            output,
        } => {
            let project_path = galaxy::install_local(&name, version.as_deref(), &output)?;
            println!("installed local galaxy package");
            println!("  name: {}", name);
            if let Some(version) = version {
                println!("  version: {}", version);
            }
            println!("  output: {}", output.display());
            println!("  project: {}", project_path.display());
        }
        cli::GalaxyCommand::InstallDeps { input } => {
            let installed = galaxy::install_project_deps(&input)?;
            if installed.installed.is_empty() {
                println!("project has no galaxy dependencies");
                println!("  project_root: {}", installed.project_root.display());
                println!("  project_plan: {}", installed.project_plan_summary);
                println!("  lock: {}", installed.lock.path.display());
            } else {
                println!("installed galaxy dependencies");
                println!("  project_root: {}", installed.project_root.display());
                println!("  project_plan: {}", installed.project_plan_summary);
                for item in installed.installed {
                    println!("  dep: {}={}", item.name, item.version);
                    println!("  output: {}", item.output.display());
                    println!("  project: {}", item.project.display());
                    println!("  bundle: {}", item.bundle.display());
                    println!("  bundle_fnv1a64: {}", item.bundle_fnv1a64);
                }
                println!("  lock: {}", installed.lock.path.display());
            }
        }
        cli::GalaxyCommand::Doctor { input } => {
            let report = galaxy::doctor_project(&input)?;
            println!("galaxy doctor");
            println!("  project_root: {}", report.project_root.display());
            println!("  project_plan: {}", report.project_plan_summary);
            println!("  deps_root: {}", report.deps_root.display());
            println!(
                "  local_registry_root: {}",
                report.local_registry_root.display()
            );
            println!("  lock_path: {}", report.lock_path.display());
            println!("  lock_status: {}", report.lock_status);
            if let Some(error) = report.lock_error {
                println!("  lock_error: {}", error);
            }
            println!("  dependencies: {}", report.dependencies.len());
            for item in report.dependencies {
                println!(
                    "  dep: {}={} local={} lock={} installed={}",
                    item.name,
                    item.version,
                    yes_no(item.local_available),
                    yes_no(item.locked),
                    yes_no(item.installed)
                );
            }
        }
        cli::GalaxyCommand::SyncDeps { input } => {
            let synced = galaxy::sync_project_deps(&input)?;
            if synced.entries.is_empty() {
                println!("galaxy lock has no dependencies");
                println!("  project_root: {}", synced.project_root.display());
                println!("  project_plan: {}", synced.project_plan_summary);
                println!("  root: {}", synced.root.display());
            } else {
                println!("synced galaxy dependencies");
                println!("  project_root: {}", synced.project_root.display());
                println!("  project_plan: {}", synced.project_plan_summary);
                println!("  root: {}", synced.root.display());
                println!("  dependencies: {}", synced.entries.len());
                for entry in synced.entries {
                    println!("  dep: {}={}", entry.name, entry.version);
                    println!("  bundle: {}", entry.bundle.display());
                    println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
                }
            }
        }
        cli::GalaxyCommand::LockDeps { input } => {
            let lock = galaxy::lock_project_deps(&input)?;
            println!("locked galaxy dependencies");
            println!("  project_root: {}", lock.project_root.display());
            println!("  project_plan: {}", lock.project_plan_summary);
            println!("  lock: {}", lock.path.display());
            println!("  dependencies: {}", lock.entries.len());
            for entry in lock.entries {
                println!("  dep: {}={}", entry.name, entry.version);
                println!("  bundle: {}", entry.bundle.display());
                println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
            }
        }
        cli::GalaxyCommand::VerifyLock { input } => {
            let lock = galaxy::verify_project_lock(&input)?;
            println!("verified galaxy lock");
            println!("  project_root: {}", lock.project_root.display());
            println!("  project_plan: {}", lock.project_plan_summary);
            println!("  lock: {}", lock.path.display());
            println!("  dependencies: {}", lock.entries.len());
            for entry in lock.entries {
                println!("  dep: {}={}", entry.name, entry.version);
                println!("  bundle: {}", entry.bundle.display());
                println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
            }
        }
        cli::GalaxyCommand::InspectLocal { name, version } => {
            let inspected = galaxy::inspect_local(&name, version.as_deref())?;
            println!("inspected local galaxy package");
            println!("  name: {}", inspected.manifest.name);
            println!("  version: {}", inspected.manifest.version);
            println!("  package_kind: {}", inspected.manifest.package_kind);
            if let Some(framework) = &inspected.manifest.framework {
                println!("  framework: {}", framework);
            }
            println!("  project: {}", inspected.manifest.project);
            println!("  summary: {}", inspected.manifest.summary);
            println!("  entries: {}", inspected.entries.len());
            for entry in inspected.entries {
                println!("  file: {} ({} bytes)", entry.path, entry.bytes);
            }
        }
        cli::GalaxyCommand::VerifyLocal { name, version } => {
            let verified = galaxy::verify_local(&name, version.as_deref())?;
            println!("verified local galaxy package");
            println!("  name: {}", verified.name);
            println!("  version: {}", verified.version);
            println!("  bundle: {}", verified.package.display());
            println!("  bundle_bytes: {}", verified.bundle_bytes);
            println!("  bundle_fnv1a64: {}", verified.bundle_fnv1a64);
            println!("  entries: {}", verified.entries);
        }
        cli::GalaxyCommand::RemoveLocal { name, version } => {
            let removed = galaxy::remove_local(&name, version.as_deref())?;
            println!("removed local galaxy package");
            println!("  name: {}", removed.name);
            println!("  version: {}", removed.version);
            println!("  bundle: {}", removed.package.display());
            println!("  index_entry: {}", removed.index_entry.display());
        }
    }
    Ok(())
}

fn print_help() {
    let frontdoor = toolchain_frontdoor_surface();
    println!("nuis toolchain frontdoor");
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!("usage:");
    println!();
    println!("  default compile workflow:");
    println!("    nuis workflow [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis project-doctor [project-dir|nuis.toml]");
    println!("    nuis check [input.ns|project-dir|nuis.toml]");
    println!(
        "    nuis test [--list] [--ignored|--include-ignored] [--exact] [input.ns|project-dir|nuis.toml] [filter]"
    );
    println!(
        "    nuis bench [--list] [--json] [--exact] [input.ns|project-dir|nuis.toml] [filter]"
    );
    println!(
        "    nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] <output-dir>"
    );
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --json");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply --json");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply --until-clean");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply --until-clean --json");
    println!(
        "    nuis run-artifact [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>"
    );
    println!(
        "    nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
    );
    println!("  general:");
    println!("    nuis status");
    println!("    nuis dev-tensor [--json]");
    println!("    nuis registry");
    println!("    nuis fmt [input.ns|project-dir|nuis.toml]");
    println!("    nuis bindings <input.ns|project-dir|nuis.toml>");
    println!("  inspection and debug:");
    println!("    nuis dump-ast [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-nir [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-yir [input.ns|project-dir|nuis.toml]");
    println!("    nuis workflow [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis scheduler-view [--json] [input.ns|project-dir|nuis.toml]");
    println!(
        "    nuis inspect-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
    );
    println!("    nuis verify-artifact [--json] <output-dir|nuis.compiled.artifact>");
    println!("    nuis unpack-artifact-support [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>");
    println!("    nuis materialize-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>");
    println!("    nuis artifact-doctor [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>");
    println!("    nuis build-report [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>");
    println!("    nuis verify-build-manifest <output-dir|nuis.build.manifest.toml>");
    println!();
    println!("  project workflow:");
    println!("    nuis project-doctor [--json] [project-dir|nuis.toml]");
    println!("    nuis project-status [--json] [project-dir|nuis.toml]");
    println!("    nuis project-imports [--json] [--apply-suggested] [project-dir|nuis.toml]");
    println!("    nuis project-lock-abi [project-dir|nuis.toml]");
    println!("  cache:");
    println!(
        "    nuis cache-status [--all] [--verbose-cache] [--json] [input.ns|project-dir|nuis.toml]"
    );
    println!("    nuis clean-cache [--all] [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]");
    println!();
    println!("  release and package:");
    println!("    nuis pack-nustar <package-id> <output.nustar>");
    println!("    nuis inspect-nustar <input.nustar>");
    println!("    nuis loader-contract <package-id>");
    println!();
    println!("  galaxy and framework projects:");
    println!("    nuis galaxy init [project-dir] [--framework <name>]");
    println!("    nuis galaxy check [project-dir|galaxy.toml]");
    println!("    nuis galaxy doctor [project-dir|nuis.toml]");
    println!("    nuis galaxy lock-deps [project-dir|nuis.toml]");
    println!("    nuis galaxy sync-deps [project-dir|nuis.toml]");
    println!("    nuis galaxy verify-lock [project-dir|nuis.toml]");
    println!("    nuis galaxy install-deps [project-dir|nuis.toml]");
    println!("    nuis galaxy pack [project-dir|galaxy.toml] [output.galaxy]");
    println!("    nuis galaxy inspect <input.galaxy>");
    println!("    nuis galaxy publish-local [project-dir|galaxy.toml] [output.galaxy]");
    println!("    nuis galaxy list");
    println!("    nuis galaxy install-local <name> [version] [output-dir]");
    println!("    nuis galaxy inspect-local <name> [version]");
    println!("    nuis galaxy verify-local <name> [version]");
    println!("    nuis galaxy remove-local <name> [version]");
    println!();
    println!("  other:");
    println!("    nuis rc <status|start|stop|track|projects|versions> [...]");
}

fn run_nuis_rc(args: &[String]) -> Result<(), String> {
    let status = std::process::Command::new("nuis-rc").args(args).status();
    match status {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "nuis-rc exited with status {}",
                    status
                        .code()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "signal".to_owned())
                ))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let fallback = std::process::Command::new("cargo")
                .args(["run", "-q", "-p", "nuis-rc", "--"])
                .args(args)
                .status();
            match fallback {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!(
                    "failed to run nuis-rc via PATH and cargo fallback exited with status {}",
                    status
                        .code()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "signal".to_owned())
                )),
                Err(fallback_error) => Err(format!(
                    "failed to run nuis-rc via PATH ({error}) and cargo fallback ({fallback_error})"
                )),
            }
        }
        Err(error) => Err(format!("failed to run nuis-rc: {error}")),
    }
}

pub(crate) fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn upsert_abi_block(
    source: &str,
    requirements: &[nuisc::project::ProjectAbiRequirement],
) -> String {
    let mut entries = requirements
        .iter()
        .map(|item| (item.domain.clone(), item.abi.clone()))
        .collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    let block = render_abi_block(&entries);

    if let Some((start, end)) = find_abi_block_span(source) {
        let mut out = String::new();
        out.push_str(&source[..start]);
        out.push_str(&block);
        out.push_str(&source[end..]);
        out
    } else if source.ends_with('\n') {
        format!("{source}\n{block}")
    } else {
        format!("{source}\n\n{block}")
    }
}

fn render_abi_block(entries: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str("abi = [\n");
    for (domain, abi) in entries {
        out.push_str(&format!("  \"{}={}\",\n", domain, abi));
    }
    out.push_str("]\n");
    out
}

fn find_abi_block_span(source: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;
    let mut start = None::<usize>;
    let mut depth = 0i32;
    let mut seen_open = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if start.is_none() && trimmed.starts_with("abi") && trimmed.contains('=') {
            start = Some(offset);
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            seen_open = line.contains('[');
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        } else if start.is_some() {
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            if line.contains('[') {
                seen_open = true;
            }
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        }
        offset += line.len();
    }
    start.map(|s| (s, source.len()))
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
