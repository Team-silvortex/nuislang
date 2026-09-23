use super::*;

#[test]
fn counted_returns_run_through_ordinary_entry_and_typed_helpers() {
    let source = "mod cpu Main {
        struct Pair { value: i64, index: i64 }
        @noinline fn choose(flag: bool, limit: i64) -> bool {
            let index = 0;
            while index < limit { if index == 1 { return flag; } let index = index + 1; }
            return false;
        }
        @noinline fn pair(seed: i64, limit: i64) -> Pair {
            let index = 0;
            while index < limit {
                let index = index + 2;
                if choose(true, index) { return Pair { value: seed + index, index: index }; }
            }
            return Pair { value: seed, index: -1 };
        }
        fn main() -> i64 {
            let index = 0;
            while index < 5 {
                let p = pair(10, 4);
                if index == 1 { return p.value + p.index; }
                let index = index + 1;
            }
            return 999 / (index - index);
        }
    }";
    assert_eq!(
        compile_and_run("counted_return_entry", source).code(),
        Some(14)
    );
    let zero = source
        .replace("while index < 5", "while index < 0")
        .replace("999 / (index - index)", "pair(8, 0).value");
    assert_eq!(
        compile_and_run("counted_return_zero", &zero).code(),
        Some(8)
    );
}

#[test]
fn counted_returns_skip_suffixes_but_preserve_selected_return_failure() {
    let source = "mod cpu Main {
        @noinline fn work(divisor: i64) -> i64 {
            let outer = 0;
            while outer < 3 {
                let outer = outer + 1;
                let inner = 0;
                while inner < 4 {
                    if inner == 1 { return (outer * 10 + inner) / divisor; }
                    let inner = inner + 1;
                }
                let invalid = 10 / (outer - outer);
            }
            return -1;
        }
        fn main() -> i64 { return work(1); }
    }";
    assert_eq!(
        compile_and_run("counted_child_return", source).code(),
        Some(11)
    );
    let status = compile_and_run(
        "counted_return_failure",
        &source.replace("work(1)", "work(0)"),
    );
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}
