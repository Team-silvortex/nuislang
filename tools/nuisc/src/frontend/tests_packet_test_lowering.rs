use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{TestClockDomain, TestClockPolicy};

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
