#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "alias_record_return_cases.rs"]
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
            if matches!(i, 1 | 3 | 4 | 13) {
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
fn alias_record_return_native_keeps_one_predicate_and_selected_fault_prefixes() {
    let failures = cases::native_failures()
        .into_iter()
        .map(|bad| probe::SequenceFailure {
            accepted: sequence(cases::prefix()),
            input: bad.words(),
            before_failure: bad.execute_native(Some(cases::LAST_STATE)).unwrap_err(),
        })
        .collect::<Vec<_>>();
    probe::check_sequences_with_limits(
        cases::SOURCE,
        &[sequence(cases::healthy())],
        &failures,
        cases::OBSERVERS,
        cases::LOOP_LIMIT,
        cases::ENTRY_LIMIT,
    );
}

#[test]
fn alias_record_return_reference_preserves_predicate_counts_and_last_state() {
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
        for (callee, _) in cases::OBSERVERS {
            assert_eq!(calls(&opened.events, callee), 0);
        }
        for row in &rows[1..] {
            let (next, trace) = row.execute(Some(previous)).unwrap();
            let event = session.event(values(*row)).unwrap();
            assert_eq!(printed(&event.events), trace);
            let (_, observed) = row.execute_native(Some(previous)).unwrap();
            for (index, (callee, _)) in cases::OBSERVERS.iter().enumerate() {
                assert_eq!(
                    calls(&event.events, callee),
                    observed
                        .iter()
                        .filter(|v| **v == cases::ENTRY_MARKERS[index])
                        .count(),
                    "{callee} {row:?}"
                );
            }
            assert_eq!(state_words(session.state()), vec![next as u64]);
            previous = next;
        }
        session.close(vec![]).unwrap();
        session.completion_status().unwrap();
        for bad in cases::native_failures() {
            let rows = cases::prefix();
            let (mut session, _) =
                ApplicationSession::open_registered(&module, &registry, "counter", values(rows[0]))
                    .unwrap();
            for row in &rows[1..] {
                session.event(values(*row)).unwrap();
            }
            assert!(session.event(values(bad)).is_err(), "{bad:?}");
            assert_eq!(state_words(session.state()), vec![cases::LAST_STATE as u64]);
            assert_eq!(
                session.phase(),
                yir_runtime_host::ApplicationSessionPhase::Faulted
            );
            assert!(session.event(values(rows[1])).is_err());
            session.close(vec![]).unwrap();
            assert_eq!(state_words(session.state()), vec![cases::LAST_STATE as u64]);
            assert!(session.completion_status().is_err());
        }
    }
}
