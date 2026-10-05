use super::super::super::tests::native::run;
use super::{integer_source, source};

#[test]
fn conditional_return_aliases_execute_default_aot_selected_atoms_calls_and_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, op, outer, nested, gate, divisor, early, expected) in [
        ("then", "&&", true, true, false, 0, false, Some("99\n19")),
        ("then", "&&", true, true, true, 2, false, Some("99\n11")),
        (
            "then",
            "&&",
            true,
            false,
            false,
            2,
            false,
            Some("99\n77\n19"),
        ),
        ("else", "||", true, true, true, 0, false, Some("99\n77\n19")),
        ("both", "||", false, false, true, 0, false, Some("99\n11")),
        ("both", "&&", false, true, false, 0, true, Some("19")),
        ("then", "&&", true, false, false, 0, false, None),
        ("both", "&&", false, false, true, 0, false, None),
    ] {
        let source = source(shape, op, outer, nested, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

#[test]
fn conditional_return_aliases_execute_default_aot_zero_return_and_continuation_checks() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (outer, nested, divisor, expected) in [
        (true, true, 2, Some("99\n0")),
        (true, false, 2, Some("99\n77\n19")),
        (false, true, 0, Some("99\n77\n19")),
        (true, false, 0, None),
    ] {
        let source = integer_source(outer, nested, divisor)
            .replace("return event(", "let result = event(")
            .replace(
                &format!("{divisor}); }} }}"),
                &format!("{divisor}); print(result); return 0; }} }}"),
            );
        run(&source, expected);
    }
}
