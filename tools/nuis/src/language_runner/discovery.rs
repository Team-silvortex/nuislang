use super::*;

pub(crate) fn run_language_tests_for_source_file(
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
                write!(line, " [reason: {}]", reason).unwrap();
            }
            if let Some(timeout_ms) = function.test_timeout_ms {
                write!(line, " [timeout_ms: {}]", timeout_ms).unwrap();
            }
            if let Some(clock_domain) = &function.test_clock_domain {
                write!(line, " [clock_domain: {}]", clock_domain.as_str()).unwrap();
            }
            if let Some(clock_policy) = &function.test_clock_policy {
                write!(line, " [clock_policy: {}]", clock_policy.as_str()).unwrap();
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
        let verdict =
            process::execute_language_test(path, &ast, function, ignored_only || include_ignored)?;
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
                    write!(line, " [reason={}]", reason).unwrap();
                }
                if let Some(clock_domain) = verdict.resolved_clock_domain {
                    write!(line, " [clock={}]", clock_domain).unwrap();
                }
                if let Some(clock_policy) = verdict.clock_policy {
                    write!(line, " [policy={}]", clock_policy).unwrap();
                }
                if let Some(note) = &verdict.note {
                    write!(line, " [note={}]", note).unwrap();
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

pub(crate) fn run_language_benchmarks_for_source_file(
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
                write!(line, " [warmup_iters: {}]", record.warmup_iters).unwrap();
            }
            write!(line, " [measure_iters: {}]", record.measure_iters).unwrap();
            if let Some(note) = &record.note {
                if record.status == "DISCOVERED" {
                    write!(line, " [note: {}]", note).unwrap();
                }
            }
            if let Some(clock_domain) = record.declared_clock_domain {
                write!(line, " [clock_domain: {}]", clock_domain).unwrap();
            }
            if let Some(clock_policy) = record.clock_policy {
                write!(line, " [clock_policy: {}]", clock_policy).unwrap();
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
                    write!(line, " [clock={}]", clock_domain).unwrap();
                }
                if let Some(clock_policy) = record.clock_policy {
                    write!(line, " [policy={}]", clock_policy).unwrap();
                }
                if let Some(note) = &record.note {
                    write!(line, " [note={}]", note).unwrap();
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

pub(super) fn collect_language_benchmarks_for_source_file(
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
    let benchmarks = ast
        .functions
        .iter()
        .filter(|function| function.benchmark_name.is_some())
        .collect::<Vec<_>>();
    let source_label = path.display().to_string();
    let mut matched = Vec::with_capacity(benchmarks.len());
    let mut discovered = Vec::with_capacity(benchmarks.len());
    for function in benchmarks {
        if !test_matches_filter(
            function.name.as_str(),
            function.benchmark_name.as_deref(),
            filter,
            exact,
        ) {
            continue;
        }
        discovered.push(discovered_benchmark_record(&source_label, function));
        matched.push(function);
    }
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
        let verdict = process::execute_language_benchmark(path, &ast, function)?;
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
            source: source_label.clone(),
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
        collected: completed + failed + timed_out,
        completed,
        failed,
        timed_out,
        text_handle_rewrite_helper_hits: text_handle_rewrite.helper_hits,
        text_handle_rewrite_local_hits: text_handle_rewrite.local_hits,
        records,
    })
}

fn discovered_benchmark_record(source: &str, function: &AstFunction) -> BenchmarkRunRecord {
    BenchmarkRunRecord {
        source: source.to_owned(),
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
    }
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
