#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "return_stateful_fields_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn values(case: cases::Case, bad: Option<usize>) -> Vec<Value> {
    let mut words = case.words();
    if let Some(slot) = bad {
        assert_eq!(words[slot], 1);
        words[slot] = 0;
    }
    words
        .into_iter()
        .enumerate()
        .map(|(i, n)| {
            if (1..=3).contains(&i) {
                Value::Bool(n != 0)
            } else if i == 12 {
                Value::F64(f64::from_bits(n as u64))
            } else {
                Value::Int(n)
            }
        })
        .collect()
}
fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|s| {
            s.contains("cpu.print")
                || (s.contains("cpu.guard_print") && s.contains("if true then print"))
        })
        .map(|s| s.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}
fn assert_state(session: &ApplicationSession<'_>, expected: cases::State) {
    assert_eq!(
        state_words(session.state()),
        expected
            .words()
            .into_iter()
            .map(|n| n as u64)
            .collect::<Vec<_>>()
    );
}

#[test]
fn return_stateful_fields_oracle_exposes_current_total_bool_and_marker_dependencies() {
    for marker in cases::MARKERS {
        for case in cases::cases() {
            let case = cases::Case { marker, ..case };
            let open = case.initial();
            let mut previous = open;
            for (index, event) in case.events().into_iter().enumerate() {
                let next = event.step(previous);
                assert_eq!(next.total, open.total + index as i64 + 1);
                assert_eq!(next.marker, marker);
                assert_ne!(event.marker, marker);
                assert_ne!(next.result, event.result(!previous.result, next.total));
                if event.gate && event.first {
                    assert_ne!(next.result, event.result(previous.result, previous.total));
                }
                if index > 0 {
                    assert_ne!(next.total, event.step(open).total);
                }
                previous = next;
            }
        }
    }
}

#[test]
fn return_stateful_fields_reference_keeps_dependent_results_raw_float_bits_and_last_publication() {
    for source in cases::source_variants() {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for marker in cases::MARKERS {
                for case in cases::cases() {
                    let case = cases::Case { marker, ..case };
                    let (mut session, opened) = ApplicationSession::open_registered(
                        &module,
                        &registry,
                        "counter",
                        values(case, None),
                    )
                    .unwrap();
                    let mut previous = case.initial();
                    assert_eq!(
                        printed(&opened.events),
                        case.trace(case.flag, previous.total, false)
                    );
                    assert_state(&session, previous);
                    for event in case.events() {
                        let next = event.step(previous);
                        let result = session.event(values(event, None)).unwrap();
                        assert_eq!(
                            printed(&result.events),
                            event.trace(previous.result, next.total, false)
                        );
                        assert_state(&session, next);
                        previous = next;
                    }
                    session.close(vec![]).unwrap();
                    assert_state(&session, previous);
                    session.completion_status().unwrap();
                }
            }
            for (bad, slot) in cases::failures() {
                let [open, first, second] = cases::failure_prefix();
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(open, None),
                )
                .unwrap();
                session.event(values(first, None)).unwrap();
                session.event(values(second, None)).unwrap();
                let expected = second.step(first.step(open.initial()));
                assert_ne!(expected.result, open.initial().result);
                assert_state(&session, expected);
                assert!(session
                    .event(values(bad, Some(slot)))
                    .unwrap_err()
                    .contains("zero"));
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert_state(&session, expected);
                assert!(session.event(values(bad, None)).is_err());
                session.close(vec![]).unwrap();
                assert_state(&session, expected);
                assert!(session.completion_status().is_err());
            }
        }
    }
}

fn sequence(rows: impl IntoIterator<Item = cases::Case>) -> probe::Sequence {
    let mut previous = None;
    probe::Sequence {
        rows: rows
            .into_iter()
            .map(|case| {
                let next = previous.map_or_else(|| case.initial(), |s| case.step(s));
                let row = probe::Row {
                    input: case.words(),
                    state: next.words(),
                    trace: case.trace(
                        previous.map_or(case.flag, |s: cases::State| s.result),
                        next.total,
                        true,
                    ),
                };
                previous = Some(next);
                row
            })
            .collect(),
    }
}

#[test]
fn return_stateful_fields_native_reuses_three_real_words_and_preserves_float_payloads() {
    let rows = cases::MARKERS
        .into_iter()
        .flat_map(|marker| {
            cases::cases().into_iter().map(move |c| {
                let case = cases::Case { marker, ..c };
                sequence(std::iter::once(case).chain(case.events()))
            })
        })
        .collect::<Vec<_>>();
    let failures = cases::failures()
        .into_iter()
        .map(|(bad, slot)| {
            let accepted = sequence(cases::failure_prefix());
            let mut input = bad.words();
            input[slot] = 0;
            probe::SequenceFailure {
                accepted,
                input,
                before_failure: bad.before_failure(slot),
            }
        })
        .collect::<Vec<_>>();
    for source in cases::source_variants() {
        probe::check_sequences(&source, &rows, &failures, cases::OBSERVER);
    }
}
