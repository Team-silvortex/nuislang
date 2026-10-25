use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_returns_cases.rs"]
mod cases;

#[test]
fn iteration_effects_returns_use_explicit_profile_and_private_typed_signal() {
    for variant in cases::variants() {
        let mut module = parse_nuis_module(&variant.source()).unwrap();
        let before = module.clone();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        let walk = module.functions.iter().find(|f| f.name == "walk").unwrap();
        assert!(!pure.contains_key("walk"));
        assert!(catalog.contains_key("walk"));
        assert!(control_loops::returns::normalize(walk, &layouts).is_none());
        let prepared = control_loops::returns::prepare_effects(walk, &layouts, &catalog)
            .unwrap()
            .unwrap();
        assert!(prepared.structs.is_empty());
        assert!(
            scalar_helpers::validate_control_body(walk, &prepared.body, &catalog, &layouts)
                .is_none()
        );
        assert!(
            scalar_helpers::validate_iteration_body(walk, &prepared.body, &catalog, &layouts)
                .is_some()
        );
        assert!(Plan::collect(&module, &layouts, &pure).contains("walk"));
        assert_eq!(module, before, "admission must not mutate source");
        let outlined = outline_buffer_loops(&mut module).unwrap();
        for name in ["walk", "returned"] {
            assert!(outlined.functions.contains(name));
        }
        for f in &module.functions {
            assert!(!control_loops::returns::contains_loop_return(&f.body));
        }
        assert!(!outlined.break_controls.is_empty());
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}

#[test]
fn iteration_effects_returns_admit_ready_i64_prints_without_pure_reclassification() {
    for (old, new) in [
        ("print(12);", "print(current);"),
        ("print(30);", "print(proof);"),
    ] {
        assert!(cases::SOURCE.contains(old));
        let module = parse_nuis_module(&cases::SOURCE.replace(old, new)).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(!pure.contains_key("walk"));
        assert!(Plan::collect(&module, &layouts, &pure).contains("walk"));
    }
}

#[test]
fn iteration_effects_returns_reject_unproven_dependencies_and_nonterminal_returns() {
    for (old, new) in [
        ("print(12);", "print(1.0);"),
        ("print(12);", "print(true);"),
        (
            "return returned(current, index, outer, 1 / root);",
            "return returned(current, index, outer, 1 / root); print(99);",
        ),
        (
            "return returned(current, index, outer, 1 / root);",
            "return missing(current);",
        ),
        (
            "return returned(current, index, outer, 1 / root);",
            "return true;",
        ),
        (
            "return returned(current, index, outer, 1 / root);",
            "return __nuis_return_value;",
        ),
        ("print(30);", "print(true);"),
        ("1 / proof;", "returned(value, index, outer, proof);"),
    ] {
        let source = cases::SOURCE.replace(old, new);
        assert_ne!(source, cases::SOURCE);
        let module = match parse_nuis_module(&source) {
            Ok(module) => module,
            Err(error) => {
                assert!(
                    error.contains("unknown") || error.contains("type"),
                    "{new}: {error}"
                );
                continue;
            }
        };
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{new}"
        );
    }
}

#[test]
fn iteration_effects_returns_keep_exact_bool_float_and_record_value_types() {
    for (ty, value, zero) in [
        ("bool", "index == 0", "false"),
        ("f64", "-2.5", "0.0"),
        (
            "Value",
            "Value { flag: true, total: index }",
            "Value { flag: false, total: 0 }",
        ),
    ] {
        let source = format!("mod cpu Main {{ struct Value {{ flag: bool, total: i64 }}
            fn work(trips: i64) -> {ty} {{ let mut index = 0;
                while index < trips {{ print(7); if index == 0 {{ return {value}; }} index = index + 1; }}
                return {zero}; }} fn main() -> i64 {{ return 0; }} }}");
        let mut module = parse_nuis_module(&source).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(!pure.contains_key("work"));
        assert!(
            Plan::collect(&module, &layouts, &pure).contains("work"),
            "{ty}"
        );
        assert!(outline_buffer_loops(&mut module)
            .unwrap()
            .functions
            .contains("work"));
    }
}

#[test]
fn iteration_effects_returns_oracle_distinguishes_return_prefix_and_shared_budget() {
    for variant in cases::variants() {
        let call = cases::base(variant.leading);
        let (state, trace) = call.execute(None, false, Some(5)).unwrap();
        assert_eq!(state.total, if variant.leading { 121 } else { 111 });
        assert_eq!(trace, [5, 8, 10, 11, 12, 30, 35, 40]);
        assert_eq!(call.execute(None, false, Some(4)).unwrap_err(), [5, 8]);
        assert_eq!(
            cases::Invocation { root: 0, ..call }
                .execute(None, false, None)
                .unwrap_err(),
            [5, 8, 10, 11, 12]
        );
        assert_eq!(
            cases::Invocation { root: 2, ..call }
                .execute(None, false, None)
                .unwrap_err(),
            [5, 8, 10, 11, 12, 30]
        );
        let skips = cases::Invocation {
            rounds: 0,
            trips: i64::MAX,
            root: 0,
            ..call
        };
        assert_eq!(skips.trace(false), [5, 31, 35, 40]);
    }
}
