use super::super::super::tests::native::run;
use super::{composed, source};

#[test]
fn conditional_computed_gates_execute_default_aot_value_roots_and_original_left_failures() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (place, kind, op, input, expected) in [
        (
            "let",
            "call",
            "&&",
            (-2, 0, 0, false, false, 0),
            Some("99\n77\n19"),
        ),
        (
            "inferred",
            "field",
            "||",
            (2, 0, 0, false, false, 0),
            Some("99\n77\n11"),
        ),
        (
            "const",
            "division",
            "&&",
            (2, 2, 0, false, false, 0),
            Some("99\n77\n11"),
        ),
        (
            "return",
            "comparison",
            "||",
            (0, 2, 0, false, false, 0),
            Some("99\n11"),
        ),
        ("return", "call", "||", (0, 2, 0, false, false, 0), None),
        ("let", "field", "&&", (0, -2, 0, false, false, 0), None),
        ("const", "call", "&&", (2, 0, 0, false, false, 0), None),
        (
            "return",
            "division",
            "||",
            (0, 0, 0, false, true, 0),
            Some("19"),
        ),
    ] {
        let source = source(place, kind, "call", op, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
    for (mode, op, input, expected) in [
        (
            "rebind",
            "||",
            (0, 0, 2, true, false, 0),
            Some("99\n77\n11"),
        ),
        ("atom", "&&", (-2, 0, 0, true, false, 0), Some("99\n19")),
    ] {
        let source = composed(mode, op, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_computed_gates_execute_default_aot_entries_signals_parent_records_and_zero_exits() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (place, kind, op, input, expected) in [
        (
            "then",
            "call",
            "||",
            (2, 0, 0, false, false, 0),
            Some("99\n19"),
        ),
        (
            "else",
            "field",
            "&&",
            (-2, 0, 0, false, false, 0),
            Some("99\n19"),
        ),
        (
            "both",
            "division",
            "&&",
            (-2, 0, 2, false, false, 0),
            Some("99\n11"),
        ),
        (
            "partial",
            "call",
            "&&",
            (2, 2, 0, true, false, 0),
            Some("99\n19"),
        ),
        (
            "suffix",
            "field",
            "||",
            (2, 0, 2, false, false, 2),
            Some("99\n77\n19"),
        ),
    ] {
        let source = source(place, kind, "call", op, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
    for (mode, op, input, expected) in [
        ("record", "||", (2, 0, 2, false, false, 0), Some("99\n19")),
        ("record", "&&", (0, 2, 2, false, false, 0), None),
        ("zero", "||", (2, 0, 0, false, false, 0), Some("99\n0")),
    ] {
        let source = composed(mode, op, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;")
            .replace("print(result); return result;", "print(result); return 0;");
        run(&source, expected);
    }
}
