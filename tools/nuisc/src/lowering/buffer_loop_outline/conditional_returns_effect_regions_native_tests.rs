use super::super::super::super::tests::native::run;
use super::*;

#[test]
fn conditional_return_staged_initializers_execute_native_order_unused_work_and_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, mode, entry, shape, index, stamp) in [
        ("division", "return", "atom", "then", 2, 2),
        ("remainder", "zero", "nested", "both", 2, -2),
        ("call", "suffix", "atom", "then", 3, -2),
        ("field", "partial", "computed", "both", 2, 2),
        ("call", "continuation", "atom", "both", 2, 2),
        ("division", "return", "atom", "then", 0, 0),
        ("division", "return", "atom", "then", 2, 0),
        ("call", "return", "nested", "both", 1, 2),
    ] {
        let text = staged_source(kind, mode, entry, shape, INPUTS[index], stamp);
        let (result, prints, _) = staged_expected(kind, mode, entry, shape, INPUTS[index], stamp);
        native_case(&text, result, &prints);
    }
    for (kind, outer, stamp) in [
        ("checked", true, -2),
        ("bool", true, 2),
        ("chain", true, 2),
        ("unused", true, 2),
        ("unused-between", true, 2),
        ("unused", false, 0),
        ("unused-checked", false, 0),
        ("unused", true, 0),
        ("unused-checked", true, 0),
        ("unused-between", true, 0),
    ] {
        let text = simple_source(kind, outer, stamp);
        let (result, prints, _) = simple_expected(kind, outer, stamp);
        native_case(&text, result, &prints);
    }
}

fn native_case(text: &str, result: Option<i64>, prints: &[i64]) {
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
