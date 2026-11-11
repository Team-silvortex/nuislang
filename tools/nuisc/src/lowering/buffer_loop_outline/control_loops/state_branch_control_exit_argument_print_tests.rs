use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_branch_control_exit_argument_print.ns"
));

#[test]
fn state_branch_control_exit_accepts_selected_checked_returns() {
    accepts(SOURCE);
}

#[test]
fn state_branch_control_exit_preserves_ready_fields_and_exact_types() {
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
        for field in ["current.total", "current.code", "current.flag"] {
            let bad = source.replace(
                "advance_control(current.control,",
                &format!("advance_control({field},"),
            );
            assert!(crate::frontend::parse_nuis_module(&bad)
                .unwrap_err()
                .contains("function `advance_control` argument 1 expects `i64`"));
        }
        let escaped = source.replace("return value + 2;", "return checked;");
        // The negative branch's local must not become the fallthrough binding.
        let escaped = escaped.replace("let checked = 0 / nonnegative_fault;", "");
        assert!(crate::frontend::parse_nuis_module(&escaped).is_err());
    }
}
