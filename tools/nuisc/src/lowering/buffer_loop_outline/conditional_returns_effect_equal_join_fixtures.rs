#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub nested: bool,
    pub stop: bool,
    pub left: i64,
    pub right: i64,
    pub shared: i64,
    pub flag: bool,
}

pub(super) const INPUTS: [Input; 8] = [
    Input {
        outer: false,
        gate: true,
        nested: true,
        stop: true,
        left: 0,
        right: 0,
        shared: 7,
        flag: true,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        stop: false,
        left: 2,
        right: 0,
        shared: 7,
        flag: true,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        stop: false,
        left: 0,
        right: -2,
        shared: -3,
        flag: false,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        stop: true,
        left: 0,
        right: 2,
        shared: 7,
        flag: true,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        stop: true,
        left: 2,
        right: 0,
        shared: -3,
        flag: true,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        stop: false,
        left: 0,
        right: 2,
        shared: 7,
        flag: false,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        stop: false,
        left: 2,
        right: 0,
        shared: 7,
        flag: true,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        stop: false,
        left: 1000,
        right: 0,
        shared: 0,
        flag: false,
    },
];

pub(super) fn source(
    kind: &str,
    word: bool,
    shared: bool,
    constant: bool,
    partial: bool,
    input: Input,
) -> String {
    let condition = match kind {
        "atom" => "gate",
        "computed" => "choose(gate)",
        "logical" => "(gate && choose(nested)) || choose(gate)",
        _ => unreachable!(),
    };
    let value = match (word, shared) {
        (true, true) => "shared",
        (false, true) => "flag",
        (true, false) => "0",
        (false, false) => "false",
    };
    let arm = |divisor, marker| {
        let early = if partial {
            format!("if stop {{ print({marker}); return 0; }}")
        } else {
            String::new()
        };
        format!("{early} let local = observe({divisor}); print(local); let unused = 10 / {divisor}; {value}")
    };
    let ty = if word { "i64" } else { "bool" };
    let binding = if constant { "const" } else { "let" };
    let end = if word {
        "print(selected); return selected;"
    } else {
        "if selected { print(11); return 11; } print(19); return 19;"
    };
    let Input {
        outer,
        gate,
        nested,
        stop,
        left,
        right,
        shared,
        flag,
    } = input;
    format!("mod cpu Main {{
        @noinline fn choose(value: bool) -> bool {{ return value; }}
        @noinline fn observe(value: i64) -> i64 {{ return 100 / value; }}
        @noinline fn event(outer: bool, gate: bool, nested: bool, stop: bool, left: i64, right: i64, shared: i64, flag: bool) -> i64 {{
            print(99); if outer {{ {binding} selected: {ty} = if {condition} {{ {} }} else {{ {} }}; print(81); {end} }}
            print(77); return 19;
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {nested}, {stop}, {left}, {right}, {shared}, {flag}); print(result); return result; }}
    }}", arm("left", 31), arm("right", 37))
}

// Evaluate source paths independently of compiler snapshots and inactive seeds.
pub(super) fn expected(
    word: bool,
    shared: bool,
    partial: bool,
    input: Input,
) -> (Option<i64>, Vec<i64>) {
    let mut prints = vec![99];
    if !input.outer {
        prints.extend([77, 19]);
        return (Some(19), prints);
    }
    if partial && input.stop {
        prints.extend([if input.gate { 31 } else { 37 }, 0]);
        return (Some(0), prints);
    }
    let divisor = if input.gate { input.left } else { input.right };
    if divisor == 0 {
        return (None, prints);
    }
    prints.extend([100 / divisor, 81]);
    let result = if word {
        if shared {
            input.shared
        } else {
            0
        }
    } else if shared && input.flag {
        11
    } else {
        19
    };
    prints.extend([result, result]);
    (Some(result), prints)
}
