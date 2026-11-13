use super::*;
const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_return_logical_exit_argument_print.ns"
));

#[test]
fn state_return_logical_exit_admits_selected_nested_results_without_purity_leaks() {
    for kind in ["f32", "f64"] {
        let text = SOURCE.replace("f64", kind);
        for reverse in [false, true] {
            assert!(admitted(&text, reverse));
            let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
            if reverse {
                module.functions.reverse();
            }
            let layouts = control_values::layouts(&module);
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            let effects = scalar_helpers::collect_iteration_calls(&module, &layouts);
            for name in ["leaf", "advance", "work"] {
                assert!(!pure.contains_key(name));
                assert!(effects.contains_key(name));
            }
        }
    }
}

#[test]
fn state_return_logical_exit_rejects_scope_readiness_types_and_unstepped_work() {
    for (from, to) in [
        ("if inner {", "if left_fault {"),
        ("let staged: Selected", "let staged: bool"),
        (
            "print(97); print(staged.payload.word);",
            "print(97); print(local);",
        ),
        (
            "let local = leaf(value + 1, left_fault);",
            "let local = leaf(staged, left_fault);",
        ),
        ("print(101); print(local);", "print(101); print(joined);"),
        (
            "print(101); print(local);",
            "print(101); print(leaf(local, left_fault));",
        ),
        ("control = control + 0;", ""),
        ("index = index + 1;", ""),
    ] {
        let bad = SOURCE.replace(from, to);
        assert!(bad != SOURCE, "missing mutation target: {from}");
        for kind in ["f32", "f64"] {
            let bad = bad.replace("f64", kind);
            if crate::frontend::parse_nuis_module(&bad).is_ok() {
                for reverse in [false, true] {
                    assert!(!admitted(&bad, reverse), "{from} -> {to}");
                }
            }
        }
    }
}
