use super::super::super::tests::native::run;
use super::{integer_source, source};

#[test]
fn conditional_return_fallthrough_trees_execute_default_aot_false_exits_and_prefix_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, outer, nested, other, gate, divisor, early, expected) in [
        (
            "then",
            "packet",
            "&&",
            false,
            true,
            false,
            true,
            0,
            false,
            Some("99\n77\n19"),
        ),
        (
            "then",
            "then",
            "&&",
            true,
            false,
            false,
            true,
            0,
            false,
            Some("99\n77\n19"),
        ),
        (
            "then",
            "then",
            "&&",
            true,
            true,
            false,
            false,
            0,
            false,
            Some("99\n19"),
        ),
        (
            "else",
            "else",
            "||",
            false,
            false,
            false,
            true,
            0,
            false,
            Some("99\n11"),
        ),
        (
            "both",
            "hygiene",
            "&&",
            false,
            true,
            false,
            true,
            2,
            false,
            Some("99\n11"),
        ),
        (
            "both",
            "split",
            "||",
            true,
            false,
            true,
            false,
            0,
            false,
            Some("99\n77\n19"),
        ),
        (
            "then",
            "packet",
            "&&",
            true,
            true,
            false,
            true,
            0,
            true,
            Some("19"),
        ),
        (
            "else",
            "then",
            "||",
            false,
            true,
            false,
            false,
            -2,
            false,
            Some("99\n19"),
        ),
        (
            "then", "packet", "&&", true, false, false, false, 0, false, None,
        ),
        (
            "both", "split", "&&", true, true, true, true, 0, false, None,
        ),
    ] {
        let source = source(shape, kind, op, outer, nested, other, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_fallthrough_trees_execute_default_aot_zero_returns_not_seed_exits() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, outer, nested, divisor, expected) in [
        ("then", true, true, 2, Some("99\n0")),
        ("else", false, false, 0, Some("99\n77\n19")),
        ("both", false, true, -2, Some("99\n-10")),
        ("both", true, true, 0, None),
    ] {
        let source = integer_source(shape, outer, nested, divisor, "let current")
            .replace("return event(", "let result = event(")
            .replace(
                &format!("{divisor}); }} }}"),
                &format!("{divisor}); print(result); return 0; }} }}"),
            );
        run(&source, expected);
    }
}
