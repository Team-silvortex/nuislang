use super::*;

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

    let report =
        run_language_tests_for_source_file(&input, Some("smoke_add"), false, false, false, true)
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

    let report =
        run_language_tests_for_source_file(&input, Some("smoke_skip"), false, true, false, true)
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

    let report = super::collect_language_benchmark_run_report(&project_root, None, false, false)
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
    let resolved = resolve_runner_clock_domain(Some(nuis_semantics::model::TestClockDomain::Wall));
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
