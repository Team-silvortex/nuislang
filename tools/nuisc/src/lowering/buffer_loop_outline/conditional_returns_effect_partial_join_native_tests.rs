use super::*;
use crate::lowering::buffer_loop_outline::conditional_returns::tests::native::run;

#[test]
fn conditional_return_partial_effect_joins_execute_default_aot_presence_exits_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, word, deep, late, continuing, index) in [
        ("computed", true, false, false, false, 1),
        ("computed", false, true, false, false, 1),
        ("atom", true, false, false, false, 2),
        ("logical", true, true, true, false, 6),
        ("computed", false, true, true, false, 9),
        ("logical", true, false, false, true, 3),
        ("computed", false, true, false, true, 4),
        ("computed", true, true, false, true, 0),
        ("atom", true, false, true, false, 2),
        ("computed", false, true, false, false, 5),
    ] {
        let computed = index % 2 == 1;
        let text = source(kind, word, deep, late, computed, continuing, INPUTS[index]);
        let (result, prints) = expected(word, deep, late, computed, continuing, INPUTS[index]);
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(&successful_main(&text), result.map(|_| output.as_str()));
    }
    let text = source("computed", false, true, false, false, false, INPUTS[4])
        .replace("let selected:", "const selected:");
    run(&successful_main(&text), Some("99\n-50\n19\n19"));
    let text = source(
        "computed",
        true,
        true,
        false,
        true,
        false,
        Input {
            left: 1000,
            tail: 2,
            ..INPUTS[3]
        },
    );
    run(&successful_main(&text), Some("99\n0\n0\n0"));
    let text = source(
        "computed",
        true,
        false,
        false,
        true,
        false,
        Input {
            tail: 0,
            ..INPUTS[3]
        },
    )
    .replace("let unused = 10 / left;", "let unused = 10 / tail;");
    run(&successful_main(&text), None);
}

fn successful_main(text: &str) -> String {
    text.replace("print(result); return result;", "print(result); return 0;")
        .replace("print(11); return 11;", "print(11); return 0;")
        .replace("print(19); return 19;", "print(19); return 0;")
}
