use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_parameter_exit_argument_print.ns"
));

#[test]
fn state_parameter_exit_argument_print_accepts_ordered_later_arguments() {
    accepts(SOURCE);
}

#[test]
fn state_parameter_exit_argument_print_keeps_exact_parameter_types_and_epochs() {
    refuses(SOURCE);
    for prefix in ["if continue_at(", "if (break_at("] {
        let exit = SOURCE
            .lines()
            .find(|line| line.trim_start().starts_with(prefix))
            .unwrap()
            .trim();
        let bad = SOURCE
            .replace(exit, "")
            .replace("print(74);", &format!("{exit} print(74);"));
        assert_ne!(bad, SOURCE);
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(!admitted(&bad.replace("f64", kind), reversed));
            }
        }
    }
    for kind in ["f32", "f64"] {
        let source = SOURCE.replace("f64", kind);
        for (call, replacement) in [
            (
                "continue_mode_value(permit,",
                "continue_mode_value(current.flag,",
            ),
            ("break_mode_value(seal,", "break_mode_value(current.total,"),
            (
                "continue_guard_value(confirm_guard - (outer * 16 + child),",
                "continue_guard_value(current.code,",
            ),
            (
                "break_guard_value(seal_guard - (outer * 16 + child),",
                "break_guard_value(current.flag,",
            ),
        ] {
            let bad = source.replace(call, replacement);
            assert_ne!(bad, source);
            assert!(crate::frontend::parse_nuis_module(&bad).is_err());
        }
        for name in [
            "continue_mode_value",
            "continue_guard_value",
            "break_mode_value",
            "break_guard_value",
        ] {
            let original = format!("fn {name}(value: i64, fault: i64) -> i64 {{ let checked = 0 / fault; return value; }}");
            let replacement = format!("fn {name}(value: i64, fault: i64) -> bool {{ let checked = 0 / fault; return value == 0; }}");
            let bad = source.replace(&original, &replacement);
            assert_ne!(bad, source);
            assert!(crate::frontend::parse_nuis_module(&bad).is_err());
        }
    }
}
