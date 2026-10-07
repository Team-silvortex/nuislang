#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub nested: bool,
    pub value: i64,
    pub right: i64,
    pub tail: i64,
    pub early: bool,
}

pub(super) const INPUTS: [Input; 8] = [
    Input {
        outer: false,
        gate: false,
        nested: false,
        value: 0,
        right: 0,
        tail: 0,
        early: false,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        value: 0,
        right: 0,
        tail: 2,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        value: 2,
        right: 2,
        tail: 2,
        early: false,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        value: -2,
        right: 0,
        tail: -2,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        value: -2,
        right: 2,
        tail: 0,
        early: false,
    },
    Input {
        outer: false,
        gate: true,
        nested: true,
        value: 2,
        right: -2,
        tail: 2,
        early: false,
    },
    Input {
        outer: false,
        gate: false,
        nested: false,
        value: -2,
        right: 2,
        tail: 0,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        value: 0,
        right: 0,
        tail: 0,
        early: true,
    },
];

pub(super) fn source(
    kind: &str,
    nested: bool,
    mode: &str,
    shape: &str,
    position: usize,
    input: Input,
) -> String {
    let predicate = match kind {
        "atom" => "gate",
        "computed" => "helper(produce(value))",
        "logical" => "(ready || gate) && helper(produce(right))",
        _ => unreachable!(),
    };
    let child = if nested {
        "if nested { const inner: i64 = 100 / tail; print(inner); } else { print(55); }"
    } else {
        ""
    };
    let arm = |marker| {
        let tail = match mode {
            "return" => "return ready;",
            "partial" => "if ready { return false; } let checked = 20 / tail;",
            "zero" => "if ready { return 0; } let checked = 20 / tail;",
            _ => unreachable!(),
        };
        let before = if position == 1 {
            format!("print({marker});")
        } else {
            String::new()
        };
        let after = if position != 2 {
            format!("print({});", marker + 1)
        } else {
            String::new()
        };
        format!("let ready = gate && helper(produce(value)); {before}
            if {predicate} {{ let local = observe(right); print(local); {child} const copy: i64 = local; print(copy); }}
            else {{ let local = observe(tail); print(local); }} {after} {tail}")
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
    let zero = mode == "zero";
    let ty = if zero { "i64" } else { "bool" };
    let fallback = if zero { "19" } else { "false" };
    let main = if zero {
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
        early,
    } = input;
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn observe(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn event(outer: bool, gate: bool, nested: bool, value: i64, right: i64, tail: i64, early: bool) -> {ty} {{
            if early {{ return {fallback}; }} print(99); if outer {{ {yes} }} else {{ {no} }} print(77); return {fallback};
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {nested}, {value}, {right}, {tail}, {early}); {main} }}
    }}")
}

pub(super) fn expected(
    kind: &str,
    nested: bool,
    mode: &str,
    shape: &str,
    position: usize,
    input: Input,
) -> (Option<i64>, Vec<i64>, usize) {
    let mut prints = Vec::new();
    let mut calls = 0;
    let mut result = 19;
    if !input.early {
        prints.push(99);
        if shape == "both" || input.outer == (shape == "then") {
            let mut predicate = |value| {
                if value == 0 {
                    None
                } else {
                    calls += 1;
                    Some(value > 0)
                }
            };
            let ready = if input.gate {
                let Some(ready) = predicate(input.value) else {
                    return (None, prints, calls);
                };
                ready
            } else {
                false
            };
            let marker = if input.outer { 88 } else { 66 };
            if position == 1 {
                prints.push(marker);
            }
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
            let value = if condition { input.right } else { input.tail };
            if value == 0 {
                return (None, prints, calls);
            }
            prints.push(100 / value);
            if condition {
                if nested {
                    if input.nested {
                        if input.tail == 0 {
                            return (None, prints, calls);
                        }
                        prints.push(100 / input.tail);
                    } else {
                        prints.push(55);
                    }
                }
                prints.push(100 / value);
            }
            if position != 2 {
                prints.push(marker + 1);
            }
            match mode {
                "return" => result = if ready { 11 } else { 19 },
                "partial" | "zero" => {
                    if ready {
                        result = if mode == "zero" { 0 } else { 19 };
                    } else {
                        if input.tail == 0 {
                            return (None, prints, calls);
                        }
                        prints.push(77);
                    }
                }
                _ => unreachable!(),
            }
        } else {
            prints.push(77);
        }
    }
    prints.push(result);
    (Some(result), prints, calls)
}
