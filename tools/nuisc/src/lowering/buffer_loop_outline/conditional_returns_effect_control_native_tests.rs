use super::super::super::super::super::tests::native::run;
use super::*;

#[test]
fn conditional_return_control_regions_execute_default_aot_order_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, nested, mode, shape, position, index) in [
        ("computed", true, "return", "then", 1, 2),
        ("logical", true, "zero", "both", 2, 5),
        ("atom", false, "partial", "then", 0, 1),
        ("computed", true, "return", "then", 2, 0),
        ("logical", true, "return", "then", 1, 0),
        ("atom", true, "return", "both", 2, 5),
        ("computed", false, "return", "else", 0, 6),
        ("logical", true, "return", "both", 2, 7),
        ("computed", false, "return", "then", 1, 1),
        ("atom", true, "return", "then", 1, 4),
        ("logical", true, "return", "then", 1, 4),
    ] {
        let text = source(kind, nested, mode, shape, position, INPUTS[index]);
        let (result, prints, _) = expected(kind, nested, mode, shape, position, INPUTS[index]);
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
    let base = source("computed", false, "return", "then", 1, INPUTS[2]);
    let condition_only = base
        .replace("let ready = gate && helper(produce(value));", "")
        .replace("return ready;", "return true;")
        .replace(
            "let local = observe(right); print(local);  const copy: i64 = local; print(copy);",
            "",
        )
        .replace("let local = observe(tail); print(local);", "");
    let unused = base.replace(
        "let local = observe(right);",
        "let ignored = observe(tail); let local = observe(right);",
    );
    let chained = base.replace(
        "print(89);",
        "let after = observe(tail); print(after); print(89);",
    );
    for (text, output) in [
        (condition_only.clone(), Some("99\n88\n89\n11")),
        (
            condition_only.replace(
                "event(true, true, true, 2, 2, 2, false)",
                "event(true, true, true, 0, 2, 2, false)",
            ),
            None,
        ),
        (unused.clone(), Some("99\n88\n50\n50\n89\n11")),
        (
            unused.replace(
                "event(true, true, true, 2, 2, 2, false)",
                "event(true, true, true, 2, 2, 0, false)",
            ),
            None,
        ),
        (chained, Some("99\n88\n50\n50\n50\n89\n11")),
    ] {
        run(
            &text
                .replace("return 11;", "return 0;")
                .replace("return 19;", "return 0;"),
            output,
        );
    }
}
