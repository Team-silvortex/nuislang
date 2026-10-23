use super::*;

pub fn source(variant: Variant) -> String {
    with_bounds(SourceKind::BranchHelperCalls.source(variant))
}

pub fn with_bounds(source: String) -> String {
    let old = "let width = extent(trips, divisor);";
    assert_eq!(source.matches(old).count(), 1);
    let source = source.replace(old, "let width = bound_extent(bound_seed(carry, trips, root + 31 - outer), divisor, after + 32 - outer);");
    let needle = "    @noinline fn work(";
    assert_eq!(source.matches(needle).count(), 1);
    source.replace(
        needle,
        &format!(
            "{}\n{needle}",
            include_str!("state_loop_return_shared_bound_helpers.ns")
        ),
    )
}

pub fn execute(
    call: Call,
    observed: bool,
    budget: Option<u64>,
) -> Result<(State, Vec<i64>), Vec<i64>> {
    let result = call.execute_with_hooks(
        observed,
        budget,
        &mut |site, index, trace| branch_calls::assignment(call, site, index, trace),
        &mut |state, outer, trace| {
            trace.push(930);
            if call.root.checked_add(31)?.checked_sub(outer)? == 0 {
                return None;
            }
            let parity = state.total % 2;
            let numerator = call.trips / (1 + parity * parity);
            trace.extend([931, 932]);
            if call.after.checked_add(32)?.checked_sub(outer)? == 0 {
                return None;
            }
            trace.push(933);
            if observed {
                trace.extend([numerator, call.divisor]);
            }
            trace.push(16);
            numerator.checked_div(call.divisor)
        },
    );
    result
        .map(|(state, trace)| (state, SourceKind::HelperCalls.trace(trace)))
        .map_err(|trace| SourceKind::HelperCalls.trace(trace))
}

// Preserve each row's intended initial selection while threading actual results,
// not the preceding fixture's differently bounded State, through later events.
pub fn sequence(mut calls: Vec<Call>) -> Vec<Call> {
    let mut previous = None;
    for call in &mut calls {
        let selected = call.first_selected();
        call.previous = previous;
        call.select_first(selected);
        previous = Some(execute(*call, false, None).unwrap().0);
    }
    calls
}

pub fn failure(variant: Variant, stage: usize) -> Call {
    let mut call = base(variant);
    let outer = call.first_outer();
    match stage {
        0 => call.root = outer - 31,
        1 => call.after = outer - 32,
        2 => call.divisor = 0,
        3 => {
            call.trips = i64::MIN;
            call.divisor = -1;
        }
        _ => unreachable!(),
    }
    call
}

pub fn state_dependent(variant: Variant) -> Call {
    let mut call = base(variant);
    call.input.total += 1;
    call.cut = 99;
    call.skip = -1;
    call.resume = -1;
    call.parent = call.first_outer();
    call.hole = 99;
    call.select_first(true);
    call
}

pub fn failure_prefix(variant: Variant, overflow: bool) -> Vec<Call> {
    let mut calls = prefix(variant);
    if overflow {
        // Keep the selected completed State even so MIN / -1 reaches the
        // extent division unchanged, rather than becoming a huge valid bound.
        calls[0].input.total += 1;
        calls[0].select_first(false);
    }
    sequence(calls)
}

pub fn after_failure_prefix(mut call: Call, prefix: &[Call]) -> Call {
    let selected = call.first_selected();
    call.previous = Some(SourceKind::BranchBoundCalls.next(*prefix.last().unwrap()));
    call.select_first(selected);
    call
}
