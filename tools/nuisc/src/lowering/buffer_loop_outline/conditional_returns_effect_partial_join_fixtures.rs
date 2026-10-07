#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub nested: bool,
    pub stop_left: bool,
    pub stop_right: bool,
    pub left: i64,
    pub right: i64,
    pub tail: i64,
}

pub(super) const INPUTS: [Input; 10] = [
    Input {
        outer: false,
        gate: true,
        nested: true,
        stop_left: true,
        stop_right: true,
        left: 0,
        right: 0,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        stop_left: true,
        stop_right: false,
        left: 0,
        right: 0,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        stop_left: false,
        stop_right: true,
        left: 0,
        right: 0,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        stop_left: false,
        stop_right: true,
        left: 2,
        right: 0,
        tail: -2,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        stop_left: true,
        stop_right: false,
        left: 0,
        right: -2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        stop_left: true,
        stop_right: true,
        left: 2,
        right: 2,
        tail: 0,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        stop_left: true,
        stop_right: true,
        left: 2,
        right: 2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        stop_left: false,
        stop_right: false,
        left: 0,
        right: 2,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        stop_left: false,
        stop_right: false,
        left: 2,
        right: 0,
        tail: 0,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        stop_left: true,
        stop_right: true,
        left: 0,
        right: -2,
        tail: -2,
    },
];

pub(super) fn source(
    kind: &str,
    word: bool,
    deep: bool,
    late: bool,
    computed_exit: bool,
    continuing: bool,
    input: Input,
) -> String {
    let condition = match kind {
        "atom" => "gate",
        "computed" => "choose(gate)",
        "logical" => "(gate || choose(nested)) && choose(gate)",
        _ => unreachable!(),
    };
    let exit = match (word, computed_exit) {
        (true, false) => "0",
        (false, false) => "false",
        (true, true) => "exit_word(tail)",
        (false, true) => "exit_bool(tail)",
    };
    let arm = |divisor: &str, stop: &str, marker| {
        let local = if deep {
            let early = format!("if {stop} {{ return {exit}; }}");
            let piece = format!("let piece = observe({divisor}); print(piece);");
            let block = if late {
                format!("{piece} {early} piece")
            } else {
                format!("{early} {piece} piece")
            };
            format!("let local: i64 = if choose(nested) {{ {block} }} else {{ {block} }};")
        } else {
            let early = format!("if {stop} {{ print({marker}); return {exit}; }}");
            let value = format!("let local = observe({divisor}); print(local);");
            if late {
                format!("{value} {early}")
            } else {
                format!("{early} {value}")
            }
        };
        let value = if word { "local" } else { "positive(local)" };
        format!("{local} let unused = 10 / {divisor}; {value}")
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
    let end = if continuing {
        "let ignored = observe(tail); print(89);"
    } else {
        "return selected;"
    };
    let Input {
        outer,
        gate,
        nested,
        stop_left,
        stop_right,
        left,
        right,
        tail,
    } = input;
    format!("mod cpu Main {{
        @noinline fn choose(value: bool) -> bool {{ return value; }}
        @noinline fn observe(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn positive(value: i64) -> bool {{ return value > 0; }}
        @noinline fn exit_word(value: i64) -> i64 {{ let unused = 10 / value; return 100 / value; }}
        @noinline fn exit_bool(value: i64) -> bool {{ let unused = 10 / value; return value < 0; }}
        @noinline fn event(outer: bool, gate: bool, nested: bool, stop_left: bool, stop_right: bool, left: i64, right: i64, tail: i64) -> {ty} {{
            print(99); if outer {{ let selected: {ty} = if {condition} {{ {} }} else {{ {} }}; {printed} {end} }}
            print(77); return {fallback};
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {nested}, {stop_left}, {stop_right}, {left}, {right}, {tail}); {main} }}
    }}", arm("left", "stop_left", 31), arm("right", "stop_right", 37))
}

// Source control flow only: no compiler slots, seeds or helper layout facts.
pub(super) fn expected(
    word: bool,
    deep: bool,
    late: bool,
    computed_exit: bool,
    continuing: bool,
    input: Input,
) -> (Option<i64>, Vec<i64>) {
    let mut prints = vec![99];
    if !input.outer {
        prints.extend([77, 19]);
        return (Some(19), prints);
    }
    let (divisor, stop, marker) = if input.gate {
        (input.left, input.stop_left, 31)
    } else {
        (input.right, input.stop_right, 37)
    };
    if late || !stop {
        if divisor == 0 {
            return (None, prints);
        }
        prints.push(100 / divisor);
    }
    if stop {
        if !deep {
            prints.push(marker);
        }
        if computed_exit && input.tail == 0 {
            return (None, prints);
        }
        let result = match (word, computed_exit) {
            (true, false) => 0,
            (true, true) => 100 / input.tail,
            (false, false) => 19,
            (false, true) => {
                if input.tail < 0 {
                    11
                } else {
                    19
                }
            }
        };
        prints.push(result);
        return (Some(result), prints);
    }
    let value = 100 / divisor;
    let result = if word {
        value
    } else if value > 0 {
        11
    } else {
        19
    };
    prints.push(result);
    if continuing {
        if input.tail == 0 {
            return (None, prints);
        }
        prints.extend([89, 77, 19]);
        (Some(19), prints)
    } else {
        prints.push(result);
        (Some(result), prints)
    }
}
