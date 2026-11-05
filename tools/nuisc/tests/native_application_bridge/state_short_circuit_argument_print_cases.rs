use super::{Case, GuardClock, Kind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exits {
    pub cut: i64,
    pub permit: i64,
    pub stop: i64,
    pub fallback: i64,
    pub continue_guard: i64,
    pub confirm_guard: i64,
    pub break_guard: i64,
    pub fallback_guard: i64,
}

impl Exits {
    pub fn continue_index(self) -> i64 {
        if self.cut == self.permit {
            self.cut
        } else {
            -1
        }
    }
    pub fn first_break(self) -> i64 {
        [self.stop, self.fallback]
            .into_iter()
            .filter(|index| *index >= 0 && *index != self.continue_index())
            .min()
            .unwrap_or(-1)
    }
    pub fn call_counts(self, completed: &[i64]) -> [usize; 4] {
        let mut counts = [0; 4];
        for count in completed {
            for index in 0..*count {
                counts[0] += 1;
                counts[1] += usize::from(index == self.cut);
                if index != self.cut || index != self.permit {
                    counts[2] += 1;
                    counts[3] += usize::from(index != self.stop);
                }
            }
        }
        counts
    }
    pub fn evaluate(
        self,
        index: i64,
        offset: i64,
        native: bool,
        trace: &mut Vec<i64>,
    ) -> Result<(bool, bool), ()> {
        let mut call = |marker, target, guard| {
            let guard = guard - offset;
            if native {
                trace.extend([marker, index, target, guard]);
            }
            if guard == 0 {
                Err(())
            } else {
                Ok(index == target)
            }
        };
        let continuing = call(603, self.cut, self.continue_guard)?
            && call(605, self.permit, self.confirm_guard)?;
        if continuing {
            return Ok((true, false));
        }
        let stopping = call(604, self.stop, self.break_guard)?
            || call(606, self.fallback, self.fallback_guard)?;
        Ok((false, stopping))
    }
}

fn checked(mut row: Case) -> Case {
    let GuardClock::StatePredicate {
        guard,
        cut,
        suffix_guard,
        stop,
        continue_guard,
        break_guard,
    } = row.clock
    else {
        panic!("checked-predicate predecessor");
    };
    row.clock = GuardClock::StateShortCircuit {
        guard,
        suffix_guard,
        exits: Exits {
            cut,
            permit: cut,
            stop,
            fallback: -1,
            continue_guard,
            break_guard,
            confirm_guard: 64,
            fallback_guard: 64,
        },
    };
    row
}

pub(super) fn seed(kind: Kind) -> Case {
    checked(super::state_predicates::seed(kind))
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = super::state_predicates::cli_sequences(kind)
        .into_iter()
        .map(|rows| rows.into_iter().map(checked).collect())
        .collect::<Vec<Vec<Case>>>();
    for i in 0..4 {
        let mut row = seed(kind);
        if let GuardClock::StateShortCircuit { exits, .. } = &mut row.clock {
            match i {
                0 => {
                    exits.cut = 99;
                    exits.permit = 0;
                    exits.stop = 0;
                    exits.fallback = 0;
                    exits.confirm_guard = 16;
                    exits.fallback_guard = 16;
                }
                1 => {
                    exits.permit = 99;
                    exits.stop = 0;
                }
                2 => {
                    exits.stop = 99;
                    exits.fallback = 1;
                }
                3 => {
                    exits.cut = 1;
                    exits.permit = 1;
                    exits.stop = 0;
                    exits.fallback = 0;
                    exits.confirm_guard = 16;
                    exits.fallback_guard = 16;
                }
                _ => unreachable!(),
            }
        }
        rows.push(vec![row, seed(kind), seed(kind)]);
    }
    rows
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    rows.extend(
        super::state_predicates::sequences(kind)
            .into_iter()
            .skip(12)
            .map(|rows| rows.into_iter().map(checked).collect()),
    );
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_predicates::prefix(kind)
        .into_iter()
        .map(checked)
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = super::state_predicates::failures(kind)
        .into_iter()
        .map(checked)
        .collect::<Vec<_>>();
    for i in 0..8 {
        let mut row = seed(kind);
        if let GuardClock::StateShortCircuit { exits, .. } = &mut row.clock {
            match i {
                0 => exits.confirm_guard = 16,
                1 => exits.fallback_guard = 17,
                2 => exits.confirm_guard = 48,
                3 => exits.fallback_guard = 49,
                4 => {
                    exits.fallback = 1;
                    exits.fallback_guard = 17;
                }
                5 => {
                    exits.permit = 99;
                    exits.confirm_guard = 16;
                }
                6 => {
                    exits.continue_guard = 16;
                    exits.confirm_guard = 16;
                }
                7 => {
                    exits.break_guard = 17;
                    exits.fallback_guard = 17;
                }
                _ => unreachable!(),
            }
        }
        rows.push(row);
    }
    rows
}
