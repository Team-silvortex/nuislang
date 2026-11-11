use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_return_gate_exit_argument_print.ns"
));

#[test]
fn state_return_gate_exit_accepts_return_selected_effects_without_pure_promotion() {
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
            assert!(
                !pure.contains_key(name),
                "{name} leaked into pure admission"
            );
            assert!(
                effects.contains_key(name),
                "missing effectful helper {name}"
            );
        }
    }
}

#[test]
fn state_return_gate_exit_keeps_scope_types_and_original_readiness() {
    for kind in ["f32", "f64"] {
        let source = SOURCE.replace("f64", kind);
        for reversed in [false, true] {
            assert!(!admitted(
                &source.replace("control = control + 0;", ""),
                reversed
            ));
            assert!(!admitted(
                &source.replace("child = child + 1; continue;", "continue;"),
                reversed
            ));
        }
        for (from, to) in [
            ("if next != 0", "if next"),
            (
                "negative_finish(next, negative_finish_fault)",
                "negative_finish(false, negative_finish_fault)",
            ),
            (
                "nonnegative_finish(next, nonnegative_finish_fault)",
                "nonnegative_finish(next, false)",
            ),
            (
                "let next = nonnegative_update(value, nonnegative_helper_fault);",
                "",
            ),
            ("return value ^ 4294967296;", "return value == 0;"),
        ] {
            let bad = source.replace(from, to);
            assert_ne!(bad, source);
            if crate::frontend::parse_nuis_module(&bad).is_ok() {
                for reversed in [false, true] {
                    assert!(!admitted(&bad, reversed), "{from} -> {to}");
                }
            }
        }
    }
}
