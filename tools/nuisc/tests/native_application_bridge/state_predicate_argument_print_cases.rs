use super::{Case, GuardClock, Kind};

fn checked(mut row: Case) -> Case {
    let GuardClock::StateMixed {
        guard,
        cut,
        suffix_guard,
        stop,
    } = row.clock
    else {
        panic!("mixed-exit predecessor");
    };
    row.clock = GuardClock::StatePredicate {
        guard,
        cut,
        suffix_guard,
        stop,
        continue_guard: 64,
        break_guard: 64,
    };
    row
}

fn guards(mut row: Case, first: i64, second: i64) -> Case {
    let GuardClock::StatePredicate {
        continue_guard,
        break_guard,
        ..
    } = &mut row.clock
    else {
        panic!("checked-exit profile");
    };
    *continue_guard = first;
    *break_guard = second;
    row
}

pub(super) fn seed(kind: Kind) -> Case {
    checked(super::state_mixed::seed(kind))
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    super::state_mixed::cli_sequences(kind)
        .into_iter()
        .enumerate()
        .map(|(i, rows)| {
            let mut rows = rows.into_iter().map(checked).collect::<Vec<_>>();
            let (first, second) = match i {
                0 | 1 => (0, 0),
                2 => (64, 16),
                3 | 6 => (64, 48),
                4 | 7 => (64, 49),
                5 => (49, 49),
                _ => (64, 64),
            };
            rows[0] = guards(rows[0], first, second);
            rows
        })
        .collect()
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    rows.extend(
        super::state_mixed::sequences(kind)
            .into_iter()
            .skip(rows.len())
            .map(|rows| rows.into_iter().map(checked).collect()),
    );
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_mixed::prefix(kind)
        .into_iter()
        .map(checked)
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = super::state_mixed::failures(kind)
        .into_iter()
        .map(checked)
        .collect::<Vec<_>>();
    for (i, (first, second)) in [
        (16, 16),
        (64, 16),
        (17, 64),
        (64, 17),
        (48, 64),
        (64, 50),
        (16, 64),
        (17, 17),
    ]
    .into_iter()
    .enumerate()
    {
        let mut row = guards(seed(kind), first, second);
        if let GuardClock::StatePredicate { cut, stop, .. } = &mut row.clock {
            if i == 1 {
                *cut = 1;
                *stop = 0;
            }
            if i == 7 {
                *cut = 1;
                *stop = 3;
            }
        }
        // The observed State update must fail before either predicate runs.
        if i == 6 {
            row.child.base.guards[2] = 16;
        }
        rows.push(row);
    }
    rows
}
