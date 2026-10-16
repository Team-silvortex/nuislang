#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_cases.rs"]
mod cases;
#[path = "state_loop_effects_profile.rs"]
mod effects;
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

#[test]
fn state_loop_effects_reference_executes_only_selected_prints_and_keeps_last_state() {
    for source in effects::source_variants() {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for marker in cases::MARKERS {
                for open in cases::cases() {
                    for poisoned in [false, true] {
                        let calls = cases::sequence(cases::Case { marker, ..open }, poisoned);
                        let (mut session, opened) = ApplicationSession::open_registered(
                            &module,
                            &registry,
                            "counter",
                            values(calls[0], None),
                        )
                        .unwrap();
                        assert_eq!(printed(&opened.events), effects::trace(calls[0], false));
                        assert_state(&session, calls[0].next());
                        for call in &calls[1..] {
                            let result = session.event(values(*call, None)).unwrap();
                            assert_eq!(printed(&result.events), effects::trace(*call, false));
                            assert_state(&session, call.next());
                        }
                        session.close(vec![]).unwrap();
                        assert_state(&session, calls.last().unwrap().next());
                        session.completion_status().unwrap();
                    }
                }
            }
            for (call, bad) in cases::failures() {
                let prefix = cases::failure_prefix();
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(prefix[0], None),
                )
                .unwrap();
                for row in &prefix[1..] {
                    session.event(values(*row, None)).unwrap();
                }
                let last = prefix.last().unwrap().next();
                assert!(session
                    .event(values(call, bad))
                    .unwrap_err()
                    .contains("zero"));
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
fn sequence(calls: Vec<cases::Invocation>) -> probe::Sequence {
    probe::Sequence {
        rows: calls
            .into_iter()
            .map(|call| probe::Row {
                input: call.words(None),
                state: call.next().words(),
                trace: effects::trace(call, true),
            })
            .collect(),
    }
}
fn observers() -> [(&'static str, &'static str); 2] {
    [
        ("refine", effects::WORK_OBSERVER),
        ("answer_work", cases::OBSERVER),
    ]
}

#[test]
fn state_loop_effects_native_observes_real_work_with_real_source_effects() {
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
            before_failure: effects::before_failure(call, bad),
        })
        .collect::<Vec<_>>();
    for source in effects::source_variants() {
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
fn state_loop_effects_native_reserves_before_any_iteration_print() {
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
            &effects::source_variants()[0],
            &[sequence(zero.clone())],
            &failures,
            &observers(),
            limit,
            100,
        );
    }
}
