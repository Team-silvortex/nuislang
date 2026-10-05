use super::super::super::tests::native::run;
use super::{integer_source, prefixed_source, source};

#[test]
fn conditional_return_computed_arms_execute_default_aot_roots_left_checks_and_short_circuit_rhs() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (mode, kind, op, input, expected) in [
        (
            "complete",
            "call",
            "&&",
            (true, false, -2, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "complete",
            "field",
            "||",
            (true, true, 2, 0, 0, false),
            Some("99\n11"),
        ),
        (
            "let",
            "call",
            "||",
            (true, true, 2, 0, 0, false),
            Some("99\n11"),
        ),
        (
            "inferred",
            "division",
            "&&",
            (true, false, -2, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "return",
            "call",
            "&&",
            (true, false, -2, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "const",
            "call",
            "||",
            (true, true, 2, 0, 0, false),
            Some("99\n19"),
        ),
        ("partial", "call", "&&", (true, true, 2, 0, 2, false), None),
        (
            "partial",
            "division",
            "||",
            (true, true, 0, 0, 2, false),
            None,
        ),
        ("return", "field", "&&", (true, false, 0, 0, 2, false), None),
        (
            "complete",
            "call",
            "&&",
            (true, true, 0, 0, 0, true),
            Some("19"),
        ),
    ] {
        let text = source("then", mode, kind, op, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&text, expected);
    }
}

#[test]
fn conditional_return_computed_arms_execute_default_aot_signals_suffixes_prefix_records_and_zero_exits(
) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (mode, op, input, expected) in [
        (
            "partial",
            "||",
            (true, false, 2, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "suffix",
            "&&",
            (true, false, -2, 0, 2, false),
            Some("99\n77\n19"),
        ),
        (
            "continuation",
            "||",
            (true, false, 2, 0, 2, false),
            Some("99\n77\n19"),
        ),
        (
            "suffix-binding",
            "&&",
            (true, true, 0, 0, 0, false),
            Some("99\n19"),
        ),
        ("suffix-binding", "&&", (true, false, -2, 0, 0, false), None),
    ] {
        let text = source("both", mode, "call", op, input)
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&text, expected);
    }
    for (kind, op, input, expected) in [
        (
            "record",
            "&&",
            (true, false, -2, 0, 2, false),
            Some("99\n77\n19"),
        ),
        ("checked", "||", (true, true, 0, 0, 2, false), None),
        (
            "comparison",
            "||",
            (true, true, 2, 0, 0, false),
            Some("99\n19"),
        ),
    ] {
        let text = prefixed_source(kind, op, input)
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&text, expected);
    }
    let text = integer_source("||", (true, true, 2, 0, 0, false));
    run(&text, Some("99\n0"));
}
