#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "return_state_routing_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn values(call: cases::Invocation, bad: Option<usize>) -> Vec<Value> {
    call.words(bad)
        .into_iter()
        .enumerate()
        .map(|(slot, word)| match slot {
            1..=3 => Value::Bool(word != 0),
            12 => Value::F64(f64::from_bits(word as u64)),
            _ => Value::Int(word),
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
fn return_state_routing_oracle_distinguishes_identical_inputs_from_poisoned_masks() {
    let mut routes = std::collections::BTreeSet::new();
    let mut repeated_root = false;
    let mut repeated_child = false;
    for open in cases::cases() {
        let plain = cases::sequence(open, false);
        let poisoned = cases::sequence(open, true);
        for (call, poison) in plain.iter().zip(&poisoned) {
            assert_eq!(call.next(), poison.next());
            assert_eq!(call.trace(false), poison.trace(false));
            assert!(poison
                .words(None)
                .iter()
                .enumerate()
                .any(|(slot, word)| (5..=11).contains(&slot) && *word == 0));
            routes.insert((call.route().gate, call.route().first));
            if let Some(previous) = call.previous {
                let wrong_bool = cases::Invocation {
                    previous: Some(cases::State {
                        result: !previous.result,
                        ..previous
                    }),
                    ..*call
                };
                let wrong_total = cases::Invocation {
                    previous: Some(cases::State {
                        total: previous.total - 1,
                        ..previous
                    }),
                    ..*call
                };
                assert_ne!(call.route().gate, wrong_bool.route().gate);
                assert_ne!(call.route().first, wrong_total.route().first);
                assert_eq!(call.next().marker, previous.marker);
                assert_ne!(call.input.marker, previous.marker);
            }
        }
        for pair in plain[1..].windows(2) {
            assert_eq!(pair[0].words(None), pair[1].words(None));
            repeated_root |= pair[0].route().gate != pair[1].route().gate;
            repeated_child |= pair[0].route().first != pair[1].route().first;
        }
    }
    assert_eq!(routes.len(), 4);
    assert!(repeated_root && repeated_child);
    for (call, slot) in cases::failures() {
        assert_eq!(call.words(None)[slot], 1);
        assert_eq!(call.words(Some(slot))[slot], 0);
    }
}

#[test]
fn return_state_routing_reference_checks_selected_work_and_faulted_last_publication() {
    for source in cases::source_variants() {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for marker in cases::MARKERS {
                for open in cases::cases() {
                    for poisoned in [false, true] {
                        let rows = cases::sequence(cases::Case { marker, ..open }, poisoned);
                        let (mut session, opened) = ApplicationSession::open_registered(
                            &module,
                            &registry,
                            "counter",
                            values(rows[0], None),
                        )
                        .unwrap();
                        assert_eq!(printed(&opened.events), rows[0].trace(false));
                        assert_state(&session, rows[0].next());
                        for call in &rows[1..] {
                            let result = session.event(values(*call, None)).unwrap();
                            assert_eq!(printed(&result.events), call.trace(false));
                            assert_state(&session, call.next());
                        }
                        session.close(vec![]).unwrap();
                        assert_state(&session, rows.last().unwrap().next());
                        session.completion_status().unwrap();
                    }
                }
            }
            for (call, slot) in cases::failures() {
                let rows = cases::failure_prefix();
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(rows[0], None),
                )
                .unwrap();
                for row in &rows[1..] {
                    session.event(values(*row, None)).unwrap();
                }
                let last = rows.last().unwrap().next();
                assert_state(&session, last);
                assert!(session
                    .event(values(call, Some(slot)))
                    .unwrap_err()
                    .contains("zero"));
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert_state(&session, last);
                assert!(session.event(values(call, None)).is_err());
                session.close(vec![]).unwrap();
                assert_state(&session, last);
                assert!(session.completion_status().is_err());
            }
        }
    }
}
fn sequence(rows: Vec<cases::Invocation>) -> probe::Sequence {
    probe::Sequence {
        rows: rows
            .into_iter()
            .map(|call| probe::Row {
                input: call.words(None),
                state: call.next().words(),
                trace: call.trace(true),
            })
            .collect(),
    }
}

#[test]
fn return_state_routing_native_branches_on_actual_previous_output_and_new_total() {
    let mut rows = vec![];
    for marker in cases::MARKERS {
        for open in cases::cases() {
            for poisoned in [false, true] {
                rows.push(sequence(cases::sequence(
                    cases::Case { marker, ..open },
                    poisoned,
                )));
            }
        }
    }
    let failures = cases::failures()
        .into_iter()
        .map(|(call, slot)| probe::SequenceFailure {
            accepted: sequence(cases::failure_prefix()),
            input: call.words(Some(slot)),
            before_failure: call.before_failure(slot),
        })
        .collect::<Vec<_>>();
    for source in cases::source_variants() {
        probe::check_sequences(&source, &rows, &failures, cases::OBSERVER);
    }
}
