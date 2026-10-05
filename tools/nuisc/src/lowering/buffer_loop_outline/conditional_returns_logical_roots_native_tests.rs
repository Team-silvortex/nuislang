use super::super::super::tests::native::run;
use super::{integer_source, prefixed_source, source};

#[test]
fn conditional_return_logical_roots_execute_default_aot_selected_conditions_and_binding_roots() {
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
            (true, false, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "complete",
            "field",
            "||",
            (true, true, 0, 0, false),
            Some("99\n11"),
        ),
        (
            "partial",
            "call",
            "||",
            (true, true, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "suffix",
            "division",
            "&&",
            (true, false, 0, 2, false),
            Some("99\n77\n19"),
        ),
        (
            "let",
            "call",
            "&&",
            (true, false, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "const",
            "field",
            "||",
            (true, true, 0, 0, false),
            Some("99\n19"),
        ),
        (
            "continuation",
            "call",
            "&&",
            (true, false, 0, 0, false),
            Some("99\n77\n19"),
        ),
        (
            "suffix-binding",
            "call",
            "||",
            (true, false, 0, 2, false),
            None,
        ),
        ("partial", "division", "&&", (true, true, 0, 2, false), None),
        ("const", "call", "&&", (true, true, 0, 0, true), Some("19")),
    ] {
        let source = source("then", mode, kind, op, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_logical_roots_execute_default_aot_local_prefixes_outer_composition_and_zero_exits(
) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, op, gate, prefix, divisor, expected) in [
        ("record", "&&", false, 0, 0, None),
        ("computed", "&&", false, -2, 0, Some("99\n77\n19")),
        ("alias", "||", true, 2, 0, Some("99\n19")),
        ("record", "||", false, 2, 0, Some("99\n19")),
    ] {
        let source = prefixed_source(kind, op, gate, prefix, divisor)
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
    for (outer_op, op, outer, gate, divisor, tail, expected) in [
        ("&&", "||", true, true, 0, 0, Some("99\n0")),
        ("||", "&&", false, false, 0, 2, Some("99\n77\n10")),
        ("&&", "&&", true, true, 0, 2, None),
        ("||", "&&", false, false, 0, 0, None),
    ] {
        let source = integer_source(outer_op, op, outer, gate, divisor, tail)
            .replace("print(result); return result;", "print(result); return 0;");
        run(&source, expected);
    }
}
