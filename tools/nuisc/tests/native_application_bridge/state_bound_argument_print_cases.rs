use super::{Case, GuardClock, Kind};

pub(super) fn negative(kind: Kind, bits: i64) -> bool {
    // Decode IEEE sign/magnitude: negative zero and signed NaNs are not less than zero.
    let sign = kind.mask();
    let magnitude = bits as u64 & (sign - 1);
    let infinity = if kind == Kind::F32 {
        0x7f800000
    } else {
        0x7ff0000000000000
    };
    bits as u64 & sign != 0 && magnitude != 0 && magnitude <= infinity
}

pub(super) fn counts(case: Case, previous: Option<i64>) -> Vec<i64> {
    let mut bits = previous.unwrap_or(case.child.base.initial);
    let mut active = false;
    (0..case.parents)
        .map(|_| {
            let count = if active {
                case.child.base.trips + i64::from(negative(case.kind(), bits))
            } else {
                0
            };
            for _ in 0..count {
                bits = (bits as u64 ^ case.kind().mask()) as i64;
            }
            active = !active;
            count
        })
        .collect()
}

pub(super) fn seed(kind: Kind) -> Case {
    let mut row = super::bounds::seed(kind);
    row.clock = GuardClock::StateBound(5);
    row
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    (0..8)
        .map(|i| {
            let mut row = seed(kind);
            match i {
                0 => {
                    row.parents = 0;
                    row.clock = GuardClock::StateBound(0);
                }
                1 => {
                    row.child.base.trips = 0;
                    row.child.base.guards = [0, 0, 0, 1];
                }
                3 => row.child.base.initial = kind.last_state(),
                4 => row.child.base.trips = 1,
                5 => {
                    row.child.base.trips = 2;
                    row.child.base.initial = kind.last_state();
                }
                6 => {
                    row.child.base.selected = false;
                    row.child.base.guards[1] = 49;
                }
                7 => row.child.base.mask = i64::from(row.child.base.right) + 4294967296,
                _ => {}
            }
            // A positive decoy must not replace the negative State accepted by earlier callbacks.
            vec![row, seed(kind), seed(kind)]
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
                    clock: GuardClock::StateBound(5),
                }
            })
            .collect()
    }));
    // Signed NaNs, negative subnormals and negative zero must not share one sign-bit rule.
    for bits in [kind.nan(), 1, 0] {
        let mut row = seed(kind);
        row.child.base.initial = (bits as u64 ^ kind.mask()) as i64;
        rows.push(vec![row, seed(kind)]);
    }
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    let mut first = seed(kind);
    first.child.base.trips = 1;
    vec![first, seed(kind)]
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = (0..4)
        .map(|parent| {
            let mut row = seed(kind);
            row.clock = GuardClock::StateBound(parent);
            row
        })
        .collect::<Vec<_>>();
    for guard in 0..3 {
        for at in [49, 50] {
            let mut row = seed(kind);
            row.child.base.guards[guard] = at;
            rows.push(row);
        }
    }
    let mut eager = seed(kind);
    eager.child.base.selected = false;
    eager.child.base.guards[0] = 51;
    rows.push(eager);
    let mut zero = seed(kind);
    zero.child.base.mask = i64::from(zero.child.base.right) + 4294967296;
    zero.child.base.guards[1] = 51;
    rows.push(zero);
    let mut tail = seed(kind);
    tail.child.base.guards[3] = 0;
    rows.push(tail);
    let mut becomes_zero = seed(kind);
    becomes_zero.child.base.trips = 0;
    becomes_zero.clock = GuardClock::StateBound(3);
    rows.push(becomes_zero);
    rows
}
