#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "logical_i32_call_argument_print_epoch_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn calls(events: &[String], callee: &str) -> usize {
    events
        .iter()
        .filter(|event| event.contains("cpu.call") && event.contains(&format!("] {callee}(")))
        .count()
}

fn values(case: cases::Case) -> Vec<Value> {
    case.words()
        .into_iter()
        .enumerate()
        .map(|(i, value)| {
            if i == 0 || i == 7 {
                if case.kind() == cases::Kind::F32 {
                    Value::F32(f32::from_bits(value as u32))
                } else {
                    Value::F64(f64::from_bits(value as u64))
                }
            } else if i == 10 || i == 11 {
                Value::I32(value as i32)
            } else if i == 1 {
                Value::Bool(value != 0)
            } else {
                Value::Int(value)
            }
        })
        .collect()
}

fn sequence(rows: Vec<cases::Case>) -> probe::Sequence {
    let mut previous = None;
    probe::Sequence {
        rows: rows
            .into_iter()
            .map(|case| {
                let (next, trace) = case.execute_native(previous).unwrap();
                previous = Some(next);
                probe::Row {
                    input: case.words(),
                    state: vec![next],
                    trace,
                }
            })
            .collect(),
    }
}

fn native(kind: cases::Kind) {
    let bodies = cases::observers(kind);
    let observers = bodies
        .iter()
        .map(|(name, body)| (*name, body.as_str()))
        .collect::<Vec<_>>();
    let failures = cases::failures(kind)
        .into_iter()
        .map(|bad| probe::SequenceFailure {
            accepted: sequence(cases::prefix(kind)),
            input: bad.words(),
            before_failure: bad.execute_native(Some(kind.last_state())).unwrap_err(),
        })
        .collect::<Vec<_>>();
    probe::check_sequences_with_i64_prints(
        &cases::source(kind),
        &cases::sequences(kind)
            .into_iter()
            .map(sequence)
            .collect::<Vec<_>>(),
        &failures,
        &observers,
        cases::LOOP_LIMIT,
        cases::ENTRY_LIMIT,
    );
}

fn budget(kind: cases::Kind) {
    let bodies = cases::observers(kind);
    let observers = bodies
        .iter()
        .map(|(name, body)| (*name, body.as_str()))
        .collect::<Vec<_>>();
    let mut case = cases::healthy(kind)
        .into_iter()
        .find(|row| row.base.row.trips == 3 && row.base.row.children == 3)
        .unwrap();
    case.base.row.initial = kind.nan();
    probe::check_i64_helper_entry_limit_rejection(
        &cases::source(kind),
        &[sequence(vec![case])],
        &observers,
        cases::LOOP_LIMIT,
        128,
    );
}

fn reference(kind: cases::Kind) {
    let project = Project::with_source(&cases::source(kind));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(
        compiled.yir.functions.len() <= 32,
        "function count: {}",
        compiled.yir.functions.len()
    );
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        for rows in cases::sequences(kind) {
            let (mut session, opened) =
                ApplicationSession::open_registered(&module, &registry, "counter", values(rows[0]))
                    .unwrap();
            let (mut previous, trace) = rows[0].execute(None).unwrap();
            let printed = |events: &[String]| {
                events
                    .iter()
                    .filter(|s| {
                        s.contains("cpu.print")
                            || (s.contains("cpu.guard_print") && s.contains("if true then print"))
                    })
                    .map(|s| s.rsplit_once(' ').unwrap().1.parse::<i64>().unwrap())
                    .collect::<Vec<_>>()
            };
            assert_eq!(printed(&opened.events), trace);
            assert_eq!(state_words(session.state()), vec![previous as u64]);
            assert_calls(&opened.events, rows[0], None);
            for row in &rows[1..] {
                let (next, trace) = row.execute(Some(previous)).unwrap();
                let event = session.event(values(*row)).unwrap();
                assert_eq!(printed(&event.events), trace);
                assert_calls(&event.events, *row, Some(previous));
                assert_eq!(state_words(session.state()), vec![next as u64]);
                previous = next;
            }
            session.close(vec![]).unwrap();
            session.completion_status().unwrap();
        }
        for bad in cases::failures(kind) {
            let rows = cases::prefix(kind);
            let (mut session, _) =
                ApplicationSession::open_registered(&module, &registry, "counter", values(rows[0]))
                    .unwrap();
            for row in &rows[1..] {
                session.event(values(*row)).unwrap();
            }
            assert!(session.event(values(bad)).is_err(), "{bad:?}");
            assert_eq!(state_words(session.state()), vec![kind.last_state() as u64]);
            assert_eq!(
                session.phase(),
                yir_runtime_host::ApplicationSessionPhase::Faulted
            );
            assert!(session.event(values(rows[1])).is_err());
            session.close(vec![]).unwrap();
            assert_eq!(state_words(session.state()), vec![kind.last_state() as u64]);
            assert!(session.completion_status().is_err());
        }
    }
}

fn assert_calls(events: &[String], case: cases::Case, previous: Option<i64>) {
    assert_eq!(
        calls(events, "choose"),
        case.observer_counts()[0],
        "choose {case:?}"
    );
    for (callee, count) in ["checked32", "select32"]
        .into_iter()
        .zip(case.helper_counts(previous))
    {
        assert_eq!(
            calls(events, callee),
            count,
            "{callee} {case:?} previous={previous:?}"
        );
    }
    // Owned-struct calls have no scalar-call event in reference execution.
    // The exact native word stream separately pins the single work-entry probe.
    for (callee, count) in ["observe", "finish"]
        .into_iter()
        .zip(case.observer_counts())
    {
        assert_eq!(calls(events, callee), count, "{callee} {case:?}");
    }
}

#[test]
fn logical_i32_call_argument_print_epoch_native_keeps_ieee_arguments_and_integer_fault_prefixes() {
    for kind in cases::KINDS {
        native(kind);
    }
}

#[test]
fn logical_i32_call_argument_print_epoch_native_keeps_entry_exhaustion_atomic() {
    for kind in cases::KINDS {
        budget(kind);
    }
}

#[test]
fn logical_i32_call_argument_print_epoch_reference_keeps_counts_and_last_state() {
    for kind in cases::KINDS {
        reference(kind);
    }
}

#[test]
fn logical_i32_call_argument_print_epoch_signed_matrix_keeps_exact_input_words() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 35);
        assert_eq!(cases::cli_sequences(kind).len(), 6);
        assert_eq!(cases::failures(kind).len(), 16);
        for (left, right) in cases::signed_pairs() {
            let mut case = cases::prefix(kind)[1];
            case.left = left;
            case.right = right;
            assert_eq!(&case.words()[10..], &[i64::from(left), i64::from(right)]);
            assert_eq!(&values(case)[10..], &[Value::I32(left), Value::I32(right)]);
            let args = case.args();
            let args = args.split(',').collect::<Vec<_>>();
            assert_eq!(args[10].parse::<i32>().unwrap(), left);
            assert_eq!(args[11].parse::<i32>().unwrap(), right);
        }
    }
}
