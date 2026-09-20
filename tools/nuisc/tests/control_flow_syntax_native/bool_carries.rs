use super::*;

#[test]
fn outer_bool_carry_keeps_zero_trip_seed_and_pre_loop_snapshot() {
    let source = "mod cpu Main {
        fn word(value: bool) -> i64 { if value { return 1; } return 0; }
        @noinline fn work(seed: bool, limit: i64) -> i64 {
            let flag = seed;
            let saved = flag;
            let index: i64 = 0;
            while index < limit {
                let index: i64 = index + 1;
                let flag = flag == false;
            }
            return word(flag) + word(saved) * 2 + index * 4;
        }
        fn main() -> i64 {
            return work(true, 0) + work(false, 0) + work(true, 3) + work(false, 4);
        }
    }";
    assert_eq!(
        compile_and_run("outer_bool_carry_entry", source).code(),
        Some(33)
    );
}

#[test]
fn outer_bool_and_flat_carries_keep_ordered_branch_updates() {
    let source = "mod cpu Main {
        struct Pair { first: i64, second: i64 }
        fn word(value: bool) -> i64 { if value { return 1; } return 0; }
        @noinline fn work(seed: bool, limit: i64) -> i64 {
            let flag = seed;
            let saved = flag;
            let other = false;
            let pair = Pair { first: 0, second: 0 };
            let index: i64 = 0;
            while index < limit {
                let index: i64 = index + 1;
                let flag = flag;
                let before = flag;
                if flag { let flag = false; } else { let flag = true; }
                let other = flag;
                if other { let flag = before; }
                let pair = Pair { first: pair.first + word(other), second: pair.second + word(flag) };
            }
            return pair.first + pair.second * 2 + word(flag) * 4 + word(saved) * 8;
        }
        fn main() -> i64 { return work(true, 3) + work(false, 3) + work(true, 0); }
    }";
    assert_eq!(
        compile_and_run("outer_bool_flat_entry", source).code(),
        Some(25)
    );
}
