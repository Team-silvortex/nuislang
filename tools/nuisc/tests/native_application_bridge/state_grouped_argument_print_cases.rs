use super::{Case, GuardClock, Kind, ShortCircuitExits};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exits {
    pub flat: ShortCircuitExits,
    pub rescue: i64,
    pub rescue_guard: i64,
    pub seal: i64,
    pub seal_guard: i64,
}

impl Exits {
    pub fn continue_index(self) -> i64 {
        if self.flat.cut == self.flat.permit || self.flat.cut == self.rescue {
            self.flat.cut
        } else {
            -1
        }
    }
    pub fn first_break(self) -> i64 {
        if (self.seal == self.flat.stop || self.seal == self.flat.fallback)
            && self.seal != self.continue_index()
        {
            self.seal
        } else {
            -1
        }
    }
    pub fn call_counts(self, completed: &[i64]) -> [usize; 6] {
        let mut counts = [0; 6];
        for count in completed {
            for index in 0..*count {
                let a = index == self.flat.cut;
                let b = index == self.flat.permit;
                let c = index == self.rescue;
                counts[0] += 1;
                counts[1] += usize::from(a);
                counts[4] += usize::from(a && !b);
                if !a || (!b && !c) {
                    counts[2] += 1;
                    counts[3] += usize::from(index != self.flat.stop);
                    counts[5] +=
                        usize::from(index == self.flat.stop || index == self.flat.fallback);
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
        let continuing = call(603, self.flat.cut, self.flat.continue_guard)?
            && (call(605, self.flat.permit, self.flat.confirm_guard)?
                || call(607, self.rescue, self.rescue_guard)?);
        if continuing {
            return Ok((true, false));
        }
        let stopping = (call(604, self.flat.stop, self.flat.break_guard)?
            || call(606, self.flat.fallback, self.flat.fallback_guard)?)
            && call(608, self.seal, self.seal_guard)?;
        Ok((false, stopping))
    }
}

fn grouped(mut row: Case) -> Case {
    let GuardClock::StateShortCircuit {
        guard,
        suffix_guard,
        exits,
    } = row.clock
    else {
        panic!("short-circuit predecessor");
    };
    row.clock = GuardClock::StateGrouped {
        guard,
        suffix_guard,
        exits: Exits {
            flat: exits,
            rescue: -1,
            rescue_guard: 64,
            seal: exits.first_break(),
            seal_guard: 64,
        },
    };
    row
}

pub fn controls(row: &mut Case) -> &mut Exits {
    let GuardClock::StateGrouped { exits, .. } = &mut row.clock else {
        panic!("grouped profile");
    };
    exits
}

pub(super) fn seed(kind: Kind) -> Case {
    grouped(super::state_short_circuit::seed(kind))
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    // Keep representative zero/one/many-entry and accepted-State predecessors.
    let old = super::state_short_circuit::cli_sequences(kind);
    let mut rows = [0, 3, 4, 5, 6, 12]
        .into_iter()
        .map(|i| old[i].iter().copied().map(grouped).collect())
        .collect::<Vec<Vec<Case>>>();
    for i in 0..6 {
        let mut row = seed(kind);
        let exits = controls(&mut row);
        match i {
            0 => {
                exits.flat.permit = 99;
                exits.rescue = 0;
            }
            1 => {
                exits.flat.permit = 99;
                exits.rescue = 99;
                exits.flat.stop = 0;
                exits.seal = 0;
            }
            2 => {
                exits.flat.stop = 99;
                exits.flat.fallback = 1;
                exits.seal = 1;
            }
            3 => {
                exits.flat.stop = 1;
                exits.flat.fallback = 2;
                exits.seal = 2;
            }
            4 => {
                exits.rescue_guard = 16;
                exits.seal_guard = 17;
            }
            5 => {
                exits.flat.cut = 99;
                exits.flat.stop = 99;
                exits.flat.fallback = 99;
                exits.rescue_guard = 16;
                exits.seal_guard = 16;
            }
            _ => unreachable!(),
        }
        rows.push(vec![row, seed(kind), seed(kind)]);
    }
    rows
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    rows.extend(
        super::state_short_circuit::sequences(kind)
            .into_iter()
            .skip(16)
            .map(|rows| rows.into_iter().map(grouped).collect()),
    );
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_short_circuit::prefix(kind)
        .into_iter()
        .map(grouped)
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let old = super::state_short_circuit::failures(kind);
    let mut rows = [0, 2, 12, 13, 22, 24, 28, 30, 31, 32, 33, 37]
        .into_iter()
        .map(|i| grouped(old[i]))
        .collect::<Vec<_>>();
    for i in 0..8 {
        let mut row = seed(kind);
        let exits = controls(&mut row);
        match i {
            0 | 1 => {
                exits.flat.permit = 99;
                exits.rescue = 0;
                exits.rescue_guard = if i == 0 { 16 } else { 48 };
            }
            2 => {
                exits.flat.permit = 99;
                exits.rescue_guard = 16;
            }
            3 | 4 => exits.seal_guard = if i == 3 { 18 } else { 50 },
            5 => {
                exits.flat.stop = 99;
                exits.flat.fallback = 1;
                exits.seal = 1;
                exits.seal_guard = 17;
            }
            6 => {
                exits.seal = 99;
                exits.seal_guard = 18;
            }
            7 => {
                exits.flat.stop = 99;
                exits.flat.fallback = 1;
                exits.seal = 1;
                exits.flat.fallback_guard = 17;
                exits.seal_guard = 17;
            }
            _ => unreachable!(),
        }
        rows.push(row);
    }
    rows
}
