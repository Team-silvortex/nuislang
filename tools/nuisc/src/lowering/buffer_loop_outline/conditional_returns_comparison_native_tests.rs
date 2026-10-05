use super::super::super::tests::native::run;
use super::{integer_source, source};

#[test]
fn conditional_return_comparisons_execute_default_aot_selected_calls_real_exits_and_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, condition, op, outer, gate, divisor, early, expected) in [
        (
            "then",
            "forwarded >= 0",
            "&&",
            true,
            false,
            0,
            false,
            Some("99\n19"),
        ),
        (
            "then",
            "forwarded > 0",
            "&&",
            true,
            true,
            2,
            false,
            Some("99\n11"),
        ),
        (
            "then",
            "forwarded < 0",
            "&&",
            true,
            false,
            2,
            false,
            Some("99\n77\n19"),
        ),
        (
            "else",
            "gate == true",
            "||",
            false,
            true,
            0,
            false,
            Some("99\n11"),
        ),
        (
            "both",
            "forwarded <= 0",
            "||",
            false,
            true,
            2,
            false,
            Some("99\n11"),
        ),
        (
            "both",
            "forwarded != 0",
            "&&",
            false,
            false,
            0,
            true,
            Some("19"),
        ),
        ("then", "forwarded != 0", "&&", true, false, 0, false, None),
        ("both", "forwarded == 0", "&&", true, true, 0, false, None),
    ] {
        let source = source(shape, condition, op, outer, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
    for (comparison, expected) in [(">", "99\n19"), ("<", "99\n77\n19")] {
        let source = source(
            "both",
            &format!("forwarded {comparison} 0"),
            "&&",
            true,
            false,
            0,
            false,
        )
        .replace("print(99);", "let divisor: i64 = 2; print(99);")
        .replace("print(11); return 11;", "print(11); return 0;")
        .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, Some(expected));
    }
}

#[test]
fn conditional_return_comparisons_execute_default_aot_zero_returns_and_continuation_checks() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (comparison, outer, divisor, expected) in [
        (">", true, 2, Some("99\n0")),
        ("<", true, 2, Some("99\n77\n19")),
        ("==", false, 0, Some("99\n77\n19")),
        ("!=", true, 0, None),
    ] {
        let source = integer_source(comparison, outer, divisor)
            .replace("return event(", "let result = event(")
            .replace(
                &format!("{divisor}); }} }}"),
                &format!("{divisor}); print(result); return 0; }} }}"),
            );
        run(&source, expected);
    }
}
