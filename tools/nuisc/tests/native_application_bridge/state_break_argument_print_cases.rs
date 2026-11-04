use super::{Case, GuardClock, Kind};

pub(super) struct ChildWork {
    pub requested: i64,
    pub completed: i64,
}

pub(super) fn schedule(case: Case, previous: Option<i64>) -> Vec<ChildWork> {
    let mut bits = previous.unwrap_or(case.child.base.initial);
    let mut active = false;
    let cut = case.break_cut().expect("break profile");
    (0..case.parents)
        .map(|_| {
            let requested = if active {
                case.child.base.trips + i64::from(super::state_bounds::negative(case.kind(), bits))
            } else {
                0
            };
            let completed = if cut >= 0 && cut < requested {
                cut + 1
            } else {
                requested
            };
            for _ in 0..completed {
                bits = (bits as u64 ^ case.kind().mask()) as i64;
            }
            active = !active;
            ChildWork {
                requested,
                completed,
            }
        })
        .collect()
}

fn with_cut(mut row: Case, cut: i64) -> Case {
    let GuardClock::StateBound(guard) = row.clock else {
        panic!("state-bound predecessor");
    };
    row.clock = GuardClock::StateBreak(guard, cut);
    row
}

pub(super) fn seed(kind: Kind) -> Case {
    with_cut(super::state_bounds::seed(kind), 0)
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    [0, 0, 0, 1, 2, 3, -1, 99]
        .into_iter()
        .enumerate()
        .map(|(i, cut)| {
            let mut row = with_cut(super::state_bounds::seed(kind), cut);
            match i {
                0 => {
                    row.parents = 0;
                    row.clock = GuardClock::StateBreak(0, cut);
                }
                1 => {
                    row.child.base.trips = 0;
                    row.child.base.guards = [0, 0, 0, 1];
                }
                2 => row.child.base.guards = [49, 49, 49, 1], // All three poisoned later-child checks are skipped.
                3 => row.child.base.initial = kind.last_state(),
                6 => {
                    row.child.base.selected = false;
                    row.child.base.guards[1] = 49;
                }
                _ => {}
            }
            vec![
                row,
                with_cut(super::state_bounds::seed(kind), -1),
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
                    .map(|row| with_cut(row, [-1, 0, 1, 2][i % 4]))
                    .collect()
            }),
    );
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_bounds::prefix(kind)
        .into_iter()
        .map(|row| with_cut(row, -1))
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = (0..4)
        .map(|parent| {
            let mut row = seed(kind);
            row.clock = GuardClock::StateBreak(parent, 2);
            row
        })
        .collect::<Vec<_>>();
    for guard in 0..3 {
        for at in [49, 50] {
            let mut row = with_cut(super::state_bounds::seed(kind), 2);
            row.child.base.guards[guard] = at;
            rows.push(row);
        }
    }
    let mut eager = with_cut(super::state_bounds::seed(kind), 2);
    eager.child.base.selected = false;
    eager.child.base.guards[0] = 50;
    rows.push(eager);
    let mut zero = with_cut(super::state_bounds::seed(kind), 2);
    zero.child.base.mask = i64::from(zero.child.base.right) + 4294967296;
    zero.child.base.guards[1] = 50;
    rows.push(zero);
    let mut tail = with_cut(super::state_bounds::seed(kind), 2);
    tail.child.base.guards[3] = 0;
    rows.push(tail);
    let mut before_break = seed(kind);
    before_break.child.base.guards[2] = 48;
    rows.push(before_break);
    let mut becomes_zero = seed(kind);
    becomes_zero.child.base.trips = 0;
    becomes_zero.clock = GuardClock::StateBreak(3, 0);
    rows.push(becomes_zero);
    rows
}
