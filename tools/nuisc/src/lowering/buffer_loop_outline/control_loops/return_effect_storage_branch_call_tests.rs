use super::tests::{cases, plans};
use super::*;

#[test]
fn effect_return_storage_branch_calls_keep_catalog_authority_and_immutable_source() {
    for variant in cases::variants() {
        for name in ["carry", "working_copy", "__nuis_return_value_0"] {
            let source = cases::SourceKind::BranchHelperCalls
                .source(variant)
                .replace("carry", name)
                .replace("branch_even", "choice_a")
                .replace("branch_odd", "choice_b");
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
            for branch in ["choice_a", "choice_b"] {
                for part in ["seed", "delta", "gate", "record"] {
                    let callee = format!("{branch}_{part}");
                    assert!(catalog.contains_key(&callee));
                    assert!(!pure.contains_key(&callee));
                    let mut denied = catalog.clone();
                    denied.remove(&callee);
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

fn mutate_branch(body: &mut Vec<NirStmt>, mutation: usize) -> bool {
    for index in 0..body.len() {
        if let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut body[index]
        {
            if matches!(then_body.first(), Some(NirStmt::Let { value: NirExpr::Call { callee, .. }, .. }) if callee.starts_with("branch_"))
            {
                match mutation {
                    0 => std::mem::swap(then_body, else_body),
                    1 => then_body.insert(0, then_body[0].clone()),
                    4 => {
                        let hoisted = then_body[0].clone();
                        body.insert(index, hoisted);
                    }
                    _ => {
                        let NirStmt::Let {
                            value: NirExpr::Call { args, .. },
                            ..
                        } = &mut then_body[0]
                        else {
                            unreachable!()
                        };
                        if mutation == 2 {
                            args.swap(1, 2);
                        } else {
                            args[0] = NirExpr::Var("carry".into());
                        }
                    }
                }
                return true;
            }
        }
        match &mut body[index] {
            NirStmt::While { body, .. } => {
                if mutate_branch(body, mutation) {
                    return true;
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if mutate_branch(then_body, mutation) || mutate_branch(else_body, mutation) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn effect_return_storage_branch_calls_reject_typed_hoisting_swaps_duplicates_and_argument_changes()
{
    let (module, layouts, catalog) = plans(
        &cases::SourceKind::BranchHelperCalls.source(cases::variants()[0]),
        "work",
    );
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    for mutation in 0..5 {
        let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        assert!(mutate_branch(&mut shared.body, mutation));
        assert!(
            scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
                .is_some(),
            "mutation {mutation}"
        );
        assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    }
}

#[test]
fn effect_return_storage_branch_calls_preserve_candidate_fallback_and_original_read_rejection() {
    let source = cases::branch_calls::with_branches(
        cases::loop_calls::with_loop_calls(cases::with_helper_calls(
            cases::source_with_alternatives(cases::variants()[0], 1),
        )),
        false,
    );
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
fn effect_return_storage_branch_source_and_oracle_preserve_semantic_roles_under_mirroring() {
    let kind = cases::SourceKind::BranchHelperCalls;
    for variant in cases::variants() {
        let source = kind.source(variant);
        assert!(source
            .split_once("print(10);")
            .unwrap()
            .1
            .trim_start()
            .starts_with("if index % 2"));
        assert!(source.contains("print(9); if outer % 2"));
        let mut call = cases::base(variant);
        call.input.gain = 8;
        call.input.marker = 9;
        call.select_first(true);
        let (plain_state, plain) = kind.execute(call, false, None).unwrap();
        let (observed_state, observed) = kind.execute(call, true, None).unwrap();
        assert_eq!(plain_state.words(), observed_state.words());
        for base in [910, 920] {
            let count = |trace: &[i64]| {
                trace
                    .windows(5)
                    .filter(|w| *w == [base, base + 1, base + 4, base + 2, base + 3])
                    .count()
            };
            assert_eq!(count(&plain), count(&observed));
        }
    }
}
