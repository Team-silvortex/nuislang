use super::*;

#[path = "nested_record_fixture.rs"]
mod fixture;

const SOURCE: &str = "mod cpu Main {
    struct Leaf { value: f64, gain: f32, tag: i32, enabled: bool }
    struct State { left: Leaf, right: Leaf, count: i64 }
    @noinline fn relay(value: State) -> State {
        return State { right: value.left, count: value.count, left: value.right };
    }
    fn start(value: f64, gain: f32, tag: i32, limit: i64) -> State {
        return State {
            left: Leaf { value: value, gain: gain, tag: tag, enabled: false },
            right: Leaf { value: 1.5, gain: 2.5, tag: i32_from_i64(7), enabled: true },
            count: limit };
    }
    fn step(state: State) -> State {
        let carry = state; let i = 0; let limit = state.count;
        while i < limit { let i = i + 1; let carry = relay(carry); }
        return carry;
    }
    fn stop(state: State) -> State { return state; }
    fn main() -> i64 { return 0; }
}";

fn cases(trips: impl Fn(i64) -> i64) -> Vec<(Vec<u64>, Vec<Vec<u64>>)> {
    let mut cases = Vec::new();
    for (value, gain, tag) in [
        (0x8000_0000_0000_0000_u64, 0x8000_0000_u64, i32::MIN),
        (0xfff8_0000_0000_1234, 0x7fc0_5678, i32::MAX),
        (0x4000_0000_0000_0001, 0x4000_0001, -17),
    ] {
        for limit in [0_i64, 1, 2, 3] {
            let initial = vec![
                value,
                gain,
                tag as i64 as u64,
                0,
                1.5_f64.to_bits(),
                u64::from(2.5_f32.to_bits()),
                7,
                1,
                limit as u64,
            ];
            let mut event = initial.clone();
            if trips(limit) % 2 != 0 {
                for slot in 0..4 {
                    event.swap(slot, slot + 4);
                }
            }
            cases.push((
                vec![value, gain, tag as i64 as u64, limit as u64],
                vec![initial, event.clone(), event],
            ));
        }
    }
    cases
}

#[test]
fn typed_nested_scoped_record_carries_preserve_paths_and_all_scalar_bits() {
    typed_record_inputs::check_flattened(SOURCE, cases(|limit| limit));
}

#[test]
fn typed_nested_scoped_record_carries_compose_nested_loops_and_snapshots() {
    let source = SOURCE.replace(
        "let carry = relay(carry);",
        "let carry = carry; let saved = carry; let j = 0;
         while j < 3 { let j = j + 1; let carry = relay(carry); }
         if i == 2 { let carry = saved; }",
    );
    typed_record_inputs::check_flattened(&source, cases(|limit| limit - i64::from(limit >= 2)));
}

#[test]
fn typed_nested_scoped_record_carries_keep_early_returns_and_dead_traps_lazy() {
    let source = SOURCE.replace(
        "let carry = relay(carry);",
        "let carry = relay(carry);
         if i == 2 { return carry; }
         if i > 2 { let trap = 1 / (i - i); }",
    );
    typed_record_inputs::check_flattened(&source, cases(|limit| limit.min(2)));
}

#[test]
fn typed_nested_i64_record_carries_do_not_alias_equal_leaf_names() {
    let source = "mod cpu Main {
        struct Leaf { value: i64 }
        struct Branch { left: Leaf, right: Leaf }
        struct State { branch: Branch, limit: i64 }
        @noinline fn relay(value: State) -> State {
            return State { limit: value.limit,
                branch: Branch { right: value.branch.left, left: value.branch.right } };
        }
        fn start(limit: i64) -> State {
            return State { branch: Branch { left: Leaf { value: 7 }, right: Leaf { value: -9 } }, limit: limit };
        }
        fn step(state: State) -> State {
            let carry = state; let i = 0; let limit = state.limit;
            while i < limit { let i = i + 1; let carry = relay(carry); }
            return carry;
        }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    let cases = [0_u64, 1, 2, 3]
        .into_iter()
        .map(|limit| {
            let initial = vec![7, (-9_i64) as u64, limit];
            let mut event = initial.clone();
            if limit % 2 != 0 {
                event.swap(0, 1);
            }
            (vec![limit], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check_flattened(source, cases);
}

#[test]
fn typed_nested_record_carries_keep_wide_private_words_allocation_free() {
    let cases = [-3_i64, 0, 5]
        .into_iter()
        .map(|seed| {
            let mut initial = (0..64).map(|i| (seed + i) as u64).collect::<Vec<_>>();
            initial[60] = (2147483646_i64 + seed) as i32 as i64 as u64;
            initial[61] = u64::from(1.5_f32.to_bits());
            initial[62] = (-1.5_f64).to_bits();
            initial[63] = u64::from(seed > 0);
            let mut event = initial.clone();
            let trips = (seed + 1).max(0);
            for (index, word) in event.iter_mut().enumerate() {
                *word = match index {
                    60 => (*word as i32).wrapping_add(trips as i32) as i64 as u64,
                    61 => u64::from((1.5_f32 + trips as f32 * 0.5).to_bits()),
                    62 => (-1.5_f64 + trips as f64 * 0.5).to_bits(),
                    63 => *word ^ (trips as u64 % 2),
                    _ => (*word as i64 + trips) as u64,
                };
            }
            event[0] = (event[0] as i64 + trips * (trips + 1) / 2) as u64;
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check(&fixture::source(), cases);
}

#[test]
fn typed_nested_record_carries_keep_selected_traps_and_shared_budgets_atomic() {
    let source = fixture::source().replace("value.left.f0 + 1", "10 / (value.left.f0 - 2)");
    assert!(source.contains("10 / (value.left.f0 - 2)"));
    typed_record_guards::check_prepared(
        &source,
        &[
            (-2, 64, 64, false, false),
            (4, 64, 64, false, false),
            (2, 64, 64, true, true),
            (4, 64, 1, false, true),
            (4, 1, 64, false, true),
        ],
    );
}
