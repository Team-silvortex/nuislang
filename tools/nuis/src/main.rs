mod cli;
mod galaxy;
mod json_surface;
mod surface_render;

use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use json_surface::workflow_contract_json_fields;
use nuis_semantics::model::{
    AstExpr, AstExternFunction, AstFunction, AstModule, AstParam, AstStmt, AstTypeRef,
    AstVisibility,
};
use surface_render::append_json_field_strings;

struct BuildReportRuntimeAdapter;

impl nuis_runtime::DomainAdapter for BuildReportRuntimeAdapter {
    fn adapter_id(&self) -> &'static str {
        "nuis-build-report-runtime-adapter"
    }

    fn supports(&self, _unit: &nuis_artifact::BuildManifestDomainBuildUnit) -> bool {
        true
    }
}

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

fn handle_test(
    input: std::path::PathBuf,
    list: bool,
    ignored_only: bool,
    include_ignored: bool,
    exact: bool,
    filter: Option<String>,
) -> Result<(), String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        if success_logs_enabled() {
            println!("test: checking project {}", project.manifest.name);
        }
        handle_check(input.clone())?;
        let mut paths = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<BTreeSet<_>>();
        let mut collected = 0usize;
        if success_logs_enabled() {
            if project.manifest.tests.is_empty() {
                println!("  no explicit tests declared");
            } else {
                println!("  declared tests: {}", project.manifest.tests.len());
            }
        }
        if !project.manifest.tests.is_empty() {
            for relative in &project.manifest.tests {
                paths.insert(project.root.join(relative));
            }
        }
        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        for path in paths {
            let report = run_language_tests_for_source_file(
                &path,
                filter.as_deref(),
                list,
                ignored_only,
                include_ignored,
                exact,
            )?;
            collected += report.collected;
            passed += report.passed;
            failed += report.failed;
            skipped += report.skipped;
        }
        if success_logs_enabled() {
            println!("  collected language tests: {}", collected);
        }
        if list {
            if success_logs_enabled() {
                println!("  listed language tests: {}", collected);
            }
            return Ok(());
        }
        if success_logs_enabled() {
            println!("  executed language tests: {}", passed + failed + skipped);
            println!("  passed: {}", passed);
            println!("  failed: {}", failed);
            println!("  skipped: {}", skipped);
        }
        if failed > 0 {
            return Err(format!(
                "project test run failed: {failed} language test(s) failed"
            ));
        }
        if success_logs_enabled() {
            if collected == 0 {
                println!("  result: project check passed");
            } else {
                println!("  result: all discovered language tests passed");
            }
        }
        Ok(())
    } else {
        if success_logs_enabled() {
            println!("test: {}", input.display());
        }
        let report = run_language_tests_for_source_file(
            &input,
            filter.as_deref(),
            list,
            ignored_only,
            include_ignored,
            exact,
        )?;
        if report.collected == 0 {
            handle_check(input.clone())?;
        }
        if list {
            if success_logs_enabled() {
                println!("  listed language tests: {}", report.collected);
            }
            return Ok(());
        }
        if report.failed > 0 {
            return Err(format!(
                "test run failed: {} language test(s) failed",
                report.failed
            ));
        }
        if success_logs_enabled() {
            println!("  result: passed");
        }
        Ok(())
    }
}

fn handle_bench(
    input: std::path::PathBuf,
    list: bool,
    json: bool,
    exact: bool,
    filter: Option<String>,
) -> Result<(), String> {
    if json {
        let source_kind = if nuisc::project::is_project_input(&input) {
            "project"
        } else {
            "single-file"
        };
        let report = collect_language_benchmark_run_report(&input, filter.as_deref(), list, exact)?;
        println!(
            "{}",
            benchmark_run_report_json(&input, source_kind, list, exact, filter.as_deref(), &report)
        );
        if !list && (report.failed > 0 || report.timed_out > 0) {
            return Err(format!(
                "benchmark run failed: {} failed, {} timed out",
                report.failed, report.timed_out
            ));
        }
        return Ok(());
    }
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        if success_logs_enabled() {
            println!("bench: checking project {}", project.manifest.name);
        }
        handle_check(input.clone())?;
        let paths = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<BTreeSet<_>>();
        let mut collected = 0usize;
        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut timed_out = 0usize;
        let mut text_handle_rewrite_helper_hits = 0usize;
        let mut text_handle_rewrite_local_hits = 0usize;
        for path in paths {
            let report =
                run_language_benchmarks_for_source_file(&path, filter.as_deref(), list, exact)?;
            collected += report.collected;
            completed += report.completed;
            failed += report.failed;
            timed_out += report.timed_out;
            text_handle_rewrite_helper_hits += report.text_handle_rewrite_helper_hits;
            text_handle_rewrite_local_hits += report.text_handle_rewrite_local_hits;
        }
        if success_logs_enabled() {
            println!("  collected language benchmarks: {}", collected);
            println!(
                "  text_handle_rewrite_helper_hits: {}",
                text_handle_rewrite_helper_hits
            );
            println!(
                "  text_handle_rewrite_local_hits: {}",
                text_handle_rewrite_local_hits
            );
            println!(
                "  text_handle_rewrite_total_hits: {}",
                text_handle_rewrite_helper_hits + text_handle_rewrite_local_hits
            );
        }
        if list {
            if success_logs_enabled() {
                println!("  listed language benchmarks: {}", collected);
            }
            return Ok(());
        }
        if success_logs_enabled() {
            println!(
                "  executed language benchmarks: {}",
                completed + failed + timed_out
            );
            println!("  completed: {}", completed);
            println!("  failed: {}", failed);
            println!("  timed_out: {}", timed_out);
        }
        if failed > 0 || timed_out > 0 {
            return Err(format!(
                "project benchmark run failed: {failed} failed, {timed_out} timed out"
            ));
        }
        if success_logs_enabled() {
            if collected == 0 {
                println!("  result: project check passed");
            } else {
                println!("  result: all discovered language benchmarks completed");
            }
        }
        Ok(())
    } else {
        println!("bench: {}", input.display());
        let report =
            run_language_benchmarks_for_source_file(&input, filter.as_deref(), list, exact)?;
        println!(
            "  text_handle_rewrite_helper_hits: {}",
            report.text_handle_rewrite_helper_hits
        );
        println!(
            "  text_handle_rewrite_local_hits: {}",
            report.text_handle_rewrite_local_hits
        );
        println!(
            "  text_handle_rewrite_total_hits: {}",
            report.text_handle_rewrite_helper_hits + report.text_handle_rewrite_local_hits
        );
        if report.collected == 0 {
            handle_check(input.clone())?;
        }
        if list {
            println!("  listed language benchmarks: {}", report.collected);
            return Ok(());
        }
        if report.failed > 0 || report.timed_out > 0 {
            return Err(format!(
                "benchmark run failed: {} failed, {} timed out",
                report.failed, report.timed_out
            ));
        }
        println!("  result: passed");
        Ok(())
    }
}

fn collect_language_benchmark_run_report(
    input: &Path,
    filter: Option<&str>,
    list_only: bool,
    exact: bool,
) -> Result<LanguageBenchmarkRunReport, String> {
    if nuisc::project::is_project_input(input) {
        let project = nuisc::project::load_project(input)?;
        // Keep `bench --json` behavior aligned with the text path without printing a check summary.
        let resolved = nuisc::pipeline::resolve_compile_input(input)?;
        let _ = resolved.compile()?;
        let paths = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<BTreeSet<_>>();
        let mut collected = 0usize;
        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut timed_out = 0usize;
        let mut text_handle_rewrite_helper_hits = 0usize;
        let mut text_handle_rewrite_local_hits = 0usize;
        let mut records = Vec::new();
        for path in paths {
            let report =
                collect_language_benchmarks_for_source_file(&path, filter, list_only, exact)?;
            collected += report.collected;
            completed += report.completed;
            failed += report.failed;
            timed_out += report.timed_out;
            text_handle_rewrite_helper_hits += report.text_handle_rewrite_helper_hits;
            text_handle_rewrite_local_hits += report.text_handle_rewrite_local_hits;
            records.extend(report.records);
        }
        Ok(LanguageBenchmarkRunReport {
            collected,
            completed,
            failed,
            timed_out,
            text_handle_rewrite_helper_hits,
            text_handle_rewrite_local_hits,
            records,
        })
    } else {
        let report = collect_language_benchmarks_for_source_file(input, filter, list_only, exact)?;
        if report.collected == 0 {
            let resolved = nuisc::pipeline::resolve_compile_input(input)?;
            let _ = resolved.compile()?;
        }
        Ok(report)
    }
}

struct LanguageTestRunReport {
    collected: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
}

struct LanguageBenchmarkRunReport {
    collected: usize,
    completed: usize,
    failed: usize,
    timed_out: usize,
    text_handle_rewrite_helper_hits: usize,
    text_handle_rewrite_local_hits: usize,
    records: Vec<BenchmarkRunRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BenchmarkRunRecord {
    source: String,
    function_name: String,
    label: String,
    status: &'static str,
    warmup_iters: usize,
    measure_iters: usize,
    note: Option<String>,
    measurement: Option<BenchmarkMeasurement>,
    clock_policy: Option<&'static str>,
    resolved_clock_bridge: Option<&'static str>,
    resolved_clock_surface: Option<&'static str>,
    declared_clock_domain: Option<&'static str>,
    declared_clock_domain_code: Option<i64>,
    resolved_clock_domain: Option<&'static str>,
    resolved_clock_domain_code: Option<i64>,
    resolved_clock_source: Option<&'static str>,
}

fn benchmark_run_report_json(
    input: &Path,
    source_kind: &str,
    list_only: bool,
    exact: bool,
    filter: Option<&str>,
    report: &LanguageBenchmarkRunReport,
) -> String {
    let benchmarks = report
        .records
        .iter()
        .map(benchmark_run_record_json)
        .collect::<Vec<_>>();
    let mut out = String::from("{");
    for field in [
        json_field("kind", "nuis_benchmark_run"),
        json_field("source_kind", source_kind),
        json_field("input", &input.display().to_string()),
        json_bool_field("list_only", list_only),
        json_bool_field("exact", exact),
        json_optional_string_field("filter", filter),
        json_usize_field("collected", report.collected),
        json_usize_field("completed", report.completed),
        json_usize_field("failed", report.failed),
        json_usize_field("timed_out", report.timed_out),
        json_usize_field(
            "text_handle_rewrite_helper_hits",
            report.text_handle_rewrite_helper_hits,
        ),
        json_usize_field(
            "text_handle_rewrite_local_hits",
            report.text_handle_rewrite_local_hits,
        ),
        json_usize_field(
            "text_handle_rewrite_total_hits",
            report.text_handle_rewrite_helper_hits + report.text_handle_rewrite_local_hits,
        ),
        json_object_array_field("benchmarks", &benchmarks),
        json_field(
            "result",
            if list_only {
                "listed"
            } else if report.failed > 0 || report.timed_out > 0 {
                "failed"
            } else {
                "passed"
            },
        ),
    ] {
        if !out.ends_with('{') {
            out.push(',');
        }
        out.push_str(&field);
    }
    out.push('}');
    out
}

fn benchmark_run_record_json(record: &BenchmarkRunRecord) -> String {
    let mut out = String::from("{");
    for field in [
        json_field("source", &record.source),
        json_field("function", &record.function_name),
        json_field("label", &record.label),
        json_field("status", record.status),
        json_usize_field("warmup_iters", record.warmup_iters),
        json_usize_field("measure_iters", record.measure_iters),
        json_optional_string_field("note", record.note.as_deref()),
        json_optional_string_field("clock_policy", record.clock_policy),
        json_optional_string_field("resolved_clock_bridge", record.resolved_clock_bridge),
        json_optional_string_field("resolved_clock_surface", record.resolved_clock_surface),
        json_optional_string_field("declared_clock_domain", record.declared_clock_domain),
        json_optional_i64_field(
            "declared_clock_domain_code",
            record.declared_clock_domain_code,
        ),
        json_optional_string_field("resolved_clock_domain", record.resolved_clock_domain),
        json_optional_i64_field(
            "resolved_clock_domain_code",
            record.resolved_clock_domain_code,
        ),
        json_optional_string_field("resolved_clock_source", record.resolved_clock_source),
    ] {
        if !out.ends_with('{') {
            out.push(',');
        }
        out.push_str(&field);
    }
    if let Some(measurement) = record.measurement {
        for field in [
            json_field("run_mode", measurement.run_mode),
            json_usize_field("sample_count", measurement.sample_count),
            json_optional_u128_field("min_ns", measurement.min_ns),
            json_u128_field("avg_ns", measurement.avg_ns),
            json_optional_u128_field("max_ns", measurement.max_ns),
            json_u128_field("total_ns", measurement.total_ns),
        ] {
            out.push(',');
            out.push_str(&field);
        }
    } else {
        for field in [
            "\"run_mode\":null",
            "\"sample_count\":null",
            "\"min_ns\":null",
            "\"avg_ns\":null",
            "\"max_ns\":null",
            "\"total_ns\":null",
        ] {
            out.push(',');
            out.push_str(field);
        }
    }
    out.push('}');
    out
}

struct TestVerdict {
    status: &'static str,
    counted_pass: bool,
    note: Option<String>,
    clock_policy: Option<&'static str>,
    resolved_clock_bridge: Option<&'static str>,
    resolved_clock_surface: Option<&'static str>,
    declared_clock_domain: Option<&'static str>,
    declared_clock_domain_code: Option<i64>,
    resolved_clock_domain: Option<&'static str>,
    resolved_clock_domain_code: Option<i64>,
    resolved_clock_source: Option<&'static str>,
}

struct BenchmarkVerdict {
    status: &'static str,
    note: Option<String>,
    warmup_iters: usize,
    measure_iters: usize,
    measurement: Option<BenchmarkMeasurement>,
    clock_policy: Option<&'static str>,
    resolved_clock_bridge: Option<&'static str>,
    resolved_clock_surface: Option<&'static str>,
    declared_clock_domain: Option<&'static str>,
    declared_clock_domain_code: Option<i64>,
    resolved_clock_domain: Option<&'static str>,
    resolved_clock_domain_code: Option<i64>,
    resolved_clock_source: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BenchmarkMeasurement {
    run_mode: &'static str,
    sample_count: usize,
    min_ns: Option<u128>,
    max_ns: Option<u128>,
    avg_ns: u128,
    total_ns: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RunnerClockResolution {
    domain: nuis_semantics::model::TestClockDomain,
    bridge: nuis_semantics::model::NirHostTimingBridge,
    source: &'static str,
}

fn run_language_tests_for_source_file(
    path: &Path,
    filter: Option<&str>,
    list_only: bool,
    ignored_only: bool,
    include_ignored: bool,
    exact: bool,
) -> Result<LanguageTestRunReport, String> {
    let verbose = success_logs_enabled();
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let ast = nuisc::frontend::parse_nuis_ast(&source)?;
    let nir = nuisc::frontend::lower_ast_to_nir(&ast)?;
    let tests = nuisc::frontend::collect_nir_tests(&nir);
    let matched = ast
        .functions
        .iter()
        .filter(|function| function.test_name.is_some())
        .filter(|function| {
            test_matches_filter(
                function.name.as_str(),
                function.test_name.as_deref(),
                filter,
                exact,
            )
        })
        .filter(|function| {
            test_matches_ignored_mode(function.test_ignored, ignored_only, include_ignored)
        })
        .collect::<Vec<_>>();
    if verbose && !matched.is_empty() {
        println!("  source: {}", path.display());
    }
    if verbose {
        println!("  collected language tests: {}", matched.len());
        for function in &tests {
            if !test_matches_filter(
                function.name.as_str(),
                function.test_name.as_deref(),
                filter,
                exact,
            ) {
                continue;
            }
            if !test_matches_ignored_mode(function.test_ignored, ignored_only, include_ignored) {
                continue;
            }
            let mut line = format!(
                "  test_fn: {} ({})",
                function.name,
                function.test_name.as_deref().unwrap_or(&function.name)
            );
            if function.test_ignored {
                line.push_str(" [ignored]");
            }
            if function.test_should_fail {
                line.push_str(" [should_fail]");
            }
            if let Some(reason) = &function.test_reason {
                line.push_str(&format!(" [reason: {}]", reason));
            }
            if let Some(timeout_ms) = function.test_timeout_ms {
                line.push_str(&format!(" [timeout_ms: {}]", timeout_ms));
            }
            if let Some(clock_domain) = &function.test_clock_domain {
                line.push_str(&format!(" [clock_domain: {}]", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.test_clock_policy {
                line.push_str(&format!(" [clock_policy: {}]", clock_policy.as_str()));
            }
            println!("{line}");
        }
    }
    if list_only {
        return Ok(LanguageTestRunReport {
            collected: matched.len(),
            passed: 0,
            failed: 0,
            skipped: 0,
        });
    }
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    for function in matched {
        let verdict = execute_language_test(path, &ast, function, ignored_only || include_ignored)?;
        let show_record = verbose || !verdict.counted_pass || verdict.status == "SKIP";
        if show_record {
            let label = function.test_name.as_deref().unwrap_or(&function.name);
            if verbose {
                println!("  {} {}", verdict.status, label);
                if let Some(reason) = &function.test_reason {
                    println!("    reason: {}", reason);
                }
                if let Some(clock_policy) = verdict.clock_policy {
                    println!("    clock_policy: {}", clock_policy);
                }
                if let Some(clock_bridge) = verdict.resolved_clock_bridge {
                    println!("    resolved_clock_bridge: {}", clock_bridge);
                }
                if let Some(clock_surface) = verdict.resolved_clock_surface {
                    println!("    resolved_clock_surface: {}", clock_surface);
                }
                if let Some(clock_domain) = verdict.declared_clock_domain {
                    let code = verdict
                        .declared_clock_domain_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    println!("    declared_clock_domain: {}{}", clock_domain, code);
                }
                if let Some(clock_domain) = verdict.resolved_clock_domain {
                    let code = verdict
                        .resolved_clock_domain_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    println!("    resolved_clock_domain: {}{}", clock_domain, code);
                }
                if let Some(source) = verdict.resolved_clock_source {
                    println!("    resolved_clock_source: {}", source);
                }
                if let Some(note) = &verdict.note {
                    println!("    note: {}", note);
                }
            } else {
                let mut line = format!("  {} {}", verdict.status, label);
                if let Some(reason) = &function.test_reason {
                    line.push_str(&format!(" [reason={}]", reason));
                }
                if let Some(clock_domain) = verdict.resolved_clock_domain {
                    line.push_str(&format!(" [clock={}]", clock_domain));
                }
                if let Some(clock_policy) = verdict.clock_policy {
                    line.push_str(&format!(" [policy={}]", clock_policy));
                }
                if let Some(note) = &verdict.note {
                    line.push_str(&format!(" [note={}]", note));
                }
                println!("{line}");
            }
        }
        if verdict.status == "SKIP" {
            skipped += 1;
        } else if verdict.counted_pass {
            passed += 1;
        } else {
            failed += 1;
        }
    }
    if verbose || failed > 0 {
        println!("  executed language tests: {}", passed + failed + skipped);
        println!("  passed: {}", passed);
        println!("  failed: {}", failed);
        println!("  skipped: {}", skipped);
    }
    Ok(LanguageTestRunReport {
        collected: tests
            .iter()
            .filter(|function| {
                test_matches_filter(
                    function.name.as_str(),
                    function.test_name.as_deref(),
                    filter,
                    exact,
                )
            })
            .filter(|function| {
                test_matches_ignored_mode(function.test_ignored, ignored_only, include_ignored)
            })
            .count(),
        passed,
        failed,
        skipped,
    })
}

fn run_language_benchmarks_for_source_file(
    path: &Path,
    filter: Option<&str>,
    list_only: bool,
    exact: bool,
) -> Result<LanguageBenchmarkRunReport, String> {
    let verbose = success_logs_enabled();
    let report = collect_language_benchmarks_for_source_file(path, filter, list_only, exact)?;
    if verbose && !report.records.is_empty() {
        println!("  source: {}", path.display());
    }
    if verbose {
        println!("  collected language benchmarks: {}", report.collected);
        println!(
            "  text_handle_rewrite_helper_hits: {}",
            report.text_handle_rewrite_helper_hits
        );
        println!(
            "  text_handle_rewrite_local_hits: {}",
            report.text_handle_rewrite_local_hits
        );
        println!(
            "  text_handle_rewrite_total_hits: {}",
            report.text_handle_rewrite_helper_hits + report.text_handle_rewrite_local_hits
        );
        for record in &report.records {
            let mut line = format!("  bench_fn: {} ({})", record.function_name, record.label);
            if record.warmup_iters > 0 {
                line.push_str(&format!(" [warmup_iters: {}]", record.warmup_iters));
            }
            line.push_str(&format!(" [measure_iters: {}]", record.measure_iters));
            if let Some(note) = &record.note {
                if record.status == "DISCOVERED" {
                    line.push_str(&format!(" [note: {}]", note));
                }
            }
            if let Some(clock_domain) = record.declared_clock_domain {
                line.push_str(&format!(" [clock_domain: {}]", clock_domain));
            }
            if let Some(clock_policy) = record.clock_policy {
                line.push_str(&format!(" [clock_policy: {}]", clock_policy));
            }
            println!("{line}");
        }
    }
    if list_only {
        return Ok(report);
    }
    for record in &report.records {
        let show_record = verbose || record.status != "OK";
        if show_record {
            if verbose {
                println!("  {} {}", record.status, record.label);
                println!("    warmup_iters: {}", record.warmup_iters);
                println!("    measure_iters: {}", record.measure_iters);
                if let Some(clock_policy) = record.clock_policy {
                    println!("    clock_policy: {}", clock_policy);
                }
                if let Some(clock_bridge) = record.resolved_clock_bridge {
                    println!("    resolved_clock_bridge: {}", clock_bridge);
                }
                if let Some(clock_surface) = record.resolved_clock_surface {
                    println!("    resolved_clock_surface: {}", clock_surface);
                }
                if let Some(clock_domain) = record.declared_clock_domain {
                    let code = record
                        .declared_clock_domain_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    println!("    declared_clock_domain: {}{}", clock_domain, code);
                }
                if let Some(clock_domain) = record.resolved_clock_domain {
                    let code = record
                        .resolved_clock_domain_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    println!("    resolved_clock_domain: {}{}", clock_domain, code);
                }
                if let Some(source) = record.resolved_clock_source {
                    println!("    resolved_clock_source: {}", source);
                }
                if let Some(measurement) = record.measurement {
                    println!("    run_mode: {}", measurement.run_mode);
                    println!("    sample_count: {}", measurement.sample_count);
                    if let Some(min_ns) = measurement.min_ns {
                        println!("    min_ns: {}", min_ns);
                    }
                    println!("    avg_ns: {}", measurement.avg_ns);
                    if let Some(max_ns) = measurement.max_ns {
                        println!("    max_ns: {}", max_ns);
                    }
                    println!("    total_ns: {}", measurement.total_ns);
                }
                if let Some(note) = &record.note {
                    println!("    note: {}", note);
                }
            } else {
                let mut line = format!(
                    "  {} {} [warmup={}] [measure={}]",
                    record.status, record.label, record.warmup_iters, record.measure_iters
                );
                if let Some(clock_domain) = record.resolved_clock_domain {
                    line.push_str(&format!(" [clock={}]", clock_domain));
                }
                if let Some(clock_policy) = record.clock_policy {
                    line.push_str(&format!(" [policy={}]", clock_policy));
                }
                if let Some(note) = &record.note {
                    line.push_str(&format!(" [note={}]", note));
                }
                println!("{line}");
            }
        }
    }
    if verbose || report.failed > 0 || report.timed_out > 0 {
        println!(
            "  executed language benchmarks: {}",
            report.completed + report.failed + report.timed_out
        );
        println!("  completed: {}", report.completed);
        println!("  failed: {}", report.failed);
        println!("  timed_out: {}", report.timed_out);
    }
    Ok(report)
}

fn collect_language_benchmarks_for_source_file(
    path: &Path,
    filter: Option<&str>,
    list_only: bool,
    exact: bool,
) -> Result<LanguageBenchmarkRunReport, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let ast = nuisc::frontend::parse_nuis_ast(&source)?;
    let nir = nuisc::frontend::lower_ast_to_nir(&ast)?;
    let text_handle_rewrite = summarize_text_handle_rewrites_from_nir(&nir);
    let benchmarks = nuisc::frontend::collect_nir_benchmarks(&nir);
    let matched = ast
        .functions
        .iter()
        .filter(|function| function.benchmark_name.is_some())
        .filter(|function| {
            test_matches_filter(
                function.name.as_str(),
                function.benchmark_name.as_deref(),
                filter,
                exact,
            )
        })
        .collect::<Vec<_>>();
    let discovered = benchmarks
        .iter()
        .filter(|function| {
            test_matches_filter(
                function.name.as_str(),
                function.benchmark_name.as_deref(),
                filter,
                exact,
            )
        })
        .map(|function| BenchmarkRunRecord {
            source: path.display().to_string(),
            function_name: function.name.clone(),
            label: function
                .benchmark_name
                .clone()
                .unwrap_or_else(|| function.name.clone()),
            status: "DISCOVERED",
            warmup_iters: function
                .benchmark_warmup_iters
                .unwrap_or(0)
                .try_into()
                .unwrap_or(0),
            measure_iters: function
                .benchmark_measure_iters
                .unwrap_or(1)
                .try_into()
                .unwrap_or(1),
            note: function
                .benchmark_timeout_ms
                .map(|timeout_ms| format!("timeout_ms={timeout_ms}")),
            measurement: None,
            clock_policy: function
                .benchmark_clock_policy
                .map(|policy| policy.as_str()),
            resolved_clock_bridge: None,
            resolved_clock_surface: None,
            declared_clock_domain: function
                .benchmark_clock_domain
                .map(|domain| domain.as_str()),
            declared_clock_domain_code: function.benchmark_clock_domain.map(|domain| domain.code()),
            resolved_clock_domain: None,
            resolved_clock_domain_code: None,
            resolved_clock_source: None,
        })
        .collect::<Vec<_>>();
    if list_only {
        return Ok(LanguageBenchmarkRunReport {
            collected: matched.len(),
            completed: 0,
            failed: 0,
            timed_out: 0,
            text_handle_rewrite_helper_hits: text_handle_rewrite.helper_hits,
            text_handle_rewrite_local_hits: text_handle_rewrite.local_hits,
            records: discovered,
        });
    }
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut timed_out = 0usize;
    let mut records = Vec::with_capacity(matched.len());
    for function in matched {
        let verdict = execute_language_benchmark(path, &ast, function)?;
        let label = function
            .benchmark_name
            .clone()
            .unwrap_or_else(|| function.name.clone());
        match verdict.status {
            "OK" => completed += 1,
            "TIMEOUT" => timed_out += 1,
            _ => failed += 1,
        }
        records.push(BenchmarkRunRecord {
            source: path.display().to_string(),
            function_name: function.name.clone(),
            label,
            status: verdict.status,
            warmup_iters: verdict.warmup_iters,
            measure_iters: verdict.measure_iters,
            note: verdict.note.clone(),
            measurement: verdict.measurement,
            clock_policy: verdict.clock_policy,
            resolved_clock_bridge: verdict.resolved_clock_bridge,
            resolved_clock_surface: verdict.resolved_clock_surface,
            declared_clock_domain: verdict.declared_clock_domain,
            declared_clock_domain_code: verdict.declared_clock_domain_code,
            resolved_clock_domain: verdict.resolved_clock_domain,
            resolved_clock_domain_code: verdict.resolved_clock_domain_code,
            resolved_clock_source: verdict.resolved_clock_source,
        });
    }
    Ok(LanguageBenchmarkRunReport {
        collected: benchmarks
            .iter()
            .filter(|function| {
                test_matches_filter(
                    function.name.as_str(),
                    function.benchmark_name.as_deref(),
                    filter,
                    exact,
                )
            })
            .count(),
        completed,
        failed,
        timed_out,
        text_handle_rewrite_helper_hits: text_handle_rewrite.helper_hits,
        text_handle_rewrite_local_hits: text_handle_rewrite.local_hits,
        records,
    })
}

fn summarize_text_handle_rewrites_from_nir(
    nir: &nuis_semantics::model::NirModule,
) -> nuisc::project::ProjectTextHandleRewriteSummary {
    let mut summary = nuisc::project::ProjectTextHandleRewriteSummary::default();
    for function in &nir.functions {
        for annotation in &function.annotations {
            if annotation.name != "__nuisc_text_handle_rewrite" {
                continue;
            }
            for arg in &annotation.args {
                let Some(name) = arg.name.as_deref() else {
                    continue;
                };
                let nuis_semantics::model::NirAttributeValue::Int(value) = arg.value else {
                    continue;
                };
                if value <= 0 {
                    continue;
                }
                match name {
                    "helper" => summary.helper_hits += value as usize,
                    "local" => summary.local_hits += value as usize,
                    _ => {}
                }
            }
        }
    }
    summary
}

fn test_matches_ignored_mode(
    test_ignored: bool,
    ignored_only: bool,
    include_ignored: bool,
) -> bool {
    if ignored_only {
        test_ignored
    } else if include_ignored {
        true
    } else {
        !test_ignored
    }
}

fn test_matches_filter(name: &str, label: Option<&str>, filter: Option<&str>, exact: bool) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    if exact {
        name == filter || label.map(|label| label == filter).unwrap_or(false)
    } else {
        name.contains(filter) || label.map(|label| label.contains(filter)).unwrap_or(false)
    }
}

fn execute_language_test(
    input_path: &Path,
    ast: &AstModule,
    test_function: &AstFunction,
    run_ignored: bool,
) -> Result<TestVerdict, String> {
    if test_function.test_ignored && !run_ignored {
        return Ok(TestVerdict {
            status: "SKIP",
            counted_pass: false,
            note: None,
            clock_policy: None,
            resolved_clock_bridge: None,
            resolved_clock_surface: None,
            declared_clock_domain: None,
            declared_clock_domain_code: None,
            resolved_clock_domain: None,
            resolved_clock_domain_code: None,
            resolved_clock_source: None,
        });
    }
    let harness_ast = build_test_harness_module(ast, test_function);
    let artifacts = nuisc::pipeline::compile_ast(harness_ast)?;
    let output_dir = temp_test_output_dir(
        test_function
            .test_name
            .as_deref()
            .unwrap_or(&test_function.name),
    );
    let cpu_target =
        nuisc::aot::resolve_cpu_build_target(Path::new("nustar-packages"), None, None, None)?;
    let written = nuisc::aot::write_and_link(
        input_path,
        &output_dir,
        &artifacts.ast,
        &artifacts.nir,
        &artifacts.yir,
        &artifacts.llvm_ir,
        &cpu_target,
    )?;
    let mut child = Command::new(&written.binary_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to run `{}`: {error}", written.binary_path))?;
    let declared_clock_domain = test_function.test_clock_domain;
    let resolved_clock = test_function
        .test_timeout_ms
        .map(|_| resolve_runner_clock_domain(declared_clock_domain));
    let raw_outcome = wait_for_test_child(
        &mut child,
        test_function.test_timeout_ms,
        resolved_clock.map(|clock| clock.domain),
    )?;
    let (status, counted_pass, note) = match raw_outcome {
        RawTestOutcome::Completed(status) => {
            let raw_ok = status.code().unwrap_or_default() != 0;
            if test_function.test_should_fail {
                if raw_ok {
                    ("XPASS", false, None)
                } else {
                    ("XFAIL", true, None)
                }
            } else if raw_ok {
                ("PASS", true, None)
            } else {
                ("FAIL", false, None)
            }
        }
        RawTestOutcome::TimedOut(timeout_ms) => {
            let note = Some(format!("timed out after {} ms", timeout_ms));
            if test_function.test_should_fail {
                ("XFAIL", true, note)
            } else {
                ("TIMEOUT", false, note)
            }
        }
    };
    Ok(TestVerdict {
        status,
        counted_pass,
        note,
        clock_policy: test_function
            .test_clock_policy
            .map(|policy| policy.as_str()),
        resolved_clock_bridge: resolved_clock.map(|clock| clock.bridge.as_str()),
        resolved_clock_surface: resolved_clock.map(|clock| clock.bridge.host_surface().as_str()),
        declared_clock_domain: declared_clock_domain.map(|domain| domain.as_str()),
        declared_clock_domain_code: declared_clock_domain.map(|domain| domain.code()),
        resolved_clock_domain: resolved_clock.map(|clock| clock.domain.as_str()),
        resolved_clock_domain_code: resolved_clock.map(|clock| clock.domain.code()),
        resolved_clock_source: resolved_clock.map(|clock| clock.source),
    })
}

fn execute_language_benchmark(
    input_path: &Path,
    ast: &AstModule,
    benchmark_function: &AstFunction,
) -> Result<BenchmarkVerdict, String> {
    let warmup_iters = benchmark_function
        .benchmark_warmup_iters
        .unwrap_or(0)
        .try_into()
        .map_err(|_| "benchmark warmup iters overflowed usize".to_owned())?;
    let measure_iters = benchmark_function
        .benchmark_measure_iters
        .unwrap_or(1)
        .try_into()
        .map_err(|_| "benchmark measure iters overflowed usize".to_owned())?;
    let declared_clock_domain = benchmark_function.benchmark_clock_domain;
    let resolved_clock = benchmark_function
        .benchmark_timeout_ms
        .map(|_| resolve_runner_clock_domain(declared_clock_domain));
    let benchmark_label = benchmark_function
        .benchmark_name
        .as_deref()
        .unwrap_or(&benchmark_function.name);
    if warmup_iters > 0 {
        let warmup_written = compile_benchmark_harness_binary(
            input_path,
            ast,
            benchmark_function,
            warmup_iters as i64,
            &format!("{benchmark_label}-warmup"),
        )?;
        let warmup_outcome = run_benchmark_process(
            &warmup_written.binary_path,
            benchmark_function.benchmark_timeout_ms,
            resolved_clock.map(|clock| clock.domain),
        )?;
        match warmup_outcome {
            RawBenchmarkOutcome::Completed { status, .. } => {
                if status.code().is_none() {
                    return Ok(BenchmarkVerdict {
                        status: "FAIL",
                        note: Some(
                            "benchmark process terminated without an exit code during warmup loop"
                                .to_owned(),
                        ),
                        warmup_iters,
                        measure_iters,
                        measurement: None,
                        clock_policy: benchmark_function
                            .benchmark_clock_policy
                            .map(|policy| policy.as_str()),
                        resolved_clock_bridge: resolved_clock.map(|clock| clock.bridge.as_str()),
                        resolved_clock_surface: resolved_clock
                            .map(|clock| clock.bridge.host_surface().as_str()),
                        declared_clock_domain: declared_clock_domain.map(|domain| domain.as_str()),
                        declared_clock_domain_code: declared_clock_domain
                            .map(|domain| domain.code()),
                        resolved_clock_domain: resolved_clock.map(|clock| clock.domain.as_str()),
                        resolved_clock_domain_code: resolved_clock.map(|clock| clock.domain.code()),
                        resolved_clock_source: resolved_clock.map(|clock| clock.source),
                    });
                }
            }
            RawBenchmarkOutcome::TimedOut { timeout_ms } => {
                return Ok(BenchmarkVerdict {
                    status: "TIMEOUT",
                    note: Some(format!(
                        "timed out during warmup loop after {} ms",
                        timeout_ms
                    )),
                    warmup_iters,
                    measure_iters,
                    measurement: None,
                    clock_policy: benchmark_function
                        .benchmark_clock_policy
                        .map(|policy| policy.as_str()),
                    resolved_clock_bridge: resolved_clock.map(|clock| clock.bridge.as_str()),
                    resolved_clock_surface: resolved_clock
                        .map(|clock| clock.bridge.host_surface().as_str()),
                    declared_clock_domain: declared_clock_domain.map(|domain| domain.as_str()),
                    declared_clock_domain_code: declared_clock_domain.map(|domain| domain.code()),
                    resolved_clock_domain: resolved_clock.map(|clock| clock.domain.as_str()),
                    resolved_clock_domain_code: resolved_clock.map(|clock| clock.domain.code()),
                    resolved_clock_source: resolved_clock.map(|clock| clock.source),
                });
            }
        }
    }
    let measured_written = compile_benchmark_harness_binary(
        input_path,
        ast,
        benchmark_function,
        measure_iters as i64,
        benchmark_label,
    )?;
    let outcome = run_benchmark_process(
        &measured_written.binary_path,
        benchmark_function.benchmark_timeout_ms,
        resolved_clock.map(|clock| clock.domain),
    )?;
    let measurement = match outcome {
        RawBenchmarkOutcome::Completed {
            elapsed_ns,
            internal_elapsed_ns,
            status,
        } => {
            if status.code().is_none() {
                return Ok(BenchmarkVerdict {
                    status: "FAIL",
                    note: Some(
                        "benchmark process terminated without an exit code during single-process loop"
                            .to_owned(),
                    ),
                    warmup_iters,
                    measure_iters,
                    measurement: None,
                    clock_policy: benchmark_function
                        .benchmark_clock_policy
                        .map(|policy| policy.as_str()),
                    resolved_clock_bridge: resolved_clock.map(|clock| clock.bridge.as_str()),
                    resolved_clock_surface: resolved_clock
                        .map(|clock| clock.bridge.host_surface().as_str()),
                    declared_clock_domain: declared_clock_domain.map(|domain| domain.as_str()),
                    declared_clock_domain_code: declared_clock_domain.map(|domain| domain.code()),
                    resolved_clock_domain: resolved_clock.map(|clock| clock.domain.as_str()),
                    resolved_clock_domain_code: resolved_clock.map(|clock| clock.domain.code()),
                    resolved_clock_source: resolved_clock.map(|clock| clock.source),
                });
            }
            let measured_total_ns = internal_elapsed_ns.unwrap_or(elapsed_ns);
            Some(BenchmarkMeasurement {
                run_mode: if internal_elapsed_ns.is_some() {
                    "in-process-clock"
                } else if warmup_iters > 0 {
                    "dual-process-loop"
                } else {
                    "single-process-loop"
                },
                sample_count: measure_iters,
                min_ns: None,
                max_ns: None,
                avg_ns: measured_total_ns / measure_iters as u128,
                total_ns: measured_total_ns,
            })
        }
        RawBenchmarkOutcome::TimedOut { timeout_ms } => {
            return Ok(BenchmarkVerdict {
                status: "TIMEOUT",
                note: Some(format!(
                    "timed out during measured loop after {} ms",
                    timeout_ms
                )),
                warmup_iters,
                measure_iters,
                measurement: None,
                clock_policy: benchmark_function
                    .benchmark_clock_policy
                    .map(|policy| policy.as_str()),
                resolved_clock_bridge: resolved_clock.map(|clock| clock.bridge.as_str()),
                resolved_clock_surface: resolved_clock
                    .map(|clock| clock.bridge.host_surface().as_str()),
                declared_clock_domain: declared_clock_domain.map(|domain| domain.as_str()),
                declared_clock_domain_code: declared_clock_domain.map(|domain| domain.code()),
                resolved_clock_domain: resolved_clock.map(|clock| clock.domain.as_str()),
                resolved_clock_domain_code: resolved_clock.map(|clock| clock.domain.code()),
                resolved_clock_source: resolved_clock.map(|clock| clock.source),
            });
        }
    };

    Ok(BenchmarkVerdict {
        status: "OK",
        note: None,
        warmup_iters,
        measure_iters,
        measurement,
        clock_policy: benchmark_function
            .benchmark_clock_policy
            .map(|policy| policy.as_str()),
        resolved_clock_bridge: resolved_clock.map(|clock| clock.bridge.as_str()),
        resolved_clock_surface: resolved_clock.map(|clock| clock.bridge.host_surface().as_str()),
        declared_clock_domain: declared_clock_domain.map(|domain| domain.as_str()),
        declared_clock_domain_code: declared_clock_domain.map(|domain| domain.code()),
        resolved_clock_domain: resolved_clock.map(|clock| clock.domain.as_str()),
        resolved_clock_domain_code: resolved_clock.map(|clock| clock.domain.code()),
        resolved_clock_source: resolved_clock.map(|clock| clock.source),
    })
}

enum RawTestOutcome {
    Completed(ExitStatus),
    TimedOut(i64),
}

enum RawBenchmarkOutcome {
    Completed {
        elapsed_ns: u128,
        internal_elapsed_ns: Option<u128>,
        status: ExitStatus,
    },
    TimedOut {
        timeout_ms: i64,
    },
}

fn compile_benchmark_harness_binary(
    input_path: &Path,
    ast: &AstModule,
    benchmark_function: &AstFunction,
    iterations: i64,
    label: &str,
) -> Result<nuisc::aot::CompileArtifacts, String> {
    let harness_ast = build_benchmark_harness_module(ast, benchmark_function, iterations)?;
    let artifacts = nuisc::pipeline::compile_ast(harness_ast)?;
    let output_dir = temp_test_output_dir(label);
    let cpu_target =
        nuisc::aot::resolve_cpu_build_target(Path::new("nustar-packages"), None, None, None)?;
    nuisc::aot::write_and_link(
        input_path,
        &output_dir,
        &artifacts.ast,
        &artifacts.nir,
        &artifacts.yir,
        &artifacts.llvm_ir,
        &cpu_target,
    )
}

fn run_benchmark_process(
    binary_path: &str,
    timeout_ms: Option<i64>,
    clock_domain: Option<nuis_semantics::model::TestClockDomain>,
) -> Result<RawBenchmarkOutcome, String> {
    let mut child = Command::new(binary_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to run `{binary_path}`: {error}"))?;
    let started = Instant::now();
    match wait_for_test_child(&mut child, timeout_ms, clock_domain)? {
        RawTestOutcome::Completed(status) => {
            let stdout = read_child_stdout(&mut child)?;
            Ok(RawBenchmarkOutcome::Completed {
                elapsed_ns: started.elapsed().as_nanos(),
                internal_elapsed_ns: parse_internal_benchmark_elapsed_ns(&stdout),
                status,
            })
        }
        RawTestOutcome::TimedOut(timeout_ms) => Ok(RawBenchmarkOutcome::TimedOut { timeout_ms }),
    }
}

fn read_child_stdout(child: &mut Child) -> Result<String, String> {
    let mut stdout = String::new();
    if let Some(mut pipe) = child.stdout.take() {
        pipe.read_to_string(&mut stdout)
            .map_err(|error| format!("failed to read benchmark stdout: {error}"))?;
    }
    Ok(stdout)
}

fn parse_internal_benchmark_elapsed_ns(stdout: &str) -> Option<u128> {
    stdout
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(|line| line.parse::<u128>().ok())
}

fn wait_for_test_child(
    child: &mut Child,
    timeout_ms: Option<i64>,
    clock_domain: Option<nuis_semantics::model::TestClockDomain>,
) -> Result<RawTestOutcome, String> {
    let Some(timeout_ms) = timeout_ms else {
        let status = child
            .wait()
            .map_err(|error| format!("failed to wait for test process: {error}"))?;
        return Ok(RawTestOutcome::Completed(status));
    };
    let timeout_ms_u64 = u64::try_from(timeout_ms)
        .map_err(|_| format!("invalid negative timeout_ms `{timeout_ms}` reached runner"))?;
    let clock_domain = clock_domain.unwrap_or(nuis_semantics::model::TestClockDomain::Monotonic);
    let monotonic_deadline = matches!(
        clock_domain,
        nuis_semantics::model::TestClockDomain::Monotonic
            | nuis_semantics::model::TestClockDomain::Global
    )
    .then(|| Instant::now() + Duration::from_millis(timeout_ms_u64));
    let wall_deadline = if clock_domain == nuis_semantics::model::TestClockDomain::Wall {
        Some(
            SystemTime::now()
                .checked_add(Duration::from_millis(timeout_ms_u64))
                .ok_or_else(|| "failed to compute wall-clock test deadline".to_owned())?,
        )
    } else {
        None
    };
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll test process: {error}"))?
        {
            return Ok(RawTestOutcome::Completed(status));
        }
        let timed_out = if let Some(deadline) = monotonic_deadline {
            Instant::now() >= deadline
        } else if let Some(deadline) = wall_deadline {
            SystemTime::now() >= deadline
        } else {
            false
        };
        if timed_out {
            child
                .kill()
                .map_err(|error| format!("failed to kill timed out test process: {error}"))?;
            let _ = child.wait();
            return Ok(RawTestOutcome::TimedOut(timeout_ms));
        }
        thread::sleep(Duration::from_millis(1));
    }
}

fn resolve_runner_clock_domain(
    declared: Option<nuis_semantics::model::TestClockDomain>,
) -> RunnerClockResolution {
    let declared = declared.unwrap_or(nuis_semantics::model::TestClockDomain::Monotonic);
    let bridge = nuis_semantics::model::NirHostTimingBridge::from_test_clock_domain(declared);
    RunnerClockResolution {
        domain: bridge.resolved_domain(),
        bridge,
        source: bridge.resolved_source(),
    }
}

fn build_test_harness_module(ast: &AstModule, test_function: &AstFunction) -> AstModule {
    let mut harness = ast.clone();
    harness.functions.retain(|function| function.name != "main");
    harness
        .functions
        .push(build_test_main_function(test_function));
    harness
}

fn build_benchmark_harness_module(
    ast: &AstModule,
    benchmark_function: &AstFunction,
    iterations: i64,
) -> Result<AstModule, String> {
    let mut harness = ast.clone();
    harness.functions.retain(|function| function.name != "main");
    ensure_benchmark_timing_externs(&mut harness);
    harness
        .functions
        .push(build_benchmark_loop_function(benchmark_function));
    harness
        .functions
        .push(build_benchmark_elapsed_text_function());
    harness.functions.push(build_benchmark_main_function(
        benchmark_function,
        iterations,
    ));
    Ok(harness)
}

fn build_test_main_function(test_function: &AstFunction) -> AstFunction {
    #[rustfmt::skip]
    let test_call = AstExpr::Call {
        callee: test_function.name.clone(), generic_args: vec![], args: vec![],
    };
    let body = match test_function.return_type.as_ref() {
        Some(return_type) if return_type.name == "bool" && !return_type.is_ref => {
            let value_expr = if test_function.is_async {
                AstExpr::Await(Box::new(test_call))
            } else {
                test_call
            };
            vec![
                AstStmt::Let {
                    mutable: false,
                    name: "passed".to_owned(),
                    ty: Some(bool_type_ref()),
                    value: value_expr,
                },
                AstStmt::If {
                    condition: AstExpr::Var("passed".to_owned()),
                    then_body: vec![AstStmt::Return(Some(AstExpr::Int(1)))],
                    else_body: vec![AstStmt::Return(Some(AstExpr::Int(0)))],
                },
            ]
        }
        _ => {
            let value_expr = if test_function.is_async {
                AstExpr::Await(Box::new(test_call))
            } else {
                test_call
            };
            vec![
                AstStmt::Let {
                    mutable: false,
                    name: "status".to_owned(),
                    ty: Some(i64_type_ref()),
                    value: value_expr,
                },
                AstStmt::Return(Some(AstExpr::Var("status".to_owned()))),
            ]
        }
    };
    AstFunction {
        name: "main".to_owned(),
        visibility: nuis_semantics::model::AstVisibility::Private,
        attributes: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: test_function.is_async,
        generic_params: vec![],
        where_bounds: vec![],
        params: vec![],
        return_type: Some(i64_type_ref()),
        body,
    }
}

fn build_benchmark_loop_function(benchmark_function: &AstFunction) -> AstFunction {
    let helper_name = benchmark_loop_function_name();
    let side_effect_name = "benchmark_side_effect".to_owned();
    let remaining_name = "benchmark_remaining".to_owned();
    let benchmark_return_type = benchmark_function
        .return_type
        .clone()
        .unwrap_or_else(i64_type_ref);
    let recursive_call = AstExpr::Call {
        callee: helper_name.clone(),
        generic_args: vec![],
        args: vec![AstExpr::Binary {
            op: nuis_semantics::model::AstBinaryOp::Sub,
            lhs: Box::new(AstExpr::Var(remaining_name.clone())),
            rhs: Box::new(AstExpr::Int(1)),
        }],
    };
    let recurse_expr = if benchmark_function.is_async {
        AstExpr::Await(Box::new(recursive_call))
    } else {
        recursive_call
    };
    AstFunction {
        name: helper_name,
        visibility: AstVisibility::Private,
        attributes: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: benchmark_function.is_async,
        generic_params: vec![],
        where_bounds: vec![],
        params: vec![nuis_semantics::model::AstParam {
            name: remaining_name.clone(),
            ty: i64_type_ref(),
        }],
        return_type: Some(i64_type_ref()),
        body: vec![
            AstStmt::If {
                condition: AstExpr::Binary {
                    op: nuis_semantics::model::AstBinaryOp::Le,
                    lhs: Box::new(AstExpr::Var(remaining_name.clone())),
                    rhs: Box::new(AstExpr::Int(0)),
                },
                then_body: vec![AstStmt::Return(Some(AstExpr::Int(0)))],
                else_body: vec![],
            },
            AstStmt::Let {
                mutable: false,
                name: side_effect_name,
                ty: Some(benchmark_return_type),
                value: benchmark_call_expr(benchmark_function),
            },
            AstStmt::Return(Some(recurse_expr)),
        ],
    }
}

fn build_benchmark_main_function(benchmark_function: &AstFunction, iterations: i64) -> AstFunction {
    let helper_call = AstExpr::Call {
        callee: benchmark_loop_function_name(),
        generic_args: vec![],
        args: vec![AstExpr::Int(iterations)],
    };
    let return_expr = if benchmark_function.is_async {
        AstExpr::Await(Box::new(helper_call))
    } else {
        helper_call
    };
    AstFunction {
        name: "main".to_owned(),
        visibility: AstVisibility::Private,
        attributes: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: benchmark_function.is_async,
        generic_params: vec![],
        where_bounds: vec![],
        params: vec![],
        return_type: Some(i64_type_ref()),
        body: vec![
            AstStmt::Let {
                mutable: false,
                name: "benchmark_started_ns".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Call {
                    callee: "host_monotonic_time_ns".to_owned(),
                    generic_args: vec![],
                    args: vec![],
                },
            },
            AstStmt::Let {
                mutable: false,
                name: "benchmark_status".to_owned(),
                ty: Some(i64_type_ref()),
                value: return_expr,
            },
            AstStmt::Let {
                mutable: false,
                name: "benchmark_ended_ns".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Call {
                    callee: "host_monotonic_time_ns".to_owned(),
                    generic_args: vec![],
                    args: vec![],
                },
            },
            AstStmt::Let {
                mutable: false,
                name: "benchmark_elapsed_ns".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Binary {
                    op: nuis_semantics::model::AstBinaryOp::Sub,
                    lhs: Box::new(AstExpr::Var("benchmark_ended_ns".to_owned())),
                    rhs: Box::new(AstExpr::Var("benchmark_started_ns".to_owned())),
                },
            },
            AstStmt::Expr(AstExpr::Call {
                callee: "host_stdout_write".to_owned(),
                generic_args: vec![],
                args: vec![AstExpr::Call {
                    callee: benchmark_elapsed_text_function_name(),
                    generic_args: vec![],
                    args: vec![AstExpr::Var("benchmark_elapsed_ns".to_owned())],
                }],
            }),
            AstStmt::Return(Some(AstExpr::Int(0))),
        ],
    }
}

fn benchmark_call_expr(benchmark_function: &AstFunction) -> AstExpr {
    #[rustfmt::skip]
    let benchmark_call = AstExpr::Call {
        callee: benchmark_function.name.clone(), generic_args: vec![], args: vec![],
    };
    if benchmark_function.is_async {
        AstExpr::Await(Box::new(benchmark_call))
    } else {
        benchmark_call
    }
}

fn benchmark_loop_function_name() -> String {
    "__nuis_benchmark_loop".to_owned()
}

fn benchmark_elapsed_text_function_name() -> String {
    "__nuis_benchmark_elapsed_text".to_owned()
}

fn build_benchmark_elapsed_text_function() -> AstFunction {
    AstFunction {
        name: benchmark_elapsed_text_function_name(),
        visibility: AstVisibility::Private,
        attributes: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: false,
        generic_params: vec![],
        where_bounds: vec![],
        params: vec![AstParam {
            name: "elapsed_ns".to_owned(),
            ty: i64_type_ref(),
        }],
        return_type: Some(i64_type_ref()),
        body: vec![
            AstStmt::Let {
                mutable: false,
                name: "buffer".to_owned(),
                ty: Some(ref_buffer_type_ref()),
                value: AstExpr::Call {
                    callee: "alloc_buffer".to_owned(),
                    generic_args: vec![],
                    args: vec![AstExpr::Int(64), AstExpr::Int(0)],
                },
            },
            AstStmt::Let {
                mutable: false,
                name: "written".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Call {
                    callee: "serialize_i64_into".to_owned(),
                    generic_args: vec![],
                    args: vec![
                        AstExpr::Var("elapsed_ns".to_owned()),
                        AstExpr::Var("buffer".to_owned()),
                        AstExpr::Int(0),
                    ],
                },
            },
            AstStmt::Return(Some(AstExpr::Call {
                callee: "deserialize_text_from".to_owned(),
                generic_args: vec![],
                args: vec![
                    AstExpr::Var("buffer".to_owned()),
                    AstExpr::Int(0),
                    AstExpr::Var("written".to_owned()),
                ],
            })),
        ],
    }
}

fn ensure_benchmark_timing_externs(module: &mut AstModule) {
    ensure_benchmark_timing_extern(module, "host_monotonic_time_ns", vec![]);
    ensure_benchmark_timing_extern(
        module,
        "host_serialize_i64_into",
        vec![
            AstParam {
                name: "value".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "buffer_handle".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "offset".to_owned(),
                ty: i64_type_ref(),
            },
        ],
    );
    ensure_benchmark_timing_extern(
        module,
        "host_deserialize_text_from",
        vec![
            AstParam {
                name: "buffer_handle".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "offset".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "len".to_owned(),
                ty: i64_type_ref(),
            },
        ],
    );
    ensure_benchmark_timing_extern(
        module,
        "host_text_len",
        vec![AstParam {
            name: "text_handle".to_owned(),
            ty: i64_type_ref(),
        }],
    );
    ensure_benchmark_timing_extern(
        module,
        "host_stdout_write",
        vec![AstParam {
            name: "text_handle".to_owned(),
            ty: i64_type_ref(),
        }],
    );
}

fn ensure_benchmark_timing_extern(module: &mut AstModule, name: &str, params: Vec<AstParam>) {
    if module.externs.iter().any(|function| function.name == name) {
        return;
    }
    module.externs.push(AstExternFunction {
        visibility: AstVisibility::Private,
        abi: "c".to_owned(),
        interface: None,
        name: name.to_owned(),
        host_symbol: None,
        params,
        return_type: i64_type_ref(),
    });
}

fn i64_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "i64".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn ref_buffer_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "Buffer".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: true,
    }
}

fn bool_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "bool".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn temp_test_output_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nuis-test-runner-{}-{}",
        sanitize_test_label(label),
        stamp
    ))
}

fn sanitize_test_label(label: &str) -> String {
    let mut out = String::new();
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "test".to_owned()
    } else {
        out
    }
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
        vec![
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
            json_bool_field("runtime_loaded_bridge_registry", false),
            json_bool_field("runtime_loaded_host_bridge_plan_index", false),
        ];
    };
    match nuis_runtime::RuntimeLoader.load_from_artifact_path(path) {
        Ok(loaded) => vec![
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
            json_bool_field(
                "runtime_loaded_bridge_registry",
                loaded.bridge_registry.is_some(),
            ),
            json_bool_field(
                "runtime_loaded_host_bridge_plan_index",
                loaded.host_bridge_plan_index.is_some(),
            ),
        ],
        Err(error) => vec![
            json_bool_field("runtime_load_attempted", true),
            json_bool_field("runtime_load_ok", false),
            json_optional_string_field("runtime_load_error", Some(&error.to_string())),
            json_usize_field("runtime_loaded_domain_units", 0),
            json_usize_field("runtime_loaded_heterogeneous_units", 0),
            json_usize_field("runtime_loaded_payload_blobs", 0),
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
        Ok((domains, plan_phases, trace_events)) => vec![
            json_bool_field("runtime_execution_attempted", true),
            json_bool_field("runtime_execution_ok", true),
            json_optional_string_field("runtime_execution_error", None),
            json_usize_field("runtime_execution_domains", domains),
            json_usize_field("runtime_execution_plan_phases", plan_phases),
            json_usize_field("runtime_execution_trace_events", trace_events),
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
    ]
}

fn runtime_execution_summary(path: &Path) -> Result<(usize, usize, usize), String> {
    let loaded = nuis_runtime::RuntimeLoader
        .load_from_artifact_path(path)
        .map_err(|error| error.to_string())?;
    let mut adapters = nuis_runtime::AdapterRegistry::new();
    adapters.register(Box::new(BuildReportRuntimeAdapter));
    let bridge = nuis_runtime::BridgeExecutor;
    let executor = nuis_runtime::Executor;
    let mut domains = 0usize;
    let mut plan_phases = 0usize;
    let mut trace_events = 0usize;
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
        domains += 1;
        plan_phases += plan.phases.len();
        trace_events += trace.events.len();
    }
    Ok((domains, plan_phases, trace_events))
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
                Ok((domains, plan_phases, trace_events)) => {
                    println!("  runtime_execution_attempted: true");
                    println!("  runtime_execution_ok: true");
                    println!("  runtime_execution_domains: {}", domains);
                    println!("  runtime_execution_plan_phases: {}", plan_phases);
                    println!("  runtime_execution_trace_events: {}", trace_events);
                }
                Err(error) => {
                    println!("  runtime_execution_attempted: true");
                    println!("  runtime_execution_ok: false");
                    println!("  runtime_execution_error: {}", error);
                }
            }
        }
    } else {
        println!("  runtime_load_attempted: false");
        println!("  runtime_load_ok: false");
        println!("  runtime_execution_attempted: false");
        println!("  runtime_execution_ok: false");
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

pub(crate) fn debug_workflow_brief() -> &'static str {
    "dump-ast -> dump-nir -> dump-yir -> scheduler-view"
}

pub(crate) fn debug_workflow_samples_brief() -> &'static str {
    "ast=nuis dump-ast <input>; nir=nuis dump-nir <input>; yir=nuis dump-yir <input>; scheduler=nuis scheduler-view <input>"
}

fn single_source_compile_workflow_brief() -> &'static str {
    "check -> test -> build -> artifact_doctor -> run_artifact -> release_check"
}

fn single_source_compile_samples_brief() -> &'static str {
    "check=nuis check <input.ns>; test=nuis test <input.ns>; build=nuis build <input.ns> <output-dir>; artifact=nuis artifact-doctor <output-dir>; run=nuis run-artifact <output-dir>; release=nuis release-check <input.ns> <output-dir>"
}

fn artifact_workflow_brief() -> &'static str {
    "build -> inspect_artifact -> verify_artifact -> artifact_doctor -> verify_build_manifest -> run_artifact"
}

fn artifact_doctor_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis artifact-doctor {}", output_dir.display())
}

fn run_artifact_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis run-artifact {}", output_dir.display())
}

fn load_link_plan_for_output_dir(output_dir: &Path) -> Option<nuisc::linker::LinkPlan> {
    let manifest = output_dir.join("nuis.build.manifest.toml");
    if !manifest.exists() {
        return None;
    }
    nuisc::linker::build_link_plan_from_manifest(&manifest).ok()
}

fn workflow_link_plan_domain_unit_record(unit: &nuisc::linker::LinkPlanDomainUnit) -> String {
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", &unit.kind),
            json_field("package_id", &unit.package_id),
            json_field("domain_family", &unit.domain_family),
            json_field("contract_family", &unit.contract_family),
            json_field("packaging_role", &unit.packaging_role),
        ],
    );
    if let Some(value) = unit.abi.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("abi", value)]);
    }
    if let Some(value) = unit.backend_family.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("backend_family", value)]);
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        append_json_field_strings(
            &mut out,
            vec![json_field("selected_lowering_target", value)],
        );
    }
    if let Some(value) = unit.machine_arch.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_arch", value)]);
    }
    if let Some(value) = unit.machine_os.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_os", value)]);
    }
    out.push('}');
    out
}

fn workflow_link_plan_json_fields(link_plan: Option<&nuisc::linker::LinkPlan>) -> Vec<String> {
    let domain_unit_records = link_plan
        .map(|plan| {
            plan.domain_units
                .iter()
                .map(workflow_link_plan_domain_unit_record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    vec![
        json_bool_field("link_plan_available", link_plan.is_some()),
        json_optional_string_field(
            "link_plan_final_stage",
            link_plan.map(|plan| plan.final_stage.kind.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_driver",
            link_plan.map(|plan| plan.final_stage.driver.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_link_mode",
            link_plan.map(|plan| plan.final_stage.link_mode.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_output",
            link_plan.map(|plan| plan.final_stage.output_path.as_str()),
        ),
        json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
        json_object_array_field("link_plan_domain_unit_records", &domain_unit_records),
    ]
}

fn compile_pipeline_stage_json(stage: &nuisc::pipeline::CompilePipelineStage) -> String {
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("id", stage.id),
            json_field("status", stage.status),
            json_field("detail", &stage.detail),
        ],
    );
    out.push('}');
    out
}

fn workflow_compile_pipeline_json_fields(input: &Path) -> Vec<String> {
    match nuisc::pipeline::resolve_compile_input(input).and_then(|resolved| {
        let artifacts = resolved.compile()?;
        Ok(resolved.compile_report(&artifacts))
    }) {
        Ok(report) => {
            let stage_records = report
                .stages
                .iter()
                .map(compile_pipeline_stage_json)
                .collect::<Vec<_>>();
            vec![
                json_bool_field("compile_pipeline_available", true),
                json_field("compile_pipeline_source_kind", report.source_kind),
                json_field("compile_pipeline_input", &report.input_path),
                json_field(
                    "compile_pipeline_effective_input",
                    &report.effective_input_path,
                ),
                json_optional_string_field(
                    "compile_pipeline_project",
                    report.project_name.as_deref(),
                ),
                json_field("compile_pipeline_domain", &report.domain),
                json_field("compile_pipeline_unit", &report.unit),
                json_usize_field("compile_pipeline_stage_count", report.stage_count()),
                json_usize_field("compile_pipeline_ok_stage_count", report.ok_stage_count()),
                json_usize_field("compile_pipeline_ast_functions", report.ast_functions),
                json_usize_field("compile_pipeline_nir_functions", report.nir_functions),
                json_usize_field("compile_pipeline_yir_nodes", report.yir_nodes),
                json_usize_field("compile_pipeline_yir_resources", report.yir_resources),
                json_usize_field("compile_pipeline_yir_edges", report.yir_edges),
                json_usize_field("compile_pipeline_llvm_ir_bytes", report.llvm_ir_bytes),
                json_usize_field(
                    "compile_pipeline_loaded_nustar_count",
                    report.loaded_nustar.len(),
                ),
                json_string_array_field("compile_pipeline_loaded_nustar", &report.loaded_nustar),
                json_object_array_field("compile_pipeline_stages", &stage_records),
                json_bool_field("compile_pipeline_ready_for_aot", report.ready_for_aot),
                json_field(
                    "compile_pipeline_recommended_next_step",
                    report.recommended_next_step,
                ),
                json_field(
                    "compile_pipeline_recommended_reason",
                    &report.recommended_reason,
                ),
                json_field("compile_pipeline_summary", &report.summary_line()),
            ]
        }
        Err(error) => vec![
            json_bool_field("compile_pipeline_available", false),
            json_field("compile_pipeline_error", &error),
        ],
    }
}

fn append_workflow_link_plan_json_fields(
    out: &mut String,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    append_json_field_strings(out, workflow_link_plan_json_fields(link_plan));
}

fn render_workflow_json(input: &Path) -> Result<String, String> {
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

fn single_source_workflow_next_step_label() -> &'static str {
    "check"
}

fn recommended_single_source_workflow_command() -> &'static str {
    "nuis check <input.ns>"
}

struct WorkflowRecommendation {
    label: &'static str,
    command: &'static str,
    reason: &'static str,
}

struct WorkflowSourceProfile {
    source_kind: &'static str,
    workflow_kind: &'static str,
    workflow_brief: &'static str,
    workflow_samples: &'static str,
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
    pub(crate) ready_to_run: bool,
    pub(crate) recommended_next_step: String,
    pub(crate) recommended_command: String,
    pub(crate) recommended_reason: String,
    pub(crate) manifest_verify_error: Option<String>,
    pub(crate) artifact_verify_error: Option<String>,
}

pub(crate) struct WorkflowFrontdoorSurface {
    pub(crate) source_kind: &'static str,
    pub(crate) workflow_kind: &'static str,
    pub(crate) workflow_brief: &'static str,
    pub(crate) workflow_samples: &'static str,
    pub(crate) recommended_next_step: &'static str,
    pub(crate) recommended_command: &'static str,
    pub(crate) recommended_reason: &'static str,
}

fn build_workflow_frontdoor_surface(
    profile: WorkflowSourceProfile,
    recommendation: WorkflowRecommendation,
) -> WorkflowFrontdoorSurface {
    WorkflowFrontdoorSurface {
        source_kind: profile.source_kind,
        workflow_kind: profile.workflow_kind,
        workflow_brief: profile.workflow_brief,
        workflow_samples: profile.workflow_samples,
        recommended_next_step: recommendation.label,
        recommended_command: recommendation.command,
        recommended_reason: recommendation.reason,
    }
}

#[allow(dead_code)]
pub(crate) fn workflow_frontdoor_json_fields(surface: &WorkflowFrontdoorSurface) -> Vec<String> {
    vec![
        json_field("source_kind", surface.source_kind),
        json_field("workflow_kind", surface.workflow_kind),
        json_field("workflow_brief", surface.workflow_brief),
        json_field("workflow_samples", surface.workflow_samples),
        json_field("recommended_next_step", surface.recommended_next_step),
        json_field("recommended_command", surface.recommended_command),
        json_field("recommended_reason", surface.recommended_reason),
    ]
}

pub(crate) fn append_workflow_frontdoor_json_fields(
    out: &mut String,
    surface: &WorkflowFrontdoorSurface,
) {
    append_json_field_strings(
        out,
        vec![
            json_field("source_kind", surface.source_kind),
            json_field("workflow_kind", surface.workflow_kind),
            json_field("workflow_brief", surface.workflow_brief),
            json_field("workflow_samples", surface.workflow_samples),
            json_field("recommended_next_step", surface.recommended_next_step),
            json_field("recommended_command", surface.recommended_command),
            json_field("recommended_reason", surface.recommended_reason),
        ],
    );
}

pub(crate) fn workflow_frontdoor_json_object_field(surface: &WorkflowFrontdoorSurface) -> String {
    let mut out = String::from("\"frontdoor\":{");
    append_workflow_frontdoor_json_fields(&mut out, surface);
    out.push('}');
    out
}

fn print_workflow_frontdoor_surface(surface: &WorkflowFrontdoorSurface) {
    println!("  frontdoor.source_kind: {}", surface.source_kind);
    println!("  frontdoor.workflow_kind: {}", surface.workflow_kind);
    println!("  frontdoor.workflow_brief: {}", surface.workflow_brief);
    print_scheduler_sample_field("frontdoor.workflow_samples", surface.workflow_samples);
    println!(
        "  frontdoor.recommended_next_step: {}",
        surface.recommended_next_step
    );
    println!(
        "  frontdoor.recommended_command: {}",
        surface.recommended_command
    );
    println!(
        "  frontdoor.recommended_reason: {}",
        surface.recommended_reason
    );
}

fn single_source_workflow_source_profile() -> WorkflowSourceProfile {
    WorkflowSourceProfile {
        source_kind: "single-file",
        workflow_kind: "compile_workflow",
        workflow_brief: single_source_compile_workflow_brief(),
        workflow_samples: single_source_compile_samples_brief(),
    }
}

fn project_compile_workflow_source_profile() -> WorkflowSourceProfile {
    WorkflowSourceProfile {
        source_kind: "project",
        workflow_kind: "project_compile_workflow",
        workflow_brief: nuisc::project_compile_workflow_brief(),
        workflow_samples: nuisc::project_compile_samples_brief(),
    }
}

pub(crate) fn project_frontdoor_surface(
    plan: &nuisc::project::ProjectCompilationPlan,
    declared_tests: &[PathBuf],
    missing_tests: &[PathBuf],
    galaxy_doctor: &galaxy::GalaxyDoctorReport,
    galaxy_check_invalid: bool,
    has_hidden_manual_only_library_modules: bool,
) -> WorkflowFrontdoorSurface {
    let recommendation = recommend_project_workflow_step(
        plan,
        declared_tests,
        missing_tests,
        galaxy_doctor,
        galaxy_check_invalid,
        has_hidden_manual_only_library_modules,
    );
    build_workflow_frontdoor_surface(project_compile_workflow_source_profile(), recommendation)
}

pub(crate) fn single_source_frontdoor_surface() -> WorkflowFrontdoorSurface {
    build_workflow_frontdoor_surface(
        single_source_workflow_source_profile(),
        WorkflowRecommendation {
            label: single_source_workflow_next_step_label(),
            command: recommended_single_source_workflow_command(),
            reason: "single-file inputs usually want direct compile truth first, so `check` stays the best default front-door step",
        },
    )
}

fn toolchain_frontdoor_surface() -> WorkflowFrontdoorSurface {
    build_workflow_frontdoor_surface(
        WorkflowSourceProfile {
            source_kind: "toolchain",
            workflow_kind: "default_compile_frontdoor",
            workflow_brief: "workflow -> project_doctor -> check -> test -> build -> artifact_doctor -> run_artifact -> release_check",
            workflow_samples: "workflow=nuis workflow [input]; doctor=nuis project-doctor [project-dir|nuis.toml]; check=nuis check [input]; test=nuis test [input]; build=nuis build [input] <output-dir>; artifact=nuis artifact-doctor <output-dir>; run=nuis run-artifact <output-dir>; release=nuis release-check [input] [output-dir]",
        },
        WorkflowRecommendation {
            label: "workflow",
            command: "nuis workflow [--json] [input.ns|project-dir|nuis.toml]",
            reason: "the compile frontdoor should classify the input shape first, then route into the right project or single-file workflow branch",
        },
    )
}

fn recommend_project_workflow_step(
    plan: &nuisc::project::ProjectCompilationPlan,
    declared_tests: &[PathBuf],
    missing_tests: &[PathBuf],
    galaxy_doctor: &galaxy::GalaxyDoctorReport,
    galaxy_check_invalid: bool,
    has_hidden_manual_only_library_modules: bool,
) -> WorkflowRecommendation {
    let deps_len = galaxy_doctor.dependencies.len();
    let any_lock_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.locked);
    let any_install_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.installed);
    if galaxy_check_invalid {
        return WorkflowRecommendation {
            label: "galaxy_check",
            command: "nuis galaxy check <project-dir|nuis.toml>",
            reason: "project packaging metadata is currently invalid, so the next step should re-check and fix the galaxy-side project contract first",
        };
    }
    match galaxy_doctor.lock_status.as_str() {
        "missing" if deps_len > 0 => {
            return WorkflowRecommendation {
                label: "galaxy_lock_deps",
                command: "nuis galaxy lock-deps <project-dir|nuis.toml>",
                reason: "the project already declares galaxy dependencies but does not yet have a lockfile",
            };
        }
        "invalid" => {
            return WorkflowRecommendation {
                label: "galaxy_verify_lock",
                command: "nuis galaxy verify-lock <project-dir|nuis.toml>",
                reason: "the current galaxy lockfile is invalid and should be repaired or regenerated before deeper compile work",
            };
        }
        _ => {}
    }
    if any_lock_missing && deps_len > 0 && galaxy_doctor.lock_status == "ok" {
        return WorkflowRecommendation {
            label: "galaxy_lock_refresh",
            command: "nuis galaxy lock-deps <project-dir|nuis.toml>",
            reason: "the lockfile exists, but some declared galaxy dependencies are not represented in it yet",
        };
    }
    if any_install_missing && galaxy_doctor.lock_status == "ok" {
        return WorkflowRecommendation {
            label: "galaxy_sync_deps",
            command: "nuis galaxy sync-deps <project-dir|nuis.toml>",
            reason: "the dependency lock is valid, but some locked galaxy packages are not materialized locally yet",
        };
    }
    if has_hidden_manual_only_library_modules {
        return WorkflowRecommendation {
            label: "project_imports_apply_suggested",
            command: "nuis project-imports --apply-suggested <project-dir|nuis.toml>",
            reason: "the project still has manual-only galaxy library modules hidden from project scope, so the highest-value next step is to write the suggested galaxy_imports entries first",
        };
    }
    if !plan.abi_resolution.explicit {
        return WorkflowRecommendation {
            label: "project_lock_abi",
            command: "nuis project-lock-abi <project-dir|nuis.toml>",
            reason: "the project is still using auto-recommended ABI selection, so freezing the current ABI choice is the highest-value stabilizing step",
        };
    }
    if !missing_tests.is_empty() {
        return WorkflowRecommendation {
            label: "project_status",
            command: "nuis project-status <project-dir|nuis.toml>",
            reason: "some declared project tests are missing on disk, so the next step should inspect and fix the declared test surface",
        };
    }
    if declared_tests.is_empty() {
        return WorkflowRecommendation {
            label: "test",
            command: "nuis test <project-dir|nuis.toml>",
            reason: "the project has no explicit declared tests yet, so the next useful step is to run the current language-level test sweep and then decide whether to add dedicated project tests",
        };
    }
    WorkflowRecommendation {
        label: "check",
        command: "nuis check <project-dir|nuis.toml>",
        reason: "the obvious project-shape blockers are already under control, so the next step is to re-check compile truth directly",
    }
}

fn default_build_output_dir(input: &Path) -> PathBuf {
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

fn default_release_check_output_dir(input: &Path) -> PathBuf {
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

fn handle_workflow(input: std::path::PathBuf, json: bool) -> Result<(), String> {
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

#[derive(Debug, Clone)]
struct SchedulerViewDomainRecord {
    shared_domain_json: String,
    shared_abi_json: Option<String>,
}

fn json_escape_local(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

pub(crate) fn json_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape_local(value))
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\":\"{}\"", name, json_escape_local(value)),
        None => format!("\"{}\":null", name),
    }
}

pub(crate) fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

pub(crate) fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{}\":{}", name, value)
}

fn json_u128_field(name: &str, value: u128) -> String {
    format!("\"{}\":{}", name, value)
}

fn json_optional_u128_field(name: &str, value: Option<u128>) -> String {
    match value {
        Some(value) => json_u128_field(name, value),
        None => format!("\"{}\":null", name),
    }
}

fn json_optional_i64_field(name: &str, value: Option<i64>) -> String {
    match value {
        Some(value) => format!("\"{}\":{}", name, value),
        None => format!("\"{}\":null", name),
    }
}

pub(crate) fn json_string_array_field(name: &str, values: &[String]) -> String {
    let mut out = String::new();
    out.push('"');
    out.push_str(name);
    out.push_str("\":[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape_local(value));
        out.push('"');
    }
    out.push(']');
    out
}

#[allow(dead_code)]
pub(crate) fn json_object_field(name: &str, fields: &[String]) -> String {
    let mut out = String::new();
    out.push('"');
    out.push_str(name);
    out.push_str("\":{");
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(field);
    }
    out.push('}');
    out
}

fn append_json_object_fields(base_json: &str, fields: &[String]) -> String {
    let mut out = base_json.to_owned();
    if out.ends_with('}') {
        out.pop();
        if !fields.is_empty() {
            out.push(',');
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(field);
            }
        }
        out.push('}');
    }
    out
}

fn json_object_array_field(name: &str, values: &[String]) -> String {
    let mut out = String::new();
    out.push('"');
    out.push_str(name);
    out.push_str("\":[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(value);
    }
    out.push(']');
    out
}

fn project_domain_registry_checks_json(
    checks: &[nuisc::registry::ProjectDomainRegistryCheck],
) -> Vec<String> {
    checks
        .iter()
        .map(nuisc::registry::project_domain_registry_check_json)
        .collect()
}

pub(crate) fn project_lowering_checks_json(
    checks: &[nuisc::project::ProjectLoweringSelectionView],
) -> Vec<String> {
    checks
        .iter()
        .map(nuisc::project::project_lowering_selection_json)
        .collect()
}

fn project_abi_checks_json(checks: &[nuisc::project::ProjectAbiSelectionCheck]) -> Vec<String> {
    checks
        .iter()
        .map(nuisc::project::project_abi_selection_check_json)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublicSurfaceModuleRecord {
    pub(crate) module: String,
    pub(crate) externs: Vec<String>,
    pub(crate) extern_interfaces: Vec<String>,
    pub(crate) consts: Vec<String>,
    pub(crate) type_aliases: Vec<String>,
    pub(crate) functions: Vec<String>,
    pub(crate) structs: Vec<String>,
    pub(crate) traits: Vec<String>,
}

impl PublicSurfaceModuleRecord {
    fn is_empty(&self) -> bool {
        self.externs.is_empty()
            && self.extern_interfaces.is_empty()
            && self.consts.is_empty()
            && self.type_aliases.is_empty()
            && self.functions.is_empty()
            && self.structs.is_empty()
            && self.traits.is_empty()
    }
}

pub(crate) fn public_surface_records(
    project: &nuisc::project::LoadedProject,
) -> Vec<PublicSurfaceModuleRecord> {
    project
        .modules
        .iter()
        .filter_map(|module| {
            let externs = module
                .ast
                .externs
                .iter()
                .filter(|function| matches!(function.visibility, AstVisibility::Public))
                .map(|function| function.name.clone())
                .collect::<Vec<_>>();
            let extern_interfaces = module
                .ast
                .extern_interfaces
                .iter()
                .filter(|interface| matches!(interface.visibility, AstVisibility::Public))
                .map(|interface| interface.name.clone())
                .collect::<Vec<_>>();
            let consts = module
                .ast
                .consts
                .iter()
                .filter(|constant| matches!(constant.visibility, AstVisibility::Public))
                .map(|constant| constant.name.clone())
                .collect::<Vec<_>>();
            let type_aliases = module
                .ast
                .type_aliases
                .iter()
                .filter(|alias| matches!(alias.visibility, AstVisibility::Public))
                .map(|alias| alias.name.clone())
                .collect::<Vec<_>>();
            let functions = module
                .ast
                .functions
                .iter()
                .filter(|function| matches!(function.visibility, AstVisibility::Public))
                .map(|function| function.name.clone())
                .collect::<Vec<_>>();
            let structs = module
                .ast
                .structs
                .iter()
                .filter(|definition| matches!(definition.visibility, AstVisibility::Public))
                .map(|definition| {
                    let public_fields = definition
                        .fields
                        .iter()
                        .filter(|field| matches!(field.visibility, AstVisibility::Public))
                        .count();
                    let hidden_fields = definition.fields.len().saturating_sub(public_fields);
                    if hidden_fields == 0 {
                        format!("{}(fields={public_fields})", definition.name)
                    } else {
                        format!(
                            "{}(fields={public_fields}, hidden={hidden_fields})",
                            definition.name
                        )
                    }
                })
                .collect::<Vec<_>>();
            let traits = module
                .ast
                .traits
                .iter()
                .filter(|definition| matches!(definition.visibility, AstVisibility::Public))
                .map(|definition| definition.name.clone())
                .collect::<Vec<_>>();
            let record = PublicSurfaceModuleRecord {
                module: format!("{}::{}", module.ast.domain, module.ast.unit),
                externs,
                extern_interfaces,
                consts,
                type_aliases,
                functions,
                structs,
                traits,
            };
            if record.is_empty() {
                None
            } else {
                Some(record)
            }
        })
        .collect()
}

pub(crate) fn describe_public_surface(records: &[PublicSurfaceModuleRecord]) -> String {
    let extern_count = records
        .iter()
        .map(|record| record.externs.len())
        .sum::<usize>();
    let extern_interface_count = records
        .iter()
        .map(|record| record.extern_interfaces.len())
        .sum::<usize>();
    let const_count = records
        .iter()
        .map(|record| record.consts.len())
        .sum::<usize>();
    let function_count = records
        .iter()
        .map(|record| record.functions.len())
        .sum::<usize>();
    let alias_count = records
        .iter()
        .map(|record| record.type_aliases.len())
        .sum::<usize>();
    let struct_count = records
        .iter()
        .map(|record| record.structs.len())
        .sum::<usize>();
    let trait_count = records
        .iter()
        .map(|record| record.traits.len())
        .sum::<usize>();
    let module_count = records.len();
    if module_count == 0 {
        return "<none>".to_owned();
    }
    format!(
        "modules={module_count} extern={extern_count} interface={extern_interface_count} const={const_count} type={alias_count} fn={function_count} struct={struct_count} trait={trait_count}"
    )
}

pub(crate) fn describe_public_surface_modules(records: &[PublicSurfaceModuleRecord]) -> String {
    if records.is_empty() {
        return "<none>".to_owned();
    }
    records
        .iter()
        .map(|record| {
            let mut segments = Vec::new();
            if !record.externs.is_empty() {
                segments.push(format!("extern={}", record.externs.join(", ")));
            }
            if !record.extern_interfaces.is_empty() {
                segments.push(format!("interface={}", record.extern_interfaces.join(", ")));
            }
            if !record.consts.is_empty() {
                segments.push(format!("const={}", record.consts.join(", ")));
            }
            if !record.type_aliases.is_empty() {
                segments.push(format!("type={}", record.type_aliases.join(", ")));
            }
            if !record.functions.is_empty() {
                segments.push(format!("fn={}", record.functions.join(", ")));
            }
            if !record.structs.is_empty() {
                segments.push(format!("struct={}", record.structs.join(", ")));
            }
            if !record.traits.is_empty() {
                segments.push(format!("trait={}", record.traits.join(", ")));
            }
            format!("{} [{}]", record.module, segments.join(" | "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn public_surface_json(records: &[PublicSurfaceModuleRecord]) -> Vec<String> {
    records
        .iter()
        .map(|record| {
            format!(
                "{{{},{},{},{},{},{},{},{}}}",
                json_field("module", &record.module),
                json_string_array_field("externs", &record.externs),
                json_string_array_field("extern_interfaces", &record.extern_interfaces),
                json_string_array_field("consts", &record.consts),
                json_string_array_field("type_aliases", &record.type_aliases),
                json_string_array_field("functions", &record.functions),
                json_string_array_field("structs", &record.structs),
                json_string_array_field("traits", &record.traits),
            )
        })
        .collect()
}

pub(crate) fn project_plan_domains_json(
    plan: &nuisc::project::ProjectCompilationPlan,
) -> Result<String, String> {
    let mut domains = Vec::new();
    for item in &plan.abi_resolution.requirements {
        domains.push(scheduler_view_domain_record(
            &item.domain,
            None,
            Some(item.abi.clone()),
        )?);
    }
    Ok(domains
        .iter()
        .map(scheduler_view_domain_record_json)
        .collect::<Vec<_>>()
        .join(","))
}

#[allow(dead_code)]
pub(crate) fn project_workflow_json_fields(
    frontdoor: &WorkflowFrontdoorSurface,
    include_galaxy_flow: bool,
) -> Vec<String> {
    workflow_contract_json_fields(frontdoor, true, true, include_galaxy_flow, false)
}

pub(crate) fn append_project_workflow_json_fields(
    out: &mut String,
    frontdoor: &WorkflowFrontdoorSurface,
    include_galaxy_flow: bool,
) {
    crate::json_surface::append_workflow_contract_json_fields(
        out,
        frontdoor,
        true,
        true,
        include_galaxy_flow,
        false,
    );
}

pub(crate) fn scheduler_view_domain_record(
    domain: &str,
    _package: Option<String>,
    abi: Option<String>,
) -> Result<SchedulerViewDomainRecord, String> {
    let registration = nuisc::registry::load_domain_registration_for_domain(
        std::path::Path::new("nustar-packages"),
        domain,
    )?;
    let shared_abi_json = if let Some(abi_name) = abi.as_deref() {
        let resolution = nuisc::project::ProjectAbiResolution {
            requirements: vec![nuisc::project::ProjectAbiRequirement {
                domain: domain.to_owned(),
                abi: abi_name.to_owned(),
            }],
            explicit: true,
        };
        nuisc::project::project_abi_selection_views(&resolution)
            .into_iter()
            .next()
            .map(|view| nuisc::project::project_abi_selection_view_json(&view))
    } else {
        None
    };
    Ok(SchedulerViewDomainRecord {
        shared_domain_json: nuisc::registry::domain_registration_json(&registration),
        shared_abi_json,
    })
}

pub(crate) fn scheduler_view_domain_record_json(record: &SchedulerViewDomainRecord) -> String {
    let mut fields = Vec::new();
    if let Some(shared_abi_json) = record.shared_abi_json.as_deref() {
        fields.push(format!("\"abi_selection\":{}", shared_abi_json));
    } else {
        fields.push("\"abi_selection\":null".to_owned());
    }
    append_json_object_fields(&record.shared_domain_json, &fields)
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
    match nova_profile.as_ref() {
        Some(profile) => {
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
        None => {}
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectImportRecord {
    galaxy: String,
    library_module: String,
    import_policy: String,
    auto_injectable: bool,
    visible: bool,
    explicit: bool,
    source_kind: Option<String>,
    source_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectImportsReport {
    project_name: String,
    root: PathBuf,
    manifest_path: PathBuf,
    galaxy_dependencies: Vec<String>,
    explicit_galaxy_imports: Vec<String>,
    suggested_galaxy_imports: Vec<String>,
    visible_library_modules: Vec<String>,
    hidden_manual_only_library_modules: Vec<String>,
    records: Vec<ProjectImportRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectImportsApplyResult {
    manifest_path: PathBuf,
    applied: Vec<String>,
    total_explicit_galaxy_imports: usize,
    manifest_updated: bool,
}

pub(crate) fn hidden_manual_only_library_modules_for_project(
    project: &nuisc::project::LoadedProject,
) -> Vec<String> {
    let explicit_galaxy_imports = project
        .manifest
        .galaxy_imports
        .iter()
        .map(|item| format!("{}:{}", item.galaxy, item.library_module))
        .collect::<BTreeSet<_>>();
    project
        .resolved_galaxies
        .iter()
        .filter(|dependency| dependency.library_import_policy.as_str() == "manual-only")
        .flat_map(|dependency| {
            dependency
                .library_modules
                .iter()
                .filter_map(|library_module| {
                    let key = format!("{}:{}", dependency.name, library_module);
                    if explicit_galaxy_imports.contains(&key) {
                        None
                    } else {
                        Some(key)
                    }
                })
        })
        .collect::<Vec<_>>()
}

fn collect_project_imports_report(input: &Path) -> Result<ProjectImportsReport, String> {
    let project = nuisc::project::load_project(input)?;
    let explicit_galaxy_imports = project
        .manifest
        .galaxy_imports
        .iter()
        .map(|item| format!("{}:{}", item.galaxy, item.library_module))
        .collect::<BTreeSet<_>>();
    let mut records = Vec::new();
    let mut visible_library_modules = Vec::new();
    let hidden_manual_only_library_modules =
        hidden_manual_only_library_modules_for_project(&project);
    let suggested_galaxy_imports = hidden_manual_only_library_modules.clone();

    for dependency in &project.resolved_galaxies {
        for (library_module, library_path) in dependency
            .library_modules
            .iter()
            .zip(dependency.resolved_library_paths.iter())
        {
            let key = format!("{}:{}", dependency.name, library_module);
            let visible_module = project
                .modules
                .iter()
                .find(|module| module.path == *library_path);
            let visible = visible_module.is_some();
            if visible {
                visible_library_modules.push(key.clone());
            }
            records.push(ProjectImportRecord {
                galaxy: dependency.name.clone(),
                library_module: library_module.clone(),
                import_policy: dependency.library_import_policy.as_str().to_owned(),
                auto_injectable: dependency.auto_injectable,
                visible,
                explicit: explicit_galaxy_imports.contains(&key),
                source_kind: visible_module.map(|module| module.origin.source_kind().to_owned()),
                source_detail: visible_module.map(|module| module.origin.source_detail()),
            });
        }
    }

    Ok(ProjectImportsReport {
        project_name: project.manifest.name.clone(),
        root: project.root.clone(),
        manifest_path: project.manifest_path.clone(),
        galaxy_dependencies: project
            .manifest
            .galaxy_dependencies
            .iter()
            .map(|item| format!("{}={}", item.name, item.version))
            .collect::<Vec<_>>(),
        explicit_galaxy_imports: explicit_galaxy_imports.into_iter().collect::<Vec<_>>(),
        suggested_galaxy_imports,
        visible_library_modules,
        hidden_manual_only_library_modules,
        records,
    })
}

fn handle_project_imports(
    input: std::path::PathBuf,
    json: bool,
    apply_suggested: bool,
) -> Result<(), String> {
    if apply_suggested {
        let applied = apply_suggested_project_imports(&input)?;
        if json {
            println!("{}", render_project_imports_apply_json(&input, &applied)?);
            return Ok(());
        }
        println!(
            "applied project imports: {}",
            applied.manifest_path.display()
        );
        println!("  applied_galaxy_imports: {}", applied.applied.len());
        println!(
            "  total_explicit_galaxy_imports: {}",
            applied.total_explicit_galaxy_imports
        );
        println!("  manifest_updated: {}", yes_no(applied.manifest_updated));
        if applied.applied.is_empty() {
            println!("  result: no suggested galaxy imports needed to be written");
        } else {
            for item in &applied.applied {
                println!("  applied_galaxy_import: {}", item);
            }
        }
        for line in render_project_imports_text_summary(&input)? {
            println!("{line}");
        }
        return Ok(());
    }
    if json {
        println!("{}", render_project_imports_json(&input)?);
        return Ok(());
    }
    for line in render_project_imports_text_summary(&input)? {
        println!("{line}");
    }
    Ok(())
}

fn apply_suggested_project_imports(input: &Path) -> Result<ProjectImportsApplyResult, String> {
    let report = collect_project_imports_report(input)?;
    let manifest_source = fs::read_to_string(&report.manifest_path).map_err(|error| {
        format!(
            "failed to read project manifest `{}`: {error}",
            report.manifest_path.display()
        )
    })?;
    let updated_source = write_manifest_galaxy_imports(
        &manifest_source,
        &report.explicit_galaxy_imports,
        &report.suggested_galaxy_imports,
    )?;
    let manifest_updated = updated_source != manifest_source;
    if manifest_updated {
        fs::write(&report.manifest_path, updated_source).map_err(|error| {
            format!(
                "failed to update project manifest `{}`: {error}",
                report.manifest_path.display()
            )
        })?;
    }
    Ok(ProjectImportsApplyResult {
        manifest_path: report.manifest_path,
        total_explicit_galaxy_imports: report.explicit_galaxy_imports.len()
            + report.suggested_galaxy_imports.len(),
        applied: report.suggested_galaxy_imports,
        manifest_updated,
    })
}

fn write_manifest_galaxy_imports(
    source: &str,
    explicit: &[String],
    suggested: &[String],
) -> Result<String, String> {
    if suggested.is_empty() {
        return Ok(source.to_owned());
    }
    let merged = merge_manifest_galaxy_imports(explicit, suggested);
    let replacement = render_manifest_galaxy_imports_block(&merged);
    if let Some((start, end)) = find_manifest_field_span(source, "galaxy_imports") {
        let mut updated = String::new();
        updated.push_str(&source[..start]);
        updated.push_str(&replacement);
        updated.push_str(&source[end..]);
        Ok(updated)
    } else {
        let mut updated = source.to_owned();
        if !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(&replacement);
        Ok(updated)
    }
}

fn merge_manifest_galaxy_imports(explicit: &[String], suggested: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();
    for item in explicit.iter().chain(suggested.iter()) {
        if seen.insert(item.clone()) {
            merged.push(item.clone());
        }
    }
    merged
}

fn render_manifest_galaxy_imports_block(values: &[String]) -> String {
    let mut rendered = String::from("galaxy_imports = [\n");
    for value in values {
        rendered.push_str("  \"");
        rendered.push_str(value);
        rendered.push_str("\",\n");
    }
    rendered.push_str("]\n");
    rendered
}

fn find_manifest_field_span(source: &str, key: &str) -> Option<(usize, usize)> {
    let prefix = format!("{key} = ");
    let mut cursor = 0usize;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let offset = line.len() - trimmed.len();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let start = cursor + offset;
            let mut end = cursor + line.len();
            if !rest.contains(']') {
                let mut scan = end;
                while scan < source.len() {
                    let remaining = &source[scan..];
                    let next_len = remaining
                        .find('\n')
                        .map(|idx| idx + 1)
                        .unwrap_or(remaining.len());
                    let next = &remaining[..next_len];
                    end += next.len();
                    if next.contains(']') {
                        break;
                    }
                    scan += next_len;
                }
            }
            return Some((start, end));
        }
        cursor += line.len();
    }
    None
}

fn render_project_imports_text_summary(input: &Path) -> Result<Vec<String>, String> {
    let report = collect_project_imports_report(input)?;
    let mut lines = vec![
        format!("project imports: {}", report.project_name),
        format!("  root: {}", report.root.display()),
        format!("  manifest: {}", report.manifest_path.display()),
        format!(
            "  galaxy_dependencies: {}",
            report.galaxy_dependencies.len()
        ),
        format!(
            "  explicit_galaxy_imports: {}",
            report.explicit_galaxy_imports.len()
        ),
        format!(
            "  visible_library_modules: {}",
            report.visible_library_modules.len()
        ),
        format!(
            "  hidden_manual_only_library_modules: {}",
            report.hidden_manual_only_library_modules.len()
        ),
        format!(
            "  suggested_galaxy_imports: {}",
            report.suggested_galaxy_imports.len()
        ),
    ];
    for item in &report.galaxy_dependencies {
        lines.push(format!("  galaxy_dependency: {}", item));
    }
    for item in &report.explicit_galaxy_imports {
        lines.push(format!("  explicit_galaxy_import: {}", item));
    }
    for item in &report.suggested_galaxy_imports {
        lines.push(format!("  suggested_galaxy_import: {}", item));
    }
    if !report.suggested_galaxy_imports.is_empty() {
        lines.push(format!(
            "  manifest_snippet: galaxy_imports = [{}]",
            report
                .suggested_galaxy_imports
                .iter()
                .map(|item| format!("\"{}\"", item))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for record in &report.records {
        let mut line = format!(
            "  library: {}:{} import_policy={} auto_injectable={} visible={} explicit={}",
            record.galaxy,
            record.library_module,
            record.import_policy,
            yes_no(record.auto_injectable),
            yes_no(record.visible),
            yes_no(record.explicit),
        );
        if let Some(source_kind) = record.source_kind.as_deref() {
            line.push_str(&format!(" source_kind={source_kind}"));
        }
        lines.push(line);
    }
    Ok(lines)
}

pub(crate) fn render_project_imports_json(input: &Path) -> Result<String, String> {
    let report = collect_project_imports_report(input)?;
    let records = report
        .records
        .iter()
        .map(|record| {
            let fields = vec![
                json_field("galaxy", &record.galaxy),
                json_field("library_module", &record.library_module),
                json_field("import_policy", &record.import_policy),
                json_bool_field("auto_injectable", record.auto_injectable),
                json_bool_field("visible", record.visible),
                json_bool_field("explicit", record.explicit),
                json_optional_string_field("source_kind", record.source_kind.as_deref()),
                json_optional_string_field("source_detail", record.source_detail.as_deref()),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>();
    let suggested_manifest_snippet = format!(
        "galaxy_imports = [{}]",
        report
            .suggested_galaxy_imports
            .iter()
            .map(|item| format!("\"{}\"", item))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut out = String::from("{");
    for field in [
        json_field("source_kind", "project"),
        json_field("input", &input.display().to_string()),
        json_field("project", &report.project_name),
        json_field("root", &report.root.display().to_string()),
        json_field("manifest", &report.manifest_path.display().to_string()),
        json_usize_field(
            "galaxy_dependencies_count",
            report.galaxy_dependencies.len(),
        ),
        json_string_array_field("galaxy_dependencies", &report.galaxy_dependencies),
        json_usize_field(
            "explicit_galaxy_imports_count",
            report.explicit_galaxy_imports.len(),
        ),
        json_string_array_field("explicit_galaxy_imports", &report.explicit_galaxy_imports),
        json_usize_field(
            "visible_library_modules_count",
            report.visible_library_modules.len(),
        ),
        json_string_array_field("visible_library_modules", &report.visible_library_modules),
        json_usize_field(
            "hidden_manual_only_library_modules_count",
            report.hidden_manual_only_library_modules.len(),
        ),
        json_string_array_field(
            "hidden_manual_only_library_modules",
            &report.hidden_manual_only_library_modules,
        ),
        json_usize_field(
            "suggested_galaxy_imports_count",
            report.suggested_galaxy_imports.len(),
        ),
        json_string_array_field("suggested_galaxy_imports", &report.suggested_galaxy_imports),
        json_field("suggested_manifest_snippet", &suggested_manifest_snippet),
        json_object_array_field("library_records", &records),
    ] {
        if !out.ends_with('{') {
            out.push(',');
        }
        out.push_str(&field);
    }
    out.push('}');
    Ok(out)
}

pub(crate) fn render_project_imports_apply_json(
    input: &Path,
    applied: &ProjectImportsApplyResult,
) -> Result<String, String> {
    let base = render_project_imports_json(input)?;
    let Some(prefix) = base.strip_suffix('}') else {
        return Err("project imports json renderer returned malformed object".to_owned());
    };
    let mut out = String::from("{");
    for field in [
        json_field("kind", "project_imports_apply"),
        json_field("action", "apply_suggested"),
        json_field(
            "manifest_path",
            &applied.manifest_path.display().to_string(),
        ),
        json_bool_field("manifest_updated", applied.manifest_updated),
        json_usize_field("applied_galaxy_imports_count", applied.applied.len()),
        json_string_array_field("applied_galaxy_imports", &applied.applied),
        json_usize_field(
            "total_explicit_galaxy_imports",
            applied.total_explicit_galaxy_imports,
        ),
    ] {
        if !out.ends_with('{') {
            out.push(',');
        }
        out.push_str(&field);
    }
    if !prefix.trim_start_matches('{').is_empty() {
        out.push(',');
        out.push_str(prefix.trim_start_matches('{'));
    }
    out.push('}');
    Ok(out)
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
mod tests {
    use super::{
        apply_suggested_project_imports, artifact_doctor_command_for_output_dir,
        artifact_workflow_brief, benchmark_run_report_json, build_workflow_frontdoor_surface,
        default_build_output_dir, find_abi_block_span, handle_build, handle_check,
        handle_materialize_artifact, handle_release_check, handle_run_artifact, handle_test,
        handle_unpack_artifact_support, project_abi_checks_json,
        project_compile_workflow_source_profile, project_domain_registry_checks_json,
        project_workflow_json_fields, recommend_project_workflow_step, render_artifact_doctor_json,
        render_build_report_json, render_project_doctor_json, render_project_imports_apply_json,
        render_project_imports_json, render_project_status_json, render_run_artifact_json,
        render_scheduler_view_json, render_workflow_json, resolve_run_artifact_binary_path,
        resolve_runner_clock_domain, run_artifact_command_for_output_dir,
        run_build_output_self_check, run_language_benchmarks_for_source_file,
        run_language_tests_for_source_file, scheduler_view_domain_record,
        scheduler_view_domain_record_json, single_source_workflow_source_profile, upsert_abi_block,
        wait_for_test_child, PublicSurfaceModuleRecord, RawTestOutcome, WorkflowRecommendation,
    };
    use crate::galaxy;
    use crate::json_surface::{
        galaxy_lock_json_fields, project_check_summary_json_fields,
        public_surface_summary_json_fields, workflow_contract_json_fields,
    };
    use crate::surface_render;
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        sync::{Mutex, Once, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn enable_test_quiet_success_logs() {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            env::set_var("NUIS_TEST_QUIET_SUCCESS_LOGS", "1");
        });
    }

    fn repo_root() -> PathBuf {
        enable_test_quiet_success_logs();
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    fn load_stdlib_source_modules(root: &Path, module_dir: &str) -> Vec<String> {
        let module_path = root.join("stdlib").join(module_dir).join("module.toml");
        let source = fs::read_to_string(&module_path)
            .unwrap_or_else(|error| panic!("{}: {error}", module_path.display()));
        let mut inside = false;
        let mut modules = Vec::new();
        for raw_line in source.lines() {
            let line = raw_line.trim();
            if !inside {
                if line.starts_with("source_modules") && line.contains('[') {
                    inside = true;
                }
                continue;
            }
            if line.starts_with(']') {
                break;
            }
            let entry = line.trim_end_matches(',').trim();
            if entry.is_empty() {
                continue;
            }
            let entry = entry.trim_matches('"');
            if !entry.is_empty() {
                modules.push(format!("stdlib/{module_dir}/{entry}"));
            }
        }
        assert!(
            !modules.is_empty(),
            "{} did not declare any source_modules",
            module_path.display()
        );
        modules
    }

    #[test]
    fn checks_stdlib_source_modules() {
        std::thread::Builder::new()
            .name("nuis-stdlib-smoke".to_owned())
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let root = repo_root();
                for module_dir in ["core", "std", "ns-nova", "pixelmagic", "witsage"] {
                    for relative in load_stdlib_source_modules(&root, module_dir) {
                        let input = root.join(relative);
                        handle_check(input.clone()).unwrap_or_else(|error| {
                            panic!("failed to check {}: {error}", input.display())
                        });
                    }
                }
            })
            .expect("spawn stdlib smoke thread")
            .join()
            .expect("join stdlib smoke thread");
    }

    fn temp_dir(label: &str) -> PathBuf {
        enable_test_quiet_success_logs();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = env::temp_dir().join(format!("nuis_{label}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_temp_project_fixture(name: &str, manifest: &str, entry_source: &str) -> PathBuf {
        let root = temp_dir(name);
        fs::write(root.join("nuis.toml"), manifest).expect("write manifest");
        fs::write(root.join("main.ns"), entry_source).expect("write entry");
        root
    }

    fn assert_checked_in_tooling_project_runs(project_root: &str, output_label: &str) {
        let project_root = PathBuf::from(project_root);
        let output_dir = temp_dir(output_label);

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        handle_run_artifact(output_dir.join("nuis.build.manifest.toml"), false)
            .expect("checked-in tooling project run-artifact passes");
    }

    fn with_repo_root_cwd<T>(f: impl FnOnce() -> T) -> T {
        static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = CWD_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = lock.lock().expect("lock cwd guard");
        let original = env::current_dir().expect("current dir");
        let root = repo_root();
        env::set_current_dir(&root).expect("set repo root cwd");
        let result = f();
        env::set_current_dir(original).expect("restore cwd");
        result
    }

    fn empty_galaxy_doctor(project_root: &Path) -> galaxy::GalaxyDoctorReport {
        galaxy::GalaxyDoctorReport {
            project_root: project_root.to_path_buf(),
            project_plan_summary: "<none>".to_owned(),
            deps_root: project_root.join(".nuis").join("deps"),
            local_registry_root: project_root.join(".nuis").join("registry"),
            lock_path: project_root.join("nuis.galaxy.lock"),
            lock_status: "missing".to_owned(),
            lock_error: None,
            dependencies: vec![],
        }
    }

    #[test]
    fn single_source_frontdoor_surface_matches_compile_contract() {
        let frontdoor = build_workflow_frontdoor_surface(
            single_source_workflow_source_profile(),
            WorkflowRecommendation {
                label: "check",
                command: "nuis check <input.ns>",
                reason: "single-file inputs should re-check compile truth first",
            },
        );
        assert_eq!(frontdoor.source_kind, "single-file");
        assert_eq!(frontdoor.workflow_kind, "compile_workflow");
        assert_eq!(
            frontdoor.workflow_brief,
            "check -> test -> build -> artifact_doctor -> run_artifact -> release_check"
        );
        assert!(frontdoor
            .workflow_samples
            .contains("nuis artifact-doctor <output-dir>"));
        assert_eq!(frontdoor.recommended_next_step, "check");
    }

    #[test]
    fn project_frontdoor_surface_uses_project_compile_profile() {
        let frontdoor = build_workflow_frontdoor_surface(
            project_compile_workflow_source_profile(),
            WorkflowRecommendation {
                label: "project_lock_abi",
                command: "nuis project-lock-abi <project-dir|nuis.toml>",
                reason: "freeze ABI choice before broader compile work",
            },
        );
        assert_eq!(frontdoor.source_kind, "project");
        assert_eq!(frontdoor.workflow_kind, "project_compile_workflow");
        assert_eq!(
            frontdoor.workflow_brief,
            nuisc::project_compile_workflow_brief()
        );
        assert_eq!(
            frontdoor.workflow_samples,
            nuisc::project_compile_samples_brief()
        );
        assert_eq!(frontdoor.recommended_next_step, "project_lock_abi");
    }

    #[test]
    fn project_compile_workflow_brief_includes_artifact_follow_up() {
        assert!(nuisc::project_compile_workflow_brief().contains("artifact_doctor"));
        assert!(nuisc::project_compile_workflow_brief().contains("run_artifact"));
        assert!(nuisc::project_compile_samples_brief().contains("nuis artifact-doctor"));
    }

    #[test]
    fn single_source_workflow_helpers_emit_artifact_follow_up_commands() {
        let input = Path::new("examples/demo.ns");
        let output_dir = default_build_output_dir(input);
        assert!(artifact_workflow_brief().contains("artifact_doctor"));
        assert!(
            artifact_doctor_command_for_output_dir(&output_dir).contains("nuis artifact-doctor")
        );
        assert!(run_artifact_command_for_output_dir(&output_dir).contains("nuis run-artifact"));
        assert_eq!(
            run_artifact_command_for_output_dir(&output_dir),
            format!("nuis run-artifact {}", output_dir.display())
        );
    }

    #[test]
    fn resolve_run_artifact_binary_path_accepts_output_dir() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo",
        );
        let output_dir = temp_dir("resolve_run_artifact_binary_path_output_dir");
        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let binary = resolve_run_artifact_binary_path(&output_dir).expect("resolve output-dir");
        assert_eq!(binary, output_dir.join("cli_runtime_demo"));
    }

    #[test]
    fn test_command_checks_declared_project_tests() {
        let dir = temp_dir("project_tests");
        let manifest = dir.join("nuis.toml");
        let entry = dir.join("main.ns");
        let tests_dir = dir.join("tests");
        fs::create_dir_all(&tests_dir).expect("create tests dir");
        let smoke = tests_dir.join("smoke.ns");
        fs::write(
            &manifest,
            r#"
name = "smoke_project"
entry = "main.ns"
tests = ["tests/smoke.ns"]
"#,
        )
        .expect("write manifest");
        fs::write(
            &entry,
            r#"
mod cpu Main {
  fn main() {
    print(1);
  }
}
"#,
        )
        .expect("write entry");
        fs::write(
            &smoke,
            r#"
mod cpu Main {
  fn main() {
    print(2);
  }
}
"#,
        )
        .expect("write smoke");
        handle_test(manifest, false, false, false, false, None).expect("project tests pass");
    }

    #[test]
    fn build_command_writes_project_compile_outputs() {
        let project_root = write_temp_project_fixture(
            "build_command_smoke",
            r#"
name = "build_command_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 7;
  }
}
"#,
        );
        let output_dir = temp_dir("build_command_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");

        for path in [
            output_dir.join("build_command_smoke.ast.txt"),
            output_dir.join("build_command_smoke.nir.txt"),
            output_dir.join("build_command_smoke.yir"),
            output_dir.join("build_command_smoke.ll"),
            output_dir.join("build_command_smoke"),
            output_dir.join("nuis.build.manifest.toml"),
            output_dir.join("nuis.executable.envelope.toml"),
            output_dir.join("nuis.compiled.artifact"),
        ] {
            assert!(path.exists(), "expected build output `{}`", path.display());
        }

        let manifest_report = nuisc::aot::verify_build_manifest(
            output_dir.join("nuis.build.manifest.toml").as_path(),
        )
        .expect("manifest verifies");
        assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
        assert_eq!(manifest_report.artifact_binary_name, "build_command_smoke");
    }

    #[test]
    fn release_check_runs_project_compile_chain_end_to_end() {
        let project_root = write_temp_project_fixture(
            "release_check_smoke",
            r#"
name = "release_check_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 9;
  }
}
"#,
        );
        let output_dir = temp_dir("release_check_outputs");

        handle_release_check(project_root, output_dir.clone(), None, None)
            .expect("release-check passes");

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        assert!(manifest_path.exists(), "expected manifest output");
        let manifest_report =
            nuisc::aot::verify_build_manifest(manifest_path.as_path()).expect("manifest verifies");
        assert_eq!(manifest_report.packaging_mode, "native-cpu-llvm");
        assert_eq!(manifest_report.artifact_binary_name, "release_check_smoke");

        let artifact_report = nuisc::aot::verify_nuis_compiled_artifact(
            output_dir.join("nuis.compiled.artifact").as_path(),
        )
        .expect("artifact verifies");
        assert!(artifact_report.lifecycle_contract_consistent);
        assert!(artifact_report.artifact_roundtrip_verified);
    }

    #[test]
    fn run_artifact_executes_binary_from_manifest_input() {
        let project_root = write_temp_project_fixture(
            "run_artifact_smoke",
            r#"
name = "run_artifact_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  fn main() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
        );
        let output_dir = temp_dir("run_artifact_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        handle_run_artifact(output_dir.join("nuis.build.manifest.toml"), false)
            .expect("run-artifact passes");
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_runtime_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo",
            "run_artifact_cli_runtime_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_session_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_session_demo",
            "run_artifact_cli_session_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_report_session_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_report_session_demo",
            "run_artifact_cli_report_session_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_workflow_runtime_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo",
            "run_artifact_workflow_runtime_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_command_runtime_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo",
            "run_artifact_command_runtime_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_subprocess_runtime_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/subprocess_runtime_demo",
            "run_artifact_subprocess_runtime_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_compile_workflow_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_compile_workflow_demo",
            "run_artifact_cli_compile_workflow_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_build_pipeline_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_build_pipeline_demo",
            "run_artifact_cli_build_pipeline_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_workflow_automation_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_workflow_automation_demo",
            "run_artifact_cli_workflow_automation_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_project_build_report_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_project_build_report_demo",
            "run_artifact_cli_project_build_report_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_pgm_info_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_info_demo",
            "run_artifact_cli_pgm_info_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_pgm_invert_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_invert_demo",
            "run_artifact_cli_pgm_invert_outputs",
        );
    }

    #[test]
    fn run_artifact_executes_checked_in_cli_pgm_threshold_project() {
        assert_checked_in_tooling_project_runs(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_threshold_demo",
            "run_artifact_cli_pgm_threshold_outputs",
        );
    }

    #[test]
    fn cli_pgm_info_binary_accepts_real_pgm_input_file() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_info_demo",
        );
        let output_dir = temp_dir("cli_pgm_info_runtime_probe_outputs");
        let input_path = output_dir.join("probe.pgm");
        fs::write(&input_path, b"P2\n2 2\n15\n0 1 2 3\n").expect("write pgm fixture");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let binary = resolve_run_artifact_binary_path(&output_dir.join("nuis.build.manifest.toml"))
            .expect("resolve built binary");
        let status = Command::new(&binary)
            .arg(&input_path)
            .status()
            .expect("launch cli pgm info binary");
        assert!(status.success(), "expected success status, got {status:?}");
    }

    #[test]
    fn cli_pgm_invert_binary_writes_inverted_pgm_output_file() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_invert_demo",
        );
        let output_dir = temp_dir("cli_pgm_invert_runtime_probe_outputs");
        let input_path = output_dir.join("probe_in.pgm");
        let output_path = output_dir.join("probe_out.pgm");
        fs::write(&input_path, b"P2\n2 2\n15\n0 1 2 3\n").expect("write pgm fixture");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let binary = resolve_run_artifact_binary_path(&output_dir.join("nuis.build.manifest.toml"))
            .expect("resolve built binary");
        let status = Command::new(&binary)
            .arg(&input_path)
            .arg(&output_path)
            .status()
            .expect("launch cli pgm invert binary");
        assert!(status.success(), "expected success status, got {status:?}");

        let output = fs::read_to_string(&output_path).expect("read inverted pgm output");
        assert_eq!(output, "P2\n2 2\n15\n15 14 13 12\n");
    }

    #[test]
    fn cli_pgm_threshold_binary_writes_mask_pgm_output_file() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_threshold_demo",
        );
        let output_dir = temp_dir("cli_pgm_threshold_runtime_probe_outputs");
        let input_path = output_dir.join("probe_in.pgm");
        let output_path = output_dir.join("probe_out.pgm");
        fs::write(&input_path, b"P2\n2 2\n15\n0 1 2 3\n").expect("write pgm fixture");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let binary = resolve_run_artifact_binary_path(&output_dir.join("nuis.build.manifest.toml"))
            .expect("resolve built binary");
        let status = Command::new(&binary)
            .arg(&input_path)
            .arg(&output_path)
            .status()
            .expect("launch cli pgm threshold binary");
        assert!(status.success(), "expected success status, got {status:?}");

        let output = fs::read_to_string(&output_path).expect("read threshold pgm output");
        assert_eq!(output, "P2\n2 2\n15\n0 0 15 15\n");
    }

    #[test]
    fn artifact_doctor_json_reports_ready_to_run_for_built_output() {
        let project_root = write_temp_project_fixture(
            "artifact_doctor_smoke",
            r#"
name = "artifact_doctor_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  fn main() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
        );
        let output_dir = temp_dir("artifact_doctor_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let json = render_artifact_doctor_json(&output_dir);

        assert!(json.contains("\"kind\":\"artifact_doctor\""));
        assert!(json.contains("\"source_kind\":\"output_dir\""));
        assert!(json.contains("\"manifest_exists\":true"));
        assert!(json.contains("\"artifact_exists\":true"));
        assert!(json.contains("\"binary_exists\":true"));
        assert!(json.contains("\"manifest_verified\":true"));
        assert!(json.contains("\"artifact_verified\":true"));
        assert!(json.contains("\"artifact_container_kind\":\"compiled-artifact-v1\""));
        assert!(json.contains("\"artifact_container_version\":1"));
        assert!(json.contains("\"artifact_section_count\":0"));
        assert!(json.contains("\"artifact_section_names\":[]"));
        assert!(json.contains("\"artifact_section_table_valid\":true"));
        assert!(json.contains("\"ready_to_run\":true"));
        assert!(json.contains("\"artifact_diagnostic_code\":\"ready_to_run\""));
        assert!(json.contains("\"self_check_ready\":true"));
        assert!(json.contains("\"self_check_code\":\"ok\""));
        assert!(json.contains("\"project_checks_available\":true"));
        assert!(json.contains("\"project_checks_code\":\"ok\""));
        assert!(json.contains("\"abi_checks_ok\":true"));
        assert!(json.contains("\"registry_checks_ok\":true"));
        assert!(json.contains("\"lowering_checks_ok\":true"));
        assert!(json.contains("\"abi_checks\":[{"));
        assert!(json.contains("\"registry_checks\":[{"));
        assert!(json.contains("\"lowering_checks\":[{"));
        assert!(json.contains("\"recommended_next_step\":\"run_artifact\""));
        assert!(json.contains("\"link_plan_available\":true"));
        assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
        assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
        assert!(json.contains("\"link_plan_final_link_mode\":\"host-toolchain-finalize\""));
        assert!(json.contains("\"link_plan_final_output\":\""));
        assert!(json.contains("\"link_plan_domain_units\":1"));
        assert!(json.contains("\"link_plan_domain_unit_records\":[{"));
        assert!(json.contains("\"domain_family\":\"cpu\""));
        assert!(json.contains("\"packaging_role\":\"host-binary\""));
    }

    #[test]
    fn build_output_self_check_accepts_built_output_dir() {
        let project_root = write_temp_project_fixture(
            "build_output_self_check_smoke",
            r#"
name = "build_output_self_check_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 42;
  }
}
"#,
        );
        let output_dir = temp_dir("build_output_self_check_outputs");
        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let doctor = run_build_output_self_check(&output_dir).expect("self-check passes");
        assert!(doctor.ready_to_run);
        assert_eq!(doctor.source_kind, "output_dir");
        assert_eq!(doctor.recommended_next_step, "run_artifact");
    }

    #[test]
    fn build_output_self_check_reports_missing_artifact_file() {
        let project_root = write_temp_project_fixture(
            "build_output_self_check_missing_artifact",
            r#"
name = "build_output_self_check_missing_artifact"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let output_dir = temp_dir("build_output_self_check_missing_artifact_outputs");
        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        fs::remove_file(output_dir.join("nuis.compiled.artifact")).expect("remove artifact");

        let error = match run_build_output_self_check(&output_dir) {
            Ok(_) => panic!("self-check should fail"),
            Err(error) => error,
        };
        assert!(error.contains("build self-check could not verify manifest"));
        assert!(error.contains("next step: verify_build_manifest"));
        assert!(error.contains("nuis.compiled.artifact"));
        let json = render_artifact_doctor_json(&output_dir);
        assert!(json.contains("\"self_check_ready\":false"));
        assert!(json.contains("\"artifact_diagnostic_code\":\"manifest_invalid\""));
        assert!(json.contains("\"self_check_code\":\"manifest_verify_failed\""));
        assert!(json.contains("\"project_checks_code\":\"unavailable\""));
        assert!(json.contains("\"self_check_error\":\"build self-check could not verify manifest"));
    }

    #[test]
    fn build_output_self_check_reports_missing_binary_as_incomplete_output() {
        let project_root = write_temp_project_fixture(
            "build_output_self_check_missing_binary",
            r#"
name = "build_output_self_check_missing_binary"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
        );
        let output_dir = temp_dir("build_output_self_check_missing_binary_outputs");
        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        fs::remove_file(output_dir.join("build_output_self_check_missing_binary"))
            .expect("remove binary");

        let error = match run_build_output_self_check(&output_dir) {
            Ok(_) => panic!("self-check should fail"),
            Err(error) => error,
        };
        assert!(error.contains("build self-check could not verify manifest"));
        assert!(error.contains("next step: verify_build_manifest"));
        assert!(error.contains("nuis verify-build-manifest"));
    }

    #[test]
    fn build_report_json_exposes_lifecycle_and_domain_unit_summary() {
        let project_root = write_temp_project_fixture(
            "build_report_smoke",
            r#"
name = "build_report_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  fn main() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
        );
        let output_dir = temp_dir("build_report_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let json = render_build_report_json(&output_dir);

        assert!(json.contains("\"kind\":\"build_report\""));
        assert!(json.contains("\"ready_to_run\":true"));
        assert!(json.contains("\"artifact_diagnostic_code\":\"ready_to_run\""));
        assert!(json.contains("\"self_check_ready\":true"));
        assert!(json.contains("\"self_check_code\":\"ok\""));
        assert!(json.contains("\"project_checks_available\":true"));
        assert!(json.contains("\"project_checks_code\":\"ok\""));
        assert!(json.contains("\"abi_checks_ok\":true"));
        assert!(json.contains("\"registry_checks_ok\":true"));
        assert!(json.contains("\"lowering_checks_ok\":true"));
        assert!(json.contains("\"abi_checks\":[{"));
        assert!(json.contains("\"registry_checks\":[{"));
        assert!(json.contains("\"lowering_checks\":[{"));
        assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
        assert!(json.contains("\"packaging_mode\":\"native-cpu-llvm\""));
        assert!(json.contains("\"lifecycle_bootstrap_entry\":\"nuis.bootstrap.lifecycle.v1\""));
        assert!(json.contains("\"lifecycle_tick_policy\":\"owned-pump.active-wait-drain\""));
        assert!(json.contains("\"domain_units_count\":1"));
        assert!(json.contains("\"domain_units\":[{"));
        assert!(json.contains("\"domain_family\":\"cpu\""));
        assert!(json.contains("\"artifact_roundtrip_verified\":true"));
        assert!(json.contains("\"lifecycle_contract_consistent\":true"));
        assert!(json.contains("\"heterogeneous_domain_count\":0"));
        assert!(json.contains("\"bridge_registry_units\":0"));
        assert!(json.contains("\"host_bridge_plan_units\":0"));
        assert!(json.contains("\"runtime_load_attempted\":true"));
        assert!(json.contains("\"runtime_load_ok\":true"));
        assert!(json.contains("\"runtime_loaded_lifecycle_entry\":\"nuis.bootstrap.lifecycle.v1\""));
        assert!(json.contains("\"runtime_loaded_domain_units\":1"));
        assert!(json.contains("\"runtime_loaded_heterogeneous_units\":0"));
        assert!(json.contains("\"runtime_loaded_payload_blobs\":0"));
        assert!(json.contains("\"runtime_execution_attempted\":true"));
        assert!(json.contains("\"runtime_execution_ok\":true"));
        assert!(json.contains("\"runtime_execution_domains\":0"));
        assert!(json.contains("\"runtime_execution_plan_phases\":0"));
        assert!(json.contains("\"runtime_execution_trace_events\":0"));
        assert!(json.contains("\"link_plan_domain_unit_records\":[{"));
    }

    #[test]
    fn run_artifact_json_reports_prelaunch_summary_for_built_output() {
        let project_root = write_temp_project_fixture(
            "run_artifact_json_smoke",
            r#"
name = "run_artifact_json_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 0;
  }
}
"#,
        );
        let output_dir = temp_dir("run_artifact_json_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

        assert!(json.contains("\"kind\":\"run_artifact\""));
        assert!(json.contains("\"ready_to_run\":true"));
        assert!(json.contains("\"binary_resolved\":true"));
        assert!(json.contains("\"binary_path\":\""));
        assert!(json.contains("\"heterogeneous_domain_count\":0"));
        assert!(json.contains("\"bridge_registry_units\":0"));
        assert!(json.contains("\"host_bridge_plan_units\":0"));
        assert!(json.contains("\"link_plan_available\":true"));
        assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
    }

    #[test]
    fn build_report_json_exposes_real_heterogeneous_runtime_summary() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo",
        );
        let output_dir = temp_dir("build_report_shader_profile_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let json = render_build_report_json(&output_dir);

        assert!(json.contains("\"domain_units_count\":2"));
        assert!(json.contains("\"heterogeneous_domain_count\":1"));
        assert!(json.contains("\"domain_family\":\"shader\""));
        assert!(json.contains("\"packaging_role\":\"hetero-contract\""));
        assert!(json.contains("\"artifact_payload_format\":\"ndpb-v2\""));
        assert!(json.contains("\"bridge_registry_units\":1"));
        assert!(json.contains("\"bridge_registry_checked\":1"));
        assert!(json.contains("\"host_bridge_plan_units\":1"));
        assert!(json.contains("\"domain_payload_blobs_checked\":1"));
        assert!(json.contains("\"domain_payload_bridge_plans_checked\":1"));
        assert!(json.contains("\"domain_bridge_stubs_checked\":1"));
        assert!(json.contains("\"link_plan_domain_units\":2"));
        assert!(json.contains("\"runtime_execution_attempted\":true"));
        assert!(json.contains("\"runtime_execution_ok\":true"));
        assert!(json.contains("\"runtime_execution_domains\":1"));
        assert!(json.contains("\"runtime_execution_plan_phases\":"));
        assert!(json.contains("\"runtime_execution_trace_events\":"));
    }

    #[test]
    fn run_artifact_json_exposes_real_heterogeneous_runtime_summary() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo",
        );
        let output_dir = temp_dir("run_artifact_shader_profile_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

        assert!(json.contains("\"binary_resolved\":true"));
        assert!(json.contains("\"heterogeneous_domain_count\":1"));
        assert!(json.contains("\"bridge_registry_units\":1"));
        assert!(json.contains("\"host_bridge_plan_units\":1"));
        assert!(json.contains("\"domain_payload_blobs_checked\":1"));
        assert!(json.contains("\"link_plan_domain_units\":2"));
        assert!(json.contains("\"domain_family\":\"shader\""));
    }

    #[test]
    fn build_report_json_exposes_bridge_bearing_exchange_summary() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo",
        );
        let output_dir = temp_dir("build_report_shader_packet_bridge_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let json = render_build_report_json(&output_dir);

        assert!(json.contains("\"packaging_mode\":\"window-aot-bundle\""));
        assert!(json.contains("\"domain_units_count\":3"));
        assert!(json.contains("\"heterogeneous_domain_count\":2"));
        assert!(json.contains("\"domain_family\":\"data\""));
        assert!(json.contains("\"domain_family\":\"shader\""));
        assert!(json.contains("\"bridge_registry_units\":2"));
        assert!(json.contains("\"bridge_registry_entries_checked\":2"));
        assert!(json.contains("\"host_bridge_plan_units\":2"));
        assert!(json.contains("\"host_bridge_plan_entries_checked\":2"));
        assert!(json.contains("\"domain_payload_blobs_checked\":2"));
        assert!(json.contains("\"domain_payload_bridge_plans_checked\":2"));
        assert!(json.contains("\"domain_bridge_stubs_checked\":2"));
        assert!(json.contains("\"link_plan_final_stage\":\"heterogeneous-bundle-pack\""));
        assert!(json.contains("\"link_plan_final_driver\":\"yir-pack-aot\""));
        assert!(json.contains("\"link_plan_domain_units\":3"));
    }

    #[test]
    fn run_artifact_json_exposes_bridge_bearing_exchange_summary() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo",
        );
        let output_dir = temp_dir("run_artifact_shader_packet_bridge_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

        assert!(json.contains("\"binary_resolved\":true"));
        assert!(json.contains("\"heterogeneous_domain_count\":2"));
        assert!(json.contains("\"bridge_registry_units\":2"));
        assert!(json.contains("\"host_bridge_plan_units\":2"));
        assert!(json.contains("\"domain_payload_blobs_checked\":2"));
        assert!(json.contains("\"domain_payload_bridge_plans_checked\":2"));
        assert!(json.contains("\"link_plan_final_stage\":\"heterogeneous-bundle-pack\""));
        assert!(json.contains("\"link_plan_final_driver\":\"yir-pack-aot\""));
        assert!(json.contains("\"link_plan_domain_units\":3"));
    }

    #[test]
    fn unpack_artifact_support_materializes_embedded_sidecars_for_bridge_project() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo",
        );
        let output_dir = temp_dir("unpack_artifact_support_bridge_build_outputs");
        let unpack_dir = temp_dir("unpack_artifact_support_bridge_unpack_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        handle_unpack_artifact_support(
            output_dir.join("nuis.compiled.artifact"),
            unpack_dir.clone(),
            false,
        )
        .expect("unpack-artifact-support passes");

        for path in [
            unpack_dir.join("nuis.bridge.registry.toml"),
            unpack_dir.join("nuis.host-bridge.plan-index.toml"),
            unpack_dir.join("nuis.domain.data.artifact.toml"),
            unpack_dir.join("nuis.domain.data.payload.toml"),
            unpack_dir.join("nuis.domain.data.payload.bin"),
            unpack_dir.join("nuis.domain.data.bridge.stub.txt"),
            unpack_dir.join("nuis.domain.shader.artifact.toml"),
            unpack_dir.join("nuis.domain.shader.payload.toml"),
            unpack_dir.join("nuis.domain.shader.payload.bin"),
            unpack_dir.join("nuis.domain.shader.bridge.stub.txt"),
        ] {
            assert!(
                path.exists(),
                "expected unpacked support `{}`",
                path.display()
            );
        }
    }

    #[test]
    fn materialize_artifact_rebuilds_frontdoor_bundle_and_support_sidecars() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo",
        );
        let build_output_dir = temp_dir("materialize_artifact_bridge_build_outputs");
        let materialize_dir = temp_dir("materialize_artifact_bridge_bundle_outputs");

        handle_build(project_root, build_output_dir.clone(), false, None, None)
            .expect("build passes");
        handle_materialize_artifact(
            build_output_dir.join("nuis.build.manifest.toml"),
            materialize_dir.clone(),
            false,
        )
        .expect("materialize-artifact passes");

        for path in [
            materialize_dir.join("nuis.executable.envelope.toml"),
            materialize_dir.join("nuis.build.manifest.toml"),
            materialize_dir.join("nuis.compiled.artifact"),
            materialize_dir.join("shader_packet_bridge_demo"),
            materialize_dir.join("nuis.bridge.registry.toml"),
            materialize_dir.join("nuis.host-bridge.plan-index.toml"),
            materialize_dir.join("nuis.domain.data.payload.bin"),
            materialize_dir.join("nuis.domain.shader.payload.bin"),
        ] {
            assert!(
                path.exists(),
                "expected materialized output `{}`",
                path.display()
            );
        }

        let report = nuisc::aot::verify_build_manifest(
            materialize_dir.join("nuis.build.manifest.toml").as_path(),
        )
        .expect("materialized manifest verifies");
        assert_eq!(report.artifact_binary_name, "shader_packet_bridge_demo");
        assert_eq!(report.packaging_mode, "window-aot-bundle");
    }

    #[test]
    fn workflow_json_reports_frontdoor_and_artifact_fields_for_project() {
        let project_root = write_temp_project_fixture(
            "workflow_json_smoke",
            r#"
name = "workflow_json_smoke"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 6;
  }
}
"#,
        );
        let tests_dir = project_root.join("tests");
        fs::create_dir_all(&tests_dir).expect("create tests dir");
        fs::write(
            tests_dir.join("smoke.ns"),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write smoke test");

        let json = render_workflow_json(&project_root).expect("render workflow json");

        assert!(json.contains("\"source_kind\":\"project\""));
        assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
        assert!(json.contains("\"artifact_workflow\":\"build -> inspect_artifact -> verify_artifact -> artifact_doctor -> verify_build_manifest -> run_artifact\""));
        assert!(json.contains("\"artifact_ready_to_run\":false"));
        assert!(json.contains("\"artifact_diagnostic_code\":\"missing_outputs\""));
        assert!(json.contains("\"artifact_self_check_ready\":false"));
        assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
        assert!(json.contains("\"artifact_self_check_error\":\""));
        assert!(json.contains("\"project_checks_available\":true"));
        assert!(json.contains("\"project_checks_code\":\"ok\""));
        assert!(json.contains("\"abi_checks_ok\":true"));
        assert!(json.contains("\"registry_checks_ok\":true"));
        assert!(json.contains("\"lowering_checks_ok\":true"));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"link_plan_final_stage\":null"));
        assert!(json.contains("\"compile_pipeline_available\":true"));
        assert!(json.contains("\"compile_pipeline_source_kind\":\"project\""));
        assert!(json.contains("\"compile_pipeline_ready_for_aot\":true"));
        assert!(json.contains("\"compile_pipeline_recommended_next_step\":\"build\""));
        assert!(json.contains("\"compile_pipeline_stage_count\":"));
        assert!(json.contains("\"compile_pipeline_ok_stage_count\":"));
        assert!(json.contains("\"id\":\"yir_lower\""));
        assert!(json.contains("\"id\":\"llvm_emit\""));
    }

    #[test]
    fn workflow_json_reports_link_plan_for_built_project_output() {
        let project_root = write_temp_project_fixture(
            "workflow_json_built_smoke",
            r#"
name = "workflow_json_built_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 8;
  }
}
"#,
        );
        let output_dir = default_build_output_dir(&project_root);

        handle_build(project_root.clone(), output_dir.clone(), false, None, None)
            .expect("build passes");

        let json = render_workflow_json(&project_root).expect("render workflow json");

        assert!(json.contains("\"artifact_ready_to_run\":true"));
        assert!(json.contains("\"artifact_diagnostic_code\":\"ready_to_run\""));
        assert!(json.contains("\"artifact_self_check_ready\":true"));
        assert!(json.contains("\"artifact_self_check_code\":\"ok\""));
        assert!(json.contains("\"artifact_recommended_next_step\":\"run_artifact\""));
        assert!(json.contains("\"artifact_self_check_error\":null"));
        assert!(json.contains("\"project_checks_available\":true"));
        assert!(json.contains("\"project_checks_code\":\"ok\""));
        assert!(json.contains("\"abi_checks_ok\":true"));
        assert!(json.contains("\"registry_checks_ok\":true"));
        assert!(json.contains("\"lowering_checks_ok\":true"));
        assert!(json.contains("\"link_plan_available\":true"));
        assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
        assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
        assert!(json.contains("\"link_plan_final_link_mode\":\"host-toolchain-finalize\""));
        assert!(json.contains("\"compile_pipeline_available\":true"));
        assert!(json.contains("\"compile_pipeline_ready_for_aot\":true"));
        assert!(json.contains("\"compile_pipeline_summary\":\"source_kind=project"));
    }

    #[test]
    fn workflow_json_reports_frontdoor_and_artifact_fields_for_single_source() {
        let dir = temp_dir("workflow_json_single_source");
        let input = dir.join("hello.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
        )
        .expect("write source");

        let json = render_workflow_json(&input).expect("render workflow json");

        assert!(json.contains("\"source_kind\":\"single-file\""));
        assert!(json.contains("\"workflow_kind\":\"compile_workflow\""));
        assert!(json.contains("\"artifact_ready_to_run\":false"));
        assert!(json.contains("\"artifact_diagnostic_code\":\"missing_outputs\""));
        assert!(json.contains("\"artifact_self_check_ready\":false"));
        assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
        assert!(json.contains("\"artifact_self_check_error\":\""));
        assert!(json.contains("\"project_checks_available\":false"));
        assert!(json.contains("\"project_checks_code\":\"unavailable\""));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"link_plan_final_stage\":null"));
        assert!(json.contains("\"compile_pipeline_available\":true"));
        assert!(json.contains("\"compile_pipeline_source_kind\":\"single_source\""));
        assert!(json.contains("\"compile_pipeline_ready_for_aot\":true"));
        assert!(json.contains("\"compile_pipeline_recommended_next_step\":\"build\""));
        assert!(json.contains("\"compile_pipeline_stage_count\":"));
        assert!(json.contains("\"compile_pipeline_ok_stage_count\":"));
    }

    #[test]
    fn workflow_json_reports_self_check_failure_for_damaged_output_dir() {
        let project_root = write_temp_project_fixture(
            "workflow_json_damaged_output",
            r#"
name = "workflow_json_damaged_output"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 9;
  }
}
"#,
        );
        let output_dir = default_build_output_dir(&project_root);
        handle_build(project_root.clone(), output_dir.clone(), false, None, None)
            .expect("build passes");
        fs::remove_file(output_dir.join("nuis.compiled.artifact")).expect("remove artifact");

        let json = render_workflow_json(&project_root).expect("render workflow json");

        assert!(json.contains("\"artifact_ready_to_run\":false"));
        assert!(json.contains("\"artifact_diagnostic_code\":\"manifest_invalid\""));
        assert!(json.contains("\"artifact_self_check_ready\":false"));
        assert!(json.contains("\"artifact_self_check_code\":\"manifest_verify_failed\""));
        assert!(json.contains("\"artifact_recommended_next_step\":\"verify_build_manifest\""));
        assert!(json.contains("\"project_checks_code\":\"ok\""));
        assert!(json.contains(
            "\"artifact_self_check_error\":\"build self-check could not verify manifest"
        ));
    }

    #[test]
    fn project_workflow_recommendation_prefers_lock_abi_for_auto_projects() {
        let project_root = write_temp_project_fixture(
            "workflow_auto_abi",
            r#"
name = "workflow_auto_abi"
entry = "main.ns"
modules = ["main.ns"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let project = nuisc::project::load_project(&project_root).expect("load project");
        let plan =
            nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
        let doctor = empty_galaxy_doctor(&project.root);

        let recommendation =
            recommend_project_workflow_step(&plan, &[], &[], &doctor, false, false);

        assert_eq!(recommendation.label, "project_lock_abi");
        assert_eq!(
            recommendation.command,
            "nuis project-lock-abi <project-dir|nuis.toml>"
        );
    }

    #[test]
    fn project_workflow_recommendation_prefers_project_status_for_missing_tests() {
        let project_root = write_temp_project_fixture(
            "workflow_missing_tests",
            r#"
name = "workflow_missing_tests"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let project = nuisc::project::load_project(&project_root).expect("load project");
        let plan =
            nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
        let doctor = empty_galaxy_doctor(&project.root);
        let missing_tests = vec![project.root.join("tests/smoke.ns")];

        let recommendation = recommend_project_workflow_step(
            &plan,
            &missing_tests,
            &missing_tests,
            &doctor,
            false,
            false,
        );

        assert_eq!(recommendation.label, "project_status");
        assert_eq!(
            recommendation.command,
            "nuis project-status <project-dir|nuis.toml>"
        );
    }

    #[test]
    fn project_workflow_recommendation_defaults_to_check_once_shape_is_stable() {
        let project_root = write_temp_project_fixture(
            "workflow_ready",
            r#"
name = "workflow_ready"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let tests_dir = project_root.join("tests");
        fs::create_dir_all(&tests_dir).expect("create tests dir");
        fs::write(
            tests_dir.join("smoke.ns"),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
        )
        .expect("write smoke test");
        let project = nuisc::project::load_project(&project_root).expect("load project");
        let plan =
            nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
        let doctor = empty_galaxy_doctor(&project.root);
        let declared_tests = vec![project.root.join("tests/smoke.ns")];

        let recommendation =
            recommend_project_workflow_step(&plan, &declared_tests, &[], &doctor, false, false);

        assert_eq!(recommendation.label, "check");
        assert_eq!(recommendation.command, "nuis check <project-dir|nuis.toml>");
    }

    #[test]
    fn project_workflow_recommendation_prefers_project_imports_apply_for_hidden_manual_only_modules(
    ) {
        let project_root = write_temp_project_fixture(
            "workflow_manual_only_imports",
            r#"
name = "workflow_manual_only_imports"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
galaxy = ["ns-nova=workspace"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let tests_dir = project_root.join("tests");
        fs::create_dir_all(&tests_dir).expect("create tests dir");
        fs::write(
            tests_dir.join("smoke.ns"),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
        )
        .expect("write smoke test");
        let project = nuisc::project::load_project(&project_root).expect("load project");
        let plan =
            nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
        let mut doctor = empty_galaxy_doctor(&project.root);
        doctor.lock_status = "ok".to_owned();
        let declared_tests = vec![project.root.join("tests/smoke.ns")];

        let recommendation =
            recommend_project_workflow_step(&plan, &declared_tests, &[], &doctor, false, true);

        assert_eq!(recommendation.label, "project_imports_apply_suggested");
        assert_eq!(
            recommendation.command,
            "nuis project-imports --apply-suggested <project-dir|nuis.toml>"
        );
    }

    #[test]
    fn project_workflow_json_fields_track_compile_and_galaxy_briefs() {
        let frontdoor = build_workflow_frontdoor_surface(
            project_compile_workflow_source_profile(),
            WorkflowRecommendation {
                label: "check",
                command: "nuis check <project-dir|nuis.toml>",
                reason: "compile truth should remain the default once the project shape is stable",
            },
        );

        let without_galaxy = project_workflow_json_fields(&frontdoor, false);
        assert!(without_galaxy.iter().any(|field| {
            field
                == &format!(
                    "\"project_compile_workflow\":\"{}\"",
                    nuisc::project_compile_workflow_brief()
                )
        }));
        assert!(without_galaxy.iter().any(|field| {
            field
                == &format!(
                    "\"project_test_workflow\":\"{}\"",
                    nuisc::project_test_workflow_brief()
                )
        }));
        assert!(!without_galaxy
            .iter()
            .any(|field| field.contains("\"project_galaxy_workflow\"")));

        let with_galaxy = project_workflow_json_fields(&frontdoor, true);
        assert!(with_galaxy.iter().any(|field| {
            field
                == &format!(
                    "\"project_galaxy_workflow\":\"{}\"",
                    nuisc::project_galaxy_workflow_brief()
                )
        }));
    }

    #[test]
    fn workflow_contract_json_fields_expose_shared_frontdoor_keys() {
        let frontdoor = build_workflow_frontdoor_surface(
            project_compile_workflow_source_profile(),
            WorkflowRecommendation {
                label: "check",
                command: "nuis check <project-dir|nuis.toml>",
                reason: "shared workflow contract should always carry the frontdoor routing fields",
            },
        );

        let fields = workflow_contract_json_fields(&frontdoor, true, true, true, true);

        for key in [
            "\"frontdoor\":{",
            "\"workflow_kind\":\"project_compile_workflow\"",
            "\"workflow_brief\":\"",
            "\"workflow_samples\":\"",
            "\"recommended_next_step\":\"check\"",
            "\"recommended_command\":\"nuis check <project-dir|nuis.toml>\"",
            "\"recommended_reason\":\"shared workflow contract should always carry the frontdoor routing fields\"",
            "\"project_compile_workflow\":\"",
            "\"project_compile_samples\":\"",
            "\"project_test_workflow\":\"",
            "\"project_galaxy_workflow\":\"",
            "\"debug_workflow\":\"",
            "\"debug_samples\":\"",
        ] {
            assert!(
                fields.iter().any(|field| field.contains(key)),
                "missing shared workflow contract key {key}"
            );
        }
    }

    #[test]
    fn galaxy_lock_json_fields_report_missing_lock_surface() {
        let dir = temp_dir("galaxy_lock_fields_missing");
        let lock_path = dir.join("nuis.galaxy.lock");

        let fields = galaxy_lock_json_fields(Err("missing".to_owned()), &lock_path, &[]);

        assert!(fields
            .iter()
            .any(|field| field == "\"galaxy_lock_status\":\"missing\""));
        assert!(fields
            .iter()
            .any(|field| field.contains("\"galaxy_lock_path\":\"")));
        assert!(!fields
            .iter()
            .any(|field| field.contains("\"galaxy_lock_error\"")));
    }

    #[test]
    fn public_surface_summary_json_fields_count_public_members() {
        let records = vec![PublicSurfaceModuleRecord {
            module: "cpu::Main".to_owned(),
            externs: vec!["ffi_print".to_owned()],
            extern_interfaces: vec!["ClockBridge".to_owned()],
            consts: vec!["DEFAULT_PORT".to_owned()],
            type_aliases: vec!["ResultCode".to_owned()],
            functions: vec!["run".to_owned(), "tick".to_owned()],
            structs: vec!["State(fields=1)".to_owned()],
            traits: vec!["Runnable".to_owned()],
        }];

        let fields = public_surface_summary_json_fields(&records);

        assert!(fields
            .iter()
            .any(|field| field == "\"public_surface_modules\":1"));
        assert!(fields.iter().any(|field| field == "\"public_externs\":1"));
        assert!(fields
            .iter()
            .any(|field| field == "\"public_extern_interfaces\":1"));
        assert!(fields.iter().any(|field| field == "\"public_consts\":1"));
        assert!(fields
            .iter()
            .any(|field| field == "\"public_type_aliases\":1"));
        assert!(fields.iter().any(|field| field == "\"public_functions\":2"));
        assert!(fields.iter().any(|field| field == "\"public_structs\":1"));
        assert!(fields.iter().any(|field| field == "\"public_traits\":1"));
    }

    #[test]
    fn project_check_summary_json_fields_report_all_green() {
        let project = nuisc::project::load_project(
            &repo_root().join("examples/projects/domains/net_session_recipe_demo"),
        )
        .expect("load project");
        let plan = nuisc::project::build_project_compilation_plan(&project).expect("build plan");
        let abi_checks =
            nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)
                .expect("abi checks");
        let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
        let lowering_checks =
            nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);

        let fields =
            project_check_summary_json_fields(&abi_checks, &registry_checks, &lowering_checks);

        assert!(fields.iter().any(|field| field == "\"abi_checks_ok\":true"));
        assert!(fields
            .iter()
            .any(|field| field == "\"registry_checks_ok\":true"));
        assert!(fields
            .iter()
            .any(|field| field == "\"lowering_checks_ok\":true"));
        assert!(fields
            .iter()
            .any(|field| field.starts_with("\"abi_checks_count\":")));
        assert!(fields
            .iter()
            .any(|field| field.starts_with("\"registry_checks_count\":")));
        assert!(fields
            .iter()
            .any(|field| field.starts_with("\"lowering_checks_count\":")));
    }

    #[test]
    fn project_status_json_reports_frontdoor_and_surface_fields() {
        let project_root = write_temp_project_fixture(
            "status_json_smoke",
            r#"
name = "status_json_smoke"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
galaxy = ["ns-nova=workspace"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  pub fn exported() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return handle;
  }

  fn main() -> i64 {
    return text_handle_helper() + exported();
  }
}
"#,
        );
        let tests_dir = project_root.join("tests");
        fs::create_dir_all(&tests_dir).expect("create tests dir");
        fs::write(
            tests_dir.join("smoke.ns"),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write smoke test");

        let json = render_project_status_json(&project_root).expect("render status json");

        assert!(json.contains("\"source_kind\":\"project\""));
        assert!(json.contains("\"project\":\"status_json_smoke\""));
        assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
        assert!(json.contains(&format!(
            "\"project_compile_workflow\":\"{}\"",
            nuisc::project_compile_workflow_brief()
        )));
        assert!(json.contains("\"recommended_next_step\":\"galaxy_lock_deps\""));
        assert!(json
            .contains("\"recommended_command\":\"nuis galaxy lock-deps <project-dir|nuis.toml>\""));
        assert!(json.contains(
            "\"recommended_reason\":\"the project already declares galaxy dependencies but does not yet have a lockfile\""
        ));
        assert!(json.contains("\"artifact_output_dir\":\""));
        assert!(json.contains("\"artifact_ready_to_run\":false"));
        assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"link_plan_final_stage\":null"));
        assert!(json.contains("\"tests_declared\":1"));
        assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
        assert!(json.contains("\"public_surface_modules\":3"));
        assert!(json.contains("\"public_functions\":10"));
        assert!(json.contains("\"galaxy_lock_status\":\"missing\""));
        assert!(json.contains("\"galaxy_surface_ids_count\":13"));
        assert!(json.contains("\"surface.ns-nova.renderer.v1\""));
        assert!(json.contains("\"contract.core.prelude.primitive-values.v1\""));
        assert!(json.contains("\"surface.std.collections.v1\""));
        assert!(json.contains("\"galaxy_records\":[{"));
        assert!(json.contains("\"galaxy_imports_count\":0"));
        assert!(json.contains("\"galaxy_hidden_manual_only_library_modules_count\":1"));
        assert!(json.contains(
            "\"galaxy_hidden_manual_only_library_modules\":[\"ns-nova:lib/nova_contracts.ns\"]"
        ));
        assert!(json.contains("\"tests\":[{"));
        assert!(json.contains("\"exists\":true"));
        assert!(json.contains("\"domains\":["));
    }

    #[test]
    fn project_status_text_summary_reports_text_handle_rewrite_hits() {
        let project_root = write_temp_project_fixture(
            "status_text_handle_summary",
            r#"
name = "status_text_handle_summary"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  fn main() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
        );

        let lines = surface_render::render_project_status_text_summary(&project_root)
            .expect("render status text summary");

        assert!(lines
            .iter()
            .any(|line| line == "  text_handle_rewrite_helper_hits: 1"));
        assert!(lines
            .iter()
            .any(|line| line == "  text_handle_rewrite_local_hits: 1"));
        assert!(lines
            .iter()
            .any(|line| line == "  text_handle_rewrite_total_hits: 2"));

        let mut written = String::new();
        surface_render::write_project_status_text_summary(&mut written, &project_root)
            .expect("write status text summary");
        assert_eq!(written.lines().collect::<Vec<_>>(), lines);
    }

    #[test]
    fn project_doctor_json_reports_missing_test_and_health_checks() {
        let project_root = write_temp_project_fixture(
            "doctor_json_smoke",
            r#"
name = "doctor_json_smoke"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/missing.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  fn main() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
        );

        let json = render_project_doctor_json(&project_root).expect("render doctor json");

        assert!(json.contains("\"source_kind\":\"project\""));
        assert!(json.contains("\"project\":\"doctor_json_smoke\""));
        assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
        assert!(json.contains("\"tests_declared\":1"));
        assert!(json.contains("\"tests_missing\":1"));
        assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
        assert!(json.contains("\"abi_checks_ok\":true"));
        assert!(json.contains("\"registry_checks_ok\":true"));
        assert!(json.contains("\"lowering_checks_ok\":true"));
        assert!(json.contains("\"artifact_output_dir\":\""));
        assert!(json.contains("\"artifact_ready_to_run\":false"));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"galaxy_check_status\":\"skipped\""));
        assert!(json.contains("\"galaxy_lock_status\":\"missing\""));
        assert!(json.contains("\"galaxy_imports_count\":1"));
        assert!(json.contains("\"galaxy_surface_ids_count\":13"));
        assert!(json.contains("\"surface.ns-nova.renderer.v1\""));
        assert!(json.contains("\"contract.core.prelude.primitive-values.v1\""));
        assert!(json.contains("\"surface.std.collections.v1\""));
        assert!(json.contains("\"galaxy_records\":[{"));
        assert!(json.contains("\"galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
        assert!(json.contains("\"galaxy_hidden_manual_only_library_modules_count\":0"));
        assert!(json.contains("\"galaxy_hidden_manual_only_library_modules\":[]"));
        assert!(json.contains("\"next_steps\":["));
        assert!(json.contains("some declared project tests are missing on disk"));
        assert!(json.contains("\"tests\":[{"));
        assert!(json.contains("\"exists\":false"));
        assert!(json.contains("\"domains\":["));
    }

    #[test]
    fn project_doctor_text_summary_reports_text_handle_rewrite_hits() {
        let project_root = write_temp_project_fixture(
            "doctor_text_handle_summary",
            r#"
name = "doctor_text_handle_summary"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  fn main() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
        );

        let lines = surface_render::render_project_doctor_text_summary(&project_root)
            .expect("render doctor text summary");

        assert!(lines
            .iter()
            .any(|line| line == "  text_handle_rewrite_helper_hits: 1"));
        assert!(lines
            .iter()
            .any(|line| line == "  text_handle_rewrite_local_hits: 1"));
        assert!(lines
            .iter()
            .any(|line| line == "  text_handle_rewrite_total_hits: 2"));

        let mut written = String::new();
        surface_render::write_project_doctor_text_summary(&mut written, &project_root)
            .expect("write doctor text summary");
        assert_eq!(written.lines().collect::<Vec<_>>(), lines);
    }

    #[test]
    fn project_doctor_json_suggests_galaxy_imports_for_hidden_manual_only_modules() {
        let project_root = write_temp_project_fixture(
            "doctor_manual_only_import_hint",
            r#"
name = "doctor_manual_only_import_hint"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 4;
  }
}
"#,
        );

        let json = render_project_doctor_json(&project_root).expect("render doctor json");

        assert!(json.contains("\"galaxy_surface_ids_count\":13"));
        assert!(json.contains("\"surface.ns-nova.renderer.v1\""));
        assert!(json.contains("\"contract.core.prelude.primitive-values.v1\""));
        assert!(json.contains("\"surface.std.collections.v1\""));
        assert!(json.contains("\"galaxy_records\":[{"));
        assert!(json.contains("\"galaxy_hidden_manual_only_library_modules_count\":1"));
        assert!(json.contains(
            "\"galaxy_hidden_manual_only_library_modules\":[\"ns-nova:lib/nova_contracts.ns\"]"
        ));
        assert!(json.contains("manual-only galaxy library modules"));
        assert!(json.contains("nuis project-imports --apply-suggested <project-dir>"));
        assert!(json.contains("galaxy_imports = [...]"));
        assert!(json.contains("ns-nova:lib/nova_contracts.ns"));
    }

    #[test]
    fn project_imports_json_reports_hidden_manual_only_library_modules() {
        let project_root = write_temp_project_fixture(
            "imports_manual_only_hint",
            r#"
name = "imports_manual_only_hint"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );

        let json = render_project_imports_json(&project_root).expect("render imports json");

        assert!(json.contains("\"source_kind\":\"project\""));
        assert!(json.contains("\"project\":\"imports_manual_only_hint\""));
        assert!(json.contains("\"explicit_galaxy_imports_count\":0"));
        assert!(json.contains("\"visible_library_modules_count\":2"));
        assert!(json.contains("\"hidden_manual_only_library_modules_count\":1"));
        assert!(json.contains(
            "\"hidden_manual_only_library_modules\":[\"ns-nova:lib/nova_contracts.ns\"]"
        ));
        assert!(json.contains("\"suggested_galaxy_imports_count\":1"));
        assert!(json.contains("\"suggested_galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
        assert!(json.contains(
            "\"suggested_manifest_snippet\":\"galaxy_imports = [\\\"ns-nova:lib/nova_contracts.ns\\\"]\""
        ));
        assert!(json.contains("\"library_records\":[{"));
        assert!(json.contains("\"import_policy\":\"manual-only\""));
        assert!(json.contains("\"visible\":false"));
        assert!(json.contains("\"explicit\":false"));
    }

    #[test]
    fn project_imports_json_reports_explicit_manual_only_library_as_visible() {
        let project_root = write_temp_project_fixture(
            "imports_explicit_manual_only",
            r#"
name = "imports_explicit_manual_only"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]
"#
            .trim_start(),
            r#"
use cpu NovaContracts;

mod cpu Main {
  fn main() -> i64 {
    return NovaContracts.runtime_score(16, 4, 3, 2, 9, 1);
  }
}
"#,
        );

        let json = render_project_imports_json(&project_root).expect("render imports json");

        assert!(json.contains("\"explicit_galaxy_imports_count\":1"));
        assert!(json.contains("\"explicit_galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
        assert!(json.contains("\"hidden_manual_only_library_modules_count\":0"));
        assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
        assert!(json.contains("\"visible\":true"));
        assert!(json.contains("\"explicit\":true"));
        assert!(json.contains("\"source_kind\":\"galaxy-explicit-import\""));
    }

    #[test]
    fn apply_suggested_project_imports_adds_manifest_field_when_missing() {
        let project_root = write_temp_project_fixture(
            "imports_apply_missing_field",
            r#"
name = "imports_apply_missing_field"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );

        let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
        assert_eq!(
            applied.applied,
            vec!["ns-nova:lib/nova_contracts.ns".to_owned()]
        );
        assert_eq!(applied.total_explicit_galaxy_imports, 1);
        assert!(applied.manifest_updated);

        let manifest = fs::read_to_string(project_root.join("nuis.toml")).expect("read manifest");
        assert!(manifest.contains("galaxy_imports = ["));
        assert!(manifest.contains("\"ns-nova:lib/nova_contracts.ns\""));

        let json = render_project_imports_json(&project_root).expect("render imports json");
        assert!(json.contains("\"explicit_galaxy_imports_count\":1"));
        assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
    }

    #[test]
    fn apply_suggested_project_imports_preserves_existing_entries_and_appends_new_ones() {
        let project_root = write_temp_project_fixture(
            "imports_apply_append",
            r#"
name = "imports_apply_append"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["pixelmagic=workspace", "ns-nova=workspace"]
galaxy_imports = [
  "pixelmagic:lib/image_contracts.ns",
]
"#
            .trim_start(),
            r#"
use cpu PixelMagicContracts;

mod cpu Main {
  fn main() -> i64 {
    return PixelMagicContracts.blur_op_kind();
  }
}
"#,
        );

        let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
        assert_eq!(
            applied.applied,
            vec!["ns-nova:lib/nova_contracts.ns".to_owned()]
        );
        assert_eq!(applied.total_explicit_galaxy_imports, 2);
        assert!(applied.manifest_updated);

        let manifest = fs::read_to_string(project_root.join("nuis.toml")).expect("read manifest");
        assert!(manifest.contains("\"pixelmagic:lib/image_contracts.ns\""));
        assert!(manifest.contains("\"ns-nova:lib/nova_contracts.ns\""));
        assert!(manifest.contains("galaxy_imports = ["));

        let pixelmagic_pos = manifest
            .find("\"pixelmagic:lib/image_contracts.ns\"")
            .expect("pixelmagic import present");
        let ns_nova_pos = manifest
            .find("\"ns-nova:lib/nova_contracts.ns\"")
            .expect("ns-nova import present");
        assert!(pixelmagic_pos < ns_nova_pos);
    }

    #[test]
    fn project_imports_apply_json_reports_mutation_result() {
        let project_root = write_temp_project_fixture(
            "imports_apply_json",
            r#"
name = "imports_apply_json"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );

        let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
        let json = render_project_imports_apply_json(&project_root, &applied)
            .expect("render imports apply json");

        assert!(json.contains("\"kind\":\"project_imports_apply\""));
        assert!(json.contains("\"action\":\"apply_suggested\""));
        assert!(json.contains("\"manifest_updated\":true"));
        assert!(json.contains("\"applied_galaxy_imports_count\":1"));
        assert!(json.contains("\"applied_galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
        assert!(json.contains("\"total_explicit_galaxy_imports\":1"));
        assert!(json.contains("\"explicit_galaxy_imports_count\":1"));
        assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
    }

    #[test]
    fn project_imports_apply_json_reports_noop_when_manifest_already_complete() {
        let project_root = write_temp_project_fixture(
            "imports_apply_json_noop",
            r#"
name = "imports_apply_json_noop"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]
"#
            .trim_start(),
            r#"
use cpu NovaContracts;

mod cpu Main {
  fn main() -> i64 {
    return NovaContracts.runtime_score(16, 4, 3, 2, 9, 1);
  }
}
"#,
        );

        let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
        assert!(!applied.manifest_updated);
        let json = render_project_imports_apply_json(&project_root, &applied)
            .expect("render imports apply json");

        assert!(json.contains("\"manifest_updated\":false"));
        assert!(json.contains("\"applied_galaxy_imports_count\":0"));
        assert!(json.contains("\"applied_galaxy_imports\":[]"));
        assert!(json.contains("\"total_explicit_galaxy_imports\":1"));
        assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
    }

    #[test]
    fn project_status_json_reports_link_plan_for_built_output() {
        let project_root = write_temp_project_fixture(
            "status_json_built_smoke",
            r#"
name = "status_json_built_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 5;
  }
}
"#,
        );
        let output_dir = default_build_output_dir(&project_root);

        handle_build(project_root.clone(), output_dir.clone(), false, None, None)
            .expect("build passes");

        let json = render_project_status_json(&project_root).expect("render status json");

        assert!(json.contains(&format!(
            "\"artifact_output_dir\":\"{}\"",
            output_dir.display()
        )));
        assert!(json.contains("\"artifact_ready_to_run\":true"));
        assert!(json.contains("\"link_plan_available\":true"));
        assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
        assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
        assert!(json.contains("\"link_plan_final_link_mode\":\"host-toolchain-finalize\""));
        assert!(json.contains("\"link_plan_domain_units\":"));
    }

    #[test]
    fn scheduler_view_json_reports_project_domains_and_frontdoor() {
        let project_root = write_temp_project_fixture(
            "scheduler_project_smoke",
            r#"
name = "scheduler_project_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 5;
  }
}
"#,
        );

        let json =
            render_scheduler_view_json(&project_root).expect("render scheduler project json");

        assert!(json.contains("\"source_kind\":\"project\""));
        assert!(json.contains("\"project\":\"scheduler_project_smoke\""));
        assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
        assert!(json.contains("\"abi_mode\":\"explicit\""));
        assert!(json.contains("\"project_plan\":\""));
        assert!(json.contains("\"project_plan_output_count\":"));
        assert!(json.contains("\"domains\":["));
        assert!(json.contains("\"abi_selection\":{"));
        assert!(json.contains("\"domain\":\"cpu\""));
        assert!(json.contains("\"abi\":\"cpu.arm64.apple_aapcs64\""));
    }

    #[test]
    fn scheduler_view_json_reports_single_file_domain_surface() {
        let input = repo_root().join("stdlib/core/basic_scalars.ns");
        let json = with_repo_root_cwd(|| {
            render_scheduler_view_json(&input).expect("render scheduler single-file json")
        });

        assert!(json.contains("\"source_kind\":\"single-file\""));
        assert!(json.contains("\"ast_domain\":\"cpu\""));
        assert!(json.contains("\"ast_unit\":\"Main\""));
        assert!(json.contains("\"workflow_kind\":\"compile_workflow\""));
        assert!(json.contains("\"recommended_next_step\":\"check\""));
        assert!(json.contains("\"domains\":["));
        assert!(json.contains("\"registration\":{"));
        assert!(json.contains("\"abi_selection\":null"));
    }

    #[test]
    fn language_test_runner_tracks_ignored_and_should_fail() {
        let dir = temp_dir("language_test_flags");
        let input = dir.join("flags.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test(ignored=true) fn skipped_case() -> i64 {
    return 1;
  }

  test(should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }

  test(should_fail=true) fn unexpected_pass() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("language tests should run");
        assert_eq!(report.collected, 2);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_run_ignored_tests() {
        let dir = temp_dir("language_test_run_ignored");
        let input = dir.join("ignored.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test(ignored=true) fn skipped_case() -> i64 {
    return 1;
  }

  test(should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, true, false, false)
            .expect("ignored language tests should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_include_ignored_tests() {
        let dir = temp_dir("language_test_include_ignored");
        let input = dir.join("include_ignored.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test(ignored=true) fn skipped_case() -> i64 {
    return 1;
  }

  test(should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }

  test(should_fail=true) fn unexpected_pass() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, true, false)
            .expect("all language tests should run");
        assert_eq!(report.collected, 3);
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_filter_exactly() {
        let dir = temp_dir("language_test_exact");
        let input = dir.join("exact.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test("smoke_add") fn smoke_add_impl() -> i64 {
    return 1;
  }

  test() fn smoke_add_extra() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(
            &input,
            Some("smoke_add"),
            false,
            false,
            false,
            true,
        )
        .expect("exact filter should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_filter_ignored_tests_exactly() {
        let dir = temp_dir("language_test_exact_ignored");
        let input = dir.join("exact_ignored.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test("smoke_skip", ignored=true) fn skipped_impl() -> i64 {
    return 1;
  }

  test(ignored=true) fn skipped_extra() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(
            &input,
            Some("smoke_skip"),
            false,
            true,
            false,
            true,
        )
        .expect("exact ignored filter should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_accepts_should_fail_reason() {
        let dir = temp_dir("language_test_reason");
        let input = dir.join("reason.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test("expected_failure", should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("reason-bearing language tests should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_benchmark_runner_executes_sync_benchmark() {
        let dir = temp_dir("language_benchmark_sync");
        let input = dir.join("bench_sync.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  benchmark("sum_loop", warmup_iters=1, measure_iters=2) fn sum_loop() -> i64 {
    return 7;
  }
}
"#,
        )
        .expect("write benchmark file");

        let report = run_language_benchmarks_for_source_file(&input, None, false, false)
            .expect("language benchmarks should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.completed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.timed_out, 0);
    }

    #[test]
    fn language_benchmark_runner_supports_bool_return() {
        let dir = temp_dir("language_benchmark_bool");
        let input = dir.join("bench_bool.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  benchmark("bool_case", measure_iters=1) fn bool_case() -> bool {
    return true;
  }
}
"#,
        )
        .expect("write bool benchmark file");

        let report = run_language_benchmarks_for_source_file(&input, None, false, false)
            .expect("bool benchmark should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.completed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.timed_out, 0);
    }

    #[test]
    fn language_benchmark_runner_can_filter_exactly() {
        let dir = temp_dir("language_benchmark_exact");
        let input = dir.join("bench_exact.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  benchmark("sum_loop", measure_iters=1) fn sum_loop_impl() -> i64 {
    return 1;
  }

  benchmark(measure_iters=1) fn sum_loop_extra() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write benchmark file");

        let report = run_language_benchmarks_for_source_file(&input, Some("sum_loop"), false, true)
            .expect("exact benchmark filter should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.completed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.timed_out, 0);
    }

    #[test]
    fn language_benchmark_runner_times_out_end_to_end() {
        let dir = temp_dir("language_benchmark_timeout");
        let input = dir.join("bench_timeout.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  extern "c" fn usleep(usec: i64) -> i32;

  benchmark("slow_async", measure_iters=1, timeout_ms=25) async fn slow_async() -> i64 {
    let _slept: i32 = usleep(100000);
    return 1;
  }
}
"#,
        )
        .expect("write timeout benchmark file");

        let report = run_language_benchmarks_for_source_file(&input, None, false, false)
            .expect("timeout benchmark should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.completed, 0);
        assert_eq!(report.failed, 0);
        assert_eq!(report.timed_out, 1);
    }

    #[test]
    fn benchmark_report_json_includes_machine_readable_measurements() {
        let dir = temp_dir("language_benchmark_json");
        let input = dir.join("bench_json.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  benchmark("sum_loop", warmup_iters=1, measure_iters=1) fn sum_loop() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
        )
        .expect("write benchmark file");

        let report = super::collect_language_benchmark_run_report(&input, None, false, false)
            .expect("json benchmark report should collect");
        let json = benchmark_run_report_json(&input, "single-file", false, false, None, &report);

        assert!(json.contains("\"kind\":\"nuis_benchmark_run\""));
        assert!(json.contains("\"source_kind\":\"single-file\""));
        assert!(json.contains("\"result\":\"passed\""));
        assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
        assert!(json.contains("\"label\":\"sum_loop\""));
        assert!(json.contains("\"status\":\"OK\""));
        assert!(json.contains("\"min_ns\":"));
        assert!(json.contains("\"avg_ns\":"));
        assert!(json.contains("\"max_ns\":"));
        assert!(json.contains("\"total_ns\":"));
    }

    #[test]
    fn benchmark_report_json_includes_project_text_handle_rewrite_summary() {
        let project_root = write_temp_project_fixture(
            "benchmark_project_json",
            r#"
name = "benchmark_project_json"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  benchmark("sum_loop", warmup_iters=1, measure_iters=1) fn sum_loop() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }

  fn main() -> i64 {
    return 0;
  }
}
"#,
        );

        let report =
            super::collect_language_benchmark_run_report(&project_root, None, false, false)
                .expect("project benchmark report should collect");
        let json = benchmark_run_report_json(&project_root, "project", false, false, None, &report);

        assert!(json.contains("\"kind\":\"nuis_benchmark_run\""));
        assert!(json.contains("\"source_kind\":\"project\""));
        assert!(json.contains("\"result\":\"passed\""));
        assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
        assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
        assert!(json.contains("\"label\":\"sum_loop\""));
        assert!(json.contains("\"status\":\"OK\""));
    }

    #[test]
    fn timeout_helper_marks_child_as_timed_out() {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep child");
        let outcome = wait_for_test_child(
            &mut child,
            Some(10),
            Some(nuis_semantics::model::TestClockDomain::Monotonic),
        )
        .expect("timeout helper should work");
        match outcome {
            RawTestOutcome::TimedOut(timeout_ms) => assert_eq!(timeout_ms, 10),
            RawTestOutcome::Completed(status) => {
                panic!("expected timeout, child exited with {:?}", status.code())
            }
        }
    }

    #[test]
    fn timeout_helper_supports_wall_clock_domain() {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep child");
        let outcome = wait_for_test_child(
            &mut child,
            Some(10),
            Some(nuis_semantics::model::TestClockDomain::Wall),
        )
        .expect("wall-clock timeout helper should work");
        match outcome {
            RawTestOutcome::TimedOut(timeout_ms) => assert_eq!(timeout_ms, 10),
            RawTestOutcome::Completed(status) => {
                panic!("expected timeout, child exited with {:?}", status.code())
            }
        }
    }

    #[test]
    fn language_test_runner_times_out_end_to_end() {
        let dir = temp_dir("language_test_timeout");
        let input = dir.join("timeout.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  extern "c" fn usleep(usec: i64) -> i32;

  test("slow_async", timeout_ms=25) async fn slow_async() -> i64 {
    let _slept: i32 = usleep(100000);
    return 1;
  }
}
"#,
        )
        .expect("write timeout test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("timeout language test should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn timeout_helper_supports_global_clock_domain() {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep child");
        let outcome = wait_for_test_child(
            &mut child,
            Some(10),
            Some(nuis_semantics::model::TestClockDomain::Global),
        )
        .expect("global-clock timeout helper should work");
        match outcome {
            RawTestOutcome::TimedOut(timeout_ms) => assert_eq!(timeout_ms, 10),
            RawTestOutcome::Completed(status) => {
                panic!("expected timeout, child exited with {:?}", status.code())
            }
        }
    }

    #[test]
    fn resolves_global_clock_domain_to_monotonic_runner_clock() {
        let resolved =
            resolve_runner_clock_domain(Some(nuis_semantics::model::TestClockDomain::Global));
        assert_eq!(
            resolved.domain,
            nuis_semantics::model::TestClockDomain::Monotonic
        );
        assert_eq!(
            resolved.bridge,
            nuis_semantics::model::NirHostTimingBridge::GlobalToMonotonicTickBridge
        );
        assert_eq!(
            resolved.bridge.host_surface(),
            nuis_semantics::model::NirHostReadSurface::ClockTick
        );
        assert_eq!(resolved.source, "host_monotonic_deadline");
    }

    #[test]
    fn resolves_wall_clock_domain_to_wall_runner_clock_source() {
        let resolved =
            resolve_runner_clock_domain(Some(nuis_semantics::model::TestClockDomain::Wall));
        assert_eq!(
            resolved.domain,
            nuis_semantics::model::TestClockDomain::Wall
        );
        assert_eq!(
            resolved.bridge,
            nuis_semantics::model::NirHostTimingBridge::WallDeadline
        );
        assert_eq!(
            resolved.bridge.host_surface(),
            nuis_semantics::model::NirHostReadSurface::ClockTick
        );
        assert_eq!(resolved.source, "host_wall_deadline");
    }

    #[test]
    fn resolves_default_clock_domain_to_monotonic_tick_bridge() {
        let resolved = resolve_runner_clock_domain(None);
        assert_eq!(
            resolved.domain,
            nuis_semantics::model::TestClockDomain::Monotonic
        );
        assert_eq!(
            resolved.bridge,
            nuis_semantics::model::NirHostTimingBridge::MonotonicTick
        );
        assert_eq!(
            resolved.bridge.host_surface(),
            nuis_semantics::model::NirHostReadSurface::ClockTick
        );
        assert_eq!(resolved.source, "host_monotonic_deadline");
    }

    #[test]
    fn scheduler_view_domain_record_json_exposes_registration_section() {
        let record = scheduler_view_domain_record("network", None, None)
            .expect("expected network scheduler registration record");
        let json = scheduler_view_domain_record_json(&record);

        assert!(json.contains("\"abi_selection\":null"));
        assert!(json.contains("\"registration\":{"));
        assert!(json.contains("\"manifest_path\":\""));
        assert!(json.contains("network.toml"));
        assert!(json.contains("\"entry_crate\":"));
        assert!(json.contains("\"ast_entry\":"));
        assert!(json.contains("\"nir_entry\":"));
        assert!(json.contains("\"yir_lowering_entry\":"));
        assert!(json.contains("\"part_verify_entry\":"));
        assert!(json.contains("\"resource_families\":["));
        assert!(json.contains("\"unit_types\":["));
        assert!(json.contains("\"lowering_targets\":["));
        assert!(json.contains("\"ops\":["));
    }

    #[test]
    fn scheduler_view_domain_record_json_exposes_shared_abi_selection_section() {
        let record = scheduler_view_domain_record(
            "network",
            None,
            Some("network.socket.macos.arm64.v1".to_owned()),
        )
        .expect("expected network scheduler registration record");
        let json = scheduler_view_domain_record_json(&record);

        assert!(json.contains("\"abi_selection\":{"));
        assert!(json.contains("\"domain\":\"network\""));
        assert!(json.contains("\"abi\":\"network.socket.macos.arm64.v1\""));
        assert!(json.contains("\"abi_target_machine\":\"arm64-darwin\""));
        assert!(json.contains("\"abi_target_host_adaptive\":false"));
    }

    #[test]
    fn project_domain_registry_checks_report_registered_abis() {
        let project = nuisc::project::load_project(
            &repo_root().join("examples/projects/domains/net_session_recipe_demo"),
        )
        .expect("load project");
        let plan = nuisc::project::build_project_compilation_plan(&project).expect("build plan");
        let checks = nuisc::registry::validate_project_domain_registry(&plan);
        assert!(!checks.is_empty());
        assert!(checks.iter().all(|check| check.ok));
        assert!(checks.iter().any(|check| check.domain == "network"));
        let network = checks
            .iter()
            .find(|check| check.domain == "network")
            .unwrap();
        assert!(network.abi_registered);
        assert!(network.issues.is_empty());
        assert_eq!(
            network.contract_schema.as_deref(),
            Some(nuisc::registry::NUSTAR_DOMAIN_CONTRACT_SCHEMA)
        );
        let json = project_domain_registry_checks_json(&checks).join(",");
        assert!(json.contains("\"issues\":[]"));
        assert!(json.contains("\"abi_registered\":true"));
    }

    #[test]
    fn project_abi_checks_report_recommended_abis() {
        let project = nuisc::project::load_project(
            &repo_root().join("examples/projects/domains/net_session_recipe_demo"),
        )
        .expect("load project");
        let plan = nuisc::project::build_project_compilation_plan(&project).expect("build plan");
        let checks =
            nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)
                .expect("abi checks");
        assert!(!checks.is_empty());
        assert!(checks.iter().all(|check| check.ok));
        assert!(checks.iter().any(|check| check.source == "recommended"));
        let json = project_abi_checks_json(&checks).join(",");
        assert!(json.contains("\"source\":\"recommended\""));
        assert!(json.contains("\"abi_registered\":true"));
        assert!(json.contains("\"issues\":[]"));
    }

    #[test]
    fn upsert_abi_block_appends_sorted_block_when_missing() {
        let source = "[package]\nname = \"demo\"\n";
        let requirements = vec![
            nuisc::project::ProjectAbiRequirement {
                domain: "shader".to_owned(),
                abi: "shader.metal.msl2_4".to_owned(),
            },
            nuisc::project::ProjectAbiRequirement {
                domain: "cpu".to_owned(),
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
            },
        ];

        let updated = upsert_abi_block(source, &requirements);

        assert_eq!(
            updated,
            "[package]\nname = \"demo\"\n\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n  \"shader=shader.metal.msl2_4\",\n]\n"
        );
    }

    #[test]
    fn upsert_abi_block_replaces_existing_block_with_normalized_sorted_entries() {
        let source = "[package]\nname = \"demo\"\nabi = [\n  \"shader=shader.cpu-fallback.v1\",\n]\nversion = \"0.1.0\"\n";
        let requirements = vec![
            nuisc::project::ProjectAbiRequirement {
                domain: "network".to_owned(),
                abi: "network.socket.macos.arm64.v1".to_owned(),
            },
            nuisc::project::ProjectAbiRequirement {
                domain: "cpu".to_owned(),
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
            },
        ];

        let updated = upsert_abi_block(source, &requirements);

        assert_eq!(
            updated,
            "[package]\nname = \"demo\"\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n  \"network=network.socket.macos.arm64.v1\",\n]\nversion = \"0.1.0\"\n"
        );
    }

    #[test]
    fn upsert_abi_block_is_idempotent_for_matching_normalized_block() {
        let source = "[package]\nname = \"demo\"\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n  \"network=network.socket.macos.arm64.v1\",\n]\n";
        let requirements = vec![
            nuisc::project::ProjectAbiRequirement {
                domain: "network".to_owned(),
                abi: "network.socket.macos.arm64.v1".to_owned(),
            },
            nuisc::project::ProjectAbiRequirement {
                domain: "cpu".to_owned(),
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
            },
        ];

        let updated = upsert_abi_block(source, &requirements);

        assert_eq!(updated, source);
    }

    #[test]
    fn find_abi_block_span_stops_at_closing_bracket_before_following_fields() {
        let source = "[package]\nname = \"demo\"\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n]\nsummary = \"kept\"\n";

        let (start, end) = find_abi_block_span(source).expect("abi block span");

        assert_eq!(
            &source[start..end],
            "abi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n]\n"
        );
        assert_eq!(&source[end..], "summary = \"kept\"\n");
    }

    #[test]
    fn language_test_runner_prints_clock_policy_metadata() {
        let dir = temp_dir("language_test_clock_policy");
        let input = dir.join("clock_policy.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  extern "c" fn usleep(usec: i64) -> i32;

  test("slow_global", should_fail=true, reason="bridge policy demo", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_global() -> i64 {
    let _slept: i32 = usleep(100000);
    return 1;
  }
}
"#,
        )
        .expect("write clock policy test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("clock policy language test should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }
}
