use super::{Case, GuardClock, Kind};

pub(super) fn execute(
    case: Case,
    previous: Option<i64>,
    native: bool,
) -> Result<(i64, Vec<i64>), Vec<i64>> {
    let mut current = previous.unwrap_or(case.child.base.initial);
    let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
    let mut completed = 0;
    for _ in 0..case.parents {
        trace.push(76);
        let mut code = case.child.base.left;
        for _ in 0..case.child.base.trips {
            // Count completed siblings instead of reusing the source's index formula.
            let mut row = case.child;
            row.base.left = code;
            row.base.trips = 1;
            for guard in &mut row.base.guards[..3] {
                *guard -= completed;
            }
            row.base.guards[3] = 1;
            match row.execute(Some(current), native) {
                Ok((next, child_trace)) => {
                    assert_eq!(child_trace.last(), Some(&73));
                    trace.extend_from_slice(&child_trace[2..child_trace.len() - 1]);
                    current = next;
                    code = case.child.base.right;
                    completed += 1;
                }
                Err(child_trace) => {
                    trace.extend_from_slice(&child_trace[2..]);
                    return Err(trace);
                }
            }
        }
        trace.push(77);
    }
    trace.push(73);
    if case.child.base.guards[3] == 0 {
        return Err(trace);
    }
    Ok((current, trace))
}

pub(super) fn seed(kind: Kind) -> Case {
    let mut row = super::seed(kind);
    row.clock = GuardClock::ChildProgress;
    row.parents = 3;
    row.child.base.trips = 3;
    row.child.base.guards = [10, 10, 10, 1];
    row
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    [(0, 3), (3, 0), (2, 2), (3, 3), (3, 2), (3, 3)]
        .into_iter()
        .enumerate()
        .map(|(index, (parents, trips))| {
            let mut row = seed(kind);
            row.parents = parents;
            row.child.base.trips = trips;
            if index < 2 {
                row.child.base.guards = [0, 0, 0, 1];
            }
            if index == 4 {
                row.child.base.selected = false;
                row.child.base.guards[1] = 5;
            }
            if index == 5 {
                row.child.base.mask = i64::from(row.child.base.right) + 4294967296;
            }
            let mut next = seed(kind);
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
                child.base.guards = [10, 10, 10, 1];
                Case {
                    child,
                    parents: 3,
                    clock: GuardClock::ChildProgress,
                }
            })
            .collect()
    }));
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    cli_sequences(kind).remove(2)
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = Vec::new();
    for guard in 0..3 {
        for after in [4, 5] {
            let mut row = seed(kind);
            row.child.base.guards[guard] = after;
            rows.push(row);
        }
    }
    let mut eager = seed(kind);
    eager.child.base.selected = false;
    eager.child.base.guards[0] = 7;
    rows.push(eager);
    let mut zero = seed(kind);
    zero.child.base.mask = i64::from(zero.child.base.right) + 4294967296;
    zero.child.base.guards[1] = 7;
    rows.push(zero);
    let mut tail = seed(kind);
    tail.child.base.guards[3] = 0;
    rows.push(tail);
    rows
}
