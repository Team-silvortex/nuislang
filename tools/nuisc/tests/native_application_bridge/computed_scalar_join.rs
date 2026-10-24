#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "computed_scalar_join_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn decisions(events: &[String]) -> usize {
    events
        .iter()
        .filter(|event| event.contains("cpu.call_bool") && event.contains("] decide("))
        .count()
}

fn values(case: cases::Case) -> Vec<Value> {
    case.words()
        .into_iter()
        .enumerate()
        .map(|(i, value)| {
            if matches!(i, 1 | 3) {
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

#[test]
fn computed_scalar_join_native_keeps_one_predicate_and_selected_fault_prefixes() {
    let failures = cases::failures()
        .into_iter()
        .map(|bad| probe::SequenceFailure {
            accepted: sequence(cases::prefix()),
            input: bad.words(),
            before_failure: bad.execute_native(Some(-2)).unwrap_err(),
        })
        .collect::<Vec<_>>();
    probe::check_sequences_with_limits(
        cases::SOURCE,
        &[sequence(cases::healthy())],
        &failures,
        &[("decide", cases::DECISION_OBSERVER)],
        cases::LOOP_LIMIT,
        cases::ENTRY_LIMIT,
    );
}

#[test]
fn computed_scalar_join_reference_preserves_predicate_counts_and_last_state() {
    let project = Project::with_source(cases::SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(compiled.yir.functions.len() <= 64);
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        let rows = cases::healthy();
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
        assert_eq!(decisions(&opened.events), 0);
        for row in &rows[1..] {
            let (next, trace) = row.execute(Some(previous)).unwrap();
            let event = session.event(values(*row)).unwrap();
            assert_eq!(printed(&event.events), trace);
            assert_eq!(
                decisions(&event.events),
                usize::from(row.input.selected && row.input.trips > 0)
            );
            assert_eq!(state_words(session.state()), vec![next as u64]);
            previous = next;
        }
        session.close(vec![]).unwrap();
        session.completion_status().unwrap();
        for bad in cases::failures() {
            let rows = cases::prefix();
            let (mut session, _) =
                ApplicationSession::open_registered(&module, &registry, "counter", values(rows[0]))
                    .unwrap();
            for row in &rows[1..] {
                session.event(values(*row)).unwrap();
            }
            assert!(session.event(values(bad)).is_err(), "{bad:?}");
            assert_eq!(state_words(session.state()), vec![(-2i64) as u64]);
            assert_eq!(
                session.phase(),
                yir_runtime_host::ApplicationSessionPhase::Faulted
            );
            assert!(session.event(values(rows[1])).is_err());
            session.close(vec![]).unwrap();
            assert_eq!(state_words(session.state()), vec![(-2i64) as u64]);
            assert!(session.completion_status().is_err());
        }
    }
}
