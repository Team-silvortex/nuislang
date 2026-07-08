use super::*;

pub(super) fn execute_language_test(
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
    let harness_ast = harness::build_test_harness_module(ast, test_function);
    let artifacts = nuisc::pipeline::compile_ast(harness_ast)?;
    let output_dir = harness::temp_test_output_dir(
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

pub(super) fn execute_language_benchmark(
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

pub(crate) enum RawTestOutcome {
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
    let harness_ast = harness::build_benchmark_harness_module(ast, benchmark_function, iterations)?;
    let artifacts = nuisc::pipeline::compile_ast(harness_ast)?;
    let output_dir = harness::temp_test_output_dir(label);
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

pub(crate) fn wait_for_test_child(
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

pub(crate) fn resolve_runner_clock_domain(
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
