use super::*;

const KIND: cases::SourceKind = cases::SourceKind::BranchBoundCalls;

#[test]
fn branch_bound_calls_reference_keeps_actual_state_and_selected_work() {
    check_reference(KIND);
}

#[test]
fn branch_bound_calls_native_keeps_once_only_evaluation_and_state_bits() {
    check_native(KIND);
}

#[test]
fn branch_bound_calls_native_preserves_reached_reservations() {
    check_bounds(KIND);
}

#[test]
fn branch_bound_calls_native_reserves_computed_width_before_any_child_effect() {
    for variant in cases::variants()
        .into_iter()
        .filter(|v| !v.mirrored && v.leading == v.parent_leading)
    {
        let mut call = cases::bound_calls::state_dependent(variant);
        call.trips = 7;
        let good = cases::skipped(variant, 0, i64::MAX);
        call.previous = Some(KIND.next(good));
        call.select_first(true);
        let width = call.trips / 2;
        for limit in [4, 5] {
            let mut failures = vec![];
            let mut healthy = vec![sequence(cases::prefix(variant), KIND)];
            if limit == 5 {
                healthy.push(sequence(vec![good, call], KIND));
            } else {
                let trace = KIND.execute(call, true, Some(limit)).unwrap_err();
                assert!(trace.ends_with(&[width, call.divisor, 16, 7]));
                assert!(!trace.contains(&10));
                failures.push(probe::SequenceFailure {
                    accepted: sequence(vec![good], KIND),
                    input: call.words(),
                    before_failure: trace,
                });
            }
            probe::check_sequences_with_limits(
                &KIND.source(variant),
                &healthy,
                &failures,
                &[
                    ("returned", cases::OBSERVER),
                    ("extent", cases::BOUND_OBSERVER),
                    ("checkpoint", cases::FALLBACK_OBSERVER),
                ],
                limit,
                KIND.helper_entry_limit(),
            );
        }
    }
}

#[test]
fn branch_bound_calls_argument_callee_and_division_failures_keep_last_state() {
    for variant in cases::variants()
        .into_iter()
        .filter(|v| !v.mirrored && v.leading == v.parent_leading)
    {
        let source = KIND.source(variant);
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        let prefix = cases::prefix(variant);
        let mut failures = vec![];
        for stage in 0..4 {
            let prefix = cases::bound_calls::failure_prefix(variant, stage == 3);
            let bad = cases::bound_calls::after_failure_prefix(
                cases::bound_calls::failure(variant, stage),
                &prefix,
            );
            let trace = KIND.execute(bad, false, None).unwrap_err();
            let suffix: &[i64] = match stage {
                0 => &[6, 930],
                1 => &[6, 930, 931, 932],
                _ => &[6, 930, 931, 932, 933, 16],
            };
            assert!(trace.ends_with(suffix), "stage={stage}: {trace:?}");
            assert!(!trace.contains(&10));
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
                assert!(session.event(values(bad)).is_err());
                assert_state_for_kind(&session, *prefix.last().unwrap(), KIND);
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0])).is_err());
                session.close(vec![]).unwrap();
                assert_state_for_kind(&session, *prefix.last().unwrap(), KIND);
                assert!(session.completion_status().is_err());
            }
            failures.push(probe::SequenceFailure {
                accepted: sequence(prefix.clone(), KIND),
                input: bad.words(),
                before_failure: KIND.execute(bad, true, None).unwrap_err(),
            });
        }
        probe::check_sequences_with_limits(
            &source,
            &[sequence(prefix, KIND)],
            &failures,
            &[
                ("returned", cases::OBSERVER),
                ("extent", cases::BOUND_OBSERVER),
                ("checkpoint", cases::FALLBACK_OBSERVER),
            ],
            cases::LOOP_LIMIT,
            KIND.helper_entry_limit(),
        );
    }
}
