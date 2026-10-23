#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[path = "state_loop_return_shared_bound_tests.rs"]
mod bound_tests;
#[allow(dead_code)]
#[path = "state_loop_return_shared_cases.rs"]
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
fn sequence(calls: Vec<cases::Call>, kind: cases::SourceKind) -> probe::Sequence {
    probe::Sequence {
        rows: calls
            .into_iter()
            .map(|call| probe::Row {
                input: call.words(),
                state: kind.next(call).words(),
                trace: kind.execute(call, true, None).unwrap().1,
            })
            .collect(),
    }
}
fn assert_state(session: &ApplicationSession<'_>, call: cases::Call) {
    assert_state_for_kind(session, call, cases::SourceKind::Plain);
}
fn assert_state_for_kind(
    session: &ApplicationSession<'_>,
    call: cases::Call,
    kind: cases::SourceKind,
) {
    assert_eq!(
        state_words(session.state()),
        kind.next(call)
            .words()
            .into_iter()
            .map(|v| v as u64)
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
fn shared_return_storage_reference_keeps_actual_state_raw_words_and_fault_retention() {
    check_reference(cases::SourceKind::Plain);
}

#[test]
fn shared_return_storage_prefix_reference_keeps_ordered_prints_and_fault_retention() {
    check_reference(cases::SourceKind::Prefix);
}

#[test]
fn shared_return_storage_candidates_reference_keeps_state_and_fault_retention() {
    check_reference(cases::SourceKind::Candidates);
}

#[test]
fn shared_return_storage_reassignment_reference_keeps_state_bits_and_fault_retention() {
    check_reference(cases::SourceKind::Reassigned);
}

#[test]
fn shared_return_storage_helper_calls_reference_keeps_state_bits_and_fault_retention() {
    check_reference(cases::SourceKind::HelperCalls);
}

#[test]
fn shared_return_storage_loop_helper_calls_reference_keeps_state_bits_and_selected_exits() {
    check_reference(cases::SourceKind::LoopHelperCalls);
}

#[test]
fn shared_return_storage_branch_helper_calls_reference_keeps_multifield_bits_and_selection() {
    check_reference(cases::SourceKind::BranchHelperCalls);
}

fn check_reference(kind: cases::SourceKind) {
    for variant in cases::variants() {
        let project = Project::with_source(&kind.source(variant));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for calls in kind.sequences(variant) {
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(calls[0]),
                )
                .unwrap();
                assert_eq!(
                    prints(&opened.events),
                    kind.execute(calls[0], false, None).unwrap().1
                );
                assert_state_for_kind(&session, calls[0], kind);
                for call in &calls[1..] {
                    let event = session.event(values(*call)).unwrap();
                    assert_eq!(
                        prints(&event.events),
                        kind.execute(*call, false, None).unwrap().1
                    );
                    assert_state_for_kind(&session, *call, kind);
                }
                session.close(vec![]).unwrap();
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
                assert!(session.event(values(cases::after_prefix(failing))).is_err());
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
fn shared_return_storage_native_keeps_once_only_prefixes_false_zeros_nan_and_local_exits() {
    check_native(cases::SourceKind::Plain);
}

#[test]
fn shared_return_storage_prefix_native_keeps_raw_words_and_once_only_effects() {
    check_native(cases::SourceKind::Prefix);
}

#[test]
fn shared_return_storage_candidates_native_keeps_raw_words_and_selected_failure_prefixes() {
    check_native(cases::SourceKind::Candidates);
}

#[test]
fn shared_return_storage_reassignment_native_keeps_raw_words_and_ordered_effects() {
    check_native(cases::SourceKind::Reassigned);
}

#[test]
fn shared_return_storage_helper_calls_native_keeps_raw_words_and_ordered_effects() {
    check_native(cases::SourceKind::HelperCalls);
}

#[test]
fn shared_return_storage_loop_helper_calls_native_keeps_raw_words_and_once_only_work() {
    check_native(cases::SourceKind::LoopHelperCalls);
}

#[test]
fn shared_return_storage_branch_helper_calls_native_keeps_unselected_poison_inert() {
    check_native(cases::SourceKind::BranchHelperCalls);
}

fn check_native(kind: cases::SourceKind) {
    for variant in cases::variants() {
        let healthy = kind
            .sequences(variant)
            .into_iter()
            .map(|calls| sequence(calls, kind))
            .collect::<Vec<_>>();
        let failures = cases::failures(variant)
            .into_iter()
            .map(|call| {
                let call = cases::after_prefix(call);
                probe::SequenceFailure {
                    accepted: sequence(cases::prefix(variant), kind),
                    input: call.words(),
                    before_failure: kind.execute(call, true, None).unwrap_err(),
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_limits(
            &kind.source(variant),
            &healthy,
            &failures,
            &[
                ("returned", cases::OBSERVER),
                ("extent", cases::BOUND_OBSERVER),
                ("checkpoint", cases::FALLBACK_OBSERVER),
            ],
            cases::LOOP_LIMIT,
            kind.helper_entry_limit(),
        );
    }
}

#[test]
fn shared_return_storage_native_still_reserves_the_full_reached_loop_bounds() {
    check_bounds(cases::SourceKind::Plain);
}

#[test]
fn shared_return_storage_prefix_native_preserves_full_reached_reservations() {
    check_bounds(cases::SourceKind::Prefix);
}

#[test]
fn shared_return_storage_candidates_native_preserves_full_reached_reservations() {
    check_bounds(cases::SourceKind::Candidates);
}

#[test]
fn shared_return_storage_reassignment_native_preserves_full_reached_reservations() {
    check_bounds(cases::SourceKind::Reassigned);
}

#[test]
fn shared_return_storage_helper_calls_native_preserves_full_reached_reservations() {
    check_bounds(cases::SourceKind::HelperCalls);
}

#[test]
fn shared_return_storage_loop_helper_calls_native_preserves_full_reached_reservations() {
    check_bounds(cases::SourceKind::LoopHelperCalls);
}

#[test]
fn shared_return_storage_branch_helper_calls_native_preserves_full_reached_reservations() {
    check_bounds(cases::SourceKind::BranchHelperCalls);
}

fn check_bounds(kind: cases::SourceKind) {
    for variant in cases::variants()
        .into_iter()
        .filter(|v| !v.mirrored && v.leading == v.parent_leading)
    {
        for limit in [4, 5] {
            let mut healthy = vec![sequence(
                cases::inactive_sequence(variant, 0, i64::MAX),
                kind,
            )];
            let mut failures = vec![];
            let mut huge = cases::base(variant);
            huge.trips = i64::MAX;
            let mut calls = vec![huge];
            if limit == 5 {
                healthy.push(sequence(cases::sequence(cases::base(variant)), kind));
            } else {
                calls.push(cases::base(variant));
            }
            for call in calls {
                let call = cases::after_prefix(call);
                failures.push(probe::SequenceFailure {
                    accepted: sequence(cases::prefix(variant), kind),
                    input: call.words(),
                    before_failure: kind.execute(call, true, Some(limit)).unwrap_err(),
                });
            }
            probe::check_sequences_with_limits(
                &kind.source(variant),
                &healthy,
                &failures,
                &[
                    ("returned", cases::OBSERVER),
                    ("extent", cases::BOUND_OBSERVER),
                    ("checkpoint", cases::FALLBACK_OBSERVER),
                ],
                limit,
                kind.helper_entry_limit(),
            );
        }
    }
}

#[test]
fn shared_return_storage_prefix_native_stops_at_a_failing_initializer_before_later_prints() {
    check_failing_prefix(
        cases::SourceKind::Prefix,
        "let mut carry = State { result: flag, total: total,",
        "let mut carry = State { result: flag, total: total + (1 / root - 1),",
        &[5, 701],
    );
}

#[test]
fn shared_return_storage_reassignment_failure_keeps_old_state_and_stops_before_later_prints() {
    check_failing_prefix(
        cases::SourceKind::Reassigned,
        "total: carry.total - 3,",
        "total: carry.total - 3 + (1 / root - 1),",
        &[5, 701, 703],
    );
}

fn check_failing_prefix(kind: cases::SourceKind, old: &str, replacement: &str, prefix: &[i64]) {
    check_prefix_failure(
        kind,
        |variant| {
            let source = kind.source(variant);
            assert_eq!(source.matches(old).count(), 1);
            source.replace(old, replacement)
        },
        prefix,
        0,
    );
}

#[test]
fn shared_return_storage_helper_calls_argument_failure_skips_later_argument_and_callee() {
    check_prefix_failure(
        cases::SourceKind::HelperCalls,
        |variant| cases::SourceKind::HelperCalls.source(variant),
        &[5, 701, 710, 711, 714, 712, 713, 703, 710, 711],
        -7,
    );
}

#[test]
fn shared_return_storage_helper_calls_callee_failure_skips_completion_and_publication() {
    check_prefix_failure(
        cases::SourceKind::HelperCalls,
        |variant| cases::SourceKind::HelperCalls.source(variant),
        &[5, 701, 710, 711, 714, 712, 713, 703, 710, 711, 714, 712],
        -8,
    );
}

fn check_prefix_failure(
    kind: cases::SourceKind,
    source: impl Fn(cases::Variant) -> String,
    prefix: &[i64],
    bad_root: i64,
) {
    for variant in cases::variants()
        .into_iter()
        .filter(|v| !v.mirrored && v.leading == v.parent_leading)
    {
        let source = source(variant);
        let good = cases::base(variant);
        let mut bad = good;
        bad.previous = Some(good.next());
        bad.root = bad_root;
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            let (mut session, _) =
                ApplicationSession::open_registered(&module, &registry, "counter", values(good))
                    .unwrap();
            assert_state(&session, good);
            assert!(session.event(values(bad)).is_err());
            assert_state(&session, good);
            assert_eq!(
                session.phase(),
                yir_runtime_host::ApplicationSessionPhase::Faulted
            );
            assert!(session.event(values(good)).is_err());
            session.close(vec![]).unwrap();
            assert_state(&session, good);
            assert!(session.completion_status().is_err());
        }
        probe::check_sequences_with_limits(
            &source,
            &[sequence(vec![good], kind)],
            &[probe::SequenceFailure {
                accepted: sequence(vec![good], kind),
                input: bad.words(),
                before_failure: prefix.to_vec(),
            }],
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
fn shared_return_storage_loop_helper_calls_later_argument_failure_retains_actual_state() {
    check_loop_call_failure(false);
}

#[test]
fn shared_return_storage_loop_helper_calls_later_callee_failure_retains_actual_state() {
    check_loop_call_failure(true);
}

#[test]
fn shared_return_storage_branch_helper_calls_selected_argument_failures_keep_actual_state() {
    for odd in [false, true] {
        check_branch_call_failure(false, odd);
    }
}

#[test]
fn shared_return_storage_branch_helper_calls_selected_callee_failures_keep_actual_state() {
    for odd in [false, true] {
        check_branch_call_failure(true, odd);
    }
}

#[test]
fn shared_return_storage_branch_helper_calls_entry_limit_100_still_rejects_long_callback() {
    let kind = cases::SourceKind::BranchHelperCalls;
    let variant = cases::variants()
        .into_iter()
        .find(|v| v.leading && !v.parent_leading && !v.mirrored)
        .unwrap();
    let call = cases::child_continued(variant);
    probe::check_helper_entry_limit_rejection(
        &kind.source(variant),
        &[sequence(vec![call], kind)],
        &[
            ("returned", cases::OBSERVER),
            ("extent", cases::BOUND_OBSERVER),
            ("checkpoint", cases::FALLBACK_OBSERVER),
        ],
        cases::LOOP_LIMIT,
        100,
    );
}

fn check_branch_call_failure(callee: bool, odd: bool) {
    let kind = cases::SourceKind::BranchHelperCalls;
    for variant in cases::variants()
        .into_iter()
        .filter(|v| !v.mirrored && v.leading == v.parent_leading)
    {
        let source = kind.source(variant);
        let prefix = cases::prefix(variant);
        let bad = cases::after_prefix(cases::branch_calls::failure(variant, callee, odd));
        let trace = kind.execute(bad, false, None).unwrap_err();
        let base = if odd { 920 } else { 910 };
        let suffix = if callee {
            vec![10, base, base + 1, base + 4, base + 2]
        } else {
            vec![10, base, base + 1]
        };
        assert!(trace.ends_with(&suffix));
        assert!(trace.iter().filter(|v| matches!(**v, 913 | 923)).count() >= 2);
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
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
            assert_state(&session, *prefix.last().unwrap());
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
        probe::check_sequences_with_limits(
            &source,
            &[sequence(prefix.clone(), kind)],
            &[probe::SequenceFailure {
                accepted: sequence(prefix, kind),
                input: bad.words(),
                before_failure: kind.execute(bad, true, None).unwrap_err(),
            }],
            &[
                ("returned", cases::OBSERVER),
                ("extent", cases::BOUND_OBSERVER),
                ("checkpoint", cases::FALLBACK_OBSERVER),
            ],
            cases::LOOP_LIMIT,
            kind.helper_entry_limit(),
        );
    }
}

fn check_loop_call_failure(callee: bool) {
    let kind = cases::SourceKind::LoopHelperCalls;
    for variant in cases::variants()
        .into_iter()
        .filter(|v| !v.mirrored && v.leading == v.parent_leading)
    {
        let source = kind.source(variant);
        let prefix = cases::prefix(variant);
        let bad = cases::after_prefix(cases::loop_calls::failure(variant, callee));
        let trace = kind.execute(bad, false, None).unwrap_err();
        assert!(trace.iter().filter(|v| **v == 813).count() >= 2);
        assert!(trace.ends_with(if callee {
            &[10, 810, 811, 814, 812]
        } else {
            &[10, 810, 811]
        }));
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
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
            assert_state(&session, *prefix.last().unwrap());
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
        probe::check_sequences_with_limits(
            &source,
            &[sequence(prefix.clone(), kind)],
            &[probe::SequenceFailure {
                accepted: sequence(prefix, kind),
                input: bad.words(),
                before_failure: kind.execute(bad, true, None).unwrap_err(),
            }],
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
