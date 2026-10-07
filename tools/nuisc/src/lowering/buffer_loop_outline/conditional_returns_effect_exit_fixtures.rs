#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub nested: bool,
    pub value: i64,
    pub right: i64,
    pub tail: i64,
}

pub(super) const INPUTS: [Input; 10] = [
    Input {
        outer: false,
        gate: true,
        nested: true,
        value: 0,
        right: 0,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        value: 2,
        right: 2,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        value: 2,
        right: 2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        value: 2,
        right: 2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        value: 2,
        right: 2,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        value: 2,
        right: -2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        value: 0,
        right: 2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        value: 0,
        right: 2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        value: 2,
        right: 0,
        tail: 2,
    },
    Input {
        outer: false,
        gate: false,
        nested: false,
        value: 2,
        right: 2,
        tail: 2,
    },
];

pub(super) fn source(kind: &str, mode: &str, deep: bool, shape: &str, input: Input) -> String {
    let condition = match kind {
        "atom" => "gate",
        "computed" => "helper(produce(value))",
        "logical" => "(ready || gate) && helper(produce(right))",
        _ => unreachable!(),
    };
    let (ty, returned, final_value, fallback) = match mode {
        "false" => ("bool", "false", "ready", "false"),
        "computed" => ("bool", "local > 0", "ready", "false"),
        "logical" => ("bool", "gate && helper(produce(tail))", "ready", "false"),
        "zero" => ("i64", "0", "23", "19"),
        "checked" => ("i64", "observe(value)", "23", "19"),
        _ => unreachable!(),
    };
    let exit = if deep {
        format!("if nested {{ if gate {{ print(44); return {returned}; }} }}")
    } else {
        format!("if nested {{ print(44); return {returned}; }}")
    };
    let arm = |marker| {
        format!(
            "print({marker}); let ready = gate && helper(produce(value));
        if {condition} {{ let local = observe(right); print(local); {exit} print(55); }}
        else {{ let local = observe(tail); print(local); {exit} print(67); }}
        let ignored = observe(tail); print(89); return {final_value};"
        )
    };
    let yes = if shape != "else" {
        arm(88)
    } else {
        String::new()
    };
    let no = if shape != "then" {
        arm(66)
    } else {
        String::new()
    };
    let main = if ty == "i64" {
        "print(result); return result;"
    } else {
        "if result { print(11); return 11; } print(19); return 19;"
    };
    let Input {
        outer,
        gate,
        nested,
        value,
        right,
        tail,
    } = input;
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn observe(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn event(outer: bool, gate: bool, nested: bool, value: i64, right: i64, tail: i64) -> {ty} {{
            print(99); if outer {{ {yes} }} else {{ {no} }} print(77); return {fallback};
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {nested}, {value}, {right}, {tail}); {main} }}
    }}")
}

pub(super) fn expected(
    kind: &str,
    mode: &str,
    deep: bool,
    shape: &str,
    input: Input,
) -> (Option<i64>, Vec<i64>, usize) {
    evaluate(kind, mode, deep, shape, input, false)
}

pub(super) fn exit_only_source(
    kind: &str,
    mode: &str,
    deep: bool,
    shape: &str,
    input: Input,
    tail: &str,
) -> String {
    let suffix = match tail {
        "empty" => "",
        "checked" => "let final_check = observe(tail);",
        "branch" => "if gate { const alias = nested; } else { const alias = gate; } let final_check = observe(tail);",
        _ => unreachable!(),
    };
    source(kind, mode, deep, shape, input)
        .replace("return ready;", suffix)
        .replace("return 23;", suffix)
}

pub(super) fn exit_only_expected(
    kind: &str,
    mode: &str,
    deep: bool,
    shape: &str,
    input: Input,
) -> (Option<i64>, Vec<i64>, usize) {
    evaluate(kind, mode, deep, shape, input, true)
}

fn evaluate(
    kind: &str,
    mode: &str,
    deep: bool,
    shape: &str,
    input: Input,
    exit_only: bool,
) -> (Option<i64>, Vec<i64>, usize) {
    let mut prints = vec![99];
    let mut calls = 0;
    if shape != "both" && input.outer != (shape == "then") {
        prints.extend([77, 19]);
        return (Some(19), prints, calls);
    }
    prints.push(if input.outer { 88 } else { 66 });
    let mut predicate = |value| {
        if value == 0 {
            None
        } else {
            calls += 1;
            Some(value > 0)
        }
    };
    let ready = if input.gate {
        let Some(value) = predicate(input.value) else {
            return (None, prints, calls);
        };
        value
    } else {
        false
    };
    let condition = match kind {
        "atom" => Some(input.gate),
        "computed" => predicate(input.value),
        "logical" => {
            if ready || input.gate {
                predicate(input.right)
            } else {
                Some(false)
            }
        }
        _ => unreachable!(),
    };
    let Some(condition) = condition else {
        return (None, prints, calls);
    };
    let divisor = if condition { input.right } else { input.tail };
    if divisor == 0 {
        return (None, prints, calls);
    }
    let local = 100 / divisor;
    prints.push(local);
    let result = if input.nested && (!deep || input.gate) {
        prints.push(44);
        match mode {
            "false" => 19,
            "zero" => 0,
            "computed" => {
                if local > 0 {
                    11
                } else {
                    19
                }
            }
            "logical" => {
                let value = if input.gate {
                    let Some(value) = predicate(input.tail) else {
                        return (None, prints, calls);
                    };
                    value
                } else {
                    false
                };
                if value {
                    11
                } else {
                    19
                }
            }
            "checked" => {
                if input.value == 0 {
                    return (None, prints, calls);
                }
                100 / input.value
            }
            _ => unreachable!(),
        }
    } else {
        prints.push(if condition { 55 } else { 67 });
        if input.tail == 0 {
            return (None, prints, calls);
        }
        prints.push(89);
        if exit_only {
            // Every nonempty fixture suffix checks the same nonzero divisor
            // already observed above, then resumes the parent fallback.
            prints.push(77);
            19
        } else if matches!(mode, "zero" | "checked") {
            23
        } else if ready {
            11
        } else {
            19
        }
    };
    prints.push(result);
    (Some(result), prints, calls)
}

pub(super) fn partial_source(mode: &str, index: usize) -> String {
    let text = source("computed", mode, false, "then", INPUTS[index]).replace(
        "let ignored = observe(tail);",
        "let ignored = observe(right);",
    );
    if mode == "zero" {
        text.replace(
            "return 23;",
            "if gate { return 0; } let final_check = 20 / tail;",
        )
    } else {
        text.replace(
            "return ready;",
            "if gate { return false; } let final_check = 20 / tail;",
        )
    }
}

pub(super) fn partial_expected(mode: &str, index: usize) -> (Option<i64>, Vec<i64>, usize) {
    let result = if mode == "zero" { 0 } else { 19 };
    match index {
        1 => (Some(result), vec![99, 88, 50, 44, result], 2),
        2 => (Some(result), vec![99, 88, 50, 55, 89, result], 2),
        3 => (Some(result), vec![99, 88, 50, 44, result], 1),
        4 => (None, vec![99, 88, 50, 55, 89], 1),
        _ => unreachable!(),
    }
}
