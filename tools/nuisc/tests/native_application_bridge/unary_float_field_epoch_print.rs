#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "unary_float_field_epoch_print_cases.rs"]
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
            if i == 0 {
                if case.kind == cases::Kind::F32 {
                    Value::F32(f32::from_bits(value as u32))
                } else {
                    Value::F64(f64::from_bits(value as u64))
                }
            } else if i == 1 {
                Value::Bool(value != 0)
            } else {
                Value::Int(value)
            }
        })
        .collect()
}

fn sequence(rows: Vec<cases::Case>, negations: usize) -> probe::Sequence {
    let mut previous = None;
    probe::Sequence {
        rows: cases::with_negations(rows, negations)
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

fn native(kind: cases::Kind, negations: usize) {
    let failures = cases::failures(kind)
        .into_iter()
        .map(|bad| bad.with_negations(negations))
        .map(|bad| probe::SequenceFailure {
            accepted: sequence(cases::prefix(kind), negations),
            input: bad.words(),
            before_failure: bad.execute_native(Some(kind.last_state())).unwrap_err(),
        })
        .collect::<Vec<_>>();
    probe::check_sequences_with_i64_prints(
        &kind.source_with_negations(negations),
        &cases::sequences(kind)
            .into_iter()
            .map(|rows| sequence(rows, negations))
            .collect::<Vec<_>>(),
        &failures,
        kind.observers(),
        cases::LOOP_LIMIT,
        cases::ENTRY_LIMIT,
    );
}

fn budget(kind: cases::Kind, negations: usize) {
    let mut case = cases::healthy(kind)
        .into_iter()
        .find(|row| row.row.trips == 3 && row.row.children == 3)
        .unwrap();
    case.row.initial = kind.nan();
    probe::check_i64_helper_entry_limit_rejection(
        &kind.source_with_negations(negations),
        &[sequence(vec![case], negations)],
        kind.observers(),
        cases::LOOP_LIMIT,
        128,
    );
}

fn reference(kind: cases::Kind, negations: usize) {
    let project = Project::with_source(&kind.source_with_negations(negations));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(compiled.yir.functions.len() <= 32);
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        for rows in cases::sequences(kind) {
            let rows = cases::with_negations(rows, negations);
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
            assert_calls(&opened.events, rows[0]);
            for row in &rows[1..] {
                let (next, trace) = row.execute(Some(previous)).unwrap();
                let event = session.event(values(*row)).unwrap();
                assert_eq!(printed(&event.events), trace);
                assert_calls(&event.events, *row);
                assert_eq!(state_words(session.state()), vec![next as u64]);
                previous = next;
            }
            session.close(vec![]).unwrap();
            session.completion_status().unwrap();
        }
        for bad in cases::failures(kind) {
            let bad = bad.with_negations(negations);
            let rows = cases::with_negations(cases::prefix(kind), negations);
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

fn assert_calls(events: &[String], case: cases::Case) {
    for ((callee, _), count) in case.kind.observers().iter().zip(case.observer_counts()) {
        assert_eq!(calls(events, callee), count, "{callee} {case:?}");
    }
}

#[test]
fn unary_float_field_epoch_print_native_keeps_raw_arguments_and_selected_fault_prefixes() {
    for kind in cases::KINDS {
        native(kind, 1);
    }
}

#[test]
fn unary_float_field_epoch_print_native_keeps_entry_exhaustion_atomic() {
    for kind in cases::KINDS {
        budget(kind, 1);
    }
}

#[test]
fn unary_float_field_epoch_print_reference_preserves_counts_and_last_state() {
    for kind in cases::KINDS {
        reference(kind, 1);
    }
}

#[test]
fn nested_unary_float_field_epoch_print_native_keeps_even_odd_raw_arguments_and_faults() {
    for kind in cases::KINDS {
        for negations in [2, 3, 6] {
            native(kind, negations);
        }
    }
}

#[test]
fn nested_unary_float_field_epoch_print_native_keeps_entry_exhaustion_atomic() {
    for kind in cases::KINDS {
        for negations in [2, 3, 6] {
            budget(kind, negations);
        }
    }
}

#[test]
fn nested_unary_float_field_epoch_print_reference_keeps_counts_and_last_state() {
    for kind in cases::KINDS {
        for negations in [2, 3, 6] {
            reference(kind, negations);
        }
    }
}
