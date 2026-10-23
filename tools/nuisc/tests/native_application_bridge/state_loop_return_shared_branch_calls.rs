use super::*;

pub fn source(variant: Variant) -> String {
    with_branches(
        SourceKind::LoopHelperCalls.source(variant),
        variant.mirrored,
    )
}

pub fn with_branches(mut source: String, mirrored: bool) -> String {
    for slot in 0..3 {
        let flip = format!("let flipped_{slot} = carry.result;");
        let at = source.find(&flip).unwrap();
        let begin = source[..at].rfind("carry = loop_record(").unwrap();
        // Mirrored source places the unselected branch before the child loop.
        // Infer the semantic site from its update, never the textual flip index.
        let call = &source[begin..at];
        let (selector, delta, argument_root, callee_root) =
            if call.contains("loop_delta(index + 1,") {
                (
                    "index",
                    "index + 1",
                    "root + 21 - index",
                    "after + 22 - index",
                )
            } else if call.contains("loop_delta(outer,") {
                ("outer", "outer", "1", "1")
            } else {
                assert!(call.contains("loop_delta(2,"));
                ("outer", "2", "1", "1")
            };
        let last = "carry = State { result: carry.result, total: carry.total, gain: carry.gain, marker: -carry.marker };";
        let end = at + source[at..].find(last).unwrap() + last.len();
        assert_eq!(source[begin..end].matches("carry =").count(), 4);
        let call = |branch: &str, safe: &str| {
            format!(
            "carry = branch_{branch}_record(branch_{branch}_seed(carry), branch_{branch}_delta({delta} + 0 / ({safe}), {argument_root}), branch_{branch}_gate({callee_root} + 0 / ({safe})));"
        )
        };
        let even = call("even", &format!("1 - {selector} % 2"));
        let odd = call("odd", &format!("{selector} % 2"));
        let replacement = if mirrored {
            format!("if {selector} % 2 != 0 {{ {odd} }} else {{ {even} }}")
        } else {
            format!("if {selector} % 2 == 0 {{ {even} }} else {{ {odd} }}")
        };
        source.replace_range(begin..end, &replacement);
    }
    let needle = "    @noinline fn work(";
    assert_eq!(source.matches(needle).count(), 1);
    source.replace(
        needle,
        &format!(
            "{}\n{needle}",
            include_str!("state_loop_return_shared_branch_helpers.ns")
        ),
    )
}

pub fn execute(
    call: Call,
    observed: bool,
    budget: Option<u64>,
) -> Result<(State, Vec<i64>), Vec<i64>> {
    let result =
        call.execute_with_assignment_effects(observed, budget, &mut |site, index, trace| {
            assignment(call, site, index, trace)
        });
    result
        .map(|(state, trace)| (state, SourceKind::HelperCalls.trace(trace)))
        .map_err(|trace| SourceKind::HelperCalls.trace(trace))
}

pub fn assignment(call: Call, site: i64, index: i64, trace: &mut Vec<i64>) -> Option<()> {
    if site == 25 {
        trace.extend([810, 811, 814, 812, 813]);
        return Some(());
    }
    let base = if index % 2 == 0 { 910 } else { 920 };
    trace.extend([base, base + 1]);
    if site == 10 && call.root.checked_add(21)?.checked_sub(index)? == 0 {
        return None;
    }
    trace.extend([base + 4, base + 2]);
    if site == 10 && call.after.checked_add(22)?.checked_sub(index)? == 0 {
        return None;
    }
    trace.push(base + 3);
    Some(())
}

pub fn failure(variant: Variant, callee: bool, odd: bool) -> Call {
    let mut call = loop_calls::failure(variant, callee);
    let index = if odd { 3 } else { 2 };
    call.trips = index + 1;
    call.root = if callee { 1 } else { index - 21 };
    call.after = if callee { index - 22 } else { 1 };
    call
}
