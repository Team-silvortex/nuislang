use super::super::super::storage as discovery;
use super::tests::{cases, plans};
use super::*;

#[test]
fn effect_return_storage_reassignment_preserves_each_typed_write_and_independent_pending() {
    for variant in cases::variants() {
        for name in ["carry", "working_copy", "__nuis_return_value_0"] {
            let source = cases::SourceKind::Reassigned
                .source(variant)
                .replace("carry", name);
            let (module, layouts, catalog) = plans(&source, "work");
            let function = module.functions.iter().find(|f| f.name == "work").unwrap();
            let before = function.clone();
            let independent =
                rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
            let baseline = independent.body.clone();
            let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert_eq!(shared.body.len() + 1, independent.body.len());
            assert_eq!(&shared.body[1..7], &function.body[..6]);
            assert_eq!(shared.signal.name(), independent.signal.name());
            assert!(verify(function, &independent, &shared, name, &layouts, &catalog).is_some());
            assert_eq!(
                prepare_effects(function, &layouts, &catalog)
                    .unwrap()
                    .unwrap()
                    .body,
                shared.body
            );
            assert_eq!(independent.body, baseline);
            assert_eq!(function, &before);
            assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("work"));
        }
    }
}

#[test]
fn effect_return_storage_reassignment_refuses_constants_parameters_missing_and_changed_types() {
    let (module, layouts, catalog) = plans(
        &cases::SourceKind::Reassigned.source(cases::variants()[0]),
        "work",
    );
    let original = module.functions.iter().find(|f| f.name == "work").unwrap();
    for mutation in 0..6 {
        let mut function = original.clone();
        match mutation {
            0 | 1 => {
                let index = if mutation == 0 { 1 } else { 2 };
                let NirStmt::Let { name, value, .. } = function.body[index].clone() else {
                    unreachable!()
                };
                function.body[index] = NirStmt::Const {
                    name,
                    value,
                    ty: scalar_type("State"),
                };
            }
            2 => {
                function.body.remove(1);
            }
            3 => {
                let NirStmt::Let { ty, .. } = &mut function.body[2] else {
                    unreachable!()
                };
                *ty = Some(scalar_type("i64"));
            }
            4 => {
                let NirStmt::Let { value, .. } = &mut function.body[2] else {
                    unreachable!()
                };
                *value = NirExpr::Int(9);
            }
            _ => {
                function.body.insert(
                    0,
                    NirStmt::If {
                        condition: NirExpr::Bool(true),
                        then_body: vec![],
                        else_body: vec![],
                    },
                );
            }
        }
        assert!(eligible(&function, "carry", &layouts, &catalog).is_none());
    }
    assert!(eligible(original, "total", &layouts, &catalog).is_none());
    let source = cases::SourceKind::Reassigned.source(cases::variants()[0]);
    assert!(
        crate::frontend::parse_nuis_module(&source.replace("let mut carry =", "let carry ="))
            .is_err()
    );
}

#[test]
fn effect_return_storage_reassignment_certificate_rejects_typed_but_reordered_or_missing_work() {
    let (module, layouts, catalog) = plans(
        &cases::SourceKind::Reassigned.source(cases::variants()[0]),
        "work",
    );
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    for mutation in 0..4 {
        let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        match mutation {
            0 => {
                shared.body.swap(3, 5);
            }
            1 => {
                shared.body.remove(3);
            }
            2 => {
                shared.body.swap(3, 4);
            }
            _ => {
                shared.body.insert(3, shared.body[3].clone());
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
fn effect_return_storage_reassignment_retains_original_read_authority_and_shared_failure_fallback()
{
    let source = "mod cpu Main { struct State { total: i64 }
        fn work(seed: State, limit: i64) -> State {
            print(701); let mut carry = seed; carry = seed; print(703); let mut index = 0;
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
    assert!(eligible(function, "carry", &layouts, &catalog).is_some());
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

    let source = include_str!(
        "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    )
    .replace(
        "let carry = state;",
        "print(701); let carry = state; let carry = carry; print(702);",
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

#[test]
fn effect_return_storage_reassignment_keeps_bounded_alternative_order_without_duplicate_attempts() {
    let source =
        cases::with_reassignments(cases::source_with_alternatives(cases::variants()[0], 1));
    let (module, layouts, catalog) = plans(&source, "work");
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    assert_eq!(
        discovery::candidate_effects(function, &layouts, &catalog).unwrap(),
        ["candidate_0", "carry"]
    );
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
    let expected = rewrite(
        function,
        &layouts,
        Some("carry".into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap();
    assert!(verify(
        function,
        &independent,
        &expected,
        "carry",
        &layouts,
        &catalog
    )
    .is_some());
    assert_eq!(
        coalesced_effects(function, &layouts, &catalog, &independent)
            .unwrap()
            .body,
        expected.body
    );
    assert_eq!(discovery::MAX_EFFECT_STORAGE_CANDIDATES, 8);
}
