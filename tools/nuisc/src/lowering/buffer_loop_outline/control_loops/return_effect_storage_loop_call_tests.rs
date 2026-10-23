use super::tests::{cases, plans};
use super::*;

#[test]
fn effect_return_storage_loop_calls_preserve_exact_work_without_new_catalog_authority() {
    for variant in cases::variants() {
        for name in ["carry", "working_copy", "__nuis_return_value_0"] {
            let mut source = cases::SourceKind::LoopHelperCalls
                .source(variant)
                .replace("carry", name);
            for (old, new) in [
                ("loop_seed", "capture"),
                ("loop_delta", "offset"),
                ("loop_gate", "gate_value"),
                ("loop_record", "transform"),
            ] {
                source = source.replace(old, new);
            }
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
            for callee in ["capture", "offset", "gate_value", "transform"] {
                assert!(catalog.contains_key(callee));
                assert!(!pure.contains_key(callee));
                let mut denied = catalog.clone();
                denied.remove(callee);
                assert!(prepare_effects(function, &layouts, &denied).is_none());
                assert!(verify(function, &independent, &shared, name, &layouts, &denied).is_none());
            }
            assert_eq!(module, before);
        }
    }
}

fn mutate_assignment(body: &mut Vec<NirStmt>, mutation: usize) -> bool {
    for index in 0..body.len() {
        // Removing the first loop write would invalidate its subsequent reads.
        // Mutate a later parent-exit write so ordinary admission still succeeds.
        if index > 0
            && matches!(&body[index - 1], NirStmt::Print(NirExpr::Int(25)))
            && matches!(&body[index], NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "loop_record")
        {
            match mutation {
                0 => {
                    body.remove(index);
                }
                1 => body.insert(index, body[index].clone()),
                2 => body.swap(index - 1, index),
                _ => {
                    let NirStmt::Let {
                        value: NirExpr::Call { args, .. },
                        ..
                    } = &mut body[index]
                    else {
                        unreachable!()
                    };
                    if mutation == 3 {
                        args.swap(1, 2);
                    } else {
                        args[0] = NirExpr::Var("carry".into());
                    }
                }
            }
            return true;
        }
        match &mut body[index] {
            NirStmt::While { body, .. } => {
                if mutate_assignment(body, mutation) {
                    return true;
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if mutate_assignment(then_body, mutation) || mutate_assignment(else_body, mutation)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn effect_return_storage_loop_calls_reject_type_valid_changed_iteration_effects() {
    let (module, layouts, catalog) = plans(
        &cases::SourceKind::LoopHelperCalls.source(cases::variants()[0]),
        "work",
    );
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    for mutation in 0..5 {
        let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        assert!(mutate_assignment(&mut shared.body, mutation));
        assert!(
            scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
                .is_some(),
            "mutation {mutation}"
        );
        assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    }
}

#[test]
fn effect_return_storage_loop_calls_keep_candidate_fallback_and_original_read_rejection() {
    let source = cases::loop_calls::with_loop_calls(cases::with_helper_calls(
        cases::source_with_alternatives(cases::variants()[0], 1),
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
fn effect_return_storage_loop_oracle_distinguishes_assignment_events_from_observed_words() {
    let kind = cases::SourceKind::LoopHelperCalls;
    for variant in cases::variants() {
        let mut call = cases::base(variant);
        call.input.gain = 8;
        call.input.marker = 9;
        call.select_first(true);
        let (plain_state, plain) = kind.execute(call, false, None).unwrap();
        let (observed_state, observed) = kind.execute(call, true, None).unwrap();
        assert_eq!(plain_state.words(), observed_state.words());
        assert!(observed.contains(&30));
        let count = |trace: &[i64]| {
            trace
                .windows(5)
                .filter(|window| *window == [810, 811, 814, 812, 813])
                .count()
        };
        assert!(count(&plain) >= 2);
        assert_eq!(count(&plain), count(&observed));
    }
}
