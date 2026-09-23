use super::*;

#[test]
fn trailing_value_break_leaves_index_before_step_and_preserves_zero_trips() {
    let source = "mod cpu Main {
        @noinline fn work(initial: i64, limit: i64, stride: i64) -> i64 {
            let index = initial;
            while index < limit { break; let index = index + stride; }
            return index;
        }
        fn main() -> i64 { return work(1, 10, 3) + work(5, 0, 0); }
    }";
    assert_eq!(
        compile_and_run("trailing_value_break", source).code(),
        Some(6)
    );
}

#[test]
fn trailing_value_loops_mix_with_leading_child_exits_and_explicit_continue_steps() {
    let source = "mod cpu Main {
        @noinline fn work() -> i64 {
            let outer = 0;
            let total = 0;
            let child = 0;
            while outer < 4 {
                let child = 0;
                while child < 5 {
                    let child = child + 1;
                    if child == 1 { continue; }
                    let total = total + child;
                    if child == 3 { break; }
                }
                let total = total + child + outer;
                if outer == 2 { break; }
                if outer == 1 { let outer = outer + 1; continue; }
                let total = total + 10;
                let outer = outer + 1;
            }
            return total + outer;
        }
        fn main() -> i64 { return work(); }
    }";
    assert_eq!(
        compile_and_run("trailing_value_mixed", source).code(),
        Some(39)
    );
    let entry = source
        .replace("@noinline fn work()", "fn main()")
        .replace("fn main() -> i64 { return work(); }", "");
    assert_eq!(
        compile_and_run("trailing_value_entry", &entry).code(),
        Some(39)
    );
}
