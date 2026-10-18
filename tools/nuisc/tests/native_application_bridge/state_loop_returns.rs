#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_returns_cases.rs"]
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
fn observers() -> [(&'static str, &'static str); 2] {
    [
        ("returned", cases::RETURN_OBSERVER),
        ("answer_work", cases::ANSWER_OBSERVER),
    ]
}

#[test]
fn state_loop_returns_reference_preserves_effect_prefix_and_faulted_state() {
    for variant in cases::variants() {
        let project = Project::with_source(&variant.source());
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for marker in cases::MARKERS {
                for open in cases::cases(variant.leading) {
                    let calls = cases::sequence(cases::Invocation { marker, ..open });
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
                let call = cases::after_prefix(call);
                assert!(call.execute(bad, false, None).is_err());
                assert!(session
                    .event(values(call, bad))
                    .unwrap_err()
                    .contains("zero"));
                assert_state(&session, prefix.last().unwrap().next());
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0], None)).is_err());
                session.close(vec![]).unwrap();
                assert_state(&session, prefix.last().unwrap().next());
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn state_loop_returns_native_uses_actual_state_and_skips_parent_suffix_and_later_iterations() {
    for variant in cases::variants() {
        let healthy = cases::MARKERS
            .into_iter()
            .flat_map(|marker| {
                cases::cases(variant.leading).into_iter().map(move |open| {
                    sequence(cases::sequence(cases::Invocation { marker, ..open }))
                })
            })
            .collect::<Vec<_>>();
        let failures = cases::failures(variant.leading)
            .into_iter()
            .map(|(call, bad)| {
                let call = cases::after_prefix(call);
                probe::SequenceFailure {
                    accepted: sequence(cases::failure_prefix(variant.leading)),
                    input: call.words(bad),
                    before_failure: call.execute(bad, true, None).unwrap_err(),
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_limits(
            &variant.source(),
            &healthy,
            &failures,
            &observers(),
            cases::LOOP_LIMIT,
            100,
        );
    }
}

#[test]
fn state_loop_returns_native_reserves_full_reached_bounds_before_any_early_return() {
    for variant in cases::variants() {
        let base = cases::base(variant.leading);
        for limit in [4, 5, 9] {
            let skipped = cases::Invocation {
                rounds: 0,
                trips: i64::MAX,
                root: 0,
                ..base
            };
            let mut healthy = vec![sequence(vec![skipped])];
            if limit >= 5 {
                healthy.push(sequence(vec![base]));
            }
            let mut failures = vec![];
            for call in [
                base,
                cases::Invocation {
                    trips: i64::MAX,
                    ..base
                },
                cases::Invocation {
                    rounds: i64::MAX,
                    ..base
                },
            ] {
                let call = cases::after_prefix(call);
                match call.execute(None, true, Some(limit)) {
                    Ok(_) => {}
                    Err(before_failure) => failures.push(probe::SequenceFailure {
                        accepted: sequence(cases::failure_prefix(variant.leading)),
                        input: call.words(None),
                        before_failure,
                    }),
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
