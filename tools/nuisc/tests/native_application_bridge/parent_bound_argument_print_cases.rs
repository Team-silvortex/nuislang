use super::{Case, GuardClock, Kind};

pub(super) fn counts(case: Case) -> Vec<i64> {
    // Toggle entry and grow the active count, independently of the source formula.
    let mut active = false;
    let mut count = case.child.base.trips;
    (0..case.parents)
        .map(|_| {
            let next = if active {
                let value = count;
                count += 2;
                value
            } else {
                0
            };
            active = !active;
            next
        })
        .collect()
}

pub(super) fn execute(
    case: Case,
    previous: Option<i64>,
    native: bool,
) -> Result<(i64, Vec<i64>), Vec<i64>> {
    let mut current = previous.unwrap_or(case.child.base.initial);
    let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
    let mut parent_guard = 0;
    for (parent, trips) in case.child_counts_from(previous).into_iter().enumerate() {
        trace.push(76);
        if native {
            trace.extend([602, case.child.base.trips, parent as i64]);
        }
        if let GuardClock::FallibleBound(guard)
        | GuardClock::StateBound(guard)
        | GuardClock::StateBreak(guard, _)
        | GuardClock::StateContinue(guard, _, _)
        | GuardClock::StateMixed { guard, .. }
        | GuardClock::StatePredicate { guard, .. }
        | GuardClock::StateShortCircuit { guard, .. }
        | GuardClock::StateGrouped { guard, .. }
        | GuardClock::StateFieldExit { guard, .. }
        | GuardClock::StateDivisionExit { guard, .. }
        | GuardClock::StateHelperExit { guard, .. }
        | GuardClock::StateParameterExit { guard, .. }
        | GuardClock::StateRecordParameterExit { guard, .. } = case.clock
        {
            let remaining = guard - parent as i64;
            if native {
                trace.push(remaining);
                if matches!(
                    case.clock,
                    GuardClock::StateBound(_)
                        | GuardClock::StateBreak(_, _)
                        | GuardClock::StateContinue(_, _, _)
                        | GuardClock::StateMixed { .. }
                        | GuardClock::StatePredicate { .. }
                        | GuardClock::StateShortCircuit { .. }
                        | GuardClock::StateGrouped { .. }
                        | GuardClock::StateFieldExit { .. }
                        | GuardClock::StateDivisionExit { .. }
                        | GuardClock::StateHelperExit { .. }
                        | GuardClock::StateParameterExit { .. }
                        | GuardClock::StateRecordParameterExit { .. }
                ) {
                    trace.push(i64::from(super::state_bounds::negative(
                        case.kind(),
                        current,
                    )));
                }
            }
            // A reached producer can fail even when its child count would be zero.
            if remaining == 0 {
                return Err(trace);
            }
        }
        let mut code = case.child.base.left;
        let mut guard_offset = parent_guard;
        let mut observed_index = 0;
        for child_index in 0..trips {
            let mut row = case.child;
            row.base.left = code;
            row.base.trips = 1;
            for guard in &mut row.base.guards[..3] {
                *guard -= guard_offset;
            }
            row.base.guards[3] = 1;
            match row.execute(Some(current), native) {
                Ok((next, child_trace)) => {
                    assert_eq!(child_trace.last(), Some(&73));
                    trace.extend_from_slice(&child_trace[2..child_trace.len() - 1]);
                    current = next;
                    code = case.child.base.right;
                    guard_offset += 1;
                }
                Err(child_trace) => {
                    trace.extend_from_slice(&child_trace[2..]);
                    return Err(trace);
                }
            }
            let decisions = if case.has_record_parameters() {
                match super::state_record_parameters::evaluate(
                    case,
                    current,
                    child_index,
                    parent_guard + child_index,
                    native,
                    &mut trace,
                ) {
                    Ok(decisions) => Some(decisions),
                    Err(()) => return Err(trace),
                }
            } else if case.parameter_faults().is_some() {
                match super::state_parameter_exits::evaluate(
                    case,
                    current,
                    child_index,
                    parent_guard + child_index,
                    native,
                    &mut trace,
                ) {
                    Ok(decisions) => Some(decisions),
                    Err(()) => return Err(trace),
                }
            } else if case.helper_operands().is_some() {
                match super::state_helper_exits::evaluate(
                    case,
                    current,
                    child_index,
                    parent_guard + child_index,
                    native,
                    &mut trace,
                ) {
                    Ok(decisions) => Some(decisions),
                    Err(()) => return Err(trace),
                }
            } else if let Some((exits, divisors)) = case.division_exits() {
                match super::state_division_exits::evaluate(
                    exits,
                    divisors,
                    case.kind(),
                    current,
                    child_index,
                    parent_guard + child_index,
                    native,
                    &mut trace,
                ) {
                    Ok(decisions) => Some(decisions),
                    Err(()) => return Err(trace),
                }
            } else if let Some(exits) = case.field_exits() {
                match super::state_field_exits::evaluate(
                    exits,
                    case.kind(),
                    current,
                    child_index,
                    parent_guard + child_index,
                    native,
                    &mut trace,
                ) {
                    Ok(decisions) => Some(decisions),
                    Err(()) => return Err(trace),
                }
            } else if let Some(exits) = case.grouped() {
                match exits.evaluate(child_index, parent_guard + child_index, native, &mut trace) {
                    Ok(decisions) => Some(decisions),
                    Err(()) => return Err(trace),
                }
            } else if let Some(exits) = case.short_circuit() {
                match exits.evaluate(child_index, parent_guard + child_index, native, &mut trace) {
                    Ok(decisions) => Some(decisions),
                    Err(()) => return Err(trace),
                }
            } else {
                None
            };
            if let Some((first, _, _)) = case.predicate_guards() {
                let guard = first - (parent_guard + child_index);
                if native {
                    trace.extend([603, child_index, case.continue_control().unwrap().0, guard]);
                }
                if guard == 0 {
                    return Err(trace);
                }
            }
            if let Some((cut, _)) = case.continue_control() {
                if decisions.map_or(child_index == cut, |(continuing, _)| continuing) {
                    observed_index = child_index + 1;
                    trace.push(78);
                    continue;
                }
            }
            if let Some((_, second, stop)) = case.predicate_guards() {
                let guard = second - (parent_guard + child_index);
                if native {
                    trace.extend([604, child_index, stop, guard]);
                }
                if guard == 0 {
                    return Err(trace);
                }
            }
            if let Some(cut) = case.break_cut() {
                if decisions.map_or(child_index == cut, |(_, stopping)| stopping) {
                    observed_index = child_index;
                    trace.push(if case.continue_control().is_some() {
                        81
                    } else {
                        78
                    });
                    break;
                }
                if case.continue_control().is_none() {
                    trace.push(79);
                }
            }
            if let Some((_, suffix_guard)) = case.continue_control() {
                trace.push(79);
                let guard = suffix_guard - (parent_guard + child_index);
                if native {
                    trace.extend([601, 0, guard]);
                }
                if guard == 0 {
                    return Err(trace);
                }
                trace.push(0);
            }
            observed_index = child_index + 1;
        }
        if case.break_cut().is_some() || case.continue_control().is_some() {
            trace.extend([80, observed_index]);
        }
        trace.push(77);
        parent_guard += 16;
    }
    trace.push(73);
    if case.child.base.guards[3] == 0 {
        return Err(trace);
    }
    Ok((current, trace))
}

pub(super) fn seed(kind: Kind) -> Case {
    let mut row = super::seed(kind);
    row.clock = GuardClock::ParentBound;
    row.parents = 4;
    row.child.base.trips = 3;
    row.child.base.guards = [64, 64, 64, 1];
    row
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    [
        (0, 3),
        (3, 0),
        (4, 0),
        (4, 1),
        (4, 3),
        (4, 2),
        (4, 3),
        (2, 1),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (parents, trips))| {
        let mut row = seed(kind);
        row.parents = parents;
        row.child.base.trips = trips;
        if i < 2 {
            row.child.base.guards = [0, 0, 0, 1];
        }
        if i == 2 {
            row.child.base.guards = [0, 32, 32, 1];
        }
        if i == 3 {
            row.child.base.guards = [32, 32, 32, 1];
        }
        if i == 5 {
            row.child.base.selected = false;
            row.child.base.guards[1] = 49;
        }
        if i == 6 {
            row.child.base.mask = i64::from(row.child.base.right) + 4294967296;
        }
        let mut next = seed(kind);
        next.parents = 2;
        next.child.base.trips = 1;
        next.child.base.initial = (kind.initial() as u64 ^ kind.mask()) as i64;
        vec![row, next]
    })
    .collect()
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    rows.extend(super::child::sequences(kind).into_iter().map(|rows| {
        rows.into_iter()
            .map(|mut child| {
                child.base.guards = [64, 64, 64, 1];
                Case {
                    child,
                    parents: 4,
                    clock: GuardClock::ParentBound,
                }
            })
            .collect()
    }));
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    cli_sequences(kind).remove(4)
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = Vec::new();
    for guard in 0..3 {
        for at in [49, 50] {
            let mut row = seed(kind);
            row.child.base.guards[guard] = at;
            rows.push(row);
        }
    }
    let mut eager = seed(kind);
    eager.child.base.selected = false;
    eager.child.base.guards[0] = 52;
    rows.push(eager);
    let mut zero = seed(kind);
    zero.child.base.mask = i64::from(zero.child.base.right) + 4294967296;
    zero.child.base.guards[1] = 52;
    rows.push(zero);
    let mut tail = seed(kind);
    tail.child.base.guards[3] = 0;
    rows.push(tail);
    rows
}
