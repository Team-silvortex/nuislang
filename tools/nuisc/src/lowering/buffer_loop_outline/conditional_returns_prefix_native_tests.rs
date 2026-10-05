use super::super::super::tests::native::run;
use super::source;

#[test]
fn conditional_return_prefixes_execute_default_source_unused_checks_local_records_and_native_traps()
{
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, outer, gate, divisor, early, expected) in [
        (
            "then",
            "const",
            "&&",
            false,
            false,
            0,
            false,
            Some("99\n77\n19"),
        ),
        ("then", "let", "&&", true, false, 0, false, Some("99\n19")),
        (
            "else",
            "inferred",
            "||",
            true,
            false,
            0,
            false,
            Some("99\n77\n19"),
        ),
        (
            "else",
            "hygiene",
            "||",
            false,
            true,
            0,
            false,
            Some("99\n11"),
        ),
        (
            "both",
            "packet",
            "&&",
            true,
            false,
            2,
            false,
            Some("99\n19"),
        ),
        (
            "both",
            "unused",
            "&&",
            false,
            true,
            2,
            false,
            Some("99\n11"),
        ),
        ("both", "packet", "||", false, false, 0, true, Some("19")),
        ("then", "packet", "&&", true, false, 0, false, None),
        ("else", "unused", "||", false, true, 0, false, None),
        ("both", "let", "&&", true, true, 0, false, None),
        (
            "then",
            "inferred",
            "&&",
            true,
            true,
            -2,
            false,
            Some("99\n19"),
        ),
        (
            "else",
            "packet",
            "||",
            false,
            false,
            2,
            false,
            Some("99\n11"),
        ),
    ] {
        let source = source(shape, kind, op, outer, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        // Reuse the deadline, kill/reap, native trap and scratch-cleanup probe.
        run(&source, expected);
    }
}
