use super::super::super::tests::native::run;
use super::{integer_source, source};

#[test]
fn conditional_return_complete_predicates_execute_default_aot_conditions_once_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, outer, gate, divisor, early, expected) in [
        (
            "then",
            "call",
            "&&",
            false,
            true,
            0,
            false,
            Some("99\n77\n19"),
        ),
        ("then", "call", "&&", true, false, 2, false, Some("99\n19")),
        (
            "else",
            "alias",
            "||",
            false,
            false,
            -2,
            false,
            Some("99\n19"),
        ),
        (
            "both",
            "inline-field",
            "&&",
            false,
            true,
            2,
            false,
            Some("99\n11"),
        ),
        ("both", "field", "||", true, true, -2, false, Some("99\n19")),
        ("then", "call", "&&", true, false, 0, true, Some("19")),
        (
            "then",
            "division",
            "&&",
            false,
            true,
            0,
            false,
            Some("99\n77\n19"),
        ),
        (
            "else",
            "remainder",
            "&&",
            true,
            false,
            0,
            false,
            Some("99\n77\n19"),
        ),
        ("then", "call", "&&", true, false, 0, false, None),
        ("both", "field", "&&", false, false, 0, false, None),
        ("then", "division", "&&", true, false, 0, false, None),
        ("else", "remainder", "&&", false, false, 0, false, None),
    ] {
        let source = source(shape, kind, op, outer, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_complete_predicates_execute_default_aot_real_zero_exits_and_checked_conditions(
) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, outer, divisor, expected) in [
        ("then", true, 2, Some("99\n0")),
        ("both", false, -2, Some("99\n0")),
        ("then", false, 0, Some("99\n77\n19")),
        ("else", false, 0, None),
    ] {
        let source = integer_source(shape, outer, divisor)
            .replace("print(result); return result;", "print(result); return 0;");
        run(&source, expected);
    }
}
