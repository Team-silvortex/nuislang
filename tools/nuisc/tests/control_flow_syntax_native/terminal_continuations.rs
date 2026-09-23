use super::*;

#[test]
fn terminal_continuations_keep_typed_results_and_lazy_failures_in_ordinary_entry() {
    let source = "mod cpu Main {
        struct Pair { value: i64, marker: i64 }
        @noinline fn checked(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn pair(skip: bool, divisor: i64) -> Pair {
            if skip { return Pair { value: 7, marker: 11 }; }
            return Pair { marker: checked(4, divisor), value: checked(8, divisor) };
        }
        @noinline fn truth(skip: bool, divisor: i64) -> bool {
            if skip { return true; } return checked(8, divisor) > 1;
        }
        @noinline fn fallback(skip: bool, divisor: i64) -> i64 {
            if skip { return 13; } return checked(8, checked(2, divisor));
        }
        fn main() -> i64 {
            let saved = pair(true, 0);
            let computed = pair(false, 2);
            let flag = truth(true, 0);
            if flag {
                return saved.value + saved.marker + computed.value + computed.marker
                    + fallback(true, 0) + fallback(false, 2);
            }
            return 99;
        }
    }";
    assert_eq!(compile_and_run("terminal_values", source).code(), Some(45));
    for (from, to) in [
        ("pair(true, 0)", "pair(false, 0)"),
        ("fallback(false, 2)", "fallback(false, 4)"),
    ] {
        let status = compile_and_run("terminal_failure", &source.replace(from, to));
        assert!(!status.success());
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}

#[test]
fn hygienic_continuations_keep_sibling_constants_symbols_and_future_names() {
    let source = "mod cpu Main {
        struct Packet { result: i64, marker: i64 }
        @noinline fn result(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn choose(divisor: i64, inner: bool) -> i64 {
            let __nuis_scalar_local_0 = 5;
            if divisor == 0 { return 41; } else {
                if inner {
                    const result: i64 = 12;
                    let unused = result(result, divisor);
                    if result == divisor { return result; }
                } else {
                    const result: bool = true;
                    if result { let unused = result(4, divisor); }
                }
                let prefix = result(16, divisor);
            }
            let __nuis_scalar_local_1 = 3;
            let result = Packet { result: result(8, divisor), marker: __nuis_scalar_local_0 };
            return result.result + result.marker + __nuis_scalar_local_1;
        }
        fn main() -> i64 {
            return choose(0, true) + choose(2, true) + choose(0 - 2, false) + choose(12, true);
        }
    }";
    assert_eq!(compile_and_run("hygienic_scopes", source).code(), Some(69));
    for (from, to) in [
        ("result(16, divisor)", "result(16, divisor - 2)"),
        ("result(4, divisor)", "result(4, divisor + 2)"),
    ] {
        let status = compile_and_run("hygienic_failure", &source.replace(from, to));
        assert!(!status.success());
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}

#[test]
fn statement_continuations_preserve_scope_snapshots_and_unused_failures() {
    let source = "mod cpu Main {
        struct Pair { value: i64, marker: i64 }
        @noinline fn checked(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn scoped(divisor: i64) -> i64 {
            if divisor == 0 { return 13; } else {
                let next = Pair { value: checked(4, divisor), marker: 1 };
                let ignored = next.value + next.marker;
            }
            let next = checked(8, divisor); return next;
        }
        @noinline fn pair(skip: bool, divisor: i64, saved: Pair) -> Pair {
            if skip { return saved; }
            let unused = checked(4, divisor);
            let next = checked(8, divisor);
            return Pair { value: next, marker: saved.marker };
        }
        @noinline fn truth(skip: bool, divisor: i64) -> bool {
            if skip { return true; }
            let next = checked(8, divisor); return next > 0;
        }
        fn main() -> i64 {
            let saved = Pair { value: 7, marker: 11 };
            let first = pair(true, 0, saved);
            let second = pair(false, 2, saved);
            if truth(true, 0) && truth(false, 2) {
                return first.value + second.value + second.marker + saved.value
                    + scoped(0) + scoped(2);
            }
            return 99;
        }
    }";
    assert_eq!(compile_and_run("statement_values", source).code(), Some(46));
    for (from, to) in [
        ("pair(true, 0, saved)", "pair(false, 0, saved)"),
        (
            "let unused = checked(4, divisor);",
            "let unused = checked(4, divisor - 2);",
        ),
        ("truth(true, 0)", "truth(false, 0)"),
    ] {
        let status = compile_and_run("statement_failure", &source.replace(from, to));
        assert!(!status.success());
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}
