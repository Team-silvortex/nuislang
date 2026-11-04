use super::{bounds, Case, GuardClock, Kind};

fn guarded(mut row: Case) -> Case {
    row.clock = GuardClock::FallibleBound(5);
    row
}

pub(super) fn seed(kind: Kind) -> Case {
    guarded(bounds::seed(kind))
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = bounds::cli_sequences(kind)
        .into_iter()
        .map(|rows| rows.into_iter().map(guarded).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    // Unreached poison is inert, but an all-zero child run still enters the producer.
    rows[0][0].clock = GuardClock::FallibleBound(0);
    rows[1][0].clock = GuardClock::FallibleBound(3);
    // Negative nonzero guards remain valid; they are not a countdown budget.
    rows[3][0].clock = GuardClock::FallibleBound(-1);
    rows
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    rows.extend(
        bounds::sequences(kind)
            .into_iter()
            .skip(8)
            .map(|rows| rows.into_iter().map(guarded).collect::<Vec<_>>()),
    );
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    cli_sequences(kind).remove(4)
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = (0..4)
        .map(|parent| {
            let mut row = seed(kind);
            row.clock = GuardClock::FallibleBound(parent);
            row
        })
        .collect::<Vec<_>>();
    let mut all_zero = seed(kind);
    all_zero.parents = 3;
    all_zero.child.base.trips = 0;
    all_zero.clock = GuardClock::FallibleBound(1);
    rows.push(all_zero);
    rows.extend(bounds::failures(kind).into_iter().map(guarded));
    rows
}
