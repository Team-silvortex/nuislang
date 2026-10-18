#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_return_values_cases.rs"]
mod cases;
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
fn state_loop_return_values_reference_preserves_actual_state_raw_words_and_fault_retention() {
    for variant in cases::variants() {
        let project = Project::with_source(&variant.source());
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for open in cases::cases(variant) {
                let calls = cases::sequence(open);
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
                assert!(session
                    .event(values(cases::after_prefix(failing)))
                    .unwrap_err()
                    .contains("zero"));
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
fn state_loop_return_values_native_preserves_false_zeros_nan_payloads_and_typed_records() {
    for variant in cases::variants() {
        let healthy = cases::cases(variant)
            .into_iter()
            .map(|open| sequence(cases::sequence(open)))
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
            &variant.source(),
            &healthy,
            &failures,
            &[("returned", cases::OBSERVER)],
            cases::LOOP_LIMIT,
            100,
        );
    }
}

#[test]
fn state_loop_return_values_native_rejects_reached_bounds_without_waiving_typed_returns() {
    for variant in cases::cli_variants() {
        let base = cases::base(variant);
        for limit in [4, 5] {
            let skipped = cases::Call {
                rounds: 0,
                trips: i64::MAX,
                root: 0,
                ..base
            };
            let mut healthy = vec![sequence(vec![skipped])];
            if limit == 5 {
                healthy.push(sequence(vec![base]));
            }
            let mut failures = vec![];
            for call in [
                base,
                cases::Call {
                    trips: i64::MAX,
                    ..base
                },
                cases::Call {
                    rounds: i64::MAX,
                    ..base
                },
            ] {
                let call = cases::after_prefix(call);
                if let Err(before_failure) = call.execute(true, Some(limit)) {
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(cases::prefix(variant)),
                        input: call.words(),
                        before_failure,
                    });
                }
            }
            probe::check_sequences_with_limits(
                &variant.source(),
                &healthy,
                &failures,
                &[("returned", cases::OBSERVER)],
                limit,
                100,
            );
        }
    }
}
