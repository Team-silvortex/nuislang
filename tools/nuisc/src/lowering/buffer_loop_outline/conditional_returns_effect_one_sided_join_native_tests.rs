use super::*;
use crate::lowering::buffer_loop_outline::conditional_returns::tests::native::run;

#[test]
fn conditional_return_one_sided_effect_joins_execute_default_source_aot_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (word, swapped, deep, partial, index) in [
        (true, false, false, false, 1),
        (false, true, false, false, 1),
        (true, false, true, false, 2),
        (false, true, true, false, 2),
        (true, false, false, true, 3),
        (false, false, true, true, 4),
        (true, false, true, false, 5),
        (false, false, true, false, 5),
        (true, false, true, false, 6),
        (false, false, true, false, 6),
        (false, true, true, false, 7),
        (true, true, true, false, 0),
    ] {
        let text = source("computed", word, swapped, deep, partial, INPUTS[index]);
        let (result, prints) = expected(word, swapped, deep, partial, INPUTS[index]);
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
    let text = source("logical", false, false, true, false, INPUTS[5])
        .replace("let selected:", "const selected:");
    run(
        &text.replace("print(result); return result;", "print(result); return 0;"),
        Some("99\n0\n19\n19"),
    );
}
