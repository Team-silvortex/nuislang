use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/native_application_bridge/state_helper_exit_argument_print.ns"
));

#[test]
fn state_helper_exit_argument_print_accepts_ordered_checked_producers() {
    accepts(SOURCE);
}

#[test]
fn state_helper_exit_argument_print_keeps_types_and_original_epochs() {
    refuses(SOURCE);
    let mut refused = Vec::new();
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
                assert!(
                    !admitted(&source.replace("f64", kind), reversed),
                    "unexpected admission: {source}"
                );
            }
        }
    }
    for kind in ["f32", "f64"] {
        let source = SOURCE.replace("f64", kind);
        let other = if kind == "f32" { "f64" } else { "f32" };
        for bad in [
            source.replace("continue_lhs(current.total,", "continue_lhs(current.flag,"),
            source.replace("break_rhs(break_divisor,", "break_rhs(current.code,"),
            source.replace(
                &format!("fn continue_lhs(value: {kind}, guard: i64) -> {kind}"),
                &format!("fn continue_lhs(value: {other}, guard: i64) -> {other}"),
            ),
            source.replace(
                &format!("fn break_rhs(value: {kind}, guard: i64) -> {kind}"),
                &format!("fn break_rhs(value: {other}, guard: i64) -> {other}"),
            ),
        ] {
            assert_ne!(bad, source);
            assert!(crate::frontend::parse_nuis_module(&bad).is_err());
        }
    }
}
