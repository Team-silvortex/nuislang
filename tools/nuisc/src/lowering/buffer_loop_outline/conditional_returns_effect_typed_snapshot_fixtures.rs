#[derive(Clone, Copy)]
pub(super) struct Input {
    pub outer: bool,
    pub gate: bool,
    pub nested: bool,
    pub left: i64,
    pub right: i64,
}

pub(super) const INPUTS: [Input; 8] = [
    Input {
        outer: false,
        gate: true,
        nested: true,
        left: 0,
        right: 0,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        left: 2,
        right: 0,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        left: 0,
        right: -2,
    },
    Input {
        outer: true,
        gate: true,
        nested: false,
        left: 2,
        right: 0,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        left: 0,
        right: 2,
    },
    Input {
        outer: true,
        gate: true,
        nested: true,
        left: 0,
        right: 2,
    },
    Input {
        outer: true,
        gate: false,
        nested: true,
        left: 2,
        right: 0,
    },
    Input {
        outer: true,
        gate: false,
        nested: false,
        left: 0,
        right: -2,
    },
];

pub(super) fn source(ty: &str, shape: &str, constant: bool, partial: bool, input: Input) -> String {
    let binding = if constant { "const" } else { "let" };
    let literal = if ty == "i32" {
        "i32_from_i64(0)"
    } else {
        "0.0"
    };
    let step = if ty == "i32" {
        "i32_from_i64(1)"
    } else {
        "0.5"
    };
    let arm = |name: &str, divisor: &str, marker: i64| {
        let value = match shape {
            "computed" => "local",
            "shared" => "shared",
            "literal" => literal,
            _ => unreachable!(),
        };
        let exit = if partial {
            "if nested { print(72); return false; }"
        } else {
            ""
        };
        format!("{exit} let local: {ty} = observe({name}, {divisor}); print({marker}); {value}")
    };
    let (left_value, right_value, shared) = if ty == "i32" {
        ("i32_from_i64(25)", "i32_from_i64(-17)", "i32_from_i64(7)")
    } else {
        ("2.5", "-1.25", "7.0")
    };
    let Input {
        outer,
        gate,
        nested,
        left,
        right,
        ..
    } = input;
    format!("mod cpu Main {{
        @noinline fn choose(value: bool) -> bool {{ return value; }}
        @noinline fn observe(value: {ty}, divisor: i64) -> {ty} {{
            let unused = 10 / divisor; return value + {step};
        }}
        @noinline fn inspect(value: {ty}, flag: bool) -> bool {{ return flag; }}
        @noinline fn event(outer: bool, gate: bool, nested: bool,
            left_value: {ty}, right_value: {ty}, shared: {ty}, left: i64, right: i64) -> bool {{
            print(99); if outer {{
                {binding} selected: {ty} = if choose(gate) {{ {} }} else {{ {} }};
                let accepted = inspect(selected, nested); print(81); return accepted;
            }} print(77); return false;
        }}
        fn main() -> i64 {{
            let result = event({outer}, {gate}, {nested}, {left_value}, {right_value}, {shared}, {left}, {right});
            if result {{ print(11); return 11; }} print(19); return 19;
        }}
    }}", arm("left_value", "left", 70), arm("right_value", "right", 71))
}

// Source paths, not inactive seeds or generated helper names, determine exits.
pub(super) fn expected(partial: bool, input: Input) -> (Option<i64>, Vec<i64>) {
    if !input.outer {
        return (Some(19), vec![99, 77, 19]);
    }
    if partial && input.nested {
        return (Some(19), vec![99, 72, 19]);
    }
    let divisor = if input.gate { input.left } else { input.right };
    if divisor == 0 {
        return (None, vec![99]);
    }
    let result = if input.nested { 11 } else { 19 };
    (
        Some(result),
        vec![99, if input.gate { 70 } else { 71 }, 81, result],
    )
}

pub(super) fn observed(ty: &str, shape: &str, gate: bool) -> String {
    match (ty, shape, gate) {
        ("i32", "computed", true) => "26i32",
        ("i32", "computed", false) => "-16i32",
        ("i32", "shared", _) => "7i32",
        ("i32", "literal", _) => "0i32",
        (_, "computed", true) if ty == "f32" => "3f32",
        (_, "computed", false) if ty == "f32" => "-0.75f32",
        (_, "computed", true) => "3f64",
        (_, "computed", false) => "-0.75f64",
        ("f32", "shared", _) => "7f32",
        ("f64", "shared", _) => "7f64",
        ("f32", "literal", _) => "0f32",
        ("f64", "literal", _) => "0f64",
        _ => unreachable!(),
    }
    .into()
}
