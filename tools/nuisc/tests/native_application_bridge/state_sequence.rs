#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_sequence_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn values(call: cases::Invocation, bad: Option<usize>) -> Vec<Value> {
    call.words(bad)
        .into_iter()
        .enumerate()
        .map(|(slot, word)| match slot {
            1..=3 => Value::Bool(word != 0),
            10 => Value::F64(f64::from_bits(word as u64)),
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
fn state_sequence_oracle_exposes_fresh_predicate_and_second_work_inputs() {
    let mut routes = std::collections::BTreeSet::new();
    let mut retained = [false; 3];
    let mut repeated_routes = false;
    let mut stale_predicates = 0;
    for marker in cases::MARKERS {
        for open in cases::cases() {
            let plain = cases::sequence(cases::Case { marker, ..open }, false);
            let poisoned = cases::sequence(cases::Case { marker, ..open }, true);
            for (call, poison) in plain.iter().zip(&poisoned) {
                assert_eq!(call.next(), poison.next());
                assert_eq!(call.trace(true), poison.trace(true));
                assert_eq!(poison.words(None)[5], i64::from(call.route()));
                assert_eq!(poison.words(None)[7], i64::from(call.second_route()));
                routes.insert((call.route(), call.second_route()));
                if call.route() {
                    assert_ne!(
                        call.second_route(),
                        call.input.second ^ ((call.entering() + 1) % 2 == 0)
                    );
                    stale_predicates += 1;
                    if call.second_route() {
                        assert_ne!(call.next().total, 2 * (call.entering() + 1) + 2);
                    }
                }
                if !call.second_route() {
                    assert_eq!(call.selected(), call.first());
                    retained[0] |= call.first() != 0;
                    retained[1] |= call.first() < 0;
                    retained[2] |= call.first() == 0;
                    if call.first() != 0 {
                        assert_ne!(call.next().total, 3);
                    }
                }
                assert_ne!(
                    call.next().result,
                    call.prior() ^ (call.selected() % 2 == 0)
                );
                if let Some(previous) = call.previous {
                    assert_eq!(call.next().marker, previous.marker);
                    assert_ne!(call.input.marker, previous.marker);
                }
            }
            for pair in plain[1..].windows(2) {
                assert_eq!(pair[0].words(None), pair[1].words(None));
                repeated_routes |= (pair[0].route(), pair[0].second_route())
                    != (pair[1].route(), pair[1].second_route());
            }
        }
    }
    assert_eq!(routes.len(), 4);
    assert_eq!(retained, [true; 3]);
    assert!(repeated_routes && stale_predicates > 0);
    assert_eq!(cases::failures().len(), 17);
    for (call, slot) in cases::failures() {
        assert_eq!(call.words(None)[slot], 1);
        assert_eq!(call.words(Some(slot))[slot], 0);
    }
}

#[test]
fn state_sequence_reference_checks_current_handoff_and_last_publication() {
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
fn state_sequence_native_observes_both_real_callees_and_current_handoff() {
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
            before_failure: call.before_failure(slot, true),
        })
        .collect::<Vec<_>>();
    for source in cases::source_variants() {
        probe::check_sequences_observed(
            &source,
            &rows,
            &failures,
            &[
                ("refine", cases::SECOND_OBSERVER),
                ("answer_work", cases::OBSERVER),
            ],
        );
    }
}
