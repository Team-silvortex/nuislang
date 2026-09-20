use super::*;

const SOURCE: &str = r#"mod cpu Main {
  @noinline fn checked(value: i64, divisor: i64) -> i64 { return value / divisor; }
  @noinline fn calculate(limit: i64, divisor: i64) -> i64 {
    let index: i64 = 0;
    let total: i64 = 0;
    while index < limit {
      let index: i64 = index + 1;
      let gate = divisor != 0;
      let saved = gate;
      if gate {
        let gate: bool = false;
        let total: i64 = total + index;
      } else {
        let gate = true;
        let total: i64 = total + 1;
      }
      let gate = gate == false;
      let gate = gate && checked(index, divisor) > 0;
      if gate { let total: i64 = total + 1; }
      if saved {
        let local = false;
        if gate { let local: bool = true; }
        if local { let total: i64 = total + 2; }
      }
    }
    return total;
  }
  fn main() -> i64 { return calculate(4, 2) + calculate(4, 0) + calculate(0, 0); }
}"#;

#[test]
fn bool_rebinding_snapshots_and_joins_run_through_default_native_entry() {
    let status = compile_and_run("bool_rebinding_entry", SOURCE);
    assert_eq!(status.code(), Some(23));
}

#[test]
fn overwritten_bool_result_keeps_native_arithmetic_failure() {
    let source = SOURCE.replace(
        "let gate = gate && checked(index, divisor) > 0;",
        "let gate = checked(index, divisor) > 0; let gate = false;",
    );
    let status = compile_and_run("overwritten_bool_result", &source);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}
