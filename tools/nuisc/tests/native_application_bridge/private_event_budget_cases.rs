//! Fixture-specific source replay and literal charge expectations, never
//! discovered from emitted LLVM or a threshold search at test runtime.
#[path = "private_event_cases.rs"]
#[allow(dead_code)]
mod cases;
pub use cases::{Case, Family};

pub fn selected(family: Family) -> [Case; 3] {
    let seed = match family {
        Family::Bool => 1,
        Family::I32 | Family::I64 => 0,
        Family::F32 => u64::from((-0.0_f32).to_bits()),
        Family::F64 => (-0.0_f64).to_bits(),
    };
    let candidates = family.cases();
    let find = |first, second| {
        candidates
            .iter()
            .find(|case| {
                case.seed_word == seed
                    && case.flags[0] == first
                    && case.flags[1] == second
                    && (case.flags.len() == 2 || case.flags[2] == second)
            })
            .unwrap()
            .clone()
    };
    [find(false, false), find(false, true), find(true, true)]
}

type Entry = (&'static str, Option<i64>);

// These names pin this fixture's private continuation topology, not public ABI.
// observe is flattened into start; the event root step adds one entry.
pub fn path(family: Family, high: bool, event: bool) -> Vec<Entry> {
    let mut entries = vec![];
    if event {
        entries.push(("step", None));
    }
    entries.extend([
        ("start", Some(99)),
        ("choose", Some(90)),
        ("__nuis_effect_call_arm_0", None),
    ]);
    if !high {
        entries.push((
            if matches!(family, Family::Bool) {
                "__nuis_effect_call_arm_17"
            } else {
                "__nuis_effect_call_arm_11"
            },
            None,
        ));
        entries.push(("initialize", Some(87)));
        if matches!(family, Family::Bool) {
            entries.push(("__nuis_effect_call_predicate_24", Some(79)));
        } else {
            entries.extend([
                ("__nuis_effect_call_predicate_3", None),
                ("answer_work", Some(97)),
            ]);
            // 79 is emitted in the same final entry, after answer_work returns.
        }
        return entries;
    }
    entries.extend([
        ("initialize", Some(87)),
        ("__nuis_effect_call_arm_1", None),
        ("child_initial", Some(91)),
        ("condition", Some(92)),
        ("__nuis_effect_call_arm_2", None),
    ]);
    if matches!(family, Family::Bool) {
        // true child skips child_work; nested current/seal branches retain
        // their private predicate and join entries despite skipping callees.
        entries.extend([
            ("__nuis_effect_call_predicate_0", None),
            ("__nuis_effect_call_predicate_1", None),
            ("parent_work", Some(95)),
            ("inner_condition", Some(88)),
            ("__nuis_effect_call_arm_3", None),
            ("__nuis_effect_call_predicate_2", None),
            ("inner_child_work", Some(94)),
            ("__nuis_effect_call_predicate_3", None),
            ("__nuis_effect_call_arm_4", None),
            ("__nuis_effect_call_arm_5", None),
            ("__nuis_effect_call_predicate_4", None),
            ("__nuis_effect_call_arm_6", None),
            ("__nuis_effect_call_predicate_6", None),
            ("__nuis_effect_call_arm_7", None),
            ("__nuis_effect_call_arm_12", None),
            ("__nuis_effect_call_predicate_15", None),
            ("after_work", Some(96)),
            ("__nuis_effect_call_predicate_16", None),
            ("__nuis_effect_call_arm_13", None),
            ("__nuis_effect_call_arm_14", None),
            ("__nuis_effect_call_predicate_19", None),
            ("__nuis_effect_call_arm_15", None),
            ("__nuis_effect_call_predicate_21", None),
            ("__nuis_effect_call_predicate_22", None),
            ("unused_work", Some(83)),
            ("__nuis_effect_call_arm_16", None),
            ("__nuis_effect_call_predicate_23", None),
            ("answer_work", Some(97)),
            ("__nuis_effect_call_arm_17", Some(79)),
        ]);
    } else {
        entries.extend([
            ("__nuis_effect_call_arm_5", None),
            ("parent_work", Some(95)),
            ("child_work", Some(93)),
            ("__nuis_effect_call_arm_6", None),
            ("child_work", Some(93)),
            ("__nuis_effect_call_arm_7", None),
            ("__nuis_effect_call_predicate_1", None),
            ("__nuis_effect_call_arm_8", None),
        ]);
        // Bitwise integer updates make the final child branch execute again.
        // IEEE signed-zero equality skips that callee for f32/f64.
        if matches!(family, Family::I32 | Family::I64) {
            entries.push(("child_work", Some(93)));
        }
        entries.extend([
            ("__nuis_effect_call_arm_9", None),
            ("after_work", Some(96)),
            ("unused_work", Some(83)),
            ("__nuis_effect_call_arm_10", None),
            ("__nuis_effect_call_predicate_2", None),
            ("answer_work", Some(97)),
            ("__nuis_effect_call_arm_11", Some(79)),
        ]);
    }
    entries
}

pub fn limit(family: Family) -> u64 {
    // Deliberately literal adjacent limits, never calibrated from actual output.
    match family {
        Family::Bool => 38,
        Family::I32 | Family::I64 => 25,
        Family::F32 | Family::F64 => 24,
    }
}

pub fn low_trace(family: Family) -> Vec<i64> {
    // Low numeric paths return to start after the answer callee.
    let mut trace = path(family, false, false)
        .iter()
        .filter_map(|e| e.1)
        .collect::<Vec<_>>();
    if !matches!(family, Family::Bool) {
        trace.push(79);
    }
    trace
}
