use super::*;

#[test]
fn leading_step_break_preserves_advanced_index_and_zero_trip_seed() {
    let source = "mod cpu Main {
        @noinline fn work(initial: i64, limit: i64, stride: i64) -> i64 {
            let index = initial;
            while index < limit { let index = index + stride; break; }
            let __nuis_advanced_index_0 = true;
            return index;
        }
        fn main() -> i64 { return work(1, 10, 3) + work(5, 0, 0); }
    }";
    assert_eq!(
        compile_and_run("leading_step_break", source).code(),
        Some(9)
    );
}

#[test]
fn leading_step_nested_exits_keep_parent_suffix_and_continue_scope() {
    let source = "mod cpu Main {
        @noinline fn work() -> i64 {
            let outer = 0;
            let total = 0;
            let child = 0;
            while outer < 3 {
                let outer = outer + 1;
                let child = 0;
                while child < 5 {
                    let child = child + 1;
                    if child == 1 { continue; }
                    let total = total + child;
                    if child == 3 { break; }
                    let total = total + 10;
                }
                let total = total + child;
                if outer == 2 { continue; }
                let total = total + 100;
            }
            return total - 200 + outer;
        }
        fn main() -> i64 { return work(); }
    }";
    assert_eq!(
        compile_and_run("nested_value_exits", source).code(),
        Some(57)
    );
    let entry = source
        .replace("@noinline fn work()", "fn main()")
        .replace("fn main() -> i64 { return work(); }", "");
    assert_eq!(
        compile_and_run("nested_value_exit_entry", &entry).code(),
        Some(57)
    );
}
