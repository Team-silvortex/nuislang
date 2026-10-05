use super::super::super::tests::native::run;
use super::*;

#[test]
fn conditional_return_effects_execute_native_print_order_exits_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (mode, entry, prefix, shape, input) in [
        ("return", "atom", "multiple", "then", INPUTS[2]),
        ("const", "computed", "current", "else", INPUTS[4]),
        ("complete", "nested", "multiple", "both", INPUTS[5]),
        ("suffix", "atom", "multiple", "then", INPUTS[3]),
        ("partial", "atom", "literal", "then", INPUTS[6]),
        ("continuation", "atom", "current", "both", INPUTS[2]),
        ("return", "atom", "multiple", "both", INPUTS[7]),
        ("return", "atom", "literal", "then", INPUTS[0]),
        ("return", "computed", "literal", "then", INPUTS[0]),
        ("return", "nested", "literal", "both", INPUTS[1]),
        ("return", "atom", "multiple", "then", INPUTS[1]),
        ("suffix", "atom", "literal", "else", INPUTS[4]),
    ] {
        let text = source(mode, entry, prefix, shape, input)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        let (result, prints, _) = expected(mode, entry, prefix, shape, input);
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(&text, result.map(|_| output.as_str()));
    }
}

#[test]
fn conditional_return_effects_execute_native_zero_return_without_fallthrough_replay() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (entry, shape, input) in [
        ("atom", "then", INPUTS[6]),
        ("nested", "both", INPUTS[2]),
        ("atom", "else", INPUTS[4]),
        ("nested", "then", INPUTS[0]),
    ] {
        let text = source("zero", entry, "multiple", shape, input)
            .replace("print(result); return result;", "print(result); return 0;");
        let (result, prints, _) = expected("zero", entry, "multiple", shape, input);
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(&text, result.map(|_| output.as_str()));
    }
}
