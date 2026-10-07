use super::super::super::super::super::tests::native::run;
use super::*;

#[test]
fn conditional_return_effect_exits_execute_default_aot_nested_returns_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, mode, deep, shape, index) in [
        ("atom", "false", false, "then", 1),
        ("computed", "zero", true, "both", 1),
        ("logical", "computed", true, "both", 5),
        ("atom", "checked", false, "then", 1),
        ("atom", "logical", false, "then", 3),
        ("computed", "logical", true, "both", 2),
        ("atom", "false", true, "then", 2),
        ("logical", "zero", true, "else", 9),
        ("computed", "checked", false, "then", 0),
        ("atom", "logical", true, "then", 1),
        ("atom", "false", false, "then", 8),
        ("computed", "false", true, "then", 7),
        ("atom", "checked", false, "then", 6),
    ] {
        let text = source(kind, mode, deep, shape, INPUTS[index]);
        let (result, prints, _) = expected(kind, mode, deep, shape, INPUTS[index]);
        let text = text
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;")
            .replace("print(result); return result;", "print(result); return 0;");
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(&text, result.map(|_| output.as_str()));
    }
    let text = source("atom", "false", false, "then", INPUTS[1]);
    let second = text.replace(
        "let ignored = observe(tail);",
        "if gate { print(43); return false; } let ignored = observe(tail);",
    );
    for (text, output) in [
        (second.clone(), Some("99\n88\n50\n44\n19")),
        (
            second.replace(
                "event(true, true, true, 2, 2, 0)",
                "event(true, true, false, 2, 2, 0)",
            ),
            Some("99\n88\n50\n55\n43\n19"),
        ),
        (
            text.replace("print(99);", "if nested { return false; } print(99);"),
            Some("19"),
        ),
        (
            text.replace("print(44);", "let ignored_exit = observe(tail); print(44);"),
            None,
        ),
    ] {
        run(
            &text
                .replace("print(11); return 11;", "print(11); return 0;")
                .replace("print(19); return 19;", "print(19); return 0;"),
            output,
        );
    }
}

#[test]
fn conditional_return_effect_exits_execute_default_aot_partial_tail_exits_and_continuation_checks()
{
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for mode in ["false", "zero"] {
        for index in [1, 2, 3, 4] {
            let text = fixtures::partial_source(mode, index)
                .replace("print(11); return 11;", "print(11); return 0;")
                .replace("print(19); return 19;", "print(19); return 0;")
                .replace("print(result); return result;", "print(result); return 0;");
            let (result, prints, _) = fixtures::partial_expected(mode, index);
            let output = prints
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            run(&text, result.map(|_| output.as_str()));
        }
    }
}
