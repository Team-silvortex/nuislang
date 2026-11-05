use super::{Case, GuardClock, Kind};

fn with_exits(mut row: Case, cut: i64, stop: i64, suffix_guard: i64) -> Case {
    let GuardClock::StateBound(guard) = row.clock else {
        panic!("state-bound predecessor");
    };
    row.clock = GuardClock::StateMixed {
        guard,
        cut,
        suffix_guard,
        stop,
    };
    row
}

pub(super) fn seed(kind: Kind) -> Case {
    with_exits(super::state_bounds::seed(kind), 0, 2, 64)
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    [
        (0, 2),
        (0, 2),
        (0, 2),
        (0, 2),
        (1, 3),
        (2, 0),
        (0, 0),
        (1, 1),
        (-1, -1),
        (99, 99),
        (0, 2),
        (0, 2),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (cut, stop))| {
        let mut row = with_exits(super::state_bounds::seed(kind), cut, stop, 64);
        match i {
            0 => {
                row.parents = 0;
                row.clock = GuardClock::StateMixed {
                    guard: 0,
                    cut,
                    stop,
                    suffix_guard: 0,
                };
            }
            1 => {
                row.child.base.trips = 0;
                row.child.base.guards = [0, 0, 0, 1];
                row.clock = GuardClock::StateMixed {
                    guard: 5,
                    cut,
                    stop,
                    suffix_guard: 0,
                };
            }
            2..=7 => {
                let suffix_guard = [16, 50, 17, 16, 48, 49][i - 2];
                row.clock = GuardClock::StateMixed {
                    guard: 5,
                    cut,
                    stop,
                    suffix_guard,
                };
                if [3, 4, 7].contains(&i) {
                    row.child.base.initial = kind.last_state();
                }
                if i == 5 {
                    row.child.base.guards = [49, 49, 49, 1];
                }
            }
            10 => {
                row.child.base.selected = false;
                row.child.base.guards[1] = 49;
            }
            11 => row.child.base.mask = i64::from(row.child.base.right) + 4294967296,
            _ => {}
        }
        vec![
            row,
            with_exits(super::state_bounds::seed(kind), -1, -1, 64),
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
            .skip(8)
            .enumerate()
            .map(|(i, rows)| {
                let (cut, stop) = [(0, 2), (1, 1), (2, 0), (-1, 99)][i % 4];
                rows.into_iter()
                    .map(|row| with_exits(row, cut, stop, 64))
                    .collect()
            }),
    );
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_bounds::prefix(kind)
        .into_iter()
        .map(|row| with_exits(row, -1, -1, 64))
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = super::state_continues::failures(kind)
        .into_iter()
        .map(|mut row| {
            let GuardClock::StateContinue(guard, cut, suffix_guard) = row.clock else {
                panic!("continue predecessor");
            };
            row.clock = GuardClock::StateMixed {
                guard,
                cut,
                suffix_guard,
                stop: 3,
            };
            row
        })
        .collect::<Vec<_>>();
    let mut before_break = seed(kind);
    before_break.child.base.guards[2] = 18;
    rows.push(before_break);
    let mut after_break = seed(kind);
    after_break.clock = GuardClock::StateMixed {
        guard: 3,
        cut: 1,
        suffix_guard: 64,
        stop: 0,
    };
    rows.push(after_break);
    rows
}
