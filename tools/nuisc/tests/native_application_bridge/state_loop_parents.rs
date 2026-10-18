#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_parents_cases.rs"]
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
fn assert_state(session: &ApplicationSession<'_>, state: cases::State) {
    assert_eq!(
        state_words(session.state()),
        state
            .words()
            .into_iter()
            .map(|n| n as u64)
            .collect::<Vec<_>>()
    );
}
fn observers() -> [(&'static str, &'static str); 5] {
    [
        ("refine", cases::WORK_OBSERVER),
        ("checkpoint", cases::CHECKPOINT_OBSERVER),
        ("combine", cases::COMBINE_OBSERVER),
        ("observe", cases::INDEX_OBSERVER),
        ("answer_work", cases::ANSWER_OBSERVER),
    ]
}
fn sequence(calls: Vec<cases::Invocation>) -> probe::Sequence {
    probe::Sequence {
        rows: calls
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
fn state_loop_parents_reference_skips_suffix_effects_and_retains_last_published_state() {
    for variant in cases::variants() {
        let project = Project::with_source(&variant.source());
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for marker in cases::MARKERS {
                for open in cases::cases() {
                    for poisoned in [false, true] {
                        let calls = cases::sequence(
                            cases::Case { marker, ..open },
                            variant.leading,
                            poisoned,
                        );
                        let (mut session, opened) = ApplicationSession::open_registered(
                            &module,
                            &registry,
                            "counter",
                            values(calls[0], None),
                        )
                        .unwrap();
                        assert_eq!(printed(&opened.events), calls[0].trace(false));
                        assert_state(&session, calls[0].next());
                        for call in &calls[1..] {
                            let event = session.event(values(*call, None)).unwrap();
                            assert_eq!(printed(&event.events), call.trace(false));
                            assert_state(&session, call.next());
                        }
                        session.close(vec![]).unwrap();
                        assert_state(&session, calls.last().unwrap().next());
                        session.completion_status().unwrap();
                    }
                }
            }
            for (call, bad) in cases::failures(variant.leading) {
                let prefix = cases::failure_prefix(variant.leading);
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(prefix[0], None),
                )
                .unwrap();
                for call in &prefix[1..] {
                    session.event(values(*call, None)).unwrap();
                }
                assert!(session
                    .event(values(call, bad))
                    .unwrap_err()
                    .contains("zero"));
                let last = prefix.last().unwrap().next();
                assert_state(&session, last);
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0], None)).is_err());
                session.close(vec![]).unwrap();
                assert_state(&session, last);
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn state_loop_parents_native_observes_recovered_index_and_selected_effect_prefixes() {
    for variant in cases::variants() {
        let mut rows = vec![];
        for marker in cases::MARKERS {
            for open in cases::cases() {
                for poisoned in [false, true] {
                    rows.push(sequence(cases::sequence(
                        cases::Case { marker, ..open },
                        variant.leading,
                        poisoned,
                    )));
                }
            }
        }
        let failures = cases::failures(variant.leading)
            .into_iter()
            .map(|(call, bad)| probe::SequenceFailure {
                accepted: sequence(cases::failure_prefix(variant.leading)),
                input: call.words(bad),
                before_failure: call.execute(bad, true).unwrap_err(),
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_limits(
            &variant.source(),
            &rows,
            &failures,
            &observers(),
            cases::LOOP_LIMIT,
            100,
        );
    }
}

#[test]
fn state_loop_parents_native_keep_child_work_and_reserve_only_reached_later_children() {
    for variant in cases::variants() {
        let zero = cases::failure_prefix(variant.leading)[0];
        for limit in [2, 5, 6, 9] {
            let mut previous = None;
            let negative = (0..4)
                .map(|_| {
                    let call = cases::Invocation {
                        input: cases::Case {
                            trips: 3,
                            rounds: 1,
                            ..zero.input
                        },
                        previous,
                        root: -1,
                        hole: -1,
                        suffix: -1,
                        ..zero
                    };
                    assert!(call.execute_with_budget(None, false, Some(limit)).is_ok());
                    previous = Some(call.next());
                    call
                })
                .collect();
            let small = cases::Case {
                trips: 1,
                rounds: 1,
                ..zero.input
            };
            let mut healthy = vec![
                sequence(cases::unselected_sequence(variant.leading, i64::MAX, 0)),
                sequence(cases::unselected_sequence(variant.leading, i64::MAX, 1)),
                sequence(negative),
                sequence(cases::sequence(small, variant.leading, false)),
            ];
            let breaks = cases::parent_sequence(variant.leading, true);
            let continues = cases::parent_sequence(variant.leading, false);
            if limit >= 5 {
                healthy.push(sequence(breaks.clone()));
            }
            if limit == 9 {
                healthy.push(sequence(continues.clone()));
            }
            let mut failures = vec![];
            if limit == 2 {
                for rounds in [3, 65537, i64::MAX] {
                    let call = cases::Invocation {
                        input: cases::Case {
                            rounds,
                            trips: 3,
                            ..zero.input
                        },
                        previous: Some(zero.next()),
                        root: 1,
                        hole: -1,
                        suffix: -1,
                        ..zero
                    };
                    assert_eq!(
                        call.execute_with_budget(None, false, Some(limit))
                            .unwrap_err(),
                        [5]
                    );
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(vec![zero]),
                        input: call.words(None),
                        before_failure: call
                            .execute_with_budget(None, true, Some(limit))
                            .unwrap_err(),
                    });
                }
                for trips in [1, 3, 65537, i64::MAX] {
                    let call = cases::Invocation {
                        input: cases::Case {
                            rounds: 2,
                            trips,
                            ..continues[1].input
                        },
                        suffix: -1,
                        ..continues[1]
                    };
                    assert_eq!(
                        call.execute_with_budget(None, false, Some(limit))
                            .unwrap_err(),
                        [5, 8, 6, 7]
                    );
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(vec![continues[0]]),
                        input: call.words(None),
                        before_failure: call
                            .execute_with_budget(None, true, Some(limit))
                            .unwrap_err(),
                    });
                }
            } else if limit == 5 {
                for trips in [4, 65537, i64::MAX] {
                    let call = cases::Invocation {
                        input: cases::Case {
                            trips,
                            ..breaks[1].input
                        },
                        suffix: -1,
                        ..breaks[1]
                    };
                    assert_eq!(
                        call.execute_with_budget(None, false, Some(limit))
                            .unwrap_err(),
                        [5, 8, 6, 7]
                    );
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(vec![breaks[0]]),
                        input: call.words(None),
                        before_failure: call
                            .execute_with_budget(None, true, Some(limit))
                            .unwrap_err(),
                    });
                }
            } else if limit == 6 {
                let call = continues[1];
                let trace = call
                    .execute_with_budget(None, false, Some(limit))
                    .unwrap_err();
                assert_eq!(trace.last(), Some(&7));
                assert_eq!(trace.iter().filter(|n| **n == 26).count(), 2);
                assert_eq!(trace.iter().filter(|n| **n == 23).count(), 2);
                assert_eq!(trace.iter().filter(|n| **n == 8).count(), 3);
                assert!(!trace.contains(&21));
                assert!(!trace.contains(&22));
                failures.push(probe::SequenceFailure {
                    accepted: sequence(vec![continues[0]]),
                    input: call.words(None),
                    before_failure: call
                        .execute_with_budget(None, true, Some(limit))
                        .unwrap_err(),
                });
            }
            probe::check_sequences_with_limits(
                &variant.source(),
                &healthy,
                &failures,
                &observers(),
                limit,
                100,
            );
        }
    }
}
