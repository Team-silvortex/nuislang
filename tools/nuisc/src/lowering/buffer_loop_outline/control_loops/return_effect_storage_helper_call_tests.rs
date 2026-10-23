use super::tests::{cases, plans};
use super::*;

#[test]
fn effect_return_storage_helper_calls_keep_catalog_authority_and_exact_prefix() {
    for variant in cases::variants() {
        for name in ["carry", "working_copy", "__nuis_return_value_0"] {
            let mut source = cases::SourceKind::HelperCalls
                .source(variant)
                .replace("carry", name);
            for (old, new) in [
                ("prefix_seed", "capture"),
                ("prefix_delta", "offset"),
                ("prefix_gate", "gate_value"),
                ("prefix_record", "transform"),
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
            assert_eq!(&shared.body[1..7], &function.body[..6]);
            assert_eq!(shared.signal.name(), independent.signal.name());
            assert!(verify(function, &independent, &shared, name, &layouts, &catalog).is_some());
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            for callee in ["capture", "offset", "gate_value", "transform"] {
                assert!(catalog.contains_key(callee));
                assert!(!pure.contains_key(callee));
                let mut denied = catalog.clone();
                denied.remove(callee);
                assert!(eligible(function, name, &layouts, &denied).is_none());
                assert!(prepare_effects(function, &layouts, &denied).is_none());
            }
            assert_eq!(module, before);
        }
    }
}

#[test]
fn effect_return_storage_helper_calls_reject_typed_reordered_deleted_or_duplicated_effects() {
    let (module, layouts, catalog) = plans(
        &cases::SourceKind::HelperCalls.source(cases::variants()[0]),
        "work",
    );
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    for mutation in 0..5 {
        let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        match mutation {
            0 => shared.body.swap(3, 5),
            1 => {
                shared.body.remove(3);
            }
            2 => shared.body.insert(3, shared.body[3].clone()),
            _ => {
                let NirStmt::Let {
                    value: NirExpr::Call { args, .. },
                    ..
                } = &mut shared.body[3]
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
        assert!(scalar_helpers::validate_iteration_body(
            function,
            &shared.body,
            &catalog,
            &layouts
        )
        .is_some());
        assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    }
}

#[test]
fn effect_return_storage_helper_calls_keep_candidate_fallback_and_reject_unadmitted_callees() {
    let source = cases::with_helper_calls(cases::source_with_alternatives(cases::variants()[0], 1));
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
    let source = source.replace("print(714); return root;", "print(root); return root;");
    let module = crate::frontend::parse_nuis_module(&source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(!catalog.contains_key("prefix_gate"));
    assert!(!catalog.contains_key("work"));
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    assert!(prepare_effects(function, &layouts, &catalog).is_none());
}
