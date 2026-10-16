#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_exits_cases.rs"]
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
fn observers() -> [(&'static str, &'static str); 3] {
    [
        ("refine", cases::WORK_OBSERVER),
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
fn state_loop_exits_reference_skips_suffix_effects_and_retains_last_published_state() {
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
fn state_loop_exits_native_observes_recovered_index_and_selected_effect_prefixes() {
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
fn state_loop_exits_native_reserves_full_bound_before_an_early_break() {
    for variant in cases::variants() {
        let open = cases::Case {
            seed: 8,
            gate: false,
            flag: false,
            trips: 0,
            cut: if variant.leading { 1 } else { 0 },
            marker: 0x8000000000000000,
        };
        let prefix = cases::sequence(open, variant.leading, true);
        // Opening State is true, so gate=false selects break on the next event.
        assert!(prefix[0].next().result);
        for limit in [0, 2] {
            let mut healthy = vec![sequence(prefix.clone())];
            if limit > 0 {
                healthy.push(sequence(cases::sequence(
                    cases::Case {
                        trips: 1,
                        gate: true,
                        ..open
                    },
                    variant.leading,
                    false,
                )));
            }
            let failures = [1, 3, 65537, i64::MAX]
                .into_iter()
                .filter(|trips| *trips > limit as i64)
                .map(|trips| {
                    let mut input = prefix[1].words(None);
                    input[3] = trips;
                    input[4] = 1;
                    input[5] = -1;
                    probe::SequenceFailure {
                        accepted: sequence(vec![prefix[0]]),
                        input,
                        before_failure: vec![5],
                    }
                })
                .collect::<Vec<_>>();
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
