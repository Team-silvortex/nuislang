#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub nested: bool,
    pub left: i64,
    pub right: i64,
    pub tail: i64,
}

pub(super) const INPUTS: [Input; 8] = [
    Input {
        outer: false,
        gate: true,
        nested: true,
        left: 0,
        right: 0,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        left: 2,
        right: 0,
        tail: 0,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        left: 0,
        right: -2,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        left: 2,
        right: 0,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        left: 0,
        right: 2,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        left: 0,
        right: 2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        left: 2,
        right: 0,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        left: 0,
        right: 2,
        tail: 2,
    },
];

pub(super) fn source(kind: &str, word: bool, deep: bool, early: bool, input: Input) -> String {
    let condition = match kind {
        "atom" => "gate",
        "computed" => "choose(gate)",
        "logical" => "(gate || choose(nested)) && choose(gate)",
        _ => unreachable!(),
    };
    let arm = |divisor| {
        let local = if deep {
            format!("let local: i64 = if nested {{ let piece = observe({divisor}); print(piece); piece }} else {{ let piece = observe({divisor}); print(piece); piece }};")
        } else {
            format!("let local = observe({divisor}); print(local);")
        };
        let result = if word {
            "local".into()
        } else {
            format!("helper(produce({divisor}))")
        };
        format!("{local} let unused = 10 / {divisor}; {result}")
    };
    let (ty, printed, fallback, main) = if word {
        (
            "i64",
            "print(selected);",
            "19",
            "print(result); return result;",
        )
    } else {
        (
            "bool",
            "if selected { print(11); } else { print(19); }",
            "false",
            "if result { print(11); return 11; } print(19); return 19;",
        )
    };
    let tail = if early {
        "if nested { print(44); return selected; } let ignored = observe(tail); print(89);"
    } else {
        "return selected;"
    };
    let Input {
        outer,
        gate,
        nested,
        left,
        right,
        tail: divisor,
    } = input;
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn choose(value: bool) -> bool {{ return value; }}
        @noinline fn observe(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn event(outer: bool, gate: bool, nested: bool, left: i64, right: i64, tail: i64) -> {ty} {{
            print(99); if outer {{ let selected: {ty} = if {condition} {{ {} }} else {{ {} }}; {printed} {tail} }}
            print(77); return {fallback};
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {nested}, {left}, {right}, {divisor}); {main} }}
    }}", arm("left"), arm("right"))
}

// An imperative source evaluator, independent of private snapshots and helpers.
pub(super) fn expected(
    kind: &str,
    word: bool,
    early: bool,
    input: Input,
) -> (Option<i64>, Vec<i64>, usize) {
    let mut prints = vec![99];
    if !input.outer {
        prints.extend([77, 19]);
        return (Some(19), prints, 0);
    }
    let selected = match kind {
        "atom" | "computed" => input.gate,
        "logical" => (input.gate || input.nested) && input.gate,
        _ => unreachable!(),
    };
    let divisor = if selected { input.left } else { input.right };
    if divisor == 0 {
        return (None, prints, 0);
    }
    let local = 100 / divisor;
    prints.push(local);
    let result = if word {
        local
    } else if divisor > 0 {
        11
    } else {
        19
    };
    let calls = usize::from(!word);
    prints.push(result);
    if early {
        if input.nested {
            prints.push(44);
        } else {
            if input.tail == 0 {
                return (None, prints, calls);
            }
            prints.extend([89, 77, 19]);
            return (Some(19), prints, calls);
        }
    }
    prints.push(result);
    (Some(result), prints, calls)
}
