use super::super::super::super::tests::native::run;
use super::*;

#[test]
fn conditional_return_print_aliases_execute_native_selected_work_traps_and_zero_exits() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (kind, mode, entry, shape, index, stamp) in [
        ("atom", "return", "atom", "both", 2, 0),
        ("division", "return", "atom", "then", 2, 2),
        ("remainder", "zero", "nested", "both", 2, -2),
        ("call", "suffix", "atom", "then", 3, -2),
        ("field", "partial", "computed", "both", 2, 2),
        ("call", "continuation", "atom", "both", 2, 2),
        ("division", "return", "atom", "then", 0, 0),
        ("call", "return", "atom", "then", 0, 0),
        ("field", "return", "nested", "both", 7, 0),
        ("division", "return", "atom", "then", 2, 0),
        ("remainder", "return", "atom", "both", 4, 0),
        ("call", "return", "atom", "then", 2, 0),
        ("field", "return", "atom", "then", 2, 0),
        ("division", "return", "computed", "then", 0, 2),
        ("call", "return", "nested", "both", 1, 2),
        ("call", "continuation", "atom", "then", 6, 2),
    ] {
        let input = INPUTS[index];
        let text = alias_source(kind, mode, entry, shape, input, stamp)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;")
            .replace("print(result); return result;", "print(result); return 0;");
        let (result, prints, _) = alias_expected(kind, mode, entry, shape, input, stamp);
        let output = prints
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        run(&text, result.map(|_| output.as_str()));
    }
}
