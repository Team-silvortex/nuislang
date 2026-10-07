use super::*;
use crate::lowering::buffer_loop_outline::conditional_returns::tests::native::run;

#[test]
fn conditional_return_exit_only_execute_default_aot_continuations_first_exits_and_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, mode, deep, shape, index, tail) in [
        ("computed", "false", false, "then", 1, "empty"),
        ("computed", "zero", false, "both", 1, "checked"),
        ("logical", "computed", true, "both", 5, "branch"),
        ("atom", "checked", false, "then", 1, "empty"),
        ("atom", "logical", false, "then", 3, "checked"),
        ("computed", "logical", true, "both", 2, "branch"),
        ("atom", "false", true, "then", 2, "empty"),
        ("logical", "zero", true, "else", 9, "branch"),
        ("computed", "checked", false, "then", 0, "checked"),
        ("atom", "logical", true, "then", 1, "empty"),
        ("atom", "false", false, "then", 8, "checked"),
        ("computed", "false", true, "then", 7, "branch"),
        ("atom", "checked", false, "then", 6, "empty"),
    ] {
        let text = source(kind, mode, deep, shape, INPUTS[index], tail);
        let (result, prints, _) = expected(kind, mode, deep, shape, INPUTS[index]);
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(&successful_main(&text), result.map(|_| output.as_str()));
    }
    for (mode, result) in [("false", 19), ("zero", 0)] {
        run(
            &successful_main(&all_exiting(mode)),
            Some(&format!("99\n88\n50\n44\n{result}")),
        );
    }
    let base = source("computed", "false", false, "then", INPUTS[2], "checked").replace(
        "let final_check = observe(tail);",
        "let final_check = observe(tail - 2);",
    );
    run(&successful_main(&base), None);
    run(
        &successful_main(&base.replace("event(true, true, false,", "event(true, true, true,")),
        Some("99\n88\n50\n44\n19"),
    );
}

fn successful_main(text: &str) -> String {
    text.replace("print(11); return 11;", "print(11); return 0;")
        .replace("print(19); return 19;", "print(19); return 0;")
        .replace("print(result); return result;", "print(result); return 0;")
}
