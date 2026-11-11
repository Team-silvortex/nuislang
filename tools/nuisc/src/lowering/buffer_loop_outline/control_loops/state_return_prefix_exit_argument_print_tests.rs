use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_return_prefix_exit_argument_print.ns"
));

#[test]
fn state_return_prefix_exit_admits_selected_arm_local_prefixes_before_later_work() {
    accepts(SOURCE);
    for kind in ["f32", "f64"] {
        let module = crate::frontend::parse_nuis_module(&SOURCE.replace("f64", kind)).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        let effects = scalar_helpers::collect_iteration_calls(&module, &layouts);
        for name in [
            "negative_update",
            "nonnegative_update",
            "negative_finish",
            "nonnegative_finish",
            "advance_control",
            "work",
        ] {
            assert!(!pure.contains_key(name), "{name} became pure");
            assert!(
                effects.contains_key(name),
                "{name} missing from effect catalog"
            );
        }
    }
}

#[test]
fn state_return_prefix_exit_keeps_types_scope_readiness_and_budgets() {
    for kind in ["f32", "f64"] {
        let source = SOURCE.replace("f64", kind);
        for (from, to) in [
            ("if next != 0", "if next"),
            ("let joined: i64", "let joined: bool"),
            ("let staged = next ^ 4294967296;", "let staged = false;"),
            ("let staged = next ^ 4294967296;", "let staged = joined;"),
            (
                "negative_finish(next, negative_finish_fault)",
                "negative_finish(false, negative_finish_fault)",
            ),
            ("control = control + 0;", ""),
            ("child = child + 1; continue;", "continue;"),
        ] {
            let bad = source.replace(from, to);
            assert!(bad != source, "missing mutation target: {from}");
            if crate::frontend::parse_nuis_module(&bad).is_ok() {
                for reversed in [false, true] {
                    assert!(!admitted(&bad, reversed), "{from} -> {to}");
                }
            }
        }
    }
}
