#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_return_parent_break_cases.rs"]
mod cases;
#[allow(dead_code)]
#[path = "state_loop_return_parent_continue_cases.rs"]
mod continuing;
#[path = "word_session_probe.rs"]
mod probe;

fn values(call: cases::Call) -> Vec<Value> {
    call.words()
        .into_iter()
        .enumerate()
        .map(|(slot, word)| match slot {
            0 | 4 => Value::Bool(word != 0),
            2 => Value::F32(f32::from_bits(word as u32)),
            3 => Value::F64(f64::from_bits(word as u64)),
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
fn sequence(calls: Vec<cases::Call>) -> probe::Sequence {
    probe::Sequence {
        rows: calls
            .into_iter()
            .map(|call| probe::Row {
                input: call.words(),
                state: call.next().words(),
                trace: call.trace(true),
            })
            .collect(),
    }
}

#[test]
fn state_loop_return_parent_break_reference_preserves_actual_state_raw_words_and_fault_retention() {
    for variant in cases::variants() {
        let project = Project::with_source(&cases::source(variant));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for calls in cases::sequences(variant) {
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(calls[0]),
                )
                .unwrap();
                assert_eq!(printed(&opened.events), calls[0].trace(false));
                assert_state(&session, calls[0].next());
                for call in &calls[1..] {
                    let event = session.event(values(*call)).unwrap();
                    assert_eq!(printed(&event.events), call.trace(false));
                    assert_state(&session, call.next());
                }
                session.close(vec![]).unwrap();
                assert_state(&session, calls.last().unwrap().next());
                session.completion_status().unwrap();
            }
            for failing in cases::failures(variant) {
                let prefix = cases::prefix(variant);
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(prefix[0]),
                )
                .unwrap();
                for call in &prefix[1..] {
                    session.event(values(*call)).unwrap();
                }
                let error = session
                    .event(values(cases::after_prefix(failing)))
                    .unwrap_err();
                assert!(
                    error.contains(if failing.divisor == -1 {
                        "overflow"
                    } else {
                        "zero"
                    }),
                    "{error}"
                );
                let last = prefix.last().unwrap().next();
                assert_state(&session, last);
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0])).is_err());
                session.close(vec![]).unwrap();
                assert_state(&session, last);
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn state_loop_return_parent_break_native_preserves_false_zeros_nan_payloads_and_typed_records() {
    for variant in cases::variants() {
        let healthy = cases::sequences(variant)
            .into_iter()
            .map(sequence)
            .collect::<Vec<_>>();
        let failures = cases::failures(variant)
            .into_iter()
            .map(|call| {
                let call = cases::after_prefix(call);
                probe::SequenceFailure {
                    accepted: sequence(cases::prefix(variant)),
                    input: call.words(),
                    before_failure: call.execute(true, None).unwrap_err(),
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_limits(
            &cases::source(variant),
            &healthy,
            &failures,
            &[
                ("returned", cases::OBSERVER),
                ("extent", cases::BOUND_OBSERVER),
                ("checkpoint", cases::FALLBACK_OBSERVER),
            ],
            cases::LOOP_LIMIT,
            100,
        );
    }
}

#[test]
fn state_loop_return_parent_break_native_reserves_full_bounds_and_replays_after_prefix_successes() {
    let mut healthy_callbacks = 0;
    let mut failed_callbacks = 0;
    for variant in cases::budget_variants() {
        for limit in [2, 4, 5] {
            let mut skipped = cases::base(variant);
            skipped.rounds = 0;
            skipped.divisor = 0;
            skipped.finish = 1;
            let mut healthy = vec![
                sequence(vec![skipped]),
                sequence(cases::inactive_sequence(variant, 0, i64::MAX)),
                sequence(cases::inactive_sequence(variant, -1, i64::MIN)),
            ];
            let mut base = cases::base(variant);
            base.finish = 1;
            let mut late = cases::late(variant);
            late.finish = 1;
            let mut huge_child = base;
            huge_child.trips = i64::MAX;
            let mut huge_parent = base;
            huge_parent.rounds = i64::MAX;
            let mut failures = vec![];
            for call in [
                base,
                late,
                cases::completed(variant),
                huge_child,
                huge_parent,
            ] {
                let call = cases::after_prefix(call);
                let prefix = cases::prefix(variant);
                match call.execute(true, Some(limit)) {
                    Ok(_) => {
                        let mut accepted = prefix;
                        accepted.push(call);
                        healthy.push(sequence(accepted));
                    }
                    Err(before_failure) => failures.push(probe::SequenceFailure {
                        accepted: sequence(prefix),
                        input: call.words(),
                        before_failure,
                    }),
                }
            }
            healthy_callbacks += healthy.iter().map(|s| s.rows.len()).sum::<usize>() * 2;
            failed_callbacks += failures.len() * 2;
            probe::check_sequences_with_limits(
                &cases::source(variant),
                &healthy,
                &failures,
                &[
                    ("returned", cases::OBSERVER),
                    ("extent", cases::BOUND_OBSERVER),
                    ("checkpoint", cases::FALLBACK_OBSERVER),
                ],
                limit,
                100,
            );
        }
    }
    eprintln!("parent_break_budget healthy={healthy_callbacks} failed={failed_callbacks}");
    assert!(healthy_callbacks > 0 && failed_callbacks > 0);
}

#[test]
fn state_loop_return_parent_break_keeps_preceding_parent_continue_native_profiles_unchanged() {
    for variant in continuing::cli_variants() {
        let healthy = [
            continuing::sequence(continuing::late(variant)),
            continuing::sequence(continuing::child_continued(variant)),
            continuing::inactive_sequence(variant, -1, i64::MIN),
        ]
        .into_iter()
        .map(|calls| probe::Sequence {
            rows: calls
                .into_iter()
                .map(|c| probe::Row {
                    input: c.words(),
                    state: c.next().words(),
                    trace: c.trace(true),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
        let failures = continuing::failures(variant)
            .into_iter()
            .take(4)
            .map(|c| {
                let c = continuing::after_prefix(c);
                probe::SequenceFailure {
                    accepted: probe::Sequence {
                        rows: continuing::prefix(variant)
                            .into_iter()
                            .map(|p| probe::Row {
                                input: p.words(),
                                state: p.next().words(),
                                trace: p.trace(true),
                            })
                            .collect(),
                    },
                    input: c.words(),
                    before_failure: c.execute(true, None).unwrap_err(),
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_limits(
            &continuing::source(variant),
            &healthy,
            &failures,
            &[
                ("returned", continuing::OBSERVER),
                ("extent", continuing::BOUND_OBSERVER),
            ],
            continuing::LOOP_LIMIT,
            100,
        );
    }
}
