use super::super::computed::{computed_expected, computed_source};
use super::*;

pub(super) fn alias_source(
    kind: &str,
    mode: &str,
    entry: &str,
    shape: &str,
    input: super::super::fixtures::Input,
    stamp: i64,
) -> String {
    let text = if kind == "atom" {
        source(mode, entry, "multiple", shape, input).replace(", 55);", &format!(", {stamp});"))
    } else {
        computed_source(kind, mode, entry, shape, input, stamp)
    };
    text.replace("print(88);", "let prefix_stamp: i64 = stamp; print(88);")
        .replace("print(66);", "let prefix_stamp: i64 = stamp; print(66);")
        .replace("print(stamp);", "print(stamp_alias);")
        .replace("/ stamp);", "/ stamp_alias);")
        .replace("% stamp);", "% stamp_alias);")
        .replace("observe(stamp));", "observe(stamp_alias));")
        .replace("produce(stamp).value);", "produce(stamp_alias).value);")
        .replace("print(88);", "print(88); const stamp_alias: i64 = prefix_stamp;")
        .replace("print(66);", "print(66); const stamp_alias: i64 = prefix_stamp;")
        .replace("print(89);", "let left_alias = left; const right_alias: i64 = right; let gate_alias: bool = gate; print(89);")
        .replace("print(67);", "let left_alias = left; const right_alias: i64 = right; let gate_alias: bool = gate; print(67);")
        .replace(
            "helper(produce(left)) && (gate || helper(produce(right)))",
            "helper(produce(left_alias)) && (gate_alias || helper(produce(right_alias)))",
        )
}

pub(super) fn alias_expected(
    kind: &str,
    mode: &str,
    entry: &str,
    shape: &str,
    input: super::super::fixtures::Input,
    stamp: i64,
) -> (Option<i64>, Vec<i64>, usize) {
    if kind != "atom" {
        return computed_expected(kind, mode, entry, shape, input, stamp);
    }
    let (result, mut prints, calls) = expected(mode, entry, "multiple", shape, input);
    for value in &mut prints {
        if *value == 55 {
            *value = stamp;
        }
    }
    (result, prints, calls)
}
