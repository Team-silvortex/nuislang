use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_division_exit_argument_print.ns"
));

#[test]
fn state_division_exit_argument_print_accepts_checked_ready_arguments() {
    accepts(SOURCE);
}

#[test]
fn state_division_exit_argument_print_keeps_exact_types_and_original_epochs() {
    refuses(SOURCE);
    for prefix in ["if continue_at(", "if (break_at("] {
        let exit = SOURCE
            .lines()
            .find(|line| line.trim_start().starts_with(prefix))
            .unwrap()
            .trim();
        let without_exit = SOURCE.replace(exit, "");
        let bad = without_exit.replace("print(74);", &format!("{exit} print(74);"));
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
        for bad in [
            source.replace(
                "current.total / continue_divisor",
                "current.flag / continue_divisor",
            ),
            source.replace(
                "current.total / break_divisor",
                "current.code / break_divisor",
            ),
            source.replace(
                "current.total / continue_divisor",
                "current.total / current.flag",
            ),
            source.replace(
                &format!("continue_divisor: {kind}"),
                &format!("continue_divisor: {other}"),
            ),
            source.replace(
                &format!("break_divisor: {kind}"),
                &format!("break_divisor: {other}"),
            ),
        ] {
            assert_ne!(bad, source);
            assert!(
                crate::frontend::parse_nuis_module(&bad).is_err(),
                "unexpected acceptance: {bad}"
            );
        }
    }
}
