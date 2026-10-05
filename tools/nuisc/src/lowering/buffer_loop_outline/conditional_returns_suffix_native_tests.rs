use super::super::super::tests::native::run;
use super::{integer_source, source};

#[test]
fn conditional_return_suffixes_execute_default_aot_selected_and_skipped_tail_checks() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, outer, gate, nested, divisor, suffix, early, expected) in [
        (
            "then",
            "partial",
            "||",
            true,
            true,
            false,
            0,
            0,
            false,
            Some("99\n11"),
        ),
        (
            "else",
            "partial",
            "&&",
            false,
            false,
            false,
            2,
            2,
            false,
            Some("99\n77\n19"),
        ),
        (
            "both",
            "complete",
            "&&",
            true,
            false,
            false,
            2,
            2,
            false,
            Some("99\n19"),
        ),
        (
            "then",
            "computed",
            "&&",
            true,
            false,
            false,
            2,
            0,
            false,
            Some("99\n19"),
        ),
        (
            "both",
            "computed",
            "||",
            false,
            true,
            true,
            2,
            0,
            false,
            Some("99\n11"),
        ),
        (
            "then",
            "sequence",
            "&&",
            true,
            true,
            true,
            0,
            0,
            false,
            Some("99\n19"),
        ),
        (
            "then",
            "diamond",
            "&&",
            true,
            false,
            true,
            0,
            -2,
            false,
            Some("99\n19"),
        ),
        (
            "both",
            "diamond",
            "&&",
            false,
            true,
            true,
            0,
            2,
            true,
            Some("19"),
        ),
        (
            "then", "partial", "&&", true, false, false, 2, 0, false, None,
        ),
        (
            "then", "computed", "&&", true, true, true, 0, 2, false, None,
        ),
        (
            "then", "sequence", "&&", true, false, false, 2, 0, false, None,
        ),
        ("both", "diamond", "&&", true, true, true, 0, 2, false, None),
    ] {
        let source = source(shape, kind, op, outer, gate, nested, divisor, suffix, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_suffixes_execute_default_aot_real_zero_and_parent_continuations() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (outer, gate, nested, divisor, suffix, expected) in [
        (true, true, true, 0, 0, Some("99\n0")),
        (true, false, true, 2, 2, Some("99\n0")),
        (true, false, false, 2, 2, Some("99\n77\n19")),
        (false, false, false, 0, 0, Some("99\n77\n19")),
        (true, false, true, 2, 0, None),
        (true, false, false, 0, 2, None),
    ] {
        let source = integer_source(outer, gate, nested, divisor, suffix)
            .replace("print(result); return result;", "print(result); return 0;");
        run(&source, expected);
    }
}
