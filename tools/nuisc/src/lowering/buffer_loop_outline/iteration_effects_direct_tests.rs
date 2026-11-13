use super::*;
use crate::frontend::parse_nuis_module;

const SOURCE: &str =
    include_str!("../../../tests/native_application_bridge/state_return_direct_eager_pair.ns");

#[test]
fn direct_effectful_records_keep_branch_locals_out_of_loop_carry_normalization() {
    for source in [
        include_str!(
            "../../../tests/native_application_bridge/state_return_local_exit_argument_print.ns"
        ),
        include_str!(
            "../../../tests/native_application_bridge/state_return_eager_exit_argument_print.ns"
        ),
    ] {
        for reverse in [false, true] {
            let mut module = parse_nuis_module(source).unwrap();
            if reverse {
                module.functions.reverse();
            }
            let outlined = outline_buffer_loops(&mut module).unwrap();
            assert!(outlined.functions.contains("advance"));
            assert!(outlined.functions.contains("work"));
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}

#[test]
fn direct_effectful_records_normalize_without_loop_or_pure_authority() {
    for kind in ["f32", "f64"] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&SOURCE.replace("f64", kind)).unwrap();
            if reversed {
                module.functions.reverse();
            }
            assert!(module
                .functions
                .iter()
                .all(|f| !control_loops::contains_loop(&f.body)));
            let layouts = control_values::layouts(&module);
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            let before = module.clone();
            let plan = Plan::collect(&module, &layouts, &pure);
            assert!(plan.contains("work"));
            for name in ["left", "right", "work", "start", "step"] {
                assert!(!pure.contains_key(name));
            }
            for name in ["left", "right", "start", "step", "stop", "main"] {
                assert!(!plan.contains(name), "straight-line entry: {name}");
            }
            assert_eq!(module, before);
            let outlined = outline_buffer_loops(&mut module).unwrap();
            for name in ["left", "right", "work"] {
                assert!(outlined.functions.contains(name));
            }
            assert!(!outlined.guarded_functions.is_empty());
            assert!(module
                .functions
                .iter()
                .all(|f| !control_loops::contains_loop(&f.body)));
            assert!(!module
                .functions
                .iter()
                .any(|f| f.name.starts_with("__nuis_scalar_iteration_")));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let pure =
                scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(!pure.contains_key("work"));
        }
    }
}

#[test]
fn direct_effectful_records_keep_dependency_scope_and_type_rejections() {
    for (from, to) in [
        ("let local: bool", "let local: i64"),
        ("left(left_value, left_guard)", "left(local, left_guard)"),
        (
            "left(left_value, left_guard)",
            "missing(left_value, left_guard)",
        ),
        ("print(20);", "print(true);"),
        ("return value;", "return left(value, guard);"),
        ("value + delta", "value + selected"),
        ("print(30);", "print(left(left_value, left_guard));"),
    ] {
        let text = SOURCE.replace(from, to);
        assert_ne!(text, SOURCE);
        for reverse in [false, true] {
            let Ok(mut module) = parse_nuis_module(&text) else {
                // Frontend type/scope rejection is earlier than profile admission.
                continue;
            };
            if reverse {
                module.functions.reverse();
            }
            let layouts = control_values::layouts(&module);
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            assert!(
                !Plan::collect(&module, &layouts, &pure).contains("work"),
                "{from} -> {to}"
            );
        }
    }
}

#[test]
fn direct_effectful_records_leave_standalone_scalar_effect_routes_unchanged() {
    let source = "mod cpu Main {
        fn scalar(gate: bool) -> i64 {
            if gate { print(1); return 2; } else { print(3); return 4; }
        }
        fn main() -> i64 { return scalar(true); }
    }";
    let module = parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
    let effects = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(effects.contains_key("scalar"));
    assert!(!pure.contains_key("scalar"));
    assert!(!Plan::collect(&module, &layouts, &pure).contains("scalar"));
}
