#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_loop_sibling_bounds_cases.rs"]
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
fn sequence(calls: Vec<cases::Call>) -> probe::Sequence {
    probe::Sequence {
        rows: calls
            .into_iter()
            .map(|call| probe::Row {
                input: call.words(),
                state: call.next().words(),
                trace: call.execute(true, None).unwrap().1,
            })
            .collect(),
    }
}
fn assert_state(session: &ApplicationSession<'_>, call: cases::Call) {
    assert_eq!(
        state_words(session.state()),
        call.next()
            .words()
            .into_iter()
            .map(|w| w as u64)
            .collect::<Vec<_>>()
    );
}
fn prints(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn sibling_bounds_reference_preserves_snapshot_state_words_and_fault_retention() {
    check_reference(cases::variants());
}

#[test]
fn preheader_bounds_reference_keeps_snapshot_across_parent_backedges_and_fault_retention() {
    check_reference(cases::preheader_variants());
}

#[test]
fn epoch_bounds_reference_keeps_distinct_snapshots_and_last_state_after_seven_faults() {
    check_reference(cases::epoch_variants());
}

#[test]
fn snapshot_returns_reference_preserves_returned_words_and_faulted_last_state() {
    check_reference(cases::snapshot_return_variants());
}

#[test]
fn payload_calls_reference_preserves_last_state_after_ordered_return_failures() {
    check_reference(cases::payload_call_variants());
}

fn check_reference(variants: Vec<cases::Variant>) {
    for variant in variants {
        let project = Project::with_source(&cases::source(variant));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for calls in cases::healthy(variant) {
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(calls[0]),
                )
                .unwrap();
                assert_eq!(
                    prints(&opened.events),
                    calls[0].execute(false, None).unwrap().1
                );
                assert_state(&session, calls[0]);
                for call in &calls[1..] {
                    let event = session.event(values(*call)).unwrap();
                    assert_eq!(prints(&event.events), call.execute(false, None).unwrap().1);
                    assert_state(&session, *call);
                }
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
            }
            for (prefix, bad) in cases::failures(variant) {
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
                assert!(session.event(values(bad)).is_err());
                assert_state(&session, *prefix.last().unwrap());
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0])).is_err());
                session.close(vec![]).unwrap();
                assert_state(&session, *prefix.last().unwrap());
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn sibling_bounds_native_keeps_frozen_record_and_exact_reached_failure_prefixes() {
    check_native(cases::variants());
}

#[test]
fn preheader_bounds_native_keeps_one_snapshot_across_parent_backedges() {
    check_native(cases::preheader_variants());
}

#[test]
fn epoch_bounds_native_keeps_two_snapshot_identities_across_parent_backedges() {
    check_native(cases::epoch_variants());
}

#[test]
fn snapshot_returns_native_preserves_older_newer_words_and_reached_fault_prefixes() {
    check_native(cases::snapshot_return_variants());
}

#[test]
fn payload_calls_native_keeps_raw_words_and_exact_argument_and_callee_failure_prefixes() {
    check_native(cases::payload_call_variants());
}

fn check_native(variants: Vec<cases::Variant>) {
    for variant in variants {
        let healthy = cases::healthy(variant)
            .into_iter()
            .map(sequence)
            .collect::<Vec<_>>();
        let failures = cases::failures(variant)
            .into_iter()
            .map(|(prefix, call)| {
                let trace = call.execute(true, None).unwrap_err();
                if matches!(call.second, -1 | 0) {
                    assert!(trace.contains(&943));
                    assert!(!trace.contains(&946));
                }
                if call.variant.two_epochs && matches!(call.second, -3 | -2) {
                    assert!(!trace.contains(&943));
                    assert!(trace.ends_with(&[if call.second == -2 { 934 } else { 930 }]));
                }
                probe::SequenceFailure {
                    accepted: sequence(prefix),
                    input: call.words(),
                    before_failure: trace,
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_limits(
            &cases::source(variant),
            &healthy,
            &failures,
            &[("extent", cases::OBSERVER)],
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn snapshot_returns_native_reserves_full_entered_bounds_but_skips_later_work() {
    check_return_reservations(cases::snapshot_return_variants());
}

#[test]
fn payload_calls_native_reserves_entered_bounds_before_any_return_expression_effects() {
    check_return_reservations(cases::payload_call_variants());
}

fn check_return_reservations(variants: Vec<cases::Variant>) {
    for variant in variants {
        for mode in [1, 6, 10, 11] {
            let required = match mode {
                1 => 4,
                6 => 8,
                10 if variant.leading => 6,
                10 => 7,
                11 if variant.leading => 8,
                11 => 9,
                _ => unreachable!(),
            };
            for limit in [required - 1, required] {
                let prefix = cases::prefix(variant, false);
                let mut call = cases::base(variant);
                call.mode = mode;
                if mode >= 10 {
                    call.trips = 5;
                    call.divisor = 2;
                }
                call.previous = Some(prefix.last().unwrap().next());
                let mut healthy = vec![sequence(prefix.clone())];
                let mut failures = vec![];
                if limit < required {
                    let trace = call.execute(true, Some(limit)).unwrap_err();
                    assert_eq!(
                        trace.last(),
                        Some(&if matches!(mode, 1 | 10) { 942 } else { 945 })
                    );
                    assert_eq!(
                        trace.iter().filter(|&&v| v == 941).count(),
                        if mode >= 10 { 2 } else { 1 }
                    );
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(prefix),
                        input: call.words(),
                        before_failure: trace,
                    });
                } else {
                    let (_, trace) = call.execute(true, Some(limit)).unwrap();
                    assert_eq!(
                        trace.last(),
                        Some(&if variant.payload_calls { 973 } else { 960 })
                    );
                    assert!(!trace.contains(&949));
                    if mode == 1 {
                        assert!(!trace.contains(&945));
                    }
                    let mut calls = prefix;
                    calls.push(call);
                    healthy.push(sequence(calls));
                }
                probe::check_sequences_with_limits(
                    &cases::source(variant),
                    &healthy,
                    &failures,
                    &[("extent", cases::OBSERVER)],
                    limit,
                    cases::ENTRY_LIMIT,
                );
            }
        }
    }
}

#[test]
fn epoch_bounds_native_reserves_distinct_changing_bounds_after_backedges() {
    for variant in cases::epoch_variants() {
        for mode in [0, 2, 5] {
            let required = if mode == 5 {
                if variant.leading {
                    5
                } else {
                    4
                }
            } else if variant.leading {
                8
            } else {
                9
            };
            for limit in [required - 1, required] {
                let prefix = cases::prefix(variant, false);
                let mut call = cases::base(variant);
                call.trips = 2;
                call.mode = mode;
                call.previous = Some(prefix.last().unwrap().next());
                let mut healthy = vec![sequence(prefix.clone())];
                let mut failures = vec![];
                if limit < required {
                    let trace = call.execute(true, Some(limit)).unwrap_err();
                    assert_eq!(trace.iter().filter(|&&v| v == 930).count(), 3);
                    assert_eq!(trace.iter().filter(|&&v| v == 941).count(), 2);
                    assert_eq!(trace.last(), Some(&if mode == 5 { 942 } else { 945 }));
                    assert_eq!(
                        trace.iter().filter(|&&v| v == 946).count(),
                        if mode == 5 { 0 } else { 2 }
                    );
                    failures.push(probe::SequenceFailure {
                        accepted: sequence(prefix),
                        input: call.words(),
                        before_failure: trace,
                    });
                } else {
                    call.execute(true, Some(limit)).unwrap();
                    let mut calls = prefix;
                    calls.push(call);
                    healthy.push(sequence(calls));
                }
                probe::check_sequences_with_limits(
                    &cases::source(variant),
                    &healthy,
                    &failures,
                    &[("extent", cases::OBSERVER)],
                    limit,
                    cases::ENTRY_LIMIT,
                );
            }
        }
    }
}

#[test]
fn preheader_bounds_native_reserves_full_bounds_again_after_parent_backedge() {
    for variant in cases::preheader_variants() {
        for (mode, limit) in [(0, 9), (2, 9), (5, 5), (4, 4), (9, 6), (0, 10)] {
            let prefix = cases::prefix(variant, false);
            let mut call = cases::base(variant);
            call.trips = 2;
            call.mode = mode;
            call.previous = Some(prefix.last().unwrap().next());
            let mut healthy = vec![sequence(prefix.clone())];
            let mut failures = vec![];
            if matches!((mode, limit), (0 | 2, 9) | (5, 5)) {
                let trace = call.execute(true, Some(limit)).unwrap_err();
                assert_eq!(trace.iter().filter(|&&v| v == 930).count(), 1);
                assert_eq!(trace.iter().filter(|&&v| v == 941).count(), 2);
                assert!(trace.ends_with(&[2, 1, 16, if mode == 5 { 942 } else { 945 }]));
                assert_eq!(
                    trace.iter().filter(|&&v| v == 946).count(),
                    if mode == 5 { 0 } else { 2 }
                );
                failures.push(probe::SequenceFailure {
                    accepted: sequence(prefix),
                    input: call.words(),
                    before_failure: trace,
                });
            } else {
                call.execute(true, Some(limit)).unwrap();
                let mut calls = prefix;
                calls.push(call);
                healthy.push(sequence(calls));
            }
            probe::check_sequences_with_limits(
                &cases::source(variant),
                &healthy,
                &failures,
                &[("extent", cases::OBSERVER)],
                limit,
                cases::ENTRY_LIMIT,
            );
        }
    }
}

#[test]
fn sibling_bounds_native_charges_each_full_reached_bound_even_after_child_break() {
    for variant in cases::variants() {
        for (mode, limit) in [(2, 5), (9, 5), (9, 6)] {
            let prefix = cases::prefix(variant, false);
            let mut call = cases::base(variant);
            call.mode = mode;
            call.previous = Some(prefix.last().unwrap().next());
            let mut healthy = vec![sequence(prefix.clone())];
            let mut failures = vec![];
            if limit == 5 {
                let trace = call.execute(true, Some(limit)).unwrap_err();
                assert!(trace.ends_with(&[2, 1, 16, 945]));
                assert!(!trace.contains(&946));
                failures.push(probe::SequenceFailure {
                    accepted: sequence(prefix),
                    input: call.words(),
                    before_failure: trace,
                });
            } else {
                let mut calls = prefix;
                calls.push(call);
                healthy.push(sequence(calls));
            }
            probe::check_sequences_with_limits(
                &cases::source(variant),
                &healthy,
                &failures,
                &[("extent", cases::OBSERVER)],
                limit,
                cases::ENTRY_LIMIT,
            );
        }
    }
}

#[test]
fn sibling_bounds_native_entry_limit_160_still_rejects_long_callback() {
    let variant = cases::variants()[0];
    let mut call = cases::base(variant);
    call.input.total = 1;
    probe::check_helper_entry_limit_rejection(
        &cases::source(variant),
        &[sequence(vec![call])],
        &[("extent", cases::OBSERVER)],
        cases::LOOP_LIMIT,
        160,
    );
}
