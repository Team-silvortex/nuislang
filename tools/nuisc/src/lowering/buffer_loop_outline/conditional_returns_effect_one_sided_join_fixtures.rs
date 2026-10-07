#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub nested: bool,
    pub stop: bool,
    pub value: i64,
    pub tail: i64,
}

pub(super) const INPUTS: [Input; 8] = [
    Input {
        outer: false,
        gate: false,
        nested: false,
        stop: false,
        value: 0,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        stop: false,
        value: 2,
        tail: 0,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        stop: false,
        value: 0,
        tail: 2,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        stop: true,
        value: 0,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        stop: true,
        value: 2,
        tail: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        stop: false,
        value: 1000,
        tail: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        stop: false,
        value: -2,
        tail: -2,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        stop: false,
        value: 2,
        tail: 2,
    },
];

pub(super) fn source(
    kind: &str,
    word: bool,
    swapped: bool,
    deep: bool,
    partial: bool,
    input: Input,
) -> String {
    let condition = match kind {
        "atom" => "gate",
        "computed" => "choose(gate)",
        "logical" => "(gate && choose(nested)) || choose(gate)",
        _ => unreachable!(),
    };
    let exiting = if deep {
        "if choose(nested) { print(31); return leave(tail); } else { print(37); return 0; }"
    } else {
        "print(31); return leave(tail);"
    };
    let early = if partial {
        "if stop { print(44); return 0; }"
    } else {
        ""
    };
    let value = if word { "local" } else { "positive(local)" };
    let continuing = format!(
        "{early} let local = observe(value); print(local); let unused = 10 / value; {value}"
    );
    let (yes, no) = if swapped {
        (exiting, continuing.as_str())
    } else {
        (continuing.as_str(), exiting)
    };
    let (ty, end) = if word {
        ("i64", "print(selected); return selected;")
    } else {
        (
            "bool",
            "if selected { print(11); return 11; } else { print(19); return 19; }",
        )
    };
    let Input {
        outer,
        gate,
        nested,
        stop,
        value,
        tail,
    } = input;
    format!("mod cpu Main {{
        @noinline fn choose(value: bool) -> bool {{ return value; }}
        @noinline fn observe(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn positive(value: i64) -> bool {{ return value > 0; }}
        @noinline fn leave(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn event(outer: bool, gate: bool, nested: bool, stop: bool, value: i64, tail: i64) -> i64 {{
            print(99); if outer {{ let selected: {ty} = if {condition} {{ {yes} }} else {{ {no} }}; {end} }}
            print(77); return 19;
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {nested}, {stop}, {value}, {tail}); print(result); return result; }}
    }}")
}

// The oracle follows source control flow, with no compiler slots or live seeds.
pub(super) fn expected(
    word: bool,
    swapped: bool,
    deep: bool,
    partial: bool,
    input: Input,
) -> (Option<i64>, Vec<i64>) {
    let mut prints = vec![99];
    if !input.outer {
        prints.extend([77, 19]);
        return (Some(19), prints);
    }
    if input.gate == swapped {
        if deep && !input.nested {
            prints.extend([37, 0]);
            return (Some(0), prints);
        }
        prints.push(31);
        if input.tail == 0 {
            return (None, prints);
        }
        let result = 100 / input.tail;
        prints.push(result);
        return (Some(result), prints);
    }
    if partial && input.stop {
        prints.extend([44, 0]);
        return (Some(0), prints);
    }
    if input.value == 0 {
        return (None, prints);
    }
    let value = 100 / input.value;
    prints.push(value);
    let result = if word {
        value
    } else if value > 0 {
        11
    } else {
        19
    };
    prints.extend([result, result]);
    (Some(result), prints)
}
