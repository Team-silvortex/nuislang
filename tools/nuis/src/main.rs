mod cli;
mod galaxy;
mod json_surface;
mod surface_render;

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use nuis_semantics::model::{AstExpr, AstFunction, AstModule, AstStmt, AstTypeRef, AstVisibility};
use json_surface::workflow_contract_json_fields;

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
        cli::CommandKind::ArtifactDoctor { input, json } => handle_artifact_doctor(input, json)?,
        cli::CommandKind::VerifyBuildManifest { manifest } => {
            nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
                manifest,
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
        cli::CommandKind::RunArtifact { input } => handle_run_artifact(input)?,
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
        println!("release-check: abi");
        for check in &abi_checks {
            for line in nuisc::project::render_project_abi_selection_check_lines(check) {
                println!("  {}", line);
            }
        }
        if abi_checks.iter().any(|check| !check.ok) {
            return Err(
                "release-check aborted because one or more project domains failed ABI selection validation"
                    .to_owned(),
            );
        }
        println!("release-check: registry");
        for check in &registry_checks {
            for line in nuisc::registry::render_project_domain_registry_check_lines(check) {
                println!("  {}", line);
            }
        }
        if registry_checks.iter().any(|check| !check.ok) {
            return Err(
                "release-check aborted because one or more project domains failed registry validation"
                    .to_owned(),
            );
        }
        println!("release-check: lowering");
        for check in &lowering_checks {
            for line in nuisc::project::render_project_lowering_selection_lines(check) {
                println!("  {}", line);
            }
        }
        if lowering_checks.iter().any(|check| !check.ok) {
            return Err(
                "release-check aborted because one or more project domains failed lowering selection validation"
                    .to_owned(),
            );
        }
    }
    println!("release-check: check");
    nuisc::run(nuisc::CommandKind::Check {
        input: input.clone(),
    })?;
    println!("release-check: build");
    nuisc::run(nuisc::CommandKind::Compile {
        input: input.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi,
        target,
    })?;
    println!("release-check: verify-build-manifest");
    let manifest = output_dir.join("nuis.build.manifest.toml");
    nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
        manifest: manifest.clone(),
        json: false,
    })?;
    println!("release-check: ok");
    println!("  output_dir: {}", output_dir.display());
    println!("  manifest: {}", manifest.display());
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
        println!("test: checking project {}", project.manifest.name);
        handle_check(input.clone())?;
        let mut paths = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<BTreeSet<_>>();
        let mut collected = 0usize;
        if project.manifest.tests.is_empty() {
            println!("  no explicit tests declared");
        } else {
            println!("  declared tests: {}", project.manifest.tests.len());
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
        println!("  collected language tests: {}", collected);
        if list {
            println!("  listed language tests: {}", collected);
            return Ok(());
        }
        println!("  executed language tests: {}", passed + failed + skipped);
        println!("  passed: {}", passed);
        println!("  failed: {}", failed);
        println!("  skipped: {}", skipped);
        if failed > 0 {
            return Err(format!(
                "project test run failed: {failed} language test(s) failed"
            ));
        }
        if collected == 0 {
            println!("  result: project check passed");
        } else {
            println!("  result: all discovered language tests passed");
        }
        Ok(())
    } else {
        println!("test: {}", input.display());
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
            println!("  listed language tests: {}", report.collected);
            return Ok(());
        }
        if report.failed > 0 {
            return Err(format!(
                "test run failed: {} language test(s) failed",
                report.failed
            ));
        }
        println!("  result: passed");
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
        let report =
            collect_language_benchmark_run_report(&input, filter.as_deref(), list, exact)?;
        println!("{}", benchmark_run_report_json(&input, source_kind, list, exact, filter.as_deref(), &report));
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
        println!("bench: checking project {}", project.manifest.name);
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
        for path in paths {
            let report = run_language_benchmarks_for_source_file(
                &path,
                filter.as_deref(),
                list,
                exact,
            )?;
            collected += report.collected;
            completed += report.completed;
            failed += report.failed;
            timed_out += report.timed_out;
        }
        println!("  collected language benchmarks: {}", collected);
        if list {
            println!("  listed language benchmarks: {}", collected);
            return Ok(());
        }
        println!("  executed language benchmarks: {}", completed + failed + timed_out);
        println!("  completed: {}", completed);
        println!("  failed: {}", failed);
        println!("  timed_out: {}", timed_out);
        if failed > 0 || timed_out > 0 {
            return Err(format!(
                "project benchmark run failed: {failed} failed, {timed_out} timed out"
            ));
        }
        if collected == 0 {
            println!("  result: project check passed");
        } else {
            println!("  result: all discovered language benchmarks completed");
        }
        Ok(())
    } else {
        println!("bench: {}", input.display());
        let report = run_language_benchmarks_for_source_file(&input, filter.as_deref(), list, exact)?;
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
        let mut records = Vec::new();
        for path in paths {
            let report =
                collect_language_benchmarks_for_source_file(&path, filter, list_only, exact)?;
            collected += report.collected;
            completed += report.completed;
            failed += report.failed;
            timed_out += report.timed_out;
            records.extend(report.records);
        }
        Ok(LanguageBenchmarkRunReport {
            collected,
            completed,
            failed,
            timed_out,
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
    let record_json = report
        .records
        .iter()
        .map(benchmark_run_record_json)
        .collect::<Vec<_>>();
    let mut fields = vec![
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
        json_object_array_field("benchmarks", &record_json),
    ];
    fields.push(json_field(
        "result",
        if list_only {
            "listed"
        } else if report.failed > 0 || report.timed_out > 0 {
            "failed"
        } else {
            "passed"
        },
    ));
    format!("{{{}}}", fields.join(","))
}

fn benchmark_run_record_json(record: &BenchmarkRunRecord) -> String {
    let mut fields = vec![
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
        json_optional_i64_field("declared_clock_domain_code", record.declared_clock_domain_code),
        json_optional_string_field("resolved_clock_domain", record.resolved_clock_domain),
        json_optional_i64_field("resolved_clock_domain_code", record.resolved_clock_domain_code),
        json_optional_string_field("resolved_clock_source", record.resolved_clock_source),
    ];
    if let Some(measurement) = record.measurement {
        fields.push(json_field("run_mode", measurement.run_mode));
        fields.push(json_usize_field("sample_count", measurement.sample_count));
        fields.push(json_optional_u128_field("min_ns", measurement.min_ns));
        fields.push(json_u128_field("avg_ns", measurement.avg_ns));
        fields.push(json_optional_u128_field("max_ns", measurement.max_ns));
        fields.push(json_u128_field("total_ns", measurement.total_ns));
    } else {
        fields.push("\"run_mode\":null".to_owned());
        fields.push("\"sample_count\":null".to_owned());
        fields.push("\"min_ns\":null".to_owned());
        fields.push("\"avg_ns\":null".to_owned());
        fields.push("\"max_ns\":null".to_owned());
        fields.push("\"total_ns\":null".to_owned());
    }
    format!("{{{}}}", fields.join(","))
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
    if !matched.is_empty() {
        println!("  source: {}", path.display());
    }
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
        println!(
            "  {} {}",
            verdict.status,
            function.test_name.as_deref().unwrap_or(&function.name)
        );
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
        if verdict.status == "SKIP" {
            skipped += 1;
        } else if verdict.counted_pass {
            passed += 1;
        } else {
            failed += 1;
        }
    }
    println!("  executed language tests: {}", passed + failed + skipped);
    println!("  passed: {}", passed);
    println!("  failed: {}", failed);
    println!("  skipped: {}", skipped);
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
    let report = collect_language_benchmarks_for_source_file(path, filter, list_only, exact)?;
    if !report.records.is_empty() {
        println!("  source: {}", path.display());
    }
    println!("  collected language benchmarks: {}", report.collected);
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
    if list_only {
        return Ok(report);
    }
    for record in &report.records {
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
    }
    println!(
        "  executed language benchmarks: {}",
        report.completed + report.failed + report.timed_out
    );
    println!("  completed: {}", report.completed);
    println!("  failed: {}", report.failed);
    println!("  timed_out: {}", report.timed_out);
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
            clock_policy: function.benchmark_clock_policy.map(|policy| policy.as_str()),
            resolved_clock_bridge: None,
            resolved_clock_surface: None,
            declared_clock_domain: function.benchmark_clock_domain.map(|domain| domain.as_str()),
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
        records,
    })
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
                        declared_clock_domain_code: declared_clock_domain.map(|domain| domain.code()),
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
        RawBenchmarkOutcome::Completed { elapsed_ns, status } => {
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
            Some(BenchmarkMeasurement {
                run_mode: if warmup_iters > 0 {
                    "dual-process-loop"
                } else {
                    "single-process-loop"
                },
                sample_count: measure_iters,
                min_ns: None,
                max_ns: None,
                avg_ns: elapsed_ns / measure_iters as u128,
                total_ns: elapsed_ns,
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
    Completed { elapsed_ns: u128, status: ExitStatus },
    TimedOut { timeout_ms: i64 },
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
        .spawn()
        .map_err(|error| format!("failed to run `{binary_path}`: {error}"))?;
    let started = Instant::now();
    match wait_for_test_child(&mut child, timeout_ms, clock_domain)? {
        RawTestOutcome::Completed(status) => Ok(RawBenchmarkOutcome::Completed {
            elapsed_ns: started.elapsed().as_nanos(),
            status,
        }),
        RawTestOutcome::TimedOut(timeout_ms) => Ok(RawBenchmarkOutcome::TimedOut {
            timeout_ms,
        }),
    }
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
    harness
        .functions
        .push(build_benchmark_loop_function(benchmark_function));
    harness
        .functions
        .push(build_benchmark_main_function(benchmark_function, iterations));
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
        body: vec![AstStmt::Return(Some(return_expr))],
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

fn i64_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "i64".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
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
        output_dir,
        verbose_cache,
        cpu_abi,
        target,
    })?;
    Ok(())
}

fn resolve_run_artifact_binary_path(input: &Path) -> Result<PathBuf, String> {
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
        "run-artifact expected a binary path, `nuis.compiled.artifact`, or `nuis.build.manifest.toml`; missing `{}`",
        input.display()
    ))
}

fn handle_run_artifact(input: PathBuf) -> Result<(), String> {
    let binary = resolve_run_artifact_binary_path(&input)?;
    let status = Command::new(&binary)
        .status()
        .map_err(|error| format!("failed to run `{}`: {error}", binary.display()))?;
    println!("run-artifact: {}", binary.display());
    println!(
        "  exit_status: {}",
        status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_owned())
    );
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
    } else if input.file_name().and_then(|value| value.to_str()) == Some("nuis.compiled.artifact")
    {
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

    if let Some(path) = manifest_path.as_ref() {
        match nuisc::aot::verify_build_manifest(path) {
            Ok(report) => {
                manifest_verified = true;
                artifact_path = Some(PathBuf::from(&report.artifact_path));
                binary_path = Some(Path::new(&report.output_dir).join(&report.artifact_binary_name));
                output_dir = Some(PathBuf::from(&report.output_dir));
            }
            Err(error) => manifest_verify_error = Some(error),
        }
    }

    if let Some(path) = artifact_path.as_ref() {
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
            format!(
                "nuis verify-build-manifest {}",
                manifest_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<nuis.build.manifest.toml>".to_owned())
            ),
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
            format!(
                "nuis run-artifact {}",
                manifest_path
                    .as_ref()
                    .or(artifact_path.as_ref())
                    .or(binary_path.as_ref())
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<artifact-input>".to_owned())
            ),
            "the binary, manifest, and compiled artifact are all present and verified, so the next step is to launch the built output through the nuis frontdoor".to_owned(),
        )
    } else if manifest_exists || artifact_exists {
        (
            "inspect_artifact".to_owned(),
            format!(
                "nuis inspect-artifact {}",
                manifest_path
                    .as_ref()
                    .or(artifact_path.as_ref())
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<artifact-input>".to_owned())
            ),
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
    }
}

pub(crate) fn render_artifact_doctor_json(input: &Path) -> String {
    let report = probe_artifact_doctor(input);
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
    let fields = vec![
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
    ];
    format!("{{{}}}", fields.join(","))
}

fn handle_artifact_doctor(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        println!("{}", render_artifact_doctor_json(&input));
        return Ok(());
    }
    let report = probe_artifact_doctor(&input);
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
    println!("  ready_to_run: {}", report.ready_to_run);
    if let Some(error) = report.manifest_verify_error.as_deref() {
        println!("  manifest_verify_error: {}", error);
    }
    if let Some(error) = report.artifact_verify_error.as_deref() {
        println!("  artifact_verify_error: {}", error);
    }
    println!("  recommended_next_step: {}", report.recommended_next_step);
    println!("  recommended_command: {}", report.recommended_command);
    println!("  recommended_reason: {}", report.recommended_reason);
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
    "check=nuis check <input.ns>; test=nuis test <input.ns>; build=nuis build <input.ns> <output-dir>; artifact=nuis artifact-doctor <output-dir>; run=nuis run-artifact <output-dir|nuis.build.manifest.toml>; release=nuis release-check <input.ns> <output-dir>"
}

fn artifact_workflow_brief() -> &'static str {
    "build -> inspect_artifact -> verify_artifact -> artifact_doctor -> verify_build_manifest -> run_artifact"
}

fn artifact_doctor_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis artifact-doctor {}", output_dir.display())
}

fn run_artifact_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nuis run-artifact {}",
        output_dir.join("nuis.build.manifest.toml").display()
    )
}

fn load_link_plan_for_output_dir(output_dir: &Path) -> Option<nuisc::linker::LinkPlan> {
    let manifest = output_dir.join("nuis.build.manifest.toml");
    if !manifest.exists() {
        return None;
    }
    nuisc::linker::build_link_plan_from_manifest(&manifest).ok()
}

fn workflow_link_plan_json_fields(
    link_plan: Option<&nuisc::linker::LinkPlan>,
) -> Vec<String> {
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
    ]
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
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
        );
        let include_galaxy_flow =
            galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
        let output_dir = default_build_output_dir(input);
        let artifact_report = probe_artifact_doctor(&output_dir);
        let link_plan = load_link_plan_for_output_dir(&output_dir);
        let mut fields = vec![
            json_field("source_kind", frontdoor.source_kind),
            json_field("input", &input.display().to_string()),
            json_field("project", &project.manifest.name),
            json_field("root", &project.root.display().to_string()),
            json_field("entry", &project.manifest.entry),
            json_field("default_build_output_dir", &output_dir.display().to_string()),
            json_field(
                "default_release_output_dir",
                &default_release_check_output_dir(input).display().to_string(),
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
        ];
        fields.extend(workflow_link_plan_json_fields(link_plan.as_ref()));
        fields.extend(workflow_contract_json_fields(
            &frontdoor,
            true,
            true,
            include_galaxy_flow,
            true,
        ));
        return Ok(format!("{{{}}}", fields.join(",")));
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
    let link_plan = load_link_plan_for_output_dir(&output_dir);
    let mut fields = vec![
        json_field("source_kind", frontdoor.source_kind),
        json_field("input", &input.display().to_string()),
        json_field("single_source_compile_workflow", frontdoor.workflow_brief),
        json_field("single_source_compile_samples", frontdoor.workflow_samples),
        json_field("default_build_output_dir", &output_dir.display().to_string()),
        json_field(
            "default_release_output_dir",
            &default_release_check_output_dir(input).display().to_string(),
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
    ];
    fields.extend(workflow_link_plan_json_fields(link_plan.as_ref()));
    fields.extend(workflow_contract_json_fields(
        &frontdoor,
        false,
        false,
        false,
        true,
    ));
    Ok(format!("{{{}}}", fields.join(",")))
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
) -> WorkflowFrontdoorSurface {
    let recommendation = recommend_project_workflow_step(
        plan,
        declared_tests,
        missing_tests,
        galaxy_doctor,
        galaxy_check_invalid,
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
            workflow_samples: "workflow=nuis workflow [input]; doctor=nuis project-doctor [project-dir|nuis.toml]; check=nuis check [input]; test=nuis test [input]; build=nuis build [input] <output-dir>; artifact=nuis artifact-doctor <output-dir>; run=nuis run-artifact <output-dir|nuis.build.manifest.toml>; release=nuis release-check [input] [output-dir]",
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
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
        );
        let include_galaxy_flow =
            galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
        let output_dir = default_build_output_dir(&input);
        let artifact_report = probe_artifact_doctor(&output_dir);
        let link_plan = load_link_plan_for_output_dir(&output_dir);
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
        println!(
            "  default_build_output_dir: {}",
            output_dir.display()
        );
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
            "  artifact_recommended_next_step: {}",
            artifact_report.recommended_next_step
        );
        println!("  link_plan_available: {}", link_plan.is_some());
        println!(
            "  link_plan_final_stage: {}",
            link_plan
                .as_ref()
                .map(|plan| plan.final_stage.kind.as_str())
                .unwrap_or("<unavailable>")
        );
        println!(
            "  link_plan_final_driver: {}",
            link_plan
                .as_ref()
                .map(|plan| plan.final_stage.driver.as_str())
                .unwrap_or("<unavailable>")
        );
        println!(
            "  link_plan_final_link_mode: {}",
            link_plan
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
    let link_plan = load_link_plan_for_output_dir(&output_dir);
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
    println!(
        "  default_build_output_dir: {}",
        output_dir.display()
    );
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
        "  artifact_recommended_next_step: {}",
        artifact_report.recommended_next_step
    );
    println!("  link_plan_available: {}", link_plan.is_some());
    println!(
        "  link_plan_final_stage: {}",
        link_plan
            .as_ref()
            .map(|plan| plan.final_stage.kind.as_str())
            .unwrap_or("<unavailable>")
    );
    println!(
        "  link_plan_final_driver: {}",
        link_plan
            .as_ref()
            .map(|plan| plan.final_stage.driver.as_str())
            .unwrap_or("<unavailable>")
    );
    println!(
        "  link_plan_final_link_mode: {}",
        link_plan
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
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape_local(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", name, entries)
}

pub(crate) fn json_object_field(name: &str, fields: &[String]) -> String {
    format!("\"{}\":{{{}}}", name, fields.join(","))
}

fn append_json_object_fields(base_json: &str, fields: &[String]) -> String {
    let mut out = base_json.to_owned();
    if out.ends_with('}') {
        out.pop();
        if !fields.is_empty() {
            out.push(',');
            out.push_str(&fields.join(","));
        }
        out.push('}');
    }
    out
}

fn json_object_array_field(name: &str, values: &[String]) -> String {
    format!("\"{}\":[{}]", name, values.join(","))
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

pub(crate) fn project_workflow_json_fields(
    frontdoor: &WorkflowFrontdoorSurface,
    include_galaxy_flow: bool,
) -> Vec<String> {
    workflow_contract_json_fields(frontdoor, true, true, include_galaxy_flow, false)
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
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
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
    for line in surface_render::render_project_status_text_summary(&input)? {
        println!("{line}");
    }
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
    for line in surface_render::render_project_doctor_text_summary(&input)? {
        println!("{line}");
    }
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let nova_profile = galaxy::inspect_ns_nova_profile(&project.root)?;
    let abi_checks =
        nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)?;
    let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
    let lowering_checks =
        nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
    for check in &abi_checks {
        for line in nuisc::project::render_project_abi_selection_check_lines(check) {
            println!("  {}", line);
        }
    }
    for check in &registry_checks {
        for line in nuisc::registry::render_project_domain_registry_check_lines(check) {
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
        "    nuis run-artifact <binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>"
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
    println!("    nuis inspect-artifact [--json] <nuis.compiled.artifact|nuis.build.manifest.toml>");
    println!("    nuis verify-artifact [--json] <nuis.compiled.artifact>");
    println!("    nuis artifact-doctor [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>");
    println!("    nuis verify-build-manifest <nuis.build.manifest.toml>");
    println!();
    println!("  project workflow:");
    println!("    nuis project-doctor [--json] [project-dir|nuis.toml]");
    println!("    nuis project-status [--json] [project-dir|nuis.toml]");
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
    use crate::galaxy;
    use crate::json_surface::{
        galaxy_lock_json_fields, project_check_summary_json_fields,
        public_surface_summary_json_fields, workflow_contract_json_fields,
    };
    use super::{
        artifact_doctor_command_for_output_dir, artifact_workflow_brief,
        benchmark_run_report_json, build_workflow_frontdoor_surface, default_build_output_dir,
        handle_build, handle_check, handle_release_check, handle_run_artifact, handle_test,
        project_abi_checks_json, project_compile_workflow_source_profile,
        project_domain_registry_checks_json, project_workflow_json_fields,
        recommend_project_workflow_step, render_artifact_doctor_json,
        render_project_doctor_json, render_project_status_json, render_scheduler_view_json,
        render_workflow_json, resolve_runner_clock_domain, run_artifact_command_for_output_dir,
        run_language_benchmarks_for_source_file, run_language_tests_for_source_file,
        scheduler_view_domain_record, scheduler_view_domain_record_json,
        single_source_workflow_source_profile, wait_for_test_child, RawTestOutcome,
        WorkflowRecommendation,
        PublicSurfaceModuleRecord,
    };
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn repo_root() -> PathBuf {
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
                for module_dir in ["core", "std", "ns-nova"] {
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
        assert!(artifact_doctor_command_for_output_dir(&output_dir).contains("nuis artifact-doctor"));
        assert!(run_artifact_command_for_output_dir(&output_dir).contains("nuis run-artifact"));
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

        let manifest_report =
            nuisc::aot::verify_build_manifest(output_dir.join("nuis.build.manifest.toml").as_path())
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
  fn main() -> i64 {
    return 0;
  }
}
"#,
        );
        let output_dir = temp_dir("run_artifact_outputs");

        handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
        handle_run_artifact(output_dir.join("nuis.build.manifest.toml"))
            .expect("run-artifact passes");
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
  fn main() -> i64 {
    return 0;
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
        assert!(json.contains("\"ready_to_run\":true"));
        assert!(json.contains("\"recommended_next_step\":\"run_artifact\""));
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
        assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"link_plan_final_stage\":null"));
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
        assert!(json.contains("\"artifact_recommended_next_step\":\"run_artifact\""));
        assert!(json.contains("\"link_plan_available\":true"));
        assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
        assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
        assert!(json.contains("\"link_plan_final_link_mode\":\"host-toolchain-finalize\""));
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
        assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"link_plan_final_stage\":null"));
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
            recommend_project_workflow_step(&plan, &[], &[], &doctor, false);

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

        let recommendation =
            recommend_project_workflow_step(&plan, &missing_tests, &missing_tests, &doctor, false);

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
            recommend_project_workflow_step(&plan, &declared_tests, &[], &doctor, false);

        assert_eq!(recommendation.label, "check");
        assert_eq!(recommendation.command, "nuis check <project-dir|nuis.toml>");
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
            field == &format!(
                "\"project_compile_workflow\":\"{}\"",
                nuisc::project_compile_workflow_brief()
            )
        }));
        assert!(without_galaxy.iter().any(|field| {
            field == &format!(
                "\"project_test_workflow\":\"{}\"",
                nuisc::project_test_workflow_brief()
            )
        }));
        assert!(!without_galaxy
            .iter()
            .any(|field| field.contains("\"project_galaxy_workflow\"")));

        let with_galaxy = project_workflow_json_fields(&frontdoor, true);
        assert!(with_galaxy.iter().any(|field| {
            field == &format!(
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
"#
            .trim_start(),
            r#"
mod cpu Main {
  pub fn exported() -> i64 {
    return 3;
  }

  fn main() -> i64 {
    return exported();
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
        assert!(json.contains("\"recommended_next_step\":\"check\""));
        assert!(json.contains("\"artifact_output_dir\":\""));
        assert!(json.contains("\"artifact_ready_to_run\":false"));
        assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"link_plan_final_stage\":null"));
        assert!(json.contains("\"tests_declared\":1"));
        assert!(json.contains("\"public_surface_modules\":1"));
        assert!(json.contains("\"public_functions\":1"));
        assert!(json.contains("\"galaxy_lock_status\":\"missing\""));
        assert!(json.contains("\"tests\":[{"));
        assert!(json.contains("\"exists\":true"));
        assert!(json.contains("\"domains\":["));
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

        assert!(json.contains("\"source_kind\":\"project\""));
        assert!(json.contains("\"project\":\"doctor_json_smoke\""));
        assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
        assert!(json.contains("\"tests_declared\":1"));
        assert!(json.contains("\"tests_missing\":1"));
        assert!(json.contains("\"abi_checks_ok\":true"));
        assert!(json.contains("\"registry_checks_ok\":true"));
        assert!(json.contains("\"lowering_checks_ok\":true"));
        assert!(json.contains("\"artifact_output_dir\":\""));
        assert!(json.contains("\"artifact_ready_to_run\":false"));
        assert!(json.contains("\"link_plan_available\":false"));
        assert!(json.contains("\"galaxy_check_status\":\"skipped\""));
        assert!(json.contains("\"galaxy_lock_status\":\"missing\""));
        assert!(json.contains("\"next_steps\":["));
        assert!(json.contains("some declared project tests are missing on disk"));
        assert!(json.contains("\"tests\":[{"));
        assert!(json.contains("\"exists\":false"));
        assert!(json.contains("\"domains\":["));
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

        let json = render_scheduler_view_json(&project_root).expect("render scheduler project json");

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

        let report =
            run_language_benchmarks_for_source_file(&input, Some("sum_loop"), false, true)
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
  benchmark("sum_loop", warmup_iters=1, measure_iters=1) fn sum_loop() -> i64 {
    return 1;
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
        assert!(json.contains("\"label\":\"sum_loop\""));
        assert!(json.contains("\"status\":\"OK\""));
        assert!(json.contains("\"min_ns\":"));
        assert!(json.contains("\"avg_ns\":"));
        assert!(json.contains("\"max_ns\":"));
        assert!(json.contains("\"total_ns\":"));
    }

    #[test]
    fn timeout_helper_marks_child_as_timed_out() {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 1")
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
