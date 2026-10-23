use super::tests::{cases, plans};
use super::*;

#[test]
fn effect_return_storage_branch_bound_calls_keep_catalog_authority_and_immutable_source() {
    for variant in cases::variants() {
        for name in ["carry", "working_copy", "__nuis_return_value_0"] {
            let source = cases::SourceKind::BranchBoundCalls
                .source(variant)
                .replace("carry", name)
                .replace("bound_", "extent_record_");
            let (module, layouts, catalog) = plans(&source, "work");
            let before = module.clone();
            let function = module.functions.iter().find(|f| f.name == "work").unwrap();
            let independent =
                rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
            let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert_eq!(shared.body.len() + 1, independent.body.len());
            assert_eq!(shared.signal.name(), independent.signal.name());
            assert!(verify(function, &independent, &shared, name, &layouts, &catalog).is_some());
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            for callee in ["extent_record_seed", "extent_record_extent", "extent"] {
                assert!(catalog.contains_key(callee));
                assert!(!pure.contains_key(callee));
                let mut denied = catalog.clone();
                denied.remove(callee);
                if callee == "extent" {
                    // Catalog entries certify complete callees. Recheck the
                    // direct caller, not an unrelated already-certified entry.
                    let leaf = module
                        .functions
                        .iter()
                        .find(|f| f.name == "extent_record_extent")
                        .unwrap();
                    assert!(scalar_helpers::validate_iteration_body(
                        leaf, &leaf.body, &denied, &layouts
                    )
                    .is_none());
                } else {
                    assert!(prepare_effects(function, &layouts, &denied).is_none());
                    assert!(
                        verify(function, &independent, &shared, name, &layouts, &denied).is_none()
                    );
                }
            }
            assert_eq!(module, before);
        }
    }
}

fn mutate_bound(body: &mut [NirStmt], mutation: usize) -> bool {
    for index in 0..body.len() {
        if let NirStmt::Let {
            name,
            value: NirExpr::Call { args, .. },
            ..
        } = &mut body[index]
        {
            if name == "width" {
                match mutation {
                    0 => args[0] = NirExpr::Var("carry".into()),
                    1 => args.swap(1, 2),
                    2 => args[2] = NirExpr::Int(1),
                    3 => {
                        assert!(matches!(body[index - 1], NirStmt::Print(NirExpr::Int(6))));
                        body.swap(index - 1, index);
                    }
                    _ => unreachable!(),
                }
                return true;
            }
        }
        match &mut body[index] {
            NirStmt::While { body, .. } => {
                if mutate_bound(body, mutation) {
                    return true;
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if mutate_bound(then_body, mutation) || mutate_bound(else_body, mutation) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn effect_return_storage_branch_bound_calls_reject_typed_bypasses_reordering_and_changed_arguments()
{
    let (module, layouts, catalog) = plans(
        &cases::SourceKind::BranchBoundCalls.source(cases::variants()[0]),
        "work",
    );
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    for mutation in 0..4 {
        let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        assert!(mutate_bound(&mut shared.body, mutation));
        assert!(
            scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
                .is_some(),
            "mutation {mutation}"
        );
        assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    }
}

#[test]
fn effect_return_storage_branch_bound_calls_preserve_fallback_and_original_read_rejection() {
    let variant = cases::variants()[0];
    let source = cases::bound_calls::with_bounds(cases::branch_calls::with_branches(
        cases::loop_calls::with_loop_calls(cases::with_helper_calls(
            cases::source_with_alternatives(variant, 1),
        )),
        false,
    ));
    let (module, layouts, catalog) = plans(&source, "work");
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    let rejected = rewrite(
        function,
        &layouts,
        Some("candidate_0".into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap();
    assert!(verify(
        function,
        &independent,
        &rejected,
        "candidate_0",
        &layouts,
        &catalog
    )
    .is_none());
    let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
    assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_some());
    let source = source.replace("print(10);", "print(10); let forbidden = carry.total;");
    let module = crate::frontend::parse_nuis_module(&source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    assert!(!catalog.contains_key("work"));
    assert!(prepare_effects(function, &layouts, &catalog).is_none());
}

#[test]
fn effect_return_storage_branch_bound_calls_oracle_depends_on_completed_state_not_input_trips_alone(
) {
    let kind = cases::SourceKind::BranchBoundCalls;
    for variant in cases::variants() {
        let call = cases::bound_calls::state_dependent(variant);
        let (state, trace) = kind.execute(call, true, None).unwrap();
        let (old, old_trace) = cases::SourceKind::BranchHelperCalls
            .execute(call, true, None)
            .unwrap();
        assert_ne!(state.words(), old.words());
        assert_eq!(trace.iter().filter(|&&v| v == 930).count(), 1);
        assert!(trace
            .windows(7)
            .any(|w| w == [930, 931, 932, 933, 1, 1, 16]));
        assert_eq!(trace.iter().filter(|&&v| v == 10).count(), 1);
        assert_eq!(old_trace.iter().filter(|&&v| v == 10).count(), 3);
        for rows in kind.sequences(variant) {
            for pair in rows.windows(2) {
                assert_eq!(pair[1].previous, Some(kind.next(pair[0])));
            }
        }
        let mut skipped = cases::skipped(variant, 0, i64::MAX);
        skipped.root = skipped.first_outer() - 31;
        skipped.after = skipped.first_outer() - 32;
        let (_, trace) = kind.execute(skipped, false, Some(2)).unwrap();
        assert!(!trace.iter().any(|v| matches!(v, 930..=933)));
    }
}
