use super::{Case, GuardClock, Kind};

fn with_cut(mut row: Case, cut: i64, suffix_guard: i64) -> Case {
    let GuardClock::StateBound(guard) = row.clock else {
        panic!("state-bound predecessor");
    };
    row.clock = GuardClock::StateContinue(guard, cut, suffix_guard);
    row
}

pub(super) fn seed(kind: Kind) -> Case {
    with_cut(super::state_bounds::seed(kind), 0, 64)
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    [0, 0, 0, 1, 2, 3, -1, 99]
        .into_iter()
        .enumerate()
        .map(|(i, cut)| {
            let mut row = with_cut(super::state_bounds::seed(kind), cut, 64);
            match i {
                0 => {
                    row.parents = 0;
                    row.clock = GuardClock::StateContinue(0, cut, 0);
                }
                1 => {
                    row.child.base.trips = 0;
                    row.child.base.guards = [0, 0, 0, 1];
                    row.clock = GuardClock::StateContinue(5, cut, 0);
                }
                2..=5 => {
                    row.clock = GuardClock::StateContinue(5, cut, 48 + cut);
                    if i == 3 || i == 5 {
                        row.child.base.initial = kind.last_state();
                    }
                }
                6 => {
                    row.child.base.selected = false;
                    row.child.base.guards[1] = 49;
                }
                7 => row.child.base.mask = i64::from(row.child.base.right) + 4294967296,
                _ => unreachable!(),
            }
            vec![
                row,
                with_cut(super::state_bounds::seed(kind), -1, 64),
                seed(kind),
            ]
        })
        .collect()
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    rows.extend(
        super::state_bounds::sequences(kind)
            .into_iter()
            .skip(rows.len())
            .enumerate()
            .map(|(i, rows)| {
                rows.into_iter()
                    .map(|row| with_cut(row, [-1, 0, 1, 2][i % 4], 64))
                    .collect()
            }),
    );
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_bounds::prefix(kind)
        .into_iter()
        .map(|row| with_cut(row, -1, 64))
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = super::state_bounds::failures(kind)
        .into_iter()
        .map(|row| with_cut(row, 0, 64))
        .collect::<Vec<_>>();
    for (cut, guard) in [(0, 49), (0, 50), (1, 16), (3, 50)] {
        let mut row = seed(kind);
        row.clock = GuardClock::StateContinue(5, cut, guard);
        rows.push(row);
    }
    let mut before_continue = seed(kind);
    before_continue.child.base.guards[2] = 48;
    rows.push(before_continue);
    // Continue skips this iteration's suffix, not the next child's eager check.
    let mut after_continue = seed(kind);
    after_continue.child.base.guards[0] = 17;
    rows.push(after_continue);
    rows
}
