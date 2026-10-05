use super::*;

pub(in super::super) fn computed_source(
    kind: &str,
    mode: &str,
    entry: &str,
    shape: &str,
    input: super::super::fixtures::Input,
    stamp: i64,
) -> String {
    let value = match kind {
        "division" => "100 / stamp",
        "remainder" => "100 % stamp",
        "call" => "observe(stamp)",
        "field" => "produce(stamp).value",
        _ => unreachable!(),
    };
    source(mode, entry, "multiple", shape, input)
        .replace("print(stamp);", &format!("print({value});"))
        .replace(", 55);", &format!(", {stamp});"))
        .replace("@noinline fn event(", "@noinline fn observe(value: i64) -> i64 { let packet = produce(value); return packet.value; } @noinline fn event(")
}

pub(in super::super) fn computed_expected(
    kind: &str,
    mode: &str,
    entry: &str,
    shape: &str,
    input: super::super::fixtures::Input,
    stamp: i64,
) -> (Option<i64>, Vec<i64>, usize) {
    let (result, mut prints, calls) = expected(mode, entry, "multiple", shape, input);
    // The source oracle's marker identifies entry before computing the argument.
    // A failed prefix prevents both its print and all following return work.
    let Some(index) = prints.iter().position(|value| *value == 55) else {
        return (result, prints, calls);
    };
    if stamp == 0 {
        prints.truncate(index);
        return (None, prints, calls);
    }
    prints[index] = match kind {
        "division" => 100 / stamp,
        "remainder" => 100 % stamp,
        "call" | "field" => stamp,
        _ => unreachable!(),
    };
    (result, prints, calls)
}
