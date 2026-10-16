#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn values(call: cases::Invocation, bad: Option<usize>) -> Vec<Value> {
    call.words(bad)
        .into_iter()
        .enumerate()
        .map(|(slot, word)| match slot {
            1..=2 => Value::Bool(word != 0),
            8 => Value::F64(f64::from_bits(word as u64)),
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
fn state_loop_oracle_distinguishes_carried_inputs_and_skipped_poison() {
    let mut trip_counts = std::collections::BTreeSet::new();
    let mut poison_positions = std::collections::BTreeSet::new();
    let mut stale_inputs = 0;
    let mut stale_predicates = 0;
    let mut history_changes = 0;
    for open in cases::cases() {
        for poisoned in [false, true] {
            let calls = cases::sequence(open, poisoned);
            for call in &calls {
                let rows = call.iterations().unwrap();
                trip_counts.insert(rows.len());
                if call.hole >= 0 && call.input.trips > 0 {
                    assert!(!rows[call.hole as usize].selected);
                    poison_positions.insert(call.hole);
                }
                for pair in rows.windows(2) {
                    assert_eq!(pair[1].entering, pair[0].current);
                    if pair[1].selected && pair[1].entering != call.entering() + 1 {
                        stale_inputs += 1;
                    }
                    if pair[1].selected
                        != (call.input.gate ^ call.prior() ^ ((call.entering() + 1) % 2 == 0))
                    {
                        stale_predicates += 1;
                    }
                }
                if let Some(previous) = call.previous {
                    assert_eq!(call.next().marker, previous.marker);
                    assert_ne!(call.input.marker, previous.marker);
                }
            }
            if !poisoned {
                for pair in calls[1..].windows(2) {
                    assert_eq!(pair[0].words(None), pair[1].words(None));
                    history_changes += usize::from(pair[0].next() != pair[1].next());
                }
            }
        }
    }
    assert_eq!(trip_counts, [0, 1, 3].into_iter().collect());
    assert_eq!(poison_positions, [0, 1, 2].into_iter().collect());
    assert!(stale_inputs > 0 && stale_predicates > 0 && history_changes > 0);
    let failed_iterations = cases::failures()
        .into_iter()
        .filter_map(|(call, bad)| bad.is_none().then(|| call.iterations().unwrap_err().len()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(failed_iterations, [0, 1, 2].into_iter().collect());
    assert_eq!(cases::failures().len(), 13);
}

#[test]
fn state_loop_reference_retains_last_state_after_later_iteration_failure() {
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
            for (call, bad) in cases::failures() {
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
                    .event(values(call, bad))
                    .unwrap_err()
                    .contains("zero"));
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert_state(&session, last);
                assert!(session.event(values(rows[0], None)).is_err());
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
fn observers() -> [(&'static str, &'static str); 3] {
    [
        ("refine", cases::WORK_OBSERVER),
        ("answer_work", cases::OBSERVER),
        ("choose", cases::CONDITION_OBSERVER),
    ]
}

#[test]
fn state_loop_native_observes_real_iteration_arguments_and_final_publication() {
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
        .map(|(call, bad)| probe::SequenceFailure {
            accepted: sequence(cases::failure_prefix()),
            input: call.words(bad),
            before_failure: call.before_failure(bad, true),
        })
        .collect::<Vec<_>>();
    for source in cases::source_variants() {
        probe::check_sequences_with_limits(
            &source,
            &rows,
            &failures,
            &observers(),
            cases::LOOP_LIMIT,
            100,
        );
    }
}

#[test]
fn state_loop_native_zero_budget_and_exact_trip_reservation_precede_body() {
    let zero = cases::sequence(
        cases::Case {
            seed: 8,
            gate: false,
            flag: false,
            trips: 0,
            marker: 0x8000000000000000,
        },
        true,
    );
    for limit in [0, 2, 3] {
        let failures = [1, 3, 4, 65537, i64::MAX]
            .into_iter()
            .filter(|trips| *trips > limit as i64)
            .map(|trips| {
                let mut input = zero[1].words(None);
                input[3] = trips;
                // If reservation were bypassed, the first iteration must be valid
                // and observable, not masked by the zero-trip poison arguments.
                input[4] = 1;
                input[5] = -1;
                probe::SequenceFailure {
                    accepted: sequence(vec![zero[0]]),
                    input,
                    before_failure: vec![5],
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_limits(
            cases::SOURCE,
            &[sequence(zero.clone())],
            &failures,
            &observers(),
            limit,
            100,
        );
    }
}
