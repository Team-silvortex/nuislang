mod artifact_device_sample;
mod artifact_doctor;
mod artifact_doctor_mirrors;
mod artifact_doctor_render;
mod artifact_host_runner;
mod artifact_launch_evidence;
mod artifact_materialization;
mod artifact_nsdb_handoff;
mod artifact_runtime_command;
mod artifact_runtime_persistence;
mod artifact_runtime_trace;
mod build_report_command;
mod build_report_nsld_status;
mod build_report_render;
mod build_report_runtime;
mod cli;
mod closure_summary;
mod dev_tensor;
mod dev_tensor_data;
mod dev_tensor_drift;
mod dev_tensor_drift_data;
mod dev_tensor_drift_data_core;
mod dev_tensor_drift_data_runtime;
mod dev_tensor_drift_data_runtime_nsld;
mod dev_tensor_hierarchy;
mod dev_tensor_manifest;
mod dev_tensor_milestones;
mod dev_tensor_render;
mod dev_tensor_status;
mod galaxy;
mod galaxy_command;
mod json_helpers;
mod json_surface;
mod language_runner;
mod project_imports;
mod project_lock_abi_command;
mod project_surface_command;
mod public_surface;
mod release_check_command;
mod release_check_handoff;
mod run_artifact;
mod runtime_host_yir;
mod scheduler_surface;
mod surface_render;
mod workflow;

use std::{
    path::{Path, PathBuf},
    thread,
};

pub(crate) use artifact_doctor::{
    collect_artifact_output_diagnostics, probe_artifact_doctor, run_build_output_self_check,
};
use artifact_doctor_render::append_artifact_output_diagnostic_json_fields;
#[cfg(test)]
pub(crate) use artifact_doctor_render::render_artifact_doctor_json;
use artifact_materialization::{handle_materialize_artifact, handle_unpack_artifact_support};
use artifact_runtime_command::{handle_artifact_doctor, handle_run_artifact};
#[cfg(test)]
pub(crate) use artifact_runtime_command::{
    render_run_artifact_json, resolve_run_artifact_binary_path,
};
use build_report_command::handle_build_report;
#[cfg(test)]
pub(crate) use build_report_render::render_build_report_json;
use galaxy_command::handle_galaxy;
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
use project_lock_abi_command::handle_project_lock_abi;
#[cfg(test)]
pub(crate) use project_lock_abi_command::{find_abi_block_span, upsert_abi_block};
use project_surface_command::{
    handle_project_doctor, handle_project_status, handle_scheduler_view,
};
pub(crate) use project_surface_command::{
    print_project_management_hints, print_scheduler_sample_field,
};
#[cfg(test)]
pub(crate) use project_surface_command::{
    render_project_doctor_json, render_project_status_json, render_scheduler_view_json,
};
pub(crate) use public_surface::{
    describe_public_surface, describe_public_surface_modules, public_surface_json,
    public_surface_records, PublicSurfaceModuleRecord,
};
pub(crate) use release_check_command::handle_release_check;
#[cfg(test)]
pub(crate) use scheduler_surface::project_workflow_json_fields;
pub(crate) use scheduler_surface::{
    append_project_workflow_json_fields, project_plan_domains_json, scheduler_view_domain_record,
    scheduler_view_domain_record_json,
};
use surface_render::append_json_field_strings;
#[cfg(test)]
pub(crate) use workflow::nsld_drive_command_set_for_output_dir;
pub(crate) use workflow::{
    append_json_object_fields, append_workflow_link_plan_json_fields, debug_workflow_brief,
    debug_workflow_samples_brief, default_build_output_dir, handle_workflow,
    json_object_array_field, load_link_plan_for_output_dir, print_workflow_frontdoor_surface,
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
            json,
        } => handle_release_check(input, output_dir, cpu_abi, target, json)?,
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
            packaging_mode,
        } => handle_build(
            input,
            output_dir,
            verbose_cache,
            cpu_abi,
            target,
            packaging_mode,
        )?,
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
    packaging_mode: Option<String>,
) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::Compile {
        input,
        output_dir: output_dir.clone(),
        verbose_cache,
        cpu_abi,
        target,
        packaging_mode,
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
        "    nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [--packaging-mode MODE] [input.ns|project-dir|nuis.toml] <output-dir>"
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
        "    nuis release-check [--json] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
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

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
