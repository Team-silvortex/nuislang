use super::super::super::super::super::tests::native::run;
use super::*;

#[test]
fn conditional_return_logical_initializers_execute_default_aot_short_circuit_and_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (tree, kind, mode, shape, index) in [
        (0, "call", "return", "then", 0),
        (0, "call", "return", "then", 1),
        (0, "call", "return", "then", 2),
        (0, "division", "unused", "then", 1),
        (0, "call", "unused", "then", 2),
        (1, "call", "return", "both", 3),
        (1, "field", "zero", "both", 5),
        (2, "call", "suffix", "both", 3),
        (2, "division", "zero", "then", 4),
        (3, "field", "return", "else", 5),
        (3, "call", "suffix", "then", 2),
        (3, "division", "return", "both", 7),
        (0, "call", "return", "then", 4),
        (0, "call", "suffix", "then", 1),
        (2, "call", "return", "both", 2),
        (3, "call", "return", "else", 6),
    ] {
        let text = source(tree, kind, mode, shape, index % 3, INPUTS[index]);
        let (result, prints, _) = expected(tree, kind, mode, shape, INPUTS[index]);
        native_case(&text, result, &prints);
    }
    for mode in ["return", "unused"] {
        let input = Input {
            outer: true,
            gate: true,
            values: [0, 2, 2],
            tail: 2,
            early: false,
        };
        let text = source(0, "call", mode, "then", 2, input);
        let (result, prints, _) = expected(0, "call", mode, "then", input);
        native_case(&text, result, &prints);
    }
    for gate in [false, true] {
        native_case(
            &chain_source(gate, 2),
            Some(if gate { 11 } else { 19 }),
            if gate {
                &[99, 88, 89, 50, 90, 11]
            } else {
                &[99, 88, 89, 50, 90, 19]
            },
        );
    }
    for input in [INPUTS[0], INPUTS[2]] {
        let text = source(0, "call", "return", "then", 1, input)
            .replace("if outer {", "if helper(produce(tail)) && outer {");
        native_case(
            &text,
            if input.tail == 0 { None } else { Some(11) },
            &[99, 88, 89, 11],
        );
    }
    for gate in [false, true] {
        let input = Input {
            outer: true,
            gate,
            values: [0, 2, 2],
            tail: 2,
            early: false,
        };
        native_case(
            &before_print_source(input),
            if gate { None } else { Some(19) },
            &[99, 88, 89, 19],
        );
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
