use std::{
    collections::BTreeSet,
    fmt::Write as _,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use nuis_semantics::model::{
    AstExpr, AstExternFunction, AstFunction, AstModule, AstParam, AstStmt, AstTypeRef,
    AstVisibility,
};

use crate::{
    handle_check, json_bool_field, json_field, json_object_array_field, json_optional_i64_field,
    json_optional_string_field, json_optional_u128_field, json_u128_field, json_usize_field,
    success_logs_enabled,
};

mod discovery;
mod harness;
mod process;

pub(crate) use discovery::{
    run_language_benchmarks_for_source_file, run_language_tests_for_source_file,
};
#[cfg(test)]
pub(crate) use process::{resolve_runner_clock_domain, wait_for_test_child, RawTestOutcome};

pub(crate) fn handle_test(
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

pub(crate) fn handle_bench(
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

pub(crate) fn collect_language_benchmark_run_report(
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
            let report = discovery::collect_language_benchmarks_for_source_file(
                &path, filter, list_only, exact,
            )?;
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
        let report = discovery::collect_language_benchmarks_for_source_file(
            input, filter, list_only, exact,
        )?;
        if report.collected == 0 {
            let resolved = nuisc::pipeline::resolve_compile_input(input)?;
            let _ = resolved.compile()?;
        }
        Ok(report)
    }
}

pub(crate) struct LanguageTestRunReport {
    pub(crate) collected: usize,
    pub(crate) passed: usize,
    pub(crate) failed: usize,
    pub(crate) skipped: usize,
}

pub(crate) struct LanguageBenchmarkRunReport {
    pub(crate) collected: usize,
    pub(crate) completed: usize,
    pub(crate) failed: usize,
    pub(crate) timed_out: usize,
    pub(crate) text_handle_rewrite_helper_hits: usize,
    pub(crate) text_handle_rewrite_local_hits: usize,
    records: Vec<BenchmarkRunRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BenchmarkRunRecord {
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

pub(crate) fn benchmark_run_report_json(
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
pub(crate) struct BenchmarkMeasurement {
    run_mode: &'static str,
    sample_count: usize,
    min_ns: Option<u128>,
    max_ns: Option<u128>,
    avg_ns: u128,
    total_ns: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunnerClockResolution {
    pub(crate) domain: nuis_semantics::model::TestClockDomain,
    pub(crate) bridge: nuis_semantics::model::NirHostTimingBridge,
    pub(crate) source: &'static str,
}
