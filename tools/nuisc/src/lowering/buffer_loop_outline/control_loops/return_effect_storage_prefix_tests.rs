use super::tests::{cases, plans};
use super::*;

#[test]
fn effect_return_storage_prefix_discovers_dominating_records_without_pure_authority() {
    for variant in cases::variants() {
        for name in ["carry", "working_copy", "__nuis_return_value_0"] {
            let source = cases::source_with_prefix(variant).replace("carry", name);
            let (module, layouts, catalog) = plans(&source, "work");
            let function = module.functions.iter().find(|f| f.name == "work").unwrap();
            let before = function.clone();
            assert!(
                super::super::super::storage::candidate(function, &layouts, &catalog).is_none()
            );
            let independent =
                rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
            let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert_eq!(shared.body.len() + 1, independent.body.len());
            assert_eq!(shared.signal.name(), independent.signal.name());
            assert!(verify(function, &independent, &shared, name, &layouts, &catalog).is_some());
            assert_eq!(&shared.body[1..4], &function.body[..3]);
            assert_eq!(
                prepare_effects(function, &layouts, &catalog)
                    .unwrap()
                    .unwrap()
                    .body,
                shared.body
            );
            assert!(verify_independent(function, &shared, &layouts).is_none());
            assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("work"));
            assert_eq!(function, &before);
        }
    }
}

#[test]
fn effect_return_storage_prefix_cannot_cross_control_flow_or_dynamic_prints() {
    let (module, layouts, catalog) =
        plans(&cases::source_with_prefix(cases::variants()[0]), "work");
    let original = module.functions.iter().find(|f| f.name == "work").unwrap();
    let conditional = NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![],
        else_body: vec![],
    };
    let loop_stmt = original
        .body
        .iter()
        .find(|s| matches!(s, NirStmt::While { .. }))
        .unwrap()
        .clone();
    for barrier in [
        conditional.clone(),
        loop_stmt,
        NirStmt::Print(NirExpr::Var("root".into())),
        NirStmt::Return(Some(NirExpr::Var("missing".into()))),
    ] {
        let mut function = original.clone();
        function.body.insert(0, barrier);
        assert!(
            super::super::super::storage::candidate_effects(&function, &layouts, &catalog)
                .is_none()
        );
        assert!(eligible(&function, "carry", &layouts, &catalog).is_none());
    }
    let mut scoped = original.clone();
    let initializer = scoped.body.remove(1);
    scoped.body[0] = NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![initializer],
        else_body: vec![],
    };
    assert!(eligible(&scoped, "carry", &layouts, &catalog).is_none());

    for barrier in [
        "if root == 99 { print(799); }",
        "let mut pre = 0; while pre < 2 { pre = pre + 1; print(799); }",
    ] {
        let source = cases::source_with_prefix(cases::variants()[0])
            .replace("print(701);", &format!("print(701); {barrier}"));
        let (module, layouts, catalog) = plans(&source, "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        assert!(coalesced_effects(function, &layouts, &catalog, &independent).is_none());
        assert_eq!(
            prepare_effects(function, &layouts, &catalog)
                .unwrap()
                .unwrap()
                .body,
            independent.body
        );
    }
}

#[test]
fn effect_return_storage_prefix_rejects_moved_deleted_or_duplicated_prints() {
    let (module, layouts, catalog) =
        plans(&cases::source_with_prefix(cases::variants()[0]), "work");
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    for mutation in 0..4 {
        let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        match mutation {
            0 => {
                shared.body.swap(1, 3);
            }
            1 => {
                shared.body.remove(1);
            }
            2 => {
                shared.body.insert(1, NirStmt::Print(NirExpr::Int(701)));
            }
            _ => {
                shared.body.swap(1, 2);
            }
        }
        assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    }
}

#[test]
fn effect_return_storage_prefix_rechecks_declarations_and_never_grants_original_reads() {
    let source = "mod cpu Main { struct State { total: i64 }
        fn work(seed: State, limit: i64) -> State {
            print(701); let mut carry = seed; print(702); let mut index = 0;
            while index < limit { index = index + 1; print(7);
                if index == 1 { return State { total: carry.total }; }
                carry = State { total: carry.total + 1 };
            } return carry;
        } fn main() -> i64 { return 0; } }";
    let module = crate::frontend::parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    let shared = rewrite(
        function,
        &layouts,
        Some("carry".into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap();
    assert!(
        scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
            .is_some()
    );
    assert!(scalar_helpers::validate_iteration_body(
        function,
        &independent.body,
        &catalog,
        &layouts
    )
    .is_none());
    assert!(!catalog.contains_key("work"));
    assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    assert!(prepare_effects(function, &layouts, &catalog).is_none());

    let (module, layouts, catalog) =
        plans(&cases::source_with_prefix(cases::variants()[0]), "work");
    let original = module.functions.iter().find(|f| f.name == "work").unwrap();
    for mutation in 0..3 {
        let mut function = original.clone();
        let initializer = function.body[1].clone();
        match mutation {
            0 => {
                function.body.insert(3, initializer);
            }
            1 => {
                let NirStmt::Let { name, value, .. } = initializer else {
                    unreachable!()
                };
                function.body[1] = NirStmt::Const {
                    name,
                    value,
                    ty: scalar_type("State"),
                };
            }
            _ => {
                let NirStmt::Let { ty, .. } = &mut function.body[1] else {
                    unreachable!()
                };
                *ty = Some(scalar_type("i64"));
            }
        }
        assert_eq!(
            eligible(&function, "carry", &layouts, &catalog).is_some(),
            mutation == 0
        );
    }
}

#[test]
fn effect_return_storage_prefix_keeps_independent_fallback_for_new_inner_writes() {
    let source = include_str!(
        "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    )
    .replace(
        "let carry = state;",
        "print(701); let carry = state; print(702);",
    )
    .replace(
        "let j = j + 1;",
        "let j = j + 1; print(7); let snapshot = carry.right;",
    );
    let (module, layouts, catalog) = plans(&source, "step");
    let function = module.functions.iter().find(|f| f.name == "step").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    assert!(eligible(function, "carry", &layouts, &catalog).is_some());
    assert!(coalesced_effects(function, &layouts, &catalog, &independent).is_none());
    assert_eq!(
        prepare_effects(function, &layouts, &catalog)
            .unwrap()
            .unwrap()
            .body,
        independent.body
    );
}
