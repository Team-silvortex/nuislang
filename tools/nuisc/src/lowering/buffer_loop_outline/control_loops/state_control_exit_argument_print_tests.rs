use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_control_exit_argument_print.ns"
));

#[test]
fn state_control_exit_argument_print_accepts_independently_ready_control_state() {
    accepts(SOURCE);
}

#[test]
fn state_control_exit_argument_print_preserves_epoch_type_and_exit_refusals() {
    let init =
        "let mut current = Pair { total: total, control: control, code: left, flag: selected };";
    let mut bad = vec![
        SOURCE.replace("control = control + 0;", ""),
        SOURCE.replace(init, "").replace(
            "while outer < parents {",
            &format!("{init} while outer < parents {{"),
        ),
        SOURCE.replace("print(74);", "let alias = epoch_word;"),
        SOURCE.replace("child = child + 1; continue;", "continue;"),
        SOURCE.replace(
            "child = child + 1; continue;",
            "child = child + 2; continue;",
        ),
        SOURCE.replace("break;", "break; print(83);"),
    ];
    for prefix in ["if continue_at(", "if (break_at("] {
        let exit = SOURCE
            .lines()
            .find(|line| line.trim_start().starts_with(prefix))
            .unwrap()
            .trim();
        bad.push(
            SOURCE
                .replace(exit, "")
                .replace("print(74);", &format!("{exit} print(74);")),
        );
    }
    for source in bad {
        assert_ne!(source, SOURCE);
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(!admitted(&source.replace("f64", kind), reversed));
            }
        }
    }
    for kind in ["f32", "f64"] {
        let source = SOURCE.replace("f64", kind);
        for (old, new) in [
            ("permit ^ current.control", "permit ^ current.flag"),
            ("seal ^ current.control", "seal ^ current.total"),
            ("control: current.control + 1", "control: current.total"),
            ("control: initial_control", "control: selected"),
        ] {
            let bad = source.replace(old, new);
            assert_ne!(bad, source);
            assert!(crate::frontend::parse_nuis_module(&bad).is_err());
        }
    }
}
