use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{TestClockDomain, TestClockPolicy};

#[test]
fn parses_benchmark_call_syntax_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          benchmark("sum_loop", warmup_iters=10, measure_iters=100, timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn sum_loop() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "sum_loop")
        .unwrap();
    assert_eq!(function.benchmark_name.as_deref(), Some("sum_loop"));
    assert_eq!(function.benchmark_warmup_iters, Some(10));
    assert_eq!(function.benchmark_measure_iters, Some(100));
    assert_eq!(function.benchmark_timeout_ms, Some(25));
    assert_eq!(function.benchmark_clock_domain, Some(TestClockDomain::Global));
    assert_eq!(function.benchmark_clock_policy, Some(TestClockPolicy::Bridge));
}

#[test]
fn lowers_at_benchmark_function_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          @benchmark("sum_loop", warmup_iters=10, measure_iters=100, timeout_ms=25, clock_domain="monotonic")
          fn sum_loop() -> i64 {
            return 1;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "sum_loop")
        .unwrap();
    assert_eq!(function.benchmark_name.as_deref(), Some("sum_loop"));
    assert_eq!(function.benchmark_warmup_iters, Some(10));
    assert_eq!(function.benchmark_measure_iters, Some(100));
    assert_eq!(function.benchmark_timeout_ms, Some(25));
    assert_eq!(
        function.benchmark_clock_domain,
        Some(TestClockDomain::Monotonic)
    );
    assert!(function
        .annotations
        .iter()
        .any(|annotation| annotation.name == "benchmark"));
}

#[test]
fn rejects_mixing_benchmark_declaration_styles() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @benchmark
          benchmark() fn sum_loop() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot use both `benchmark(...)` and `@benchmark(...)`"));
}

#[test]
fn rejects_mixing_test_and_benchmark_metadata() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke") benchmark("sum_loop") fn mixed() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot be both a test and a benchmark"));
}

#[test]
fn rejects_benchmark_function_with_parameters() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          benchmark("sum_loop") fn sum_loop(value: i64) -> i64 {
            return value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot take parameters"));
}

#[test]
fn rejects_benchmark_function_with_unsupported_return_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          benchmark("sum_loop") fn sum_loop() -> String {
            return "nope";
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("must return integer or float scalar"));
}

#[test]
fn rejects_non_positive_benchmark_measure_iters() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          benchmark("sum_loop", measure_iters=0) fn sum_loop() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("must use `measure_iters > 0`"));
}

#[test]
fn rejects_negative_benchmark_warmup_iters() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          benchmark("sum_loop", warmup_iters=-1, measure_iters=1) fn sum_loop() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("must use `warmup_iters >= 0`"));
}

#[test]
fn rejects_benchmark_clock_policy_without_global_domain() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          benchmark("sum_loop", measure_iters=10, timeout_ms=25, clock_domain="monotonic", clock_policy="bridge") async fn sum_loop() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains(
        "can only use `clock_policy=\"bridge\"` together with `clock_domain=\"global\"`"
    ));
}
