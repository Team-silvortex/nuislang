use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{TestClockDomain, TestClockPolicy};

#[test]
fn lowers_at_test_function_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          @test("smoke_add", should_fail=true, reason="must reject zero", timeout_ms=25, clock_domain="monotonic")
          fn smoke_add() -> i64 {
            return 0;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "smoke_add")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
    assert!(function.test_should_fail);
    assert_eq!(function.test_reason.as_deref(), Some("must reject zero"));
    assert_eq!(function.test_timeout_ms, Some(25));
    assert_eq!(function.test_clock_domain, Some(TestClockDomain::Monotonic));
    assert!(function
        .annotations
        .iter()
        .any(|annotation| annotation.name == "test"));
}

#[test]
fn rejects_mixing_test_declaration_styles() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @test
          test() fn smoke_add() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot use both `test(...)` and `@test(...)`"));
}

#[test]
fn accepts_bool_and_i64_test_functions() {
    parse_nuis_module(
        r#"
        mod cpu Main {
          test() fn smoke_bool() -> bool {
            return true;
          }

          test() async fn smoke_i64() -> i64 {
            return 1;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn rejects_test_function_with_parameters() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test() fn smoke(value: i64) -> i64 {
            return value;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot take parameters"));
}

#[test]
fn rejects_test_function_with_unsupported_return_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test() fn smoke() -> String {
            return "nope";
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("must return `bool` or integer scalar"));
}

#[test]
fn rejects_test_function_with_conflicting_modifiers() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test(ignored=true, should_fail=true) fn smoke() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot be both `ignored` and `should_fail`"));
}

#[test]
fn rejects_test_reason_without_should_fail() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke", reason="must reject zero") fn smoke() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("can only use `reason=\"...\"` together with `should_fail=true`"));
}

#[test]
fn rejects_non_positive_test_timeout() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke", timeout_ms=0) fn smoke() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("must use `timeout_ms` > 0"));
}

#[test]
fn rejects_test_clock_domain_without_timeout() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke", clock_domain="wall") fn smoke() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("can only use `clock_domain=\"...\"` together with `timeout_ms=...`"));
}

#[test]
fn rejects_unknown_test_clock_domain() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke", timeout_ms=25, clock_domain="gpu_global") fn smoke() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("unsupported `clock_domain=\"gpu_global\"`"));
}

#[test]
fn rejects_wall_clock_domain_on_async_tests() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("slow_async", timeout_ms=25, clock_domain="wall") async fn slow_async() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot use `clock_domain=\"wall\"` on `async fn`"));
}

#[test]
fn rejects_test_clock_policy_without_timeout() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("slow_global", clock_policy="bridge") async fn slow_global() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("can only use `clock_policy=\"...\"` together with `timeout_ms=...`"));
}

#[test]
fn rejects_test_clock_policy_without_global_domain() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test("slow_mono", timeout_ms=25, clock_domain="monotonic", clock_policy="bridge") async fn slow_mono() -> i64 {
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

#[test]
fn rejects_test_function_outside_cpu_domain() {
    let error = parse_nuis_module(
        r#"
        mod shader SurfaceShader {
          test() fn smoke() -> i64 {
            return 1;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("only supported in `mod cpu`"));
}

#[test]
fn rejects_legacy_test_prefix_syntax() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          test fn smoke() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("test declarations now require `test(...) fn ...`"));
}

#[test]
fn lowers_test_function_modifiers_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          test(ignored=true, should_fail=true) fn smoke_add() -> i64 {
            return 0;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        module.contains("cannot be both `ignored` and `should_fail`"),
        "unexpected error: {module}"
    );
}

#[test]
fn lowers_test_function_call_syntax_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke_add", reason="kept for docs") fn smoke_add() -> i64 {
            return 1;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(module.contains("can only use `reason=\"...\"` together with `should_fail=true`"));
}

#[test]
fn lowers_test_function_reason_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke_add", should_fail=true, reason="must reject zero", timeout_ms=25, clock_domain="monotonic") fn smoke_add() -> i64 {
            return 0;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "smoke_add")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
    assert!(!function.test_ignored);
    assert!(function.test_should_fail);
    assert_eq!(function.test_reason.as_deref(), Some("must reject zero"));
    assert_eq!(function.test_timeout_ms, Some(25));
    assert_eq!(function.test_clock_domain, Some(TestClockDomain::Monotonic));
    assert_eq!(function.test_clock_policy, None);
}

#[test]
fn parses_test_clock_policy_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          test("slow_global", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_global() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "slow_global")
        .unwrap();
    assert_eq!(function.test_clock_domain, Some(TestClockDomain::Global));
    assert_eq!(function.test_clock_policy, Some(TestClockPolicy::Bridge));
}

#[test]
fn lowers_test_clock_policy_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          test("slow_global", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_global() -> i64 {
            return 1;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "slow_global")
        .unwrap();
    assert_eq!(function.test_clock_domain, Some(TestClockDomain::Global));
    assert_eq!(function.test_clock_policy, Some(TestClockPolicy::Bridge));
    assert!(function
        .annotations
        .iter()
        .any(|annotation| annotation.name == "test"));
}
