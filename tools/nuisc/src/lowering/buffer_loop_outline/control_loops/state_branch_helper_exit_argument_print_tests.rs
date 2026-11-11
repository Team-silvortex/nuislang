use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_branch_helper_exit_argument_print.ns"
));

#[test]
fn state_branch_helper_exit_accepts_checked_effectful_leaf_calls() {
    accepts(SOURCE);
    let module = crate::frontend::parse_nuis_module(SOURCE).unwrap();
    let layouts = control_values::layouts(&module);
    let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
    let effects = scalar_helpers::collect_iteration_calls(&module, &layouts);
    for name in [
        "negative_update",
        "nonnegative_update",
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

#[test]
fn state_branch_helper_exit_preserves_readiness_and_leaf_types() {
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
            (
                "negative_update(value, negative_helper_fault)",
                "negative_update(value, false)",
            ),
            (
                "nonnegative_update(value, nonnegative_helper_fault)",
                "nonnegative_update(value, false)",
            ),
            ("return value - 3;", "return value < 0;"),
            ("return value + 2;", "return value >= 0;"),
        ] {
            assert_ne!(source.replace(from, to), source);
            let bad = source.replace(from, to);
            // Call signatures are checked by the frontend; return bodies by the outliner.
            if crate::frontend::parse_nuis_module(&bad).is_ok() {
                for reversed in [false, true] {
                    assert!(
                        !admitted(&bad, reversed),
                        "accepted wrong leaf type: {from} -> {to}"
                    );
                }
            }
        }
    }
}
