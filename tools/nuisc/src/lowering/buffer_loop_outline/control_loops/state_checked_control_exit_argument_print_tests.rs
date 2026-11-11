use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_checked_control_exit_argument_print.ns"
));

#[test]
fn state_checked_control_exit_argument_print_accepts_scoped_update_producer() {
    accepts(SOURCE);
}

#[test]
fn state_checked_control_exit_argument_print_keeps_original_epochs_and_exact_types() {
    let mut refused = vec![
        SOURCE.replace("control = control + 0;", ""),
        SOURCE.replace("print(74);", "let alias = epoch_word;"),
        SOURCE.replace("child = child + 1; continue;", "continue;"),
        SOURCE.replace(
            "child = child + 1; continue;",
            "child = child + 2; continue;",
        ),
    ];
    for prefix in ["if continue_at(", "if (break_at("] {
        let exit = SOURCE
            .lines()
            .find(|line| line.trim_start().starts_with(prefix))
            .unwrap()
            .trim();
        refused.push(
            SOURCE
                .replace(exit, "")
                .replace("print(74);", &format!("{exit} print(74);")),
        );
    }
    for source in refused {
        assert_ne!(source, SOURCE);
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(!admitted(&source.replace("f64", kind), reversed));
            }
        }
    }
    for kind in ["f32", "f64"] {
        let source = SOURCE.replace("f64", kind);
        for field in ["current.total", "current.flag", "current.code"] {
            let bad = source.replace(
                "advance_control(current.control,",
                &format!("advance_control({field},"),
            );
            assert_ne!(bad, source);
            let error = crate::frontend::parse_nuis_module(&bad).unwrap_err();
            assert!(
                error.contains("function `advance_control` argument 1 expects `i64`"),
                "{error}"
            );
        }
        let bad = source
            .replace(
                "advance_control(value: i64, fault: i64) -> i64",
                "advance_control(value: i64, fault: i64) -> bool",
            )
            .replace("return value + 1;", "return value == 0;");
        assert!(crate::frontend::parse_nuis_module(&bad).is_err());
    }
}
