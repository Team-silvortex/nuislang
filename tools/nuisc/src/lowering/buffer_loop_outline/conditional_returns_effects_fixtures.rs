use super::*;

pub(super) type Input = (bool, bool, i64, i64, i64, bool);
pub(super) const INPUTS: [Input; 8] = [
    (false, false, 0, 0, 0, false),
    (true, false, 2, 0, 2, false),
    (true, true, 2, 0, 2, false),
    (true, false, -2, 0, 2, false),
    (false, false, 2, 2, 0, false),
    (true, false, 2, -2, 2, false),
    (true, false, 2, 2, 0, false),
    (true, false, 0, 0, 0, true),
];

pub(super) fn source(mode: &str, entry: &str, prefix: &str, shape: &str, input: Input) -> String {
    let (outer, gate, left, right, tail, early) = input;
    let decision = "helper(produce(left)) && (gate || helper(produce(right)))";
    let pure = match mode {
        "return" => format!("return {decision};"),
        "inferred" => format!("let value = {decision}; return value;"),
        "const" => format!("const value: bool = {decision}; return value;"),
        "complete" => format!("if {decision} {{ return true; }} else {{ return false; }}"),
        "partial" => format!("if {decision} {{ return false; }} else {{ let checked = 20 / tail; }}"),
        "suffix" => format!("if {decision} {{ return false; }} let checked = 20 / tail;"),
        "continuation" => format!("if gate {{ return false; }} else {{ let observed = {decision}; }} let checked = 20 / tail;"),
        "zero" => format!("if {decision} {{ return 0; }} let checked = 20 / tail;"),
        _ => unreachable!(),
    };
    let entry = match entry {
        "atom" => "outer",
        "computed" => "helper(produce(left)) && outer",
        "nested" => "outer && (gate || helper(produce(right)))",
        _ => unreachable!(),
    };
    let prints = |yes| match (prefix, yes) {
        ("literal", true) => "print(88);",
        ("literal", false) => "print(66);",
        ("current", _) => "print(stamp);",
        ("multiple", true) => "print(88); print(stamp); print(89);",
        ("multiple", false) => "print(66); print(stamp); print(67);",
        _ => unreachable!(),
    };
    let yes = if shape == "else" {
        String::new()
    } else {
        format!("{} {pure}", prints(true))
    };
    let no = if shape == "then" {
        String::new()
    } else {
        format!("{} {pure}", prints(false))
    };
    let (result, fallback, main) = if mode == "zero" {
        ("i64", "19", "print(result); return result;")
    } else {
        (
            "bool",
            "false",
            "if result { print(11); return 11; } print(19); return 19;",
        )
    };
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, gate: bool, left: i64, right: i64, tail: i64, early: bool, stamp: i64) -> {result} {{
            if early {{ return {fallback}; }}
            print(99);
            if {entry} {{ {yes} }} else {{ {no} }}
            print(77); return {fallback};
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {left}, {right}, {tail}, {early}, 55); {main} }}
    }}")
}

pub(super) fn expected(
    mode: &str,
    entry: &str,
    prefix: &str,
    shape: &str,
    input: Input,
) -> (Option<i64>, Vec<i64>, usize) {
    // Independent source semantics, not a traversal of the outlined NIR/YIR.
    let (outer, gate, left, right, tail, early) = input;
    if early {
        return (Some(19), vec![19], 0);
    }
    let mut prints = vec![99];
    let mut calls = 0;
    let mut evaluate = || -> Option<(bool, bool)> {
        let mut leaf = |value| {
            calls += 1;
            (value != 0).then_some(value > 0)
        };
        let entry = match entry {
            "atom" => outer,
            "computed" => leaf(left)? && outer,
            "nested" => outer && (gate || leaf(right)?),
            _ => unreachable!(),
        };
        let selected = shape == "both" || if shape == "then" { entry } else { !entry };
        if !selected {
            prints.push(77);
            return Some((false, false));
        }
        match prefix {
            "literal" => prints.push(if entry { 88 } else { 66 }),
            "current" => prints.push(55),
            "multiple" => prints.extend(if entry { [88, 55, 89] } else { [66, 55, 67] }),
            _ => unreachable!(),
        }
        if mode == "continuation" && gate {
            return Some((false, true));
        }
        let value = leaf(left)? && (gate || leaf(right)?);
        if matches!(mode, "return" | "inferred" | "const" | "complete") {
            return Some((value, true));
        }
        if mode != "continuation" && value {
            return Some((false, true));
        }
        if tail == 0 {
            return None;
        }
        prints.push(77);
        Some((false, false))
    };
    let Some((value, exited)) = evaluate() else {
        return (None, prints, calls);
    };
    let result = if mode == "zero" {
        if exited {
            0
        } else {
            19
        }
    } else if value {
        11
    } else {
        19
    };
    prints.push(result);
    (Some(result), prints, calls)
}

pub(super) fn event(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|function| function.name == "event")
        .unwrap()
}
