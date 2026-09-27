use super::*;

fn source() -> String {
    typed_scoped_record_inputs::fixture::source(64, false)
        .replace("f63: i64", "f63: bool")
        .replace("f63: seed + 63", "f63: seed > 0")
        .replace("f63: value.f63 + 1", "f63: !value.f63")
}

#[test]
fn typed_mixed_scoped_record_carries_keep_counted_return_source_timing() {
    let source = "mod cpu Main {
        struct State { count: i64, enabled: bool }
        @noinline fn relay(value: State) -> State {
            return State { enabled: !value.enabled, count: value.count + 1 };
        }
        fn start(seed: i64) -> State { return State { count: seed, enabled: seed > 0 }; }
        fn step(state: State) -> State {
            let carry = state; let i = 0;
            while i < 3 {
                let i = i + 1; let carry = relay(carry);
                if i == 2 { return carry; }
            }
            return State { count: state.count / 0, enabled: false };
        }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    let cases = [-3_i64, 0, 5]
        .into_iter()
        .map(|seed| {
            let initial = vec![seed as u64, u64::from(seed > 0)];
            let event = vec![(seed + 2) as u64, initial[1]];
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check_flattened(source, cases);
}

#[test]
fn typed_mixed_scoped_record_carries_keep_multiple_record_and_scalar_slots_distinct() {
    let source = "mod cpu Main {
        struct State { count: i64, checksum: i64, first: bool, second: bool }
        @noinline fn relay(value: State) -> State {
            return State { second: !value.second, first: !value.first,
                checksum: value.checksum + 2, count: value.count + 1 };
        }
        fn start(seed: i64) -> State {
            return State { count: seed, checksum: seed + 7, first: seed > 0, second: seed < 0 };
        }
        fn step(state: State) -> State {
            let first = state; let second = relay(state); let flag = true; let i = 0;
            while i < 3 {
                let i = i + 1; let first = relay(first); let second = relay(second); let flag = !flag;
            }
            return State { count: first.count + second.count, checksum: first.checksum + second.checksum,
                first: first.first || flag, second: second.second || flag };
        }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    let cases = [-3_i64, 0, 5]
        .into_iter()
        .map(|seed| {
            let initial = vec![
                seed as u64,
                (seed + 7) as u64,
                u64::from(seed > 0),
                u64::from(seed < 0),
            ];
            let event = vec![
                (2 * seed + 7) as u64,
                (2 * seed + 28) as u64,
                1 - initial[2],
                initial[3],
            ];
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check_flattened(source, cases);
}

#[test]
fn typed_mixed_scoped_record_carries_compose_nested_iterations_and_immutable_inputs() {
    let source = "mod cpu Main {
        struct State { count: i64, enabled: bool }
        @noinline fn relay(value: State) -> State {
            return State { enabled: !value.enabled, count: value.count + 1 };
        }
        fn start(seed: i64) -> State { return State { count: seed, enabled: seed > 0 }; }
        fn step(state: State) -> State {
            let saved = state; let carry = state; let i = 0;
            while i < 2 {
                let i = i + 1;
                let j = 0;
                while j < 3 {
                    let j = j + 1;
                    let carry = relay(carry);
                }
                let marker = relay(saved);
                let carry = State { count: carry.count + marker.count, enabled: carry.enabled };
            }
            return carry;
        }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    let cases = [-3_i64, 0, 5]
        .into_iter()
        .map(|seed| {
            let initial = vec![seed as u64, u64::from(seed > 0)];
            let event = vec![(seed + 6 + 2 * (seed + 1)) as u64, initial[1]];
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check_flattened(source, cases);
}

#[test]
fn typed_mixed_scoped_record_carries_keep_checked_failures_and_work_limits_atomic() {
    typed_record_guards::check_limits(
        source(),
        &[
            (-2, 0, 64, false),
            (4, 64, 64, false),
            (2, 64, 64, true),
            (4, 64, 1, true),
            (4, 2, 64, true),
        ],
    );
}

#[test]
fn typed_mixed_scoped_record_carries_preserve_breaks_multiple_flags_and_snapshots() {
    let source = typed_scoped_record_inputs::fixture::source(4, true)
        .replace("f2: i64", "f2: bool")
        .replace("f3: i64", "f3: bool")
        .replace("f2: seed + 2", "f2: seed > 0")
        .replace("f3: seed + 3", "f3: seed < 0")
        .replace("f2: value.f2 + 1", "f2: !value.f2")
        .replace("f3: value.f3 + 1", "f3: !value.f3")
        .replace("f2: carry.f2 + 2", "f2: !carry.f2")
        .replace("f3: carry.f3 + 2", "f3: !carry.f3")
        .replace(
            "let carry = state;",
            "let carry = state; let saved = state;",
        )
        .replace(
            "return carry;",
            "return State { f3: carry.f3, f2: carry.f2, f1: saved.f1, f0: carry.f0 + saved.f0 };",
        );
    let cases = [-3_i64, 0, 1, 5]
        .into_iter()
        .map(|seed| {
            let initial = vec![
                seed as u64,
                (seed + 1) as u64,
                u64::from(seed > 0),
                u64::from(seed < 0),
            ];
            let trips = (seed + 1).clamp(0, 3);
            let delta = trips.min(2) + 2 * (trips - 2).max(0);
            let event = vec![
                (2 * seed + delta) as u64,
                (seed + 1) as u64,
                initial[2] ^ (trips % 2) as u64,
                initial[3] ^ (trips % 2) as u64,
            ];
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check_flattened(&source, cases);
}

#[test]
fn typed_mixed_scoped_record_carries_preserve_boolean_backedges_and_zero_trips() {
    let cases = [-3_i64, 0, 1, 5]
        .into_iter()
        .map(|seed| {
            let mut initial = (0..63).map(|i| (seed + i) as u64).collect::<Vec<_>>();
            initial.push(u64::from(seed > 0));
            let trips = (seed + 1).max(0);
            let mut event = initial.clone();
            for (index, word) in event[..63].iter_mut().enumerate() {
                *word = (*word as i64
                    + trips
                    + if index == 0 {
                        trips * (trips + 1) / 2
                    } else {
                        0
                    }) as u64;
            }
            event[63] ^= (trips % 2) as u64;
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check(&source(), cases);
}
