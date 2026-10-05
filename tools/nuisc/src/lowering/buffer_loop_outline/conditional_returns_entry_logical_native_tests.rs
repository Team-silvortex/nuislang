use super::super::super::tests::native::run;
use super::{integer_source, partial_source, source};

#[test]
fn conditional_return_logical_entries_execute_default_aot_selected_skipped_rhs_and_earlier_exits() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, input, expected) in [
        (
            "then",
            "call",
            "&&",
            (false, 0, true, 0, false),
            Some("99\n77\n19"),
        ),
        (
            "then",
            "call",
            "||",
            (true, 0, false, 0, false),
            Some("99\n19"),
        ),
        (
            "then",
            "call",
            "&&",
            (true, 2, true, 2, false),
            Some("99\n11"),
        ),
        (
            "else",
            "field",
            "&&",
            (true, -2, true, 2, false),
            Some("99\n11"),
        ),
        (
            "both",
            "division",
            "||",
            (false, -2, false, -2, false),
            Some("99\n19"),
        ),
        ("then", "call", "&&", (true, 0, false, 2, false), None),
        ("then", "call", "||", (false, 0, false, 2, false), None),
        ("then", "call", "||", (true, 0, true, 0, false), None),
        ("both", "field", "&&", (true, 0, true, 0, true), Some("19")),
        (
            "then",
            "division",
            "||",
            (true, 0, false, 0, false),
            Some("99\n19"),
        ),
    ] {
        let source = source(shape, kind, op, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_logical_entries_execute_default_aot_partial_signals_and_real_zero_exits() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, op, outer, entry, gate, divisor, expected) in [
        ("replay", "&&", true, 2, true, 0, Some("99\n19")),
        ("stored", "||", true, 0, false, -2, Some("99\n77\n19")),
        ("suffix", "||", true, 0, false, 0, None),
        ("stored", "&&", false, 0, true, 0, Some("99\n77\n19")),
    ] {
        let source = partial_source(kind, op, outer, entry, gate, divisor)
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
    for (op, outer, entry, gate, late, expected) in [
        ("||", true, 0, true, 0, Some("99\n0")),
        ("&&", false, 0, true, 2, Some("99\n77\n10")),
        ("&&", true, 2, false, 0, None),
        ("||", false, 0, true, 2, None),
    ] {
        let source = integer_source(op, outer, entry, gate, late)
            .replace("print(result); return result;", "print(result); return 0;");
        run(&source, expected);
    }
}
