use super::*;

#[test]
fn literal_nested_loops_keep_persistent_and_iteration_local_induction() {
    let source = "mod cpu Main {
        struct Pair { sum: i64, visits: i64 }
        fn word(value: bool) -> i64 { if value { return 1; } return 0; }
        @noinline fn work(rounds: i64, reset: bool) -> i64 {
            let index: i64 = 0;
            let child: i64 = 0;
            let flag = false;
            let saved = flag;
            let pair = Pair { sum: 0, visits: 0 };
            while index < rounds {
                let index = index + 1;
                if reset { let child = 0; }
                while child < index {
                    let child = child + 1;
                    let flag = flag == false;
                    let pair = Pair { sum: pair.sum + child + word(flag), visits: pair.visits + 1 };
                }
            }
            return pair.sum + pair.visits + child + word(saved);
        }
        fn main() -> i64 { return work(3, true) + work(3, false) + work(0, true); }
    }";
    assert_eq!(
        compile_and_run("literal_nested_loops", source).code(),
        Some(36)
    );
}

#[test]
fn literal_child_predicates_remain_scoped_even_for_a_single_scalar_update() {
    let source = "mod cpu Main {
        @noinline fn work(selected: bool) -> i64 {
            let index: i64 = 0;
            let total: i64 = 0;
            while index < 3 {
                let index = index + 1;
                let child: i64 = 0;
                while child < index {
                    let child = child + 1;
                    if selected { let total = total + child; }
                }
            }
            return total;
        }
        fn main() -> i64 { return work(true) + work(false); }
    }";
    assert_eq!(
        compile_and_run("literal_child_predicate", source).code(),
        Some(10)
    );
}
