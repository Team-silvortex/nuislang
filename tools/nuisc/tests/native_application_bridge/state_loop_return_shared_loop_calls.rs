use super::*;

pub fn source(variant: Variant) -> String {
    with_loop_calls(SourceKind::HelperCalls.source(variant))
}

pub fn with_loop_calls(mut source: String) -> String {
    for (delta, argument_root, callee_root) in [
        ("outer", "1", "1"),
        ("index + 1", "root + 11 - index", "after + 12 - index"),
        ("2", "1", "1"),
    ] {
        let old = format!("carry = State {{ result: carry.result, total: carry.total + {delta}, gain: carry.gain, marker: carry.marker }};");
        assert_eq!(
            source.matches(&old).count(),
            if delta == "outer" { 3 } else { 1 }
        );
        source = source.replace(&old, &format!("carry = loop_record(loop_seed(carry), loop_delta({delta}, {argument_root}), loop_gate({callee_root}));"));
    }
    let needle = "    @noinline fn work(";
    assert_eq!(source.matches(needle).count(), 1);
    source.replace(
        needle,
        &format!(
            "{}\n{needle}",
            include_str!("state_loop_return_shared_loop_helpers.ns")
        ),
    )
}

// Hook semantic assignment sites, not numeric print values: observed argument
// words may themselves be 8, 9 or 10 and must never become synthetic events.
pub fn execute(
    call: Call,
    observed: bool,
    budget: Option<u64>,
) -> Result<(State, Vec<i64>), Vec<i64>> {
    let result =
        call.execute_with_assignment_effects(observed, budget, &mut |site, index, trace| {
            trace.extend([810, 811]);
            if site == 10 && call.root.checked_add(11)?.checked_sub(index)? == 0 {
                return None;
            }
            trace.extend([814, 812]);
            if site == 10 && call.after.checked_add(12)?.checked_sub(index)? == 0 {
                return None;
            }
            trace.push(813);
            Some(())
        });
    result
        .map(|(state, trace)| (state, SourceKind::HelperCalls.trace(trace)))
        .map_err(|trace| SourceKind::HelperCalls.trace(trace))
}

pub fn failure(variant: Variant, callee: bool) -> Call {
    let mut call = base(variant);
    call.cut = 99;
    call.hole = 99;
    call.trips = 3;
    call.skip = -1;
    call.resume = -1;
    call.parent = -1;
    call.finish = 1;
    call.root = if callee { 1 } else { -9 };
    call.after = if callee { -10 } else { 1 };
    call.select_first(true);
    call
}
