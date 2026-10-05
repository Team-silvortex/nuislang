use super::super::super::tests::native::run;
use super::*;

fn supported() -> bool {
    cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    ))
}

#[test]
fn conditional_return_stored_signals_execute_default_aot_computed_exits_and_continuations() {
    if !supported() {
        return;
    }
    for (shape, kind, outer, gate, divisor, early, expected) in [
        ("then", "call", false, true, 0, false, Some("99\n77\n19")),
        ("then", "call", true, false, 2, false, Some("99\n19")),
        ("else", "alias", false, true, 2, false, Some("99\n11")),
        ("both", "field", false, true, -2, false, Some("99\n77\n19")),
        ("both", "inline-field", true, true, 2, false, Some("99\n11")),
        (
            "then",
            "division",
            true,
            false,
            -2,
            false,
            Some("99\n77\n19"),
        ),
        ("both", "call", false, true, 0, true, Some("19")),
        ("then", "call", true, false, 0, false, None),
        ("else", "field", false, true, 0, false, None),
        ("both", "division", true, false, 0, false, None),
    ] {
        run(
            &source(shape, kind, outer, gate, divisor, early)
                .replace("print(11); return 11;", "print(11); return 0;")
                .replace("print(19); return 19;", "print(19); return 0;"),
            expected,
        );
    }
}

#[test]
fn conditional_return_stored_signals_execute_default_aot_zero_results_and_parent_traps() {
    if !supported() {
        return;
    }
    for (shape, outer, divisor, tail_trap, expected) in [
        ("then", true, 2, false, Some("99\n0")),
        ("else", false, -2, false, Some("99\n77\n19")),
        ("both", false, 2, true, Some("99\n0")),
        ("then", false, 0, false, Some("99\n77\n19")),
        ("then", true, -2, true, None),
        ("both", true, 0, false, None),
    ] {
        let mut source = integer_source(shape, outer, divisor)
            .replace("print(result); return result;", "print(result); return 0;");
        if tail_trap {
            source = source.replace("print(77); return 19;", "print(77); return 10 / 0;");
        }
        run(&source, expected);
    }
    for (op, gate, expected) in [
        ("&&", false, Some("99\n19")),
        ("&&", true, None),
        ("||", true, Some("99\n11")),
        ("||", false, None),
    ] {
        let source = source("then", "call", true, gate, 2, false)
            .replace(
                "return gate && helper(produce(divisor));",
                &format!("return gate {op} helper(produce(0));"),
            )
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}
