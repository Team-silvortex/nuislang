#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub values: [i64; 3],
    pub tail: i64,
    pub early: bool,
}

pub(super) const INPUTS: [Input; 8] = [
    Input {
        outer: false,
        gate: false,
        values: [0, 0, 0],
        tail: 0,
        early: false,
    },
    Input {
        outer: true,
        gate: false,
        values: [0, 0, 0],
        tail: 0,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        values: [2, 0, 0],
        tail: 2,
        early: false,
    },
    Input {
        outer: true,
        gate: false,
        values: [-2, 2, 0],
        tail: 2,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        values: [-2, -2, 2],
        tail: 0,
        early: false,
    },
    Input {
        outer: false,
        gate: true,
        values: [2, -2, 2],
        tail: 2,
        early: false,
    },
    Input {
        outer: false,
        gate: false,
        values: [-2, -2, 0],
        tail: 0,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        values: [0, 0, 0],
        tail: 0,
        early: true,
    },
];

enum Tree {
    Gate,
    Leaf(usize),
    Edge(bool, Box<Tree>, Box<Tree>),
}

impl Tree {
    fn source(&self, kind: &str) -> String {
        match self {
            Self::Gate => "gate".into(),
            Self::Leaf(index) => {
                let value = ["left", "middle", "right"][*index];
                match kind {
                    "call" => format!("helper(produce({value}))"),
                    "field" => format!("produce({value}).value > 0"),
                    "division" => format!("10 / {value} > 0"),
                    _ => unreachable!(),
                }
            }
            Self::Edge(or, left, right) => format!(
                "({}) {} ({})",
                left.source(kind),
                if *or { "||" } else { "&&" },
                right.source(kind)
            ),
        }
    }

    fn evaluate(&self, kind: &str, input: Input, calls: &mut usize) -> Option<bool> {
        match self {
            Self::Gate => Some(input.gate),
            Self::Leaf(index) => {
                let value = input.values[*index];
                if value == 0 {
                    return None;
                }
                *calls += usize::from(kind == "call");
                Some(value > 0)
            }
            Self::Edge(or, left, right) => {
                let left = left.evaluate(kind, input, calls)?;
                if left == *or {
                    Some(left)
                } else {
                    right.evaluate(kind, input, calls)
                }
            }
        }
    }
}

fn tree(index: usize) -> Tree {
    let edge = |or, left, right| Tree::Edge(or, Box::new(left), Box::new(right));
    match index {
        0 => edge(false, Tree::Gate, Tree::Leaf(0)),
        1 => edge(true, Tree::Leaf(0), Tree::Gate),
        2 => edge(
            false,
            edge(true, Tree::Gate, Tree::Leaf(0)),
            edge(true, Tree::Leaf(1), Tree::Leaf(2)),
        ),
        3 => edge(
            true,
            edge(false, Tree::Leaf(0), Tree::Leaf(1)),
            edge(false, Tree::Gate, Tree::Leaf(2)),
        ),
        _ => unreachable!(),
    }
}

pub(super) fn source(
    index: usize,
    kind: &str,
    mode: &str,
    shape: &str,
    declaration: usize,
    input: Input,
) -> String {
    let declaration = ["let ready", "let ready: bool", "const ready: bool"][declaration];
    let arm = |print| {
        let tail = match mode {
            "return" => "return copy;",
            "suffix" => "if copy { return false; } let checked = 20 / tail;",
            "zero" => "if copy { return 0; } let checked = 20 / tail;",
            "unused" => "return true;",
            _ => unreachable!(),
        };
        format!(
            "print({print}); {declaration} = {}; const copy: bool = ready; print({}); {tail}",
            tree(index).source(kind),
            print + 1
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
    let zero = mode == "zero";
    let result_type = if zero { "i64" } else { "bool" };
    let fallback = if zero { "19" } else { "false" };
    let main = if zero {
        "print(result); return result;"
    } else {
        "if result { print(11); return 11; } print(19); return 19;"
    };
    let Input {
        outer,
        gate,
        values: [left, middle, right],
        tail,
        early,
    } = input;
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn observe(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn to_word(flag: bool) -> i64 {{ if flag {{ return 1; }} return 0; }}
        @noinline fn event(outer: bool, gate: bool, left: i64, middle: i64, right: i64, tail: i64, early: bool) -> {result_type} {{
            if early {{ return {fallback}; }} print(99);
            if outer {{ {yes} }} else {{ {no} }} print(77); return {fallback};
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {left}, {middle}, {right}, {tail}, {early}); {main} }}
    }}")
}

pub(super) fn expected(
    index: usize,
    kind: &str,
    mode: &str,
    shape: &str,
    input: Input,
) -> (Option<i64>, Vec<i64>, usize) {
    let mut prints = Vec::new();
    let mut calls = 0;
    let mut result = 19;
    if !input.early {
        prints.push(99);
        if shape == "both" || input.outer == (shape == "then") {
            let marker = if input.outer { 88 } else { 66 };
            prints.push(marker);
            let Some(value) = tree(index).evaluate(kind, input, &mut calls) else {
                return (None, prints, calls);
            };
            prints.push(marker + 1);
            match mode {
                "unused" => result = 11,
                "return" => result = if value { 11 } else { 19 },
                "suffix" | "zero" => {
                    if value {
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

pub(super) fn chain_source(gate: bool, left: i64) -> String {
    let input = Input {
        outer: true,
        gate,
        values: [left, 2, 2],
        tail: 2,
        early: false,
    };
    source(0, "call", "return", "then", 0, input).replace(
        "print(89); return copy;",
        "print(89); let sample = observe(tail); print(sample); let again = copy && helper(produce(right)); print(90); return again;",
    )
}

pub(super) fn before_print_source(input: Input) -> String {
    source(0, "call", "return", "then", 0, input)
        .replace("print(88); let ready =", "let ready =")
        .replace(
            "; const copy: bool = ready;",
            "; print(88); const copy: bool = ready;",
        )
}
