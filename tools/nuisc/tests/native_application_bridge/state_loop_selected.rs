#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_selected_cases.rs"]
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
fn observers() -> [(&'static str, &'static str); 4] {
    [
        ("refine", cases::WORK_OBSERVER),
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
fn state_loop_selected_reference_skips_suffix_effects_and_retains_last_published_state() {
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
fn state_loop_selected_native_observes_recovered_index_and_selected_effect_prefixes() {
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
fn state_loop_selected_native_reserves_only_selected_bounds_and_keeps_parent_progress() {
    for variant in cases::variants() {
        let zero = cases::failure_prefix(variant.leading)[0];
        for limit in [2, 5] {
            let negative = cases::Invocation {
                input: cases::Case {
                    trips: 3,
                    rounds: 2,
                    ..zero.input
                },
                root: -1,
                hole: -1,
                suffix: -1,
                ..zero
            };
            let small = cases::Case {
                trips: if limit == 2 { 0 } else { 1 },
                rounds: 2,
                ..zero.input
            };
            let mut previous = None;
            let negative_calls = (0..4)
                .map(|_| {
                    let call = cases::Invocation {
                        previous,
                        ..negative
                    };
                    assert!(call.execute_with_budget(None, false, Some(limit)).is_ok());
                    previous = Some(call.next());
                    call
                })
                .collect();
            let healthy = [
                sequence(cases::unselected_sequence(variant.leading, i64::MAX, 0)),
                sequence(cases::unselected_sequence(variant.leading, i64::MAX, 1)),
                sequence(negative_calls),
                sequence(cases::sequence(small, variant.leading, false)),
            ];
            let mut failures = vec![];
            if limit == 2 {
                // The accepted State is -3/false, not an injected expected value.
                let prefix = cases::Invocation {
                    input: cases::Case {
                        seed: -7,
                        ..zero.input
                    },
                    ..zero
                };
                assert_eq!(prefix.next().total, -3);
                assert!(!prefix.next().result);
                for trips in [1, 3, 65537, i64::MAX] {
                    let call = cases::Invocation {
                        input: cases::Case {
                            trips,
                            rounds: 2,
                            cut: 2,
                            ..prefix.input
                        },
                        previous: Some(prefix.next()),
                        root: 1,
                        hole: -1,
                        suffix: -1,
                        ..prefix
                    };
                    assert_eq!(
                        call.execute_with_budget(None, false, Some(limit))
                            .unwrap_err(),
                        [5, 8, 6, 7]
                    );
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(vec![prefix]),
                        input: call.words(None),
                        before_failure: call
                            .execute_with_budget(None, true, Some(limit))
                            .unwrap_err(),
                    });
                }
            } else {
                for (seed, cut, gate) in [
                    (-7, 2, false),
                    (4, if variant.leading { 1 } else { 0 }, false),
                ] {
                    let prefix = cases::Invocation {
                        input: cases::Case { seed, ..zero.input },
                        ..zero
                    };
                    let call = cases::Invocation {
                        input: cases::Case {
                            gate,
                            trips: 3,
                            rounds: 2,
                            cut,
                            ..prefix.input
                        },
                        previous: Some(prefix.next()),
                        root: 1,
                        hole: -1,
                        suffix: -1,
                        ..prefix
                    };
                    assert!(call.execute_with_budget(None, false, Some(8)).is_ok());
                    let trace = call
                        .execute_with_budget(None, false, Some(limit))
                        .unwrap_err();
                    assert_eq!(trace.last(), Some(&7));
                    assert_eq!(trace.iter().filter(|n| **n == 6).count(), 2);
                    assert_eq!(trace.iter().filter(|n| **n == 22).count(), 1);
                    assert!(trace.contains(&(if seed == 4 { 12 } else { 13 })));
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(vec![prefix]),
                        input: call.words(None),
                        before_failure: call
                            .execute_with_budget(None, true, Some(limit))
                            .unwrap_err(),
                    });
                }
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
