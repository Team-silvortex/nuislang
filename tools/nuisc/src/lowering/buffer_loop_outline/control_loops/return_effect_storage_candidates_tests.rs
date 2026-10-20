use super::super::super::storage as discovery;
use super::tests::{cases, plans};
use super::*;

fn shared(function: &NirFunction, layouts: &control_values::CarryLayouts, name: &str) -> Prepared {
    rewrite(
        function,
        layouts,
        Some(name.into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap()
}

#[test]
fn effect_return_storage_candidates_reject_first_and_certify_later_without_partial_publication() {
    for variant in cases::variants() {
        for name in ["carry", "a_later", "__nuis_return_value_0"] {
            let source = cases::source_with_alternatives(variant, 1).replace("carry", name);
            let (module, layouts, catalog) = plans(&source, "work");
            let function = module.functions.iter().find(|f| f.name == "work").unwrap();
            let before = function.clone();
            let independent =
                rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
            let baseline = independent.body.clone();
            assert_eq!(
                discovery::candidate_effects(function, &layouts, &catalog).unwrap(),
                ["candidate_0", name]
            );
            assert!(eligible(function, "candidate_0", &layouts, &catalog).is_some());
            let rejected = shared(function, &layouts, "candidate_0");
            assert!(scalar_helpers::validate_iteration_body(
                function,
                &rejected.body,
                &catalog,
                &layouts
            )
            .is_none());
            assert!(verify(
                function,
                &independent,
                &rejected,
                "candidate_0",
                &layouts,
                &catalog
            )
            .is_none());
            let expected = shared(function, &layouts, name);
            assert!(verify(function, &independent, &expected, name, &layouts, &catalog).is_some());
            assert_eq!(
                coalesced_effects(function, &layouts, &catalog, &independent)
                    .unwrap()
                    .body,
                expected.body
            );
            assert_eq!(
                prepare_effects(function, &layouts, &catalog)
                    .unwrap()
                    .unwrap()
                    .body,
                expected.body
            );
            assert_eq!(independent.body, baseline);
            assert_eq!(function, &before);
        }
    }
}

#[test]
fn effect_return_storage_candidates_use_declaration_order_not_sorted_names_or_function_order() {
    let source = cases::source_with_alternatives(cases::variants()[0], 2)
        .replace("candidate_0", "z_first")
        .replace("candidate_1", "a_second")
        .replace(
            "let snapshot_0 = z_first.total;",
            "z_first = z_first; let snapshot_0 = z_first.total;",
        )
        .replace(
            "let snapshot_1 = a_second.total;",
            "a_second = a_second; let snapshot_1 = a_second.total;",
        );
    let (mut module, layouts, _) = plans(&source, "work");
    for _ in 0..2 {
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        assert_eq!(
            discovery::candidate_effects(function, &layouts, &catalog).unwrap(),
            ["z_first", "a_second", "carry"]
        );
        for name in ["z_first", "a_second", "carry"] {
            assert!(verify(
                function,
                &independent,
                &shared(function, &layouts, name),
                name,
                &layouts,
                &catalog
            )
            .is_some());
        }
        assert_eq!(
            coalesced_effects(function, &layouts, &catalog, &independent)
                .unwrap()
                .body,
            shared(function, &layouts, "z_first").body
        );
        module.functions.reverse();
    }
}

#[test]
fn effect_return_storage_candidates_bound_attempts_and_keep_exact_fallback_after_exhaustion() {
    let cap = discovery::MAX_EFFECT_STORAGE_CANDIDATES;
    assert_eq!(cap, 8);
    for rejected in [cap - 1, cap] {
        let (module, layouts, catalog) = plans(
            &cases::source_with_alternatives(cases::variants()[0], rejected),
            "work",
        );
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        let candidates = discovery::candidate_effects(function, &layouts, &catalog).unwrap();
        assert_eq!(candidates.len(), cap);
        assert_eq!(
            candidates.last().unwrap(),
            if rejected < cap {
                "carry"
            } else {
                "candidate_7"
            }
        );
        for name in candidates
            .iter()
            .filter(|name| name.starts_with("candidate_"))
        {
            assert!(verify(
                function,
                &independent,
                &shared(function, &layouts, name),
                name,
                &layouts,
                &catalog
            )
            .is_none());
        }
        let selected = coalesced_effects(function, &layouts, &catalog, &independent);
        let expected = if rejected < cap {
            assert!(selected.is_some());
            shared(function, &layouts, "carry").body
        } else {
            assert!(selected.is_none());
            assert!(verify(
                function,
                &independent,
                &shared(function, &layouts, "carry"),
                "carry",
                &layouts,
                &catalog
            )
            .is_some());
            independent.body
        };
        assert_eq!(
            prepare_effects(function, &layouts, &catalog)
                .unwrap()
                .unwrap()
                .body,
            expected
        );
    }
}

#[test]
fn effect_return_storage_candidates_retain_barriers_and_the_pure_first_candidate_policy() {
    let source = cases::source_with_alternatives(cases::variants()[0], 1);
    for barrier in [
        "if root == 99 { print(799); }",
        "let mut pre = 0; while pre < 2 { pre = pre + 1; print(799); }",
    ] {
        let source = source.replace("let mut carry =", &format!("{barrier} let mut carry ="));
        let (module, layouts, catalog) = plans(&source, "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        assert_eq!(
            discovery::candidate_effects(function, &layouts, &catalog).unwrap(),
            ["candidate_0"]
        );
        assert!(coalesced_effects(function, &layouts, &catalog, &independent).is_none());
        assert_eq!(
            prepare_effects(function, &layouts, &catalog)
                .unwrap()
                .unwrap()
                .body,
            independent.body
        );
    }
    let source = source.replace("print(701);", "").replace("print(702);", "");
    let (module, layouts, catalog) = plans(&source, "work");
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    assert_eq!(
        discovery::candidate(function, &layouts, &catalog).as_deref(),
        Some("candidate_0")
    );
}

#[test]
fn effect_return_storage_candidates_never_bypass_original_read_authority() {
    let source = "mod cpu Main { struct State { total: i64 }
        fn work(seed: State, limit: i64) -> State {
            print(701); let mut first = seed; let mut carry = seed; print(702); let mut index = 0;
            while index < limit { index = index + 1; print(7); first = first;
                if index == 1 { return State { total: carry.total }; }
                carry = State { total: carry.total + 1 };
            } return carry;
        } fn main() -> i64 { return 0; } }";
    let module = crate::frontend::parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    let candidate = shared(function, &layouts, "carry");
    assert!(
        scalar_helpers::validate_iteration_body(function, &candidate.body, &catalog, &layouts)
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
    assert!(verify(
        function,
        &independent,
        &candidate,
        "carry",
        &layouts,
        &catalog
    )
    .is_none());
    assert!(coalesced_effects(function, &layouts, &catalog, &independent).is_none());
    assert!(prepare_effects(function, &layouts, &catalog).is_none());
}
