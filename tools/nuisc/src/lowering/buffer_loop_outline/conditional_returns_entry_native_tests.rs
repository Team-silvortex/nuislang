use super::super::super::tests::native::run;
use super::{partial_source, source};

#[test]
fn conditional_return_entry_predicates_execute_default_aot_original_site_and_short_circuit_paths() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, entry, gate, divisor, early, expected) in [
        ("then", "call", "&&", -2, true, 0, false, Some("99\n77\n19")),
        ("then", "call", "&&", 2, false, 0, false, Some("99\n19")),
        ("then", "call", "||", 2, true, 0, false, Some("99\n11")),
        ("then", "call", "&&", 2, true, 0, false, None),
        ("then", "call", "||", 2, false, 0, false, None),
        ("both", "field", "&&", -2, false, 2, false, Some("99\n11")),
        (
            "else",
            "call-compare",
            "||",
            -2,
            false,
            -2,
            false,
            Some("99\n19"),
        ),
        ("then", "division", "&&", 0, false, 2, false, None),
        ("both", "call", "&&", 0, true, 0, true, Some("19")),
        (
            "then",
            "nested",
            "&&",
            0,
            true,
            0,
            false,
            Some("99\n77\n19"),
        ),
    ] {
        let source = source(shape, kind, op, entry, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_entry_predicates_execute_default_aot_partial_exits_and_continuation_work() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (stored, entry, gate, divisor, expected) in [
        (false, 2, true, 0, Some("99\n19")),
        (false, 2, false, 2, Some("99\n77\n19")),
        (false, 2, false, 0, None),
        (true, 2, true, 2, Some("99\n19")),
        (true, 2, true, -2, Some("99\n77\n19")),
        (true, -2, true, 0, Some("99\n77\n19")),
        (true, 0, true, 2, None),
        (true, 2, false, 0, None),
    ] {
        let source = partial_source(stored, entry, gate, divisor)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}
