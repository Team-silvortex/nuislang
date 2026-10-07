use super::*;
use crate::lowering::buffer_loop_outline::conditional_returns::tests::native::run;

#[test]
fn conditional_return_effect_joins_execute_default_aot_nested_selection_and_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, word, deep, early, index) in [
        ("atom", true, false, false, 1),
        ("computed", false, true, true, 2),
        ("logical", true, true, false, 3),
        ("computed", true, true, true, 0),
        ("atom", false, false, true, 7),
        ("logical", false, true, true, 1),
        ("computed", true, true, true, 4),
        ("atom", true, false, false, 5),
        ("computed", false, true, false, 6),
    ] {
        let text = source(kind, word, deep, early, INPUTS[index]);
        let (result, prints, _) = expected(kind, word, early, INPUTS[index]);
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(&successful_main(&text), result.map(|_| output.as_str()));
    }
    let text = source(
        "computed",
        true,
        true,
        true,
        Input {
            right: 1000,
            ..INPUTS[2]
        },
    );
    run(&successful_main(&text), Some("99\n0\n0\n44\n0"));
    let text = source("computed", false, true, true, INPUTS[2])
        .replace("let selected:", "const selected:");
    run(&successful_main(&text), Some("99\n-50\n19\n44\n19"));
    for word in [false, true] {
        let exit = if word { "0" } else { "false" };
        let text = source("computed", word, false, true, INPUTS[5]).replace(
            "if outer {",
            &format!("if outer {{ if nested {{ print(33); return {exit}; }}"),
        );
        let output = if word { "99\n33\n0" } else { "99\n33\n19" };
        run(&successful_main(&text), Some(output));
    }
}

fn successful_main(text: &str) -> String {
    text.replace("print(result); return result;", "print(result); return 0;")
        .replace("print(11); return 11;", "print(11); return 0;")
        .replace("print(19); return 19;", "print(19); return 0;")
}
