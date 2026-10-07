use super::*;
use crate::lowering::buffer_loop_outline::conditional_returns::tests::native::run;

#[test]
fn conditional_return_equal_effect_joins_execute_default_aot_work_exits_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, word, shared, constant, partial, index) in [
        ("computed", true, false, false, false, 1),
        ("logical", false, false, true, false, 2),
        ("atom", true, true, true, false, 2),
        ("computed", false, true, false, false, 1),
        ("logical", true, false, false, true, 3),
        ("computed", false, false, true, true, 4),
        ("computed", true, true, false, false, 0),
        ("atom", false, true, true, false, 7),
        ("computed", true, false, false, false, 5),
        ("computed", false, true, false, false, 6),
    ] {
        let text = source(kind, word, shared, constant, partial, INPUTS[index]);
        let (result, prints) = expected(word, shared, partial, INPUTS[index]);
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(
            &text.replace("print(result); return result;", "print(result); return 0;"),
            result.map(|_| output.as_str()),
        );
    }
    let text =
        source("computed", true, false, false, false, INPUTS[1]).replace("10 / left", "10 / right");
    run(
        &text.replace("print(result); return result;", "print(result); return 0;"),
        None,
    );
}
