use super::super::super::tests::native::run;
use super::source;

#[test]
fn conditional_return_terminal_trees_execute_default_aot_skipped_leaf_and_prefix_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, outer, nested, gate, divisor, early, expected) in [
        (
            "then",
            "packet",
            "&&",
            false,
            true,
            true,
            0,
            false,
            Some("99\n77\n19"),
        ),
        (
            "then",
            "leaf",
            "&&",
            true,
            false,
            true,
            0,
            false,
            Some("99\n19"),
        ),
        (
            "else",
            "local",
            "||",
            false,
            true,
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
            true,
            2,
            false,
            Some("99\n11"),
        ),
        (
            "both",
            "packet",
            "||",
            true,
            false,
            false,
            2,
            false,
            Some("99\n19"),
        ),
        (
            "then",
            "packet",
            "&&",
            true,
            true,
            true,
            0,
            true,
            Some("19"),
        ),
        (
            "else",
            "leaf",
            "||",
            false,
            true,
            false,
            -2,
            false,
            Some("99\n19"),
        ),
        ("then", "packet", "&&", true, false, false, 0, false, None),
        ("else", "packet", "||", false, true, true, 0, false, None),
        ("both", "leaf", "&&", true, true, true, 0, false, None),
    ] {
        let source = source(shape, kind, op, outer, nested, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}
