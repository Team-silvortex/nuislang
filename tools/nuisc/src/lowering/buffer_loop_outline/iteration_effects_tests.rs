use super::*;
use crate::frontend::parse_nuis_module;

const BASE: &str = include_str!("../../../tests/native_application_bridge/state_loop.ns");

#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_cases.rs"]
mod cases;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_statements_profile.rs"]
mod statements;

#[test]
fn iteration_effects_direct_session_source_has_complete_shape_admission() {
    for text in statements::source_variants() {
        let module = parse_nuis_module(&text).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(!pure.contains_key("walk"));
        assert!(Plan::collect(&module, &layouts, &pure).contains("walk"));
    }
}

fn source() -> String {
    BASE.replace(
        "{ return value == (proof == 1); }",
        "{ print(10); return value == (proof == 1); }",
    )
    .replace(
        "{ return value * 2 + proof + index; }",
        "{ print(20); return value * 2 + proof + index; }",
    )
}

#[test]
fn iteration_effects_unvalidated_dependencies_never_gain_provisional_authority() {
    for callee in ["missing", "refine", "choose"] {
        let mut module = parse_nuis_module(&source()).unwrap();
        let leaf = module
            .functions
            .iter_mut()
            .find(|f| f.name == "refine")
            .unwrap();
        *leaf.body.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Call {
            callee: callee.into(),
            args: vec![
                NirExpr::Var("value".into()),
                NirExpr::Var("proof".into()),
                NirExpr::Var("index".into()),
            ],
        }));
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{callee}"
        );
    }
}

#[test]
fn iteration_effects_catalog_is_separate_and_retains_real_prints() {
    let mut module = parse_nuis_module(&source()).unwrap();
    let layouts = control_values::layouts(&module);
    let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
    for name in ["choose", "refine", "update", "walk"] {
        assert!(!pure.contains_key(name), "{name} gained pure authority");
    }
    let before = module.clone();
    let plan = Plan::collect(&module, &layouts, &pure);
    assert!(plan.contains("walk"));
    for name in ["choose", "refine", "start", "step"] {
        assert!(!plan.contains(name));
    }
    assert_eq!(module, before, "discovery mutated source");
    let outlined = outline_buffer_loops(&mut module).unwrap();
    for name in ["walk", "update", "choose", "refine"] {
        assert!(outlined.functions.contains(name));
    }
    for name in ["choose", "refine"] {
        let f = module.functions.iter().find(|f| f.name == name).unwrap();
        assert!(matches!(
            f.body.first(),
            Some(NirStmt::Print(NirExpr::Int(_)))
        ));
    }
    assert!(module
        .functions
        .iter()
        .any(|f| f.name.starts_with("__nuis_scalar_iteration_")));
    assert!(
        !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
            .contains_key("walk")
    );
}

#[test]
fn iteration_effects_reject_cycles_types_and_unscoped_effects() {
    for replacement in [
        "print(true); return value * 2 + proof + index;",
        "print(20); return refine(value, proof, index);",
        "print(20); return missing(value);",
        "print(20); return choose(value, proof);",
    ] {
        let text = source().replace("print(20); return value * 2 + proof + index;", replacement);
        let Ok(module) = parse_nuis_module(&text) else {
            continue;
        };
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{replacement}"
        );
    }
    for replacement in ["return current; current = update(current, gate, root, hole, index);"] {
        let text = source().replace(
            "current = update(current, gate, root, hole, index);",
            replacement,
        );
        let module = parse_nuis_module(&text).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{replacement}"
        );
    }
}

#[test]
fn iteration_effects_direct_and_print_only_loops_keep_explicit_authority() {
    for body in [
        "print(7); index = index + 1;",
        "index = index + 1; if gate { print(7); } else { print(8); }",
        "print(7); current = current + 1; index = index + 1;",
        "let mut child = 0; while child < 2 { print(7); child = child + 1; } index = index + 1;",
    ] {
        let text = BASE.replace(
            "current = update(current, gate, root, hole, index);\n            index = index + 1;",
            body,
        );
        let mut module = parse_nuis_module(&text).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(!pure.contains_key("walk"), "{body}");
        assert!(
            Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{body}"
        );
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(outlined.functions.contains("walk"));
        assert!(
            module
                .functions
                .iter()
                .any(|f| f.name.starts_with("__nuis_scalar_iteration_")),
            "{body}"
        );
        assert!(
            module
                .functions
                .iter()
                .any(|f| f.name.starts_with("__nuis_") && validation::contains_print(&f.body)),
            "{body}"
        );
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}

#[test]
fn iteration_effects_direct_statements_reject_wrong_types_and_foreign_effects() {
    for effect in [
        "print(true);",
        "print(1.0);",
        "choose(true, 1);",
        "return current;",
    ] {
        let text = BASE.replace(
            "current = update(current, gate, root, hole, index);",
            effect,
        );
        let module = parse_nuis_module(&text).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{effect}"
        );
    }
}

#[test]
fn iteration_effects_direct_validation_does_not_grant_buffer_or_pure_authority() {
    let catalog = ScalarHelpers::new();
    let layouts = control_values::CarryLayouts::new();
    let mutations = MutationScope {
        writable: BTreeSet::new(),
        protected: BTreeSet::new(),
    };
    for (types, accepted) in [
        (validation::EffectTypes::Buffer(&catalog), false),
        (
            validation::EffectTypes::Values(&catalog, &layouts, validation::LiteralPrints::Reject),
            false,
        ),
        (
            validation::EffectTypes::Values(&catalog, &layouts, validation::LiteralPrints::Allow),
            true,
        ),
    ] {
        assert_eq!(
            validate_effects(
                &[NirStmt::Print(NirExpr::Int(7))],
                &mut Scope::new(),
                &mut BTreeSet::new(),
                types,
                &mutations,
                &mut vec![]
            )
            .is_some(),
            accepted
        );
    }
}
