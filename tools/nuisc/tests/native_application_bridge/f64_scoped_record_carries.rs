use super::*;

const SOURCE: &str = "mod cpu Main {
    struct State { value: f64, saved: f64, count: i64, enabled: bool }
    @noinline fn relay(value: State) -> State {
        return State { enabled: !value.enabled, count: value.count,
            saved: value.value, value: value.saved };
    }
    fn start(seed: f64, other: f64, limit: i64) -> State {
        return State { value: seed, saved: other, count: limit, enabled: false };
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
    // Compare words, not floats: NaNs are unequal and signed zeros compare equal.
    for (seed, other) in [
        (0x8000_0000_0000_0000_u64, 0),
        (0x7ff8_0000_0000_1234, 0xfff8_0000_0000_5678),
        (0x7ff0_0000_0000_0001, 0xfff0_0000_0000_0001),
        (1, 0x800f_ffff_ffff_ffff),
        (0x7ff0_0000_0000_0000, 0xfff0_0000_0000_0000),
        (0x3ff8_0000_0000_0000, 0xc004_0000_0000_0000),
        (0x4000_0000_0000_0000, 0x4000_0000_0000_0001),
        (0xffff_ffff_ffff_ffff, 0x7fff_ffff_ffff_ffff),
    ] {
        for limit in [0, 1, 2, 3] {
            let initial = vec![seed, other, limit as u64, 0];
            let mut event = initial.clone();
            if trips(limit) % 2 != 0 {
                event.swap(0, 1);
                event[3] = 1;
            }
            cases.push((
                vec![seed, other, limit as u64],
                vec![initial, event.clone(), event],
            ));
        }
    }
    cases
}

#[test]
fn typed_f64_scoped_record_carries_preserve_all_seed_and_backedge_bits() {
    typed_record_inputs::check_flattened(SOURCE, cases(|limit| limit));
}

#[test]
fn typed_f64_scoped_record_carries_compose_nested_iterations() {
    let source = SOURCE.replace(
        "let carry = relay(carry);",
        "let j = 0; while j < 2 { let j = j + 1; let carry = relay(carry); }",
    );
    typed_record_inputs::check_flattened(&source, cases(|limit| 2 * limit));
}

#[test]
fn typed_f64_scoped_record_carries_keep_counted_returns_lazy() {
    let source = SOURCE.replace("let carry = relay(carry);", "let carry = relay(carry); if i == 2 { return carry; }")
        .replace("return carry;\n", "if limit > 1 { return State { value: 0.0, saved: 0.0, count: 1 / 0, enabled: false }; } return carry;\n");
    typed_record_inputs::check_flattened(&source, cases(|limit| limit.min(2)));
}

#[test]
fn typed_f64_scoped_record_carries_keep_scalar_slots_and_snapshots() {
    let source = SOURCE
        .replace("let carry = state;", "let carry = state; let scalar = state.value; let saved = state.value;")
        .replace("let carry = relay(carry);", "let carry = relay(carry); let scalar = carry.value;")
        .replace("return carry;", "return State { value: scalar, saved: saved, count: carry.count, enabled: carry.enabled };");
    let cases = cases(|limit| limit)
        .into_iter()
        .map(|(inputs, mut states)| {
            states[1][1] = inputs[0];
            states[2][1] = inputs[0];
            (inputs, states)
        })
        .collect();
    typed_record_inputs::check_flattened(&source, cases);
}

#[test]
fn typed_f64_scoped_record_carries_update_with_float_arithmetic() {
    let source = SOURCE.replace(
        "value: value.saved",
        "value: (value.value * 1.5 + 0.25) - 0.5",
    );
    let cases = [-2.5_f64, 0.0, 1.5]
        .into_iter()
        .flat_map(|seed| {
            [0_i64, 1, 3].into_iter().map(move |limit| {
                let initial = vec![seed.to_bits(), 0.25_f64.to_bits(), limit as u64, 0];
                let mut event = initial.clone();
                let mut value = seed;
                for _ in 0..limit {
                    event[1] = value.to_bits();
                    value = (value * 1.5 + 0.25) - 0.5;
                }
                event[0] = value.to_bits();
                event[3] = limit as u64 % 2;
                (initial[..3].to_vec(), vec![initial, event.clone(), event])
            })
        })
        .collect();
    typed_record_inputs::check_flattened(&source, cases);
}

#[test]
fn typed_f64_scoped_record_carries_compose_all_five_scalar_kinds() {
    let source = "mod cpu Main {
        struct State { value: f64, gain: f32, tag: i32, count: i64, enabled: bool }
        @noinline fn relay(value: State) -> State {
            return State { value: value.value, gain: value.gain,
                tag: value.tag + i32_from_i64(1), count: value.count, enabled: !value.enabled };
        }
        fn start(seed: f64, gain: f32, tag: i32, limit: i64) -> State {
            return State { value: seed, gain: gain, tag: tag, count: limit, enabled: false };
        }
        fn step(state: State) -> State {
            let carry = state; let scalar = state.value; let gain = state.gain; let i = 0;
            let limit = state.count;
            while i < limit {
                let i = i + 1; let carry = relay(carry);
                let scalar = carry.value; let gain = carry.gain;
            }
            return State { value: scalar, gain: gain, tag: carry.tag,
                count: carry.count, enabled: carry.enabled };
        }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    let mut cases = Vec::new();
    for (seed, gain) in [
        (0x8000_0000_0000_0000_u64, 0x8000_0000_u64),
        (0xfff8_0000_0000_1234, 0x7fc0_5678),
        (0x4000_0000_0000_0001, 0x4000_0001),
    ] {
        for (tag, limit) in [(i32::MIN, 0), (i32::MAX, 1), (-2_i32, 3)] {
            let initial = vec![seed, gain, tag as i64 as u64, limit as u64, 0];
            let mut event = initial.clone();
            event[2] = tag.wrapping_add(limit) as i64 as u64;
            event[4] = limit as u64 % 2;
            cases.push((initial[..4].to_vec(), vec![initial, event.clone(), event]));
        }
    }
    typed_record_inputs::check_flattened(source, cases);
}

#[test]
fn typed_f64_scoped_record_carries_reject_wrong_word_kinds() {
    let project = Project::with_source(SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for (from, to) in [
        ("pack_f64_word", "pack_f32_word"),
        ("unpack_f64_word", "unpack_f32_word"),
        ("pack_f64_word", "cast_i32_to_i64"),
        ("unpack_f64_word", "cast_i64_to_i32"),
        ("pack_f64_word", "cast_f64_to_i64"),
        ("unpack_f64_word", "cast_i64_to_f64"),
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
        assert!(
            emit_registered(&module, "counter").is_err(),
            "{from} -> {to}"
        );
    }
}

fn wide_source() -> String {
    typed_scoped_record_inputs::fixture::source(64, false)
        .replace(
            "fn start(seed: i64)",
            "fn start(seed: i64, left: f64, right: f64)",
        )
        .replace("f62: i64", "f62: f64")
        .replace("f63: i64", "f63: f64")
        .replace("f62: seed + 62", "f62: left")
        .replace("f63: seed + 63", "f63: right")
        .replace("f62: value.f62 + 1", "f62: value.f63")
        .replace("f63: value.f63 + 1", "f63: value.f62")
}

#[test]
fn typed_f64_scoped_record_carries_transport_wide_private_word_inputs() {
    let cases = [-3_i64, 0, 1, 2]
        .into_iter()
        .map(|seed| {
            let mut initial = (0..62).map(|i| (seed + i) as u64).collect::<Vec<_>>();
            initial.extend([0x8000_0000_0000_0000, 0x7ff8_0000_0000_1234]);
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
            if trips % 2 != 0 {
                event.swap(62, 63);
            }
            (
                vec![seed as u64, initial[62], initial[63]],
                vec![initial, event.clone(), event],
            )
        })
        .collect();
    typed_record_inputs::check(&wide_source(), cases);
}

#[test]
fn typed_f64_scoped_record_carries_keep_traps_and_work_limits_atomic() {
    let source = wide_source()
        .replace(
            "fn start(seed: i64, left: f64, right: f64)",
            "fn start(seed: i64)",
        )
        .replace("f62: left", "f62: 1.5")
        .replace("f63: right", "f63: 2.5");
    typed_record_guards::check_limits(
        source,
        &[
            (-2, 0, 64, false),
            (4, 64, 64, false),
            (2, 64, 64, true),
            (4, 64, 1, true),
            (4, 2, 64, true),
        ],
    );
}
