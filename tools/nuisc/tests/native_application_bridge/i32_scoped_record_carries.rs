use super::*;

const SOURCE: &str = "mod cpu Main {
    struct State { value: i32, saved: i32, count: i64, enabled: bool }
    @noinline fn relay(value: State) -> State {
        return State { enabled: !value.enabled, count: value.count, saved: value.saved,
            value: value.value + i32_from_i64(2) };
    }
    fn start(seed: i64, limit: i64) -> State {
        return State { value: i32_from_i64(seed), saved: i32_from_i64(seed),
            count: limit, enabled: seed > 0 };
    }
    fn step(state: State) -> State {
        let saved = state.value; let carry = state; let i = 0; let limit = state.count;
        while i < limit { let i = i + 1; let carry = relay(carry); }
        return State { value: carry.value, saved: saved, count: carry.count, enabled: carry.enabled };
    }
    fn stop(state: State) -> State { return state; }
    fn main() -> i64 { return 0; }
}";

fn cases(trips: impl Fn(i64) -> i64) -> Vec<(Vec<u64>, Vec<Vec<u64>>)> {
    let mut cases = Vec::new();
    for seed in [i32::MIN as i64, -1, 0, i32::MAX as i64, 4294967295] {
        for limit in [0, 1, 3] {
            let narrow = seed as i32;
            let initial = vec![
                narrow as i64 as u64,
                narrow as i64 as u64,
                limit as u64,
                u64::from(seed > 0),
            ];
            let mut event = initial.clone();
            event[0] = narrow.wrapping_add(2 * trips(limit) as i32) as i64 as u64;
            event[3] ^= trips(limit) as u64 % 2;
            cases.push((
                vec![seed as u64, limit as u64],
                vec![initial, event.clone(), event],
            ));
        }
    }
    cases
}

#[test]
fn typed_i32_scoped_record_carries_preserve_signed_words_zero_trips_and_snapshots() {
    typed_record_inputs::check_flattened(SOURCE, cases(|limit| limit));
}

#[test]
fn typed_i32_scoped_record_carries_keep_counted_returns_lazy() {
    let source = SOURCE.replace("let carry = relay(carry);", "let carry = relay(carry); if i == 2 { return carry; }")
        .replace("return State { value: carry.value", "if limit > 1 { return State { value: i32_from_i64(1 / 0), saved: saved, count: limit, enabled: false }; } return State { value: carry.value");
    typed_record_inputs::check_flattened(&source, cases(|limit| limit.min(2)));
}

#[test]
fn typed_i32_scoped_record_carries_compose_nested_iterations() {
    let source = SOURCE.replace(
        "let carry = relay(carry);",
        "let j = 0; while j < 2 { let j = j + 1; let carry = relay(carry); }",
    );
    typed_record_inputs::check_flattened(&source, cases(|limit| 2 * limit));
}

#[test]
fn typed_i32_scoped_record_carries_keep_independent_scalar_slots() {
    let source = SOURCE
        .replace(
            "let saved = state.value;",
            "let saved = state.value; let scalar = state.value;",
        )
        .replace(
            "let carry = relay(carry);",
            "let carry = relay(carry); let scalar = scalar + i32_from_i64(2);",
        )
        .replace("value: carry.value", "value: scalar");
    typed_record_inputs::check_flattened(&source, cases(|limit| limit));
}

fn wide_source() -> String {
    typed_scoped_record_inputs::fixture::source(64, false)
        .replace("f62: i64", "f62: bool")
        .replace("f62: seed + 62", "f62: seed > 0")
        .replace("f62: value.f62 + 1", "f62: !value.f62")
        .replace("f63: i64", "f63: i32")
        .replace("f63: seed + 63", "f63: i32_from_i64(2147483646 + seed)")
        .replace("f63: value.f63 + 1", "f63: value.f63 + i32_from_i64(1)")
}

#[test]
fn typed_i32_scoped_record_carries_transport_wide_signed_backedges() {
    let cases = [-3_i64, 0, 1, 5]
        .into_iter()
        .map(|seed| {
            let mut initial = (0..62).map(|i| (seed + i) as u64).collect::<Vec<_>>();
            initial.push(u64::from(seed > 0));
            initial.push((2147483646_i64 + seed) as i32 as i64 as u64);
            let trips = (seed + 1).max(0);
            let mut event = initial.clone();
            for (index, word) in event[..62].iter_mut().enumerate() {
                *word = (*word as i64
                    + trips
                    + if index == 0 {
                        trips * (trips + 1) / 2
                    } else {
                        0
                    }) as u64;
            }
            event[62] ^= trips as u64 % 2;
            event[63] = (initial[63] as i32).wrapping_add(trips as i32) as i64 as u64;
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check(&wide_source(), cases);
}

#[test]
fn typed_i32_scoped_record_carries_keep_traps_and_work_limits_atomic() {
    typed_record_guards::check_limits(
        wide_source(),
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
fn typed_i32_scoped_record_carries_reject_wrong_cast_kinds() {
    let project = Project::with_source(SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for (from, to) in [
        ("cast_i32_to_i64", "cast_bool_to_i64"),
        ("cast_i64_to_i32", "cast_i64_to_bool"),
    ] {
        let mut module = compiled.yir.clone();
        let mut changed = 0;
        for node in &mut module.nodes {
            if node.op.instruction == from {
                node.op.instruction = to.into();
                changed += 1;
            }
        }
        assert!(changed > 0);
        assert!(emit_registered(&module, "counter").is_err(), "{from}");
    }
}
