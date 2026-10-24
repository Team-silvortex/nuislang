use super::State;

pub(super) fn typed_source(source: String) -> String {
    let old = include_str!("state_loop_return_conditional_helpers.ns");
    assert_eq!(source.matches(old).count(), 1);
    source.replace(
        old,
        include_str!("state_loop_return_scalar_join_helpers.ns"),
    )
}

pub(super) fn source(source: String) -> String {
    let old = include_str!("state_loop_return_unprojected_helpers.ns");
    assert_eq!(source.matches(old).count(), 1);
    source.replace(
        old,
        include_str!("state_loop_return_conditional_helpers.ns"),
    )
}

pub(super) fn payload(
    live: State,
    older: State,
    newer: State,
    mode: i64,
    child: usize,
    trace: &mut Vec<i64>,
) -> Option<State> {
    let base = if child == 0 { 28 } else { 30 };
    let seeds = if child == 0 {
        [older, live, newer, live]
    } else {
        [newer, live, older, live]
    };
    let mut projected = vec![];
    let result = super::projected::payload(live, older, newer, mode, child, &mut projected);
    let mut calls = 0;
    for effect in projected {
        trace.push(effect);
        if effect != 987 {
            continue;
        }
        let choice = seeds[calls].result;
        calls += 1;
        let (before, after) = if calls == 3 {
            (
                (mode - base) * (mode - base - 4),
                (mode - base - 1) * (mode - base - 5),
            )
        } else {
            (1, 1)
        };
        trace.extend([991, 997]);
        if choice {
            trace.push(998);
            if before == 0 {
                return None;
            }
            trace.push(999);
        }
        trace.push(if choice && before != 1 { 1000 } else { 1001 });
        if before == 0 {
            return None;
        }
        trace.extend([992, 993, 994, 995, 997]);
        if !choice {
            trace.push(998);
            if after == 0 {
                return None;
            }
            trace.push(999);
        }
        trace.push(if choice || after != 1 { 1002 } else { 1003 });
        if after == 0 {
            return None;
        }
        trace.push(996);
    }
    result
}
