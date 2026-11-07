use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_record_parameter_exit_argument_print.ns"
));

#[test]
fn state_record_parameter_exit_argument_print_accepts_updated_state_parameters() {
    accepts(SOURCE);
}

#[test]
fn state_record_parameter_exit_argument_print_keeps_field_types_and_original_epochs() {
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
        let other = if kind == "f32" { "f64" } else { "f32" };
        for name in [
            "continue_mode_value",
            "continue_guard_value",
            "break_mode_value",
            "break_guard_value",
        ] {
            for field in ["current.flag", "current.code"] {
                let bad = source.replace(
                    &format!("{name}(current.total,"),
                    &format!("{name}({field},"),
                );
                assert_ne!(bad, source);
                let error = crate::frontend::parse_nuis_module(&bad).unwrap_err();
                assert!(
                    error.contains(&format!("function `{name}` argument 1 expects `{kind}`")),
                    "{error}"
                );
            }
            let bad = source.replace(
                &format!("fn {name}(value: {kind},"),
                &format!("fn {name}(value: {other},"),
            );
            assert_ne!(bad, source);
            assert!(crate::frontend::parse_nuis_module(&bad).is_err());
        }
        for bad in [
            source.replace(
                "continue_mode_value(current.total, permit,",
                "continue_mode_value(current.total, current.flag,",
            ),
            source.replace(
                "break_mode_value(current.total, seal,",
                "break_mode_value(current.total, current.total,",
            ),
        ] {
            assert_ne!(bad, source);
            assert!(crate::frontend::parse_nuis_module(&bad).is_err());
        }
    }
}
