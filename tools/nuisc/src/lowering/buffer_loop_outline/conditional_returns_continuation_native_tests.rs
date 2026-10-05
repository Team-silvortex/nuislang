use super::super::super::tests::native::run;
use super::{integer_source, source, suffix_source};

#[test]
fn conditional_return_continuations_execute_default_aot_selected_unused_work_and_real_exits() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, kind, op, outer, nested, other, gate, divisor, early, expected) in [
        (
            "outer-then",
            "record",
            "&&",
            true,
            false,
            false,
            false,
            0,
            false,
            Some("99\n19"),
        ),
        (
            "outer-then",
            "calls",
            "&&",
            false,
            false,
            false,
            true,
            2,
            false,
            Some("99\n77\n19"),
        ),
        (
            "outer-else",
            "record",
            "&&",
            false,
            false,
            false,
            true,
            2,
            false,
            Some("99\n11"),
        ),
        (
            "then",
            "record",
            "||",
            true,
            false,
            false,
            true,
            -2,
            false,
            Some("99\n77\n19"),
        ),
        (
            "then",
            "record",
            "&&",
            false,
            false,
            false,
            true,
            0,
            false,
            Some("99\n77\n19"),
        ),
        (
            "both",
            "hygiene",
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
            "tree",
            "&&",
            true,
            false,
            true,
            false,
            2,
            false,
            Some("99\n77\n19"),
        ),
        (
            "outer-else",
            "record",
            "&&",
            true,
            false,
            false,
            true,
            0,
            true,
            Some("19"),
        ),
        (
            "outer-else",
            "record",
            "&&",
            true,
            false,
            false,
            true,
            0,
            false,
            None,
        ),
        (
            "both", "tree", "&&", false, true, false, false, 0, false, None,
        ),
    ] {
        let source = source(shape, kind, op, outer, nested, other, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
    for (returned, expected) in [(true, Some("99\n19")), (false, None)] {
        let source = suffix_source(returned, false)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_continuations_execute_default_aot_zero_results_and_continuation_checks() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, outer, nested, divisor, expected) in [
        ("outer-then", true, false, 2, Some("99\n0")),
        ("outer-else", true, false, 2, Some("99\n77\n19")),
        ("both", false, false, -2, Some("99\n-10")),
        ("then", true, false, 0, None),
    ] {
        let source = integer_source(shape, outer, nested, divisor, "let")
            .replace("return event(", "let result = event(")
            .replace(
                &format!("{divisor}); }} }}"),
                &format!("{divisor}); print(result); return 0; }} }}"),
            );
        run(&source, expected);
    }
}
