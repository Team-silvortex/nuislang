mod build_report_runtime;
mod cli;
mod galaxy;
mod json_helpers;
mod json_surface;
mod language_runner;
mod project_imports;
mod public_surface;
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
#[cfg(test)]
pub(crate) use scheduler_surface::project_workflow_json_fields;
pub(crate) use scheduler_surface::{
    append_project_workflow_json_fields, project_plan_domains_json, scheduler_view_domain_record,
    scheduler_view_domain_record_json,
};
use surface_render::append_json_field_strings;
pub(crate) use workflow::{
    append_json_object_fields, append_workflow_link_plan_json_fields, artifact_lowering_units_json,
    debug_workflow_brief, debug_workflow_samples_brief, default_build_output_dir, handle_workflow,
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
        }
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

fn run_build_output_self_check(output_dir: &Path) -> Result<ArtifactDoctorReport, String> {
    let manifest_path = resolve_frontdoor_build_manifest_path(output_dir)?;
    let doctor = probe_artifact_doctor(output_dir);
    let manifest_report = nuisc::aot::verify_build_manifest(&manifest_path).map_err(|error| {
        format!(
            "build self-check could not verify manifest `{}`: {error}; next step: {} ({})",
            manifest_path.display(),
            doctor.recommended_next_step,
            doctor.recommended_command
        )
    })?;
    nuisc::aot::verify_nuis_compiled_artifact(Path::new(&manifest_report.artifact_path)).map_err(
        |error| {
            format!(
                "build self-check could not verify artifact `{}`: {error}; next step: {} ({})",
                manifest_report.artifact_path,
                doctor.recommended_next_step,
                doctor.recommended_command
            )
        },
    )?;
    if !doctor.ready_to_run {
        return Err(format!(
            "build self-check found incomplete runnable output in `{}`; next step: {} ({})",
            output_dir.display(),
            doctor.recommended_next_step,
            doctor.recommended_command
        ));
    }
    Ok(doctor)
}

fn build_output_self_check_status(output_dir: Option<&Path>) -> (bool, Option<String>) {
    let Some(output_dir) = output_dir else {
        return (
            false,
            Some("no output_dir available for self-check".to_owned()),
        );
    };
    match run_build_output_self_check(output_dir) {
        Ok(_) => (true, None),
        Err(error) => (false, Some(error)),
    }
}

fn artifact_diagnostic_code(report: &ArtifactDoctorReport) -> &'static str {
    if !report.manifest_exists && !report.artifact_exists && !report.binary_exists {
        "missing_outputs"
    } else if report.manifest_exists && !report.manifest_verified {
        "manifest_invalid"
    } else if report.artifact_exists && !report.artifact_verified {
        "artifact_invalid"
    } else if report.ready_to_run {
        "ready_to_run"
    } else if report.manifest_exists || report.artifact_exists {
        "partial_outputs"
    } else {
        "binary_only"
    }
}

fn self_check_code(output_dir: Option<&Path>, self_check_error: Option<&str>) -> &'static str {
    match self_check_error {
        None => "ok",
        Some(_) if output_dir.is_none() => "no_output_dir",
        Some(error) if error.contains("does not contain `nuis.build.manifest.toml`") => {
            "missing_build_manifest"
        }
        Some(error)
            if error.contains("expected an output directory or `nuis.build.manifest.toml`") =>
        {
            "invalid_artifact_input"
        }
        Some(error) if error.contains("could not verify manifest") => "manifest_verify_failed",
        Some(error) if error.contains("could not verify artifact") => "artifact_verify_failed",
        Some(error) if error.contains("incomplete runnable output") => "incomplete_runnable_output",
        Some(_) => "self_check_failed",
    }
}

struct ProjectValidationSnapshot {
    project_root: PathBuf,
    abi_checks: Vec<nuisc::project::ProjectAbiSelectionCheck>,
    registry_checks: Vec<nuisc::registry::ProjectDomainRegistryCheck>,
    lowering_checks: Vec<nuisc::project::ProjectLoweringSelectionView>,
}

fn project_checks_code(snapshot: Option<&ProjectValidationSnapshot>) -> &'static str {
    let Some(snapshot) = snapshot else {
        return "unavailable";
    };
    if snapshot.abi_checks.iter().any(|check| !check.ok) {
        "abi_checks_failed"
    } else if snapshot.registry_checks.iter().any(|check| !check.ok) {
        "registry_checks_failed"
    } else if snapshot.lowering_checks.iter().any(|check| !check.ok) {
        "lowering_checks_failed"
    } else {
        "ok"
    }
}

fn collect_project_validation_snapshot(
    input: &Path,
    doctor: Option<&ArtifactDoctorReport>,
) -> Option<ProjectValidationSnapshot> {
    let mut candidates = vec![input.to_path_buf()];
    if let Some(manifest_path) = doctor
        .and_then(|report| report.manifest_path.clone())
        .or_else(|| resolve_frontdoor_build_manifest_path(input).ok())
    {
        if let Ok(manifest_report) = nuisc::aot::verify_build_manifest(&manifest_path) {
            let source_input = PathBuf::from(&manifest_report.input);
            candidates.push(source_input.clone());
            if let Some(parent) = source_input.parent() {
                candidates.push(parent.to_path_buf());
            }
        }
    }
    for candidate in candidates {
        let Ok(project) = nuisc::project::load_project(&candidate) else {
            continue;
        };
        let Ok(plan) = nuisc::project::build_project_compilation_plan(&project) else {
            continue;
        };
        let Ok(abi_checks) =
            nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)
        else {
            continue;
        };
        let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
        let lowering_checks =
            nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
        return Some(ProjectValidationSnapshot {
            project_root: project.root.clone(),
            abi_checks,
            registry_checks,
            lowering_checks,
        });
    }
    None
}

struct SelfCheckSummary {
    ready: bool,
    error: Option<String>,
    code: &'static str,
}

struct ProjectCheckSummary {
    snapshot: Option<ProjectValidationSnapshot>,
    code: &'static str,
}

impl ProjectCheckSummary {
    fn available(&self) -> bool {
        self.snapshot.is_some()
    }
}

struct LinkPlanSummary {
    plan: Option<nuisc::linker::LinkPlan>,
}

impl LinkPlanSummary {
    fn as_ref(&self) -> Option<&nuisc::linker::LinkPlan> {
        self.plan.as_ref()
    }
}

struct ArtifactOutputDiagnostics {
    artifact_diagnostic_code: &'static str,
    self_check: SelfCheckSummary,
    project_checks: ProjectCheckSummary,
    link_plan: LinkPlanSummary,
}

fn collect_artifact_output_diagnostics(
    input: &Path,
    report: &ArtifactDoctorReport,
) -> ArtifactOutputDiagnostics {
    let (self_check_ready, self_check_error) =
        build_output_self_check_status(report.output_dir.as_deref());
    let project_snapshot = collect_project_validation_snapshot(input, Some(report));
    ArtifactOutputDiagnostics {
        artifact_diagnostic_code: artifact_diagnostic_code(report),
        self_check: SelfCheckSummary {
            ready: self_check_ready,
            code: self_check_code(report.output_dir.as_deref(), self_check_error.as_deref()),
            error: self_check_error,
        },
        project_checks: ProjectCheckSummary {
            code: project_checks_code(project_snapshot.as_ref()),
            snapshot: project_snapshot,
        },
        link_plan: LinkPlanSummary {
            plan: report
                .output_dir
                .as_ref()
                .and_then(|output_dir| load_link_plan_for_output_dir(output_dir)),
        },
    }
}

fn append_artifact_output_diagnostic_json_fields(
    out: &mut String,
    diagnostics: &ArtifactOutputDiagnostics,
    self_check_ready_key: &str,
    self_check_code_key: &str,
    self_check_error_key: &str,
    include_project_details: bool,
) {
    append_json_field_strings(
        out,
        vec![
            json_field(
                "artifact_diagnostic_code",
                diagnostics.artifact_diagnostic_code,
            ),
            json_bool_field(self_check_ready_key, diagnostics.self_check.ready),
            json_field(self_check_code_key, diagnostics.self_check.code),
            json_optional_string_field(
                self_check_error_key,
                diagnostics.self_check.error.as_deref(),
            ),
        ],
    );
    append_project_validation_summary_json_fields(
        out,
        diagnostics.project_checks.snapshot.as_ref(),
        include_project_details,
    );
    append_json_field_strings(
        out,
        vec![json_field(
            "project_checks_code",
            diagnostics.project_checks.code,
        )],
    );
}

fn append_project_validation_summary_json_fields(
    out: &mut String,
    snapshot: Option<&ProjectValidationSnapshot>,
    include_details: bool,
) {
    append_json_field_strings(
        out,
        vec![json_bool_field(
            "project_checks_available",
            snapshot.is_some(),
        )],
    );
    if let Some(snapshot) = snapshot {
        append_json_field_strings(
            out,
            vec![json_field(
                "project_checks_root",
                &snapshot.project_root.display().to_string(),
            )],
        );
        append_json_field_strings(
            out,
            json_surface::project_check_summary_json_fields(
                &snapshot.abi_checks,
                &snapshot.registry_checks,
                &snapshot.lowering_checks,
            ),
        );
        if include_details {
            append_json_field_strings(
                out,
                vec![
                    json_object_array_field(
                        "abi_checks",
                        &project_abi_checks_json(&snapshot.abi_checks),
                    ),
                    json_object_array_field(
                        "registry_checks",
                        &project_domain_registry_checks_json(&snapshot.registry_checks),
                    ),
                    json_object_array_field(
                        "lowering_checks",
                        &project_lowering_checks_json(&snapshot.lowering_checks),
                    ),
                ],
            );
        }
    }
}

fn resolve_frontdoor_build_manifest_path(input: &Path) -> Result<PathBuf, String> {
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

fn load_frontdoor_compiled_artifact(
    input: &Path,
) -> Result<nuisc::aot::NuisCompiledArtifact, String> {
    if input.is_dir() {
        let artifact_path = input.join("nuis.compiled.artifact");
        if artifact_path.is_file() {
            return nuisc::aot::parse_nuis_compiled_artifact(&artifact_path);
        }
        let manifest_path = resolve_frontdoor_build_manifest_path(input)?;
        let report = nuisc::aot::verify_build_manifest(&manifest_path)?;
        return nuisc::aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path));
    }
    let file_name = input.file_name().and_then(|value| value.to_str());
    if file_name == Some("nuis.compiled.artifact") {
        return nuisc::aot::parse_nuis_compiled_artifact(input);
    }
    if file_name == Some("nuis.build.manifest.toml") {
        let report = nuisc::aot::verify_build_manifest(input)?;
        return nuisc::aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path));
    }
    Err(format!(
        "artifact materialization expected an output directory, `nuis.compiled.artifact`, or `nuis.build.manifest.toml`; got `{}`",
        input.display()
    ))
}

fn success_logs_enabled() -> bool {
    std::env::var_os("NUIS_TEST_QUIET_SUCCESS_LOGS").is_none()
}

fn render_artifact_materialization_json(
    kind: &str,
    input: &Path,
    output_dir: &Path,
    written_files: &[PathBuf],
) -> String {
    let files = written_files
        .iter()
        .map(|path| format!("\"{}\"", json_escape_local(&path.display().to_string())))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{}}}",
        [
            json_field("kind", kind),
            json_field("input", &input.display().to_string()),
            json_field("output_dir", &output_dir.display().to_string()),
            json_usize_field("written_files_count", written_files.len()),
            format!("\"written_files\":[{}]", files),
        ]
        .join(",")
    )
}

fn materialize_artifact_bundle(input: &Path, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let artifact = load_frontdoor_compiled_artifact(input)?;
    nuisc::aot::validate_nuis_compiled_artifact_layout(input, &artifact)?;
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;
    let envelope_path = output_dir.join("nuis.executable.envelope.toml");
    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    let binary_path = output_dir.join(&artifact.binary_name);
    nuisc::aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
    fs::write(&binary_path, &artifact.binary_blob)
        .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
    let relocated_manifest = nuisc::aot::render_relocated_unpacked_build_manifest(
        &artifact,
        output_dir,
        &envelope_path,
        &artifact_path,
        &binary_path,
    )?;
    let mut relocated_artifact = artifact.clone();
    relocated_artifact.build_manifest_source = relocated_manifest.clone();
    relocated_artifact.build_manifest_bytes = relocated_manifest.len();
    nuisc::aot::write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
    fs::write(&manifest_path, relocated_manifest)
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
    let mut written = vec![envelope_path, manifest_path, artifact_path, binary_path];
    written.extend(
        nuis_artifact::materialize_embedded_artifact_support(&relocated_artifact, output_dir)
            .map_err(|error| error.to_string())?,
    );
    written.sort();
    written.dedup();
    Ok(written)
}

fn handle_unpack_artifact_support(
    input: PathBuf,
    output_dir: PathBuf,
    json: bool,
) -> Result<(), String> {
    let artifact = load_frontdoor_compiled_artifact(&input)?;
    nuisc::aot::validate_nuis_compiled_artifact_layout(&input, &artifact)?;
    let mut written = nuis_artifact::materialize_embedded_artifact_support(&artifact, &output_dir)
        .map_err(|error| error.to_string())?;
    written.sort();
    written.dedup();
    if json {
        println!(
            "{}",
            render_artifact_materialization_json(
                "unpack_artifact_support",
                &input,
                &output_dir,
                &written,
            )
        );
        return Ok(());
    }
    if success_logs_enabled() {
        println!("unpacked artifact support: {}", output_dir.display());
        println!("  source: {}", input.display());
        println!("  written_files: {}", written.len());
        for path in &written {
            println!("  file: {}", path.display());
        }
    }
    Ok(())
}

fn handle_materialize_artifact(
    input: PathBuf,
    output_dir: PathBuf,
    json: bool,
) -> Result<(), String> {
    let written = materialize_artifact_bundle(&input, &output_dir)?;
    if json {
        println!(
            "{}",
            render_artifact_materialization_json(
                "materialize_artifact",
                &input,
                &output_dir,
                &written,
            )
        );
        return Ok(());
    }
    if success_logs_enabled() {
        println!("materialized artifact: {}", output_dir.display());
        println!("  source: {}", input.display());
        println!("  written_files: {}", written.len());
        for path in &written {
            println!("  file: {}", path.display());
        }
    }
    Ok(())
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

pub(crate) fn probe_artifact_doctor(input: &Path) -> ArtifactDoctorReport {
    let mut source_kind = "binary".to_owned();
    let mut output_dir = if input.is_dir() {
        Some(input.to_path_buf())
    } else {
        input.parent().map(Path::to_path_buf)
    };
    let mut manifest_path = None;
    let mut artifact_path = None;
    let mut binary_path = None;

    if input.is_dir() {
        source_kind = "output_dir".to_owned();
        let candidate_manifest = input.join("nuis.build.manifest.toml");
        let candidate_artifact = input.join("nuis.compiled.artifact");
        if candidate_manifest.exists() {
            manifest_path = Some(candidate_manifest);
        }
        if candidate_artifact.exists() {
            artifact_path = Some(candidate_artifact);
        }
    } else if input.file_name().and_then(|value| value.to_str()) == Some("nuis.build.manifest.toml")
    {
        source_kind = "manifest".to_owned();
        manifest_path = Some(input.to_path_buf());
        output_dir = input.parent().map(Path::to_path_buf);
    } else if input.file_name().and_then(|value| value.to_str()) == Some("nuis.compiled.artifact") {
        source_kind = "artifact".to_owned();
        artifact_path = Some(input.to_path_buf());
        output_dir = input.parent().map(Path::to_path_buf);
    } else {
        binary_path = Some(input.to_path_buf());
        output_dir = input.parent().map(Path::to_path_buf);
        if let Some(dir) = output_dir.as_ref() {
            let candidate_manifest = dir.join("nuis.build.manifest.toml");
            let candidate_artifact = dir.join("nuis.compiled.artifact");
            if candidate_manifest.exists() {
                manifest_path = Some(candidate_manifest);
            }
            if candidate_artifact.exists() {
                artifact_path = Some(candidate_artifact);
            }
        }
    }

    let mut manifest_verified = false;
    let mut artifact_verified = false;
    let mut manifest_verify_error = None;
    let mut artifact_verify_error = None;
    let mut artifact_container_kind = None;
    let mut artifact_container_version = None;
    let mut artifact_section_count = None;
    let mut artifact_section_names = Vec::new();
    let mut artifact_section_table_valid = None;
    let mut lowering_unit_count = None;
    let mut lowering_domain_families = Vec::new();
    let mut lowering_targets = Vec::new();
    let mut lowering_units = Vec::new();

    if let Some(path) = manifest_path.as_ref() {
        match nuisc::aot::verify_build_manifest(path) {
            Ok(report) => {
                manifest_verified = true;
                artifact_path = Some(PathBuf::from(&report.artifact_path));
                binary_path =
                    Some(Path::new(&report.output_dir).join(&report.artifact_binary_name));
                output_dir = Some(PathBuf::from(&report.output_dir));
            }
            Err(error) => manifest_verify_error = Some(error),
        }
    }

    if let Some(path) = artifact_path.as_ref() {
        match nuisc::aot::inspect_nuis_compiled_artifact_container(path) {
            Ok(container) => {
                artifact_container_kind = Some(container.container_kind);
                artifact_container_version = Some(container.binary_version);
                artifact_section_count = Some(container.section_count);
                artifact_section_names = container.section_names;
                artifact_section_table_valid = Some(container.section_table_valid);
                lowering_unit_count = Some(container.lowering_unit_count);
                lowering_domain_families = container.lowering_domain_families;
                lowering_targets = container.lowering_targets;
                lowering_units = container.lowering_units;
            }
            Err(error) => {
                artifact_verify_error = Some(error);
            }
        }
        match nuisc::aot::verify_nuis_compiled_artifact(path) {
            Ok(report) => {
                artifact_verified = true;
                if binary_path.is_none() {
                    let base = path.parent().unwrap_or_else(|| Path::new("."));
                    binary_path = Some(base.join(report.binary_name));
                }
            }
            Err(error) => {
                artifact_verify_error = Some(error);
                if binary_path.is_none() {
                    if let Ok(artifact) = nuisc::aot::parse_nuis_compiled_artifact(path) {
                        let base = path.parent().unwrap_or_else(|| Path::new("."));
                        binary_path = Some(base.join(artifact.binary_name));
                    }
                }
            }
        }
    }

    let manifest_exists = manifest_path.as_ref().is_some_and(|path| path.exists());
    let artifact_exists = artifact_path.as_ref().is_some_and(|path| path.exists());
    let binary_exists = binary_path.as_ref().is_some_and(|path| path.exists());
    let ready_to_run = binary_exists && manifest_verified && artifact_verified;

    let (recommended_next_step, recommended_command, recommended_reason) = if !manifest_exists
        && !artifact_exists
        && !binary_exists
    {
        (
            "build".to_owned(),
            "nuis build <input> <output-dir>".to_owned(),
            "no recognizable native artifact outputs were found yet, so the next step is to rebuild a fresh output directory".to_owned(),
        )
    } else if manifest_exists && !manifest_verified {
        (
            "verify_build_manifest".to_owned(),
            output_dir
                .as_ref()
                .map(|path| format!("nuis verify-build-manifest {}", path.display()))
                .or_else(|| {
                    manifest_path
                        .as_ref()
                        .map(|path| format!("nuis verify-build-manifest {}", path.display()))
                })
                .unwrap_or_else(|| "nuis verify-build-manifest <output-dir>".to_owned()),
            "the manifest exists but does not currently pass verification, so the next step is to inspect that contract boundary directly".to_owned(),
        )
    } else if artifact_exists && !artifact_verified {
        (
            "verify_artifact".to_owned(),
            format!(
                "nuis verify-artifact {}",
                artifact_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<nuis.compiled.artifact>".to_owned())
            ),
            "the compiled artifact exists but does not currently pass verification, so the next step is to inspect the packaged binary bundle directly".to_owned(),
        )
    } else if ready_to_run {
        (
            "run_artifact".to_owned(),
            output_dir
                .as_ref()
                .map(|path| format!("nuis run-artifact {}", path.display()))
                .or_else(|| {
                    manifest_path
                        .as_ref()
                        .or(artifact_path.as_ref())
                        .or(binary_path.as_ref())
                        .map(|path| format!("nuis run-artifact {}", path.display()))
                })
                .unwrap_or_else(|| "nuis run-artifact <output-dir>".to_owned()),
            "the binary, manifest, and compiled artifact are all present and verified, so the next step is to launch the built output through the nuis frontdoor".to_owned(),
        )
    } else if manifest_exists || artifact_exists {
        (
            "inspect_artifact".to_owned(),
            output_dir
                .as_ref()
                .map(|path| format!("nuis inspect-artifact {}", path.display()))
                .or_else(|| {
                    manifest_path
                        .as_ref()
                        .or(artifact_path.as_ref())
                        .map(|path| format!("nuis inspect-artifact {}", path.display()))
                })
                .unwrap_or_else(|| "nuis inspect-artifact <output-dir>".to_owned()),
            "some artifact outputs are present, but the closure is not fully ready yet, so the next step is to inspect the available bundle metadata".to_owned(),
        )
    } else {
        (
            "run_artifact".to_owned(),
            format!(
                "nuis run-artifact {}",
                binary_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<binary-path>".to_owned())
            ),
            "only the binary path is currently visible, so the next step is to launch it through the nuis artifact runner".to_owned(),
        )
    };

    ArtifactDoctorReport {
        source_kind,
        input: input.to_path_buf(),
        output_dir,
        manifest_path,
        artifact_path,
        binary_path,
        manifest_exists,
        artifact_exists,
        binary_exists,
        manifest_verified,
        artifact_verified,
        ready_to_run,
        recommended_next_step,
        recommended_command,
        recommended_reason,
        manifest_verify_error,
        artifact_verify_error,
        artifact_container_kind,
        artifact_container_version,
        artifact_section_count,
        artifact_section_names,
        artifact_section_table_valid,
        lowering_unit_count,
        lowering_domain_families,
        lowering_targets,
        lowering_units,
    }
}

pub(crate) fn render_artifact_doctor_json(input: &Path) -> String {
    let report = probe_artifact_doctor(input);
    let diagnostics = collect_artifact_output_diagnostics(input, &report);
    let output_dir = report
        .output_dir
        .as_ref()
        .map(|path| path.display().to_string());
    let manifest_path = report
        .manifest_path
        .as_ref()
        .map(|path| path.display().to_string());
    let artifact_path = report
        .artifact_path
        .as_ref()
        .map(|path| path.display().to_string());
    let binary_path = report
        .binary_path
        .as_ref()
        .map(|path| path.display().to_string());
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", "artifact_doctor"),
            json_field("source_kind", &report.source_kind),
            json_field("input", &report.input.display().to_string()),
            json_optional_string_field("output_dir", output_dir.as_deref()),
            json_optional_string_field("manifest_path", manifest_path.as_deref()),
            json_optional_string_field("artifact_path", artifact_path.as_deref()),
            json_optional_string_field("binary_path", binary_path.as_deref()),
            json_bool_field("manifest_exists", report.manifest_exists),
            json_bool_field("artifact_exists", report.artifact_exists),
            json_bool_field("binary_exists", report.binary_exists),
            json_bool_field("manifest_verified", report.manifest_verified),
            json_bool_field("artifact_verified", report.artifact_verified),
            json_optional_string_field(
                "artifact_container_kind",
                report.artifact_container_kind.as_deref(),
            ),
            match report.artifact_container_version {
                Some(version) => format!("\"artifact_container_version\":{}", version),
                None => "\"artifact_container_version\":null".to_owned(),
            },
            match report.artifact_section_count {
                Some(count) => json_usize_field("artifact_section_count", count),
                None => "\"artifact_section_count\":null".to_owned(),
            },
            json_string_array_field("artifact_section_names", &report.artifact_section_names),
            match report.artifact_section_table_valid {
                Some(valid) => json_bool_field("artifact_section_table_valid", valid),
                None => "\"artifact_section_table_valid\":null".to_owned(),
            },
            match report.lowering_unit_count {
                Some(count) => json_usize_field("lowering_unit_count", count),
                None => "\"lowering_unit_count\":null".to_owned(),
            },
            json_string_array_field("lowering_domain_families", &report.lowering_domain_families),
            json_string_array_field("lowering_targets", &report.lowering_targets),
            artifact_lowering_units_json(&report.lowering_units),
            json_bool_field("ready_to_run", report.ready_to_run),
            json_field("recommended_next_step", &report.recommended_next_step),
            json_field("recommended_command", &report.recommended_command),
            json_field("recommended_reason", &report.recommended_reason),
            json_optional_string_field(
                "manifest_verify_error",
                report.manifest_verify_error.as_deref(),
            ),
            json_optional_string_field(
                "artifact_verify_error",
                report.artifact_verify_error.as_deref(),
            ),
        ],
    );
    append_artifact_output_diagnostic_json_fields(
        &mut out,
        &diagnostics,
        "self_check_ready",
        "self_check_code",
        "self_check_error",
        true,
    );
    append_workflow_link_plan_json_fields(&mut out, diagnostics.link_plan.plan.as_ref());
    out.push('}');
    out
}

fn build_report_domain_unit_record(unit: &nuisc::aot::BuildManifestDomainBuildUnit) -> String {
    let mut fields = vec![
        json_field("package_id", &unit.package_id),
        json_field("domain_family", &unit.domain_family),
        json_field("contract_family", &unit.contract_family),
        json_field("packaging_role", &unit.packaging_role),
        json_bool_field("heterogeneous", unit.is_heterogeneous()),
    ];
    if let Some(value) = unit.abi.as_deref() {
        fields.push(json_field("abi", value));
    }
    if let Some(value) = unit.machine_arch.as_deref() {
        fields.push(json_field("machine_arch", value));
    }
    if let Some(value) = unit.machine_os.as_deref() {
        fields.push(json_field("machine_os", value));
    }
    if let Some(value) = unit.backend_family.as_deref() {
        fields.push(json_field("backend_family", value));
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        fields.push(json_field("selected_lowering_target", value));
    }
    if let Some(value) = unit.artifact_payload_format.as_deref() {
        fields.push(json_field("artifact_payload_format", value));
    }
    if let Some(value) = unit.artifact_payload_blob_bytes {
        fields.push(json_usize_field("artifact_payload_blob_bytes", value));
    }
    format!("{{{}}}", fields.join(","))
}

fn runtime_session_json_fields(
    manifest_verify: Option<&nuisc::aot::BuildManifestVerifyReport>,
) -> Vec<String> {
    let Some(report) = manifest_verify else {
        return vec![
            json_usize_field("heterogeneous_domain_count", 0),
            json_optional_string_field("bridge_registry_path", None),
            json_usize_field("bridge_registry_units", 0),
            json_usize_field("bridge_registry_checked", 0),
            json_usize_field("bridge_registry_entries_checked", 0),
            json_optional_string_field("host_bridge_plan_index_path", None),
            json_usize_field("host_bridge_plan_units", 0),
            json_usize_field("host_bridge_plan_checked", 0),
            json_usize_field("host_bridge_plan_entries_checked", 0),
            json_optional_string_field("lowering_plan_index_path", None),
            json_usize_field("lowering_plan_units", 0),
            json_usize_field("lowering_plan_index_checked", 0),
            json_usize_field("lowering_plan_entries_checked", 0),
            json_usize_field("domain_payload_blobs_checked", 0),
            json_usize_field("domain_payload_blob_sections_checked", 0),
            json_usize_field("domain_payload_contract_sections_checked", 0),
            json_usize_field("domain_payload_lowering_plans_checked", 0),
            json_usize_field("domain_payload_backend_stubs_checked", 0),
            json_usize_field("domain_payload_bridge_plans_checked", 0),
            json_usize_field("domain_bridge_stubs_checked", 0),
        ];
    };
    vec![
        json_usize_field(
            "heterogeneous_domain_count",
            report.heterogeneous_domain_count,
        ),
        json_optional_string_field(
            "bridge_registry_path",
            report.bridge_registry_path.as_deref(),
        ),
        json_usize_field("bridge_registry_units", report.bridge_registry_units),
        json_usize_field("bridge_registry_checked", report.bridge_registry_checked),
        json_usize_field(
            "bridge_registry_entries_checked",
            report.bridge_registry_entries_checked,
        ),
        json_optional_string_field(
            "host_bridge_plan_index_path",
            report.host_bridge_plan_index_path.as_deref(),
        ),
        json_usize_field("host_bridge_plan_units", report.host_bridge_plan_units),
        json_usize_field("host_bridge_plan_checked", report.host_bridge_plan_checked),
        json_usize_field(
            "host_bridge_plan_entries_checked",
            report.host_bridge_plan_entries_checked,
        ),
        json_optional_string_field(
            "lowering_plan_index_path",
            report.lowering_plan_index_path.as_deref(),
        ),
        json_usize_field("lowering_plan_units", report.lowering_plan_units),
        json_usize_field(
            "lowering_plan_index_checked",
            report.lowering_plan_index_checked,
        ),
        json_usize_field(
            "lowering_plan_entries_checked",
            report.lowering_plan_entries_checked,
        ),
        json_usize_field(
            "domain_payload_blobs_checked",
            report.domain_payload_blobs_checked,
        ),
        json_usize_field(
            "domain_payload_blob_sections_checked",
            report.domain_payload_blob_sections_checked,
        ),
        json_usize_field(
            "domain_payload_contract_sections_checked",
            report.domain_payload_contract_sections_checked,
        ),
        json_usize_field(
            "domain_payload_lowering_plans_checked",
            report.domain_payload_lowering_plans_checked,
        ),
        json_usize_field(
            "domain_payload_backend_stubs_checked",
            report.domain_payload_backend_stubs_checked,
        ),
        json_usize_field(
            "domain_payload_bridge_plans_checked",
            report.domain_payload_bridge_plans_checked,
        ),
        json_usize_field(
            "domain_bridge_stubs_checked",
            report.domain_bridge_stubs_checked,
        ),
    ]
}

fn append_runtime_session_json_fields(
    out: &mut String,
    manifest_verify: Option<&nuisc::aot::BuildManifestVerifyReport>,
) {
    append_json_field_strings(out, runtime_session_json_fields(manifest_verify));
}

fn runtime_load_json_fields(artifact_path: Option<&Path>, artifact_verified: bool) -> Vec<String> {
    if !artifact_verified {
        return vec![
            json_bool_field("runtime_load_attempted", false),
            json_bool_field("runtime_load_ok", false),
            json_optional_string_field("runtime_load_error", None),
            json_usize_field("runtime_loaded_domain_units", 0),
            json_usize_field("runtime_loaded_heterogeneous_units", 0),
            json_usize_field("runtime_loaded_payload_blobs", 0),
            json_usize_field("runtime_payload_backed_heterogeneous_units", 0),
            json_usize_field("runtime_cpu_fallback_units", 0),
            json_usize_field("runtime_host_consumable_units", 0),
            json_bool_field("runtime_loaded_bridge_registry", false),
            json_bool_field("runtime_loaded_host_bridge_plan_index", false),
        ];
    }
    let Some(path) = artifact_path else {
        return vec![
            json_bool_field("runtime_load_attempted", false),
            json_bool_field("runtime_load_ok", false),
            json_optional_string_field("runtime_load_error", None),
            json_usize_field("runtime_loaded_domain_units", 0),
            json_usize_field("runtime_loaded_heterogeneous_units", 0),
            json_usize_field("runtime_loaded_payload_blobs", 0),
            json_usize_field("runtime_payload_backed_heterogeneous_units", 0),
            json_usize_field("runtime_cpu_fallback_units", 0),
            json_usize_field("runtime_host_consumable_units", 0),
            json_bool_field("runtime_loaded_bridge_registry", false),
            json_bool_field("runtime_loaded_host_bridge_plan_index", false),
        ];
    };
    match nuis_runtime::RuntimeLoader.load_from_artifact_path(path) {
        Ok(loaded) => {
            let host_consumable = loaded.host_consumable_summary();
            vec![
                json_bool_field("runtime_load_attempted", true),
                json_bool_field("runtime_load_ok", true),
                json_optional_string_field("runtime_load_error", None),
                json_field(
                    "runtime_loaded_lifecycle_entry",
                    &loaded.artifact.lifecycle.bootstrap_entry,
                ),
                json_usize_field("runtime_loaded_domain_units", loaded.domain_units.len()),
                json_usize_field(
                    "runtime_loaded_heterogeneous_units",
                    loaded.heterogeneous_units().count(),
                ),
                json_usize_field(
                    "runtime_loaded_payload_blobs",
                    loaded.domain_payload_blobs.len(),
                ),
                json_usize_field(
                    "runtime_payload_backed_heterogeneous_units",
                    host_consumable.payload_backed_units,
                ),
                json_usize_field(
                    "runtime_cpu_fallback_units",
                    host_consumable.cpu_fallback_units,
                ),
                json_usize_field(
                    "runtime_host_consumable_units",
                    host_consumable.host_consumable_units,
                ),
                json_bool_field(
                    "runtime_loaded_bridge_registry",
                    loaded.bridge_registry.is_some(),
                ),
                json_bool_field(
                    "runtime_loaded_host_bridge_plan_index",
                    loaded.host_bridge_plan_index.is_some(),
                ),
            ]
        }
        Err(error) => vec![
            json_bool_field("runtime_load_attempted", true),
            json_bool_field("runtime_load_ok", false),
            json_optional_string_field("runtime_load_error", Some(&error.to_string())),
            json_usize_field("runtime_loaded_domain_units", 0),
            json_usize_field("runtime_loaded_heterogeneous_units", 0),
            json_usize_field("runtime_loaded_payload_blobs", 0),
            json_usize_field("runtime_payload_backed_heterogeneous_units", 0),
            json_usize_field("runtime_cpu_fallback_units", 0),
            json_usize_field("runtime_host_consumable_units", 0),
            json_bool_field("runtime_loaded_bridge_registry", false),
            json_bool_field("runtime_loaded_host_bridge_plan_index", false),
        ],
    }
}

fn runtime_execution_json_fields(
    artifact_path: Option<&Path>,
    artifact_verified: bool,
) -> Vec<String> {
    if !artifact_verified {
        return runtime_execution_unavailable_fields(false, None);
    }
    let Some(path) = artifact_path else {
        return runtime_execution_unavailable_fields(false, None);
    };
    match runtime_execution_summary(path) {
        Ok((
            domains,
            plan_phases,
            trace_events,
            host_fallback_events,
            kernel_host_reference_events,
        )) => vec![
            json_bool_field("runtime_execution_attempted", true),
            json_bool_field("runtime_execution_ok", true),
            json_optional_string_field("runtime_execution_error", None),
            json_usize_field("runtime_execution_domains", domains),
            json_usize_field("runtime_execution_plan_phases", plan_phases),
            json_usize_field("runtime_execution_trace_events", trace_events),
            json_usize_field(
                "runtime_execution_host_fallback_events",
                host_fallback_events,
            ),
            json_usize_field(
                "runtime_execution_kernel_host_reference_events",
                kernel_host_reference_events,
            ),
        ],
        Err(error) => runtime_execution_unavailable_fields(true, Some(&error)),
    }
}

fn runtime_execution_unavailable_fields(attempted: bool, error: Option<&str>) -> Vec<String> {
    vec![
        json_bool_field("runtime_execution_attempted", attempted),
        json_bool_field("runtime_execution_ok", false),
        json_optional_string_field("runtime_execution_error", error),
        json_usize_field("runtime_execution_domains", 0),
        json_usize_field("runtime_execution_plan_phases", 0),
        json_usize_field("runtime_execution_trace_events", 0),
        json_usize_field("runtime_execution_host_fallback_events", 0),
        json_usize_field("runtime_execution_kernel_host_reference_events", 0),
    ]
}

fn runtime_execution_summary(path: &Path) -> Result<(usize, usize, usize, usize, usize), String> {
    let loaded = nuis_runtime::RuntimeLoader
        .load_from_artifact_path(path)
        .map_err(|error| error.to_string())?;
    let mut adapters = nuis_runtime::AdapterRegistry::new();
    adapters.register(Box::new(build_report_runtime::BuildReportRuntimeAdapter));
    let bridge = nuis_runtime::BridgeExecutor;
    let executor = nuis_runtime::Executor;
    let mut domains = 0usize;
    let mut plan_phases = 0usize;
    let mut trace_events = 0usize;
    let mut host_fallback_events = 0usize;
    let mut kernel_host_reference_events = 0usize;
    for unit in loaded.heterogeneous_units() {
        let prepared = bridge
            .prepare(&loaded, &adapters, &unit.domain_family)
            .map_err(|error| error.to_string())?;
        let plan = executor
            .plan(&prepared)
            .map_err(|error| error.to_string())?;
        let trace = executor
            .execute_prepared_plan(prepared.adapter, &plan)
            .map_err(|error| error.to_string())?;
        host_fallback_events += trace
            .events
            .iter()
            .filter(|event| event.outcome.status == "host-cpu-fallback-complete")
            .count();
        kernel_host_reference_events += trace
            .events
            .iter()
            .filter(|event| event.outcome.status == "kernel-host-reference-dispatch-complete")
            .count();
        domains += 1;
        plan_phases += plan.phases.len();
        trace_events += trace.events.len();
    }
    Ok((
        domains,
        plan_phases,
        trace_events,
        host_fallback_events,
        kernel_host_reference_events,
    ))
}

pub(crate) fn render_build_report_json(input: &Path) -> String {
    let doctor = probe_artifact_doctor(input);
    let diagnostics = collect_artifact_output_diagnostics(input, &doctor);
    let manifest_verify = doctor
        .manifest_path
        .as_ref()
        .filter(|_| doctor.manifest_verified)
        .and_then(|path| nuisc::aot::verify_build_manifest(path).ok());
    let artifact_verify = doctor
        .artifact_path
        .as_ref()
        .filter(|_| doctor.artifact_verified)
        .and_then(|path| nuisc::aot::verify_nuis_compiled_artifact(path).ok());
    let domain_unit_records = manifest_verify
        .as_ref()
        .map(|report| {
            report
                .domain_build_units
                .iter()
                .map(build_report_domain_unit_record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", "build_report"),
            json_field("source_kind", &doctor.source_kind),
            json_field("input", &doctor.input.display().to_string()),
            json_optional_string_field(
                "output_dir",
                doctor
                    .output_dir
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_optional_string_field(
                "manifest_path",
                doctor
                    .manifest_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_optional_string_field(
                "artifact_path",
                doctor
                    .artifact_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_optional_string_field(
                "binary_path",
                doctor
                    .binary_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_bool_field("manifest_verified", doctor.manifest_verified),
            json_bool_field("artifact_verified", doctor.artifact_verified),
            json_bool_field("ready_to_run", doctor.ready_to_run),
            json_field("recommended_next_step", &doctor.recommended_next_step),
            json_field("recommended_command", &doctor.recommended_command),
            json_field("recommended_reason", &doctor.recommended_reason),
            json_usize_field("domain_units_count", domain_unit_records.len()),
            json_object_array_field("domain_units", &domain_unit_records),
        ],
    );
    append_artifact_output_diagnostic_json_fields(
        &mut out,
        &diagnostics,
        "self_check_ready",
        "self_check_code",
        "self_check_error",
        true,
    );
    if let Some(report) = manifest_verify.as_ref() {
        append_json_field_strings(
            &mut out,
            vec![
                json_usize_field(
                    "text_handle_rewrite_helper_hits",
                    report.project_text_handle_rewrite_helper_hits,
                ),
                json_usize_field(
                    "text_handle_rewrite_local_hits",
                    report.project_text_handle_rewrite_local_hits,
                ),
                json_usize_field(
                    "text_handle_rewrite_total_hits",
                    report.project_text_handle_rewrite_helper_hits
                        + report.project_text_handle_rewrite_local_hits,
                ),
                json_field("packaging_mode", &report.packaging_mode),
                json_field("binary_name", &report.artifact_binary_name),
                json_usize_field("binary_bytes", report.artifact_binary_bytes),
                json_field("lifecycle_schema", &report.lifecycle_schema),
                json_field(
                    "lifecycle_bootstrap_entry",
                    &report.lifecycle_bootstrap_entry,
                ),
                json_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
                json_field(
                    "lifecycle_shutdown_policy",
                    &report.lifecycle_shutdown_policy,
                ),
                json_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
                json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
                json_string_array_field(
                    "lifecycle_export_surface",
                    &report.lifecycle_export_surface,
                ),
                json_string_array_field(
                    "lifecycle_runtime_capability_flags",
                    &report.lifecycle_runtime_capability_flags,
                ),
                json_field("cpu_target_abi", &report.cpu_target_abi),
                json_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
                json_field("cpu_target_machine_os", &report.cpu_target_machine_os),
            ],
        );
    }
    if let Some(report) = artifact_verify.as_ref() {
        append_json_field_strings(
            &mut out,
            vec![
                json_bool_field(
                    "artifact_roundtrip_verified",
                    report.artifact_roundtrip_verified,
                ),
                json_bool_field(
                    "lifecycle_contract_consistent",
                    report.lifecycle_contract_consistent,
                ),
                json_bool_field(
                    "lifecycle_runtime_capability_flags_consistent",
                    report.lifecycle_runtime_capability_flags_consistent,
                ),
            ],
        );
    }
    append_runtime_session_json_fields(&mut out, manifest_verify.as_ref());
    append_json_field_strings(
        &mut out,
        runtime_load_json_fields(doctor.artifact_path.as_deref(), doctor.artifact_verified),
    );
    append_json_field_strings(
        &mut out,
        runtime_execution_json_fields(doctor.artifact_path.as_deref(), doctor.artifact_verified),
    );
    append_json_field_strings(
        &mut out,
        runtime_host_yir::runtime_host_yir_json_fields(
            doctor.artifact_path.as_deref(),
            doctor.artifact_verified,
        ),
    );
    append_workflow_link_plan_json_fields(&mut out, diagnostics.link_plan.plan.as_ref());
    out.push('}');
    out
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
        "  link_plan_domain_units: {}",
        diagnostics
            .link_plan
            .as_ref()
            .map(|plan| plan.domain_units.len())
            .unwrap_or(0)
    );
    if let Some(plan) = diagnostics.link_plan.plan.as_ref() {
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

fn handle_build_report(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        println!("{}", render_build_report_json(&input));
        return Ok(());
    }
    let doctor = probe_artifact_doctor(&input);
    let diagnostics = collect_artifact_output_diagnostics(&input, &doctor);
    let manifest_verify = doctor
        .manifest_path
        .as_ref()
        .filter(|_| doctor.manifest_verified)
        .and_then(|path| nuisc::aot::verify_build_manifest(path).ok());
    let artifact_verify = doctor
        .artifact_path
        .as_ref()
        .filter(|_| doctor.artifact_verified)
        .and_then(|path| nuisc::aot::verify_nuis_compiled_artifact(path).ok());
    println!("build report: {}", doctor.input.display());
    println!("  source_kind: {}", doctor.source_kind);
    println!("  ready_to_run: {}", doctor.ready_to_run);
    println!(
        "  artifact_diagnostic_code: {}",
        diagnostics.artifact_diagnostic_code
    );
    println!("  self_check_ready: {}", diagnostics.self_check.ready);
    println!("  self_check_code: {}", diagnostics.self_check.code);
    println!("  recommended_next_step: {}", doctor.recommended_next_step);
    println!("  recommended_command: {}", doctor.recommended_command);
    if let Some(error) = diagnostics.self_check.error.as_deref() {
        println!("  self_check_error: {}", error);
    }
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
    if let Some(report) = manifest_verify.as_ref() {
        println!(
            "  text_handle_rewrite_helper_hits: {}",
            report.project_text_handle_rewrite_helper_hits
        );
        println!(
            "  text_handle_rewrite_local_hits: {}",
            report.project_text_handle_rewrite_local_hits
        );
        println!(
            "  text_handle_rewrite_total_hits: {}",
            report.project_text_handle_rewrite_helper_hits
                + report.project_text_handle_rewrite_local_hits
        );
        println!("  packaging_mode: {}", report.packaging_mode);
        println!("  binary_name: {}", report.artifact_binary_name);
        println!("  binary_bytes: {}", report.artifact_binary_bytes);
        println!("  cpu_target_abi: {}", report.cpu_target_abi);
        println!(
            "  lifecycle_bootstrap_entry: {}",
            report.lifecycle_bootstrap_entry
        );
        println!("  lifecycle_tick_policy: {}", report.lifecycle_tick_policy);
        println!(
            "  lifecycle_shutdown_policy: {}",
            report.lifecycle_shutdown_policy
        );
        println!("  lifecycle_yalivia_rpc: {}", report.lifecycle_yalivia_rpc);
        println!(
            "  lifecycle_runtime_capability_flags: {}",
            if report.lifecycle_runtime_capability_flags.is_empty() {
                "<none>".to_owned()
            } else {
                report.lifecycle_runtime_capability_flags.join(", ")
            }
        );
        println!(
            "  heterogeneous_domain_count: {}",
            report.heterogeneous_domain_count
        );
        println!(
            "  bridge_registry_path: {}",
            report.bridge_registry_path.as_deref().unwrap_or("<none>")
        );
        println!("  bridge_registry_units: {}", report.bridge_registry_units);
        println!(
            "  bridge_registry_checked: {}",
            report.bridge_registry_checked
        );
        println!(
            "  bridge_registry_entries_checked: {}",
            report.bridge_registry_entries_checked
        );
        println!(
            "  host_bridge_plan_index_path: {}",
            report
                .host_bridge_plan_index_path
                .as_deref()
                .unwrap_or("<none>")
        );
        println!(
            "  host_bridge_plan_units: {}",
            report.host_bridge_plan_units
        );
        println!(
            "  host_bridge_plan_checked: {}",
            report.host_bridge_plan_checked
        );
        println!(
            "  host_bridge_plan_entries_checked: {}",
            report.host_bridge_plan_entries_checked
        );
        println!(
            "  lowering_plan_index_path: {}",
            report
                .lowering_plan_index_path
                .as_deref()
                .unwrap_or("<none>")
        );
        println!("  lowering_plan_units: {}", report.lowering_plan_units);
        println!(
            "  lowering_plan_index_checked: {}",
            report.lowering_plan_index_checked
        );
        println!(
            "  lowering_plan_entries_checked: {}",
            report.lowering_plan_entries_checked
        );
        println!("  domain_units: {}", report.domain_build_units.len());
        for unit in &report.domain_build_units {
            let abi = unit.abi.as_deref().unwrap_or("<none>");
            let lowering = unit.selected_lowering_target.as_deref().unwrap_or("<none>");
            let backend = unit.backend_family.as_deref().unwrap_or("<none>");
            println!(
                "  domain_unit: {} package={} role={} abi={} lowering={} backend={}",
                unit.domain_family, unit.package_id, unit.packaging_role, abi, lowering, backend
            );
        }
    } else {
        println!("  packaging_mode: <unavailable>");
        println!("  domain_units: 0");
    }
    if let Some(report) = artifact_verify.as_ref() {
        println!(
            "  artifact_roundtrip_verified: {}",
            report.artifact_roundtrip_verified
        );
        println!(
            "  lifecycle_contract_consistent: {}",
            report.lifecycle_contract_consistent
        );
        println!(
            "  lifecycle_runtime_capability_flags_consistent: {}",
            report.lifecycle_runtime_capability_flags_consistent
        );
    }
    if doctor.artifact_verified {
        if let Some(path) = doctor.artifact_path.as_ref() {
            match nuis_runtime::RuntimeLoader.load_from_artifact_path(path) {
                Ok(loaded) => {
                    let host_consumable = loaded.host_consumable_summary();
                    println!("  runtime_load_attempted: true");
                    println!("  runtime_load_ok: true");
                    println!(
                        "  runtime_loaded_lifecycle_entry: {}",
                        loaded.artifact.lifecycle.bootstrap_entry
                    );
                    println!(
                        "  runtime_loaded_domain_units: {}",
                        loaded.domain_units.len()
                    );
                    println!(
                        "  runtime_loaded_heterogeneous_units: {}",
                        loaded.heterogeneous_units().count()
                    );
                    println!(
                        "  runtime_loaded_payload_blobs: {}",
                        loaded.domain_payload_blobs.len()
                    );
                    println!(
                        "  runtime_payload_backed_heterogeneous_units: {}",
                        host_consumable.payload_backed_units
                    );
                    println!(
                        "  runtime_cpu_fallback_units: {}",
                        host_consumable.cpu_fallback_units
                    );
                    println!(
                        "  runtime_host_consumable_units: {}",
                        host_consumable.host_consumable_units
                    );
                    println!(
                        "  runtime_loaded_bridge_registry: {}",
                        loaded.bridge_registry.is_some()
                    );
                    println!(
                        "  runtime_loaded_host_bridge_plan_index: {}",
                        loaded.host_bridge_plan_index.is_some()
                    );
                }
                Err(error) => {
                    println!("  runtime_load_attempted: true");
                    println!("  runtime_load_ok: false");
                    println!("  runtime_load_error: {}", error);
                }
            }
            match runtime_execution_summary(path) {
                Ok((
                    domains,
                    plan_phases,
                    trace_events,
                    host_fallback_events,
                    kernel_host_reference_events,
                )) => {
                    println!("  runtime_execution_attempted: true");
                    println!("  runtime_execution_ok: true");
                    println!("  runtime_execution_domains: {}", domains);
                    println!("  runtime_execution_plan_phases: {}", plan_phases);
                    println!("  runtime_execution_trace_events: {}", trace_events);
                    println!(
                        "  runtime_execution_host_fallback_events: {}",
                        host_fallback_events
                    );
                    println!(
                        "  runtime_execution_kernel_host_reference_events: {}",
                        kernel_host_reference_events
                    );
                }
                Err(error) => {
                    println!("  runtime_execution_attempted: true");
                    println!("  runtime_execution_ok: false");
                    println!("  runtime_execution_error: {}", error);
                }
            }
            match runtime_host_yir::summary(path) {
                Ok(Some((yir_path, summary))) => {
                    println!("  runtime_host_yir_attempted: true");
                    println!("  runtime_host_yir_ok: true");
                    println!("  runtime_host_yir_path: {}", yir_path);
                    println!("  runtime_host_yir_nodes: {}", summary.nodes_executed);
                    println!(
                        "  runtime_host_yir_kernel_nodes: {}",
                        summary.kernel_nodes_executed
                    );
                    println!(
                        "  runtime_host_yir_tensor_values: {}",
                        summary.tensor_values
                    );
                    println!(
                        "  runtime_host_yir_scalar_values: {}",
                        summary.scalar_values
                    );
                    println!("  runtime_host_yir_frame_values: {}", summary.frame_values);
                    println!(
                        "  runtime_host_yir_integer_checksum: {}",
                        summary.integer_checksum
                    );
                    println!(
                        "  runtime_host_yir_kernel_integer_checksum: {}",
                        summary.kernel_integer_checksum
                    );
                }
                Ok(None) => {
                    println!("  runtime_host_yir_attempted: false");
                    println!("  runtime_host_yir_ok: false");
                    println!("  runtime_host_yir_skip_reason: host_ffi_externs_present_or_no_yir");
                }
                Err(error) => {
                    println!("  runtime_host_yir_attempted: true");
                    println!("  runtime_host_yir_ok: false");
                    println!("  runtime_host_yir_error: {}", error);
                }
            }
        }
    } else {
        println!("  runtime_load_attempted: false");
        println!("  runtime_load_ok: false");
        println!("  runtime_execution_attempted: false");
        println!("  runtime_execution_ok: false");
        println!("  runtime_execution_host_fallback_events: 0");
        println!("  runtime_execution_kernel_host_reference_events: 0");
        println!("  runtime_host_yir_attempted: false");
        println!("  runtime_host_yir_ok: false");
    }
    println!(
        "  link_plan_available: {}",
        diagnostics.link_plan.plan.is_some()
    );
    if let Some(plan) = diagnostics.link_plan.plan.as_ref() {
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

pub(crate) struct ArtifactDoctorReport {
    pub(crate) source_kind: String,
    pub(crate) input: PathBuf,
    pub(crate) output_dir: Option<PathBuf>,
    pub(crate) manifest_path: Option<PathBuf>,
    pub(crate) artifact_path: Option<PathBuf>,
    pub(crate) binary_path: Option<PathBuf>,
    pub(crate) manifest_exists: bool,
    pub(crate) artifact_exists: bool,
    pub(crate) binary_exists: bool,
    pub(crate) manifest_verified: bool,
    pub(crate) artifact_verified: bool,
    pub(crate) artifact_container_kind: Option<String>,
    pub(crate) artifact_container_version: Option<u16>,
    pub(crate) artifact_section_count: Option<usize>,
    pub(crate) artifact_section_names: Vec<String>,
    pub(crate) artifact_section_table_valid: Option<bool>,
    pub(crate) lowering_unit_count: Option<usize>,
    pub(crate) lowering_domain_families: Vec<String>,
    pub(crate) lowering_targets: Vec<String>,
    pub(crate) lowering_units: Vec<nuisc::aot::NuisCompiledArtifactLoweringUnitInspect>,
    pub(crate) ready_to_run: bool,
    pub(crate) recommended_next_step: String,
    pub(crate) recommended_command: String,
    pub(crate) recommended_reason: String,
    pub(crate) manifest_verify_error: Option<String>,
    pub(crate) artifact_verify_error: Option<String>,
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
    println!(
        "    nuis run-artifact [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>"
    );
    println!(
        "    nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
    );
    println!("  general:");
    println!("    nuis status");
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
