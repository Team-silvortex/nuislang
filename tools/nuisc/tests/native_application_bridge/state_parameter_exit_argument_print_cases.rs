pub use super::state_division_exits::{plan, quotient};
use super::{Case, GroupedExits, GuardClock, Kind};

pub fn controls(row: &mut Case) -> &mut GroupedExits {
    let GuardClock::StateParameterExit { exits, .. } = &mut row.clock else {
        panic!("parameter-exit profile");
    };
    exits
}

pub fn divisors(row: &mut Case) -> &mut [i64; 2] {
    let GuardClock::StateParameterExit { divisors, .. } = &mut row.clock else {
        panic!("parameter-exit profile");
    };
    divisors
}

pub fn faults(row: &mut Case) -> &mut [i64; 4] {
    let GuardClock::StateParameterExit {
        parameter_faults, ..
    } = &mut row.clock
    else {
        panic!("parameter-exit profile");
    };
    parameter_faults
}

pub fn helper(mut row: Case) -> Case {
    let GuardClock::StateParameterExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
        ..
    } = row.clock
    else {
        panic!("parameter-exit profile");
    };
    row.clock = GuardClock::StateHelperExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
    };
    row
}

pub fn promote(mut row: Case) -> Case {
    let GuardClock::StateHelperExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
    } = row.clock
    else {
        panic!("helper-exit predecessor");
    };
    row.clock = GuardClock::StateParameterExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
        parameter_faults: [64; 4],
    };
    row
}

pub fn call_counts(row: Case, previous: Option<i64>) -> [usize; 4] {
    let calls = plan(row, previous).calls;
    [calls[1], calls[1], calls[5], calls[5]]
}

pub fn evaluate(
    case: Case,
    bits: i64,
    index: i64,
    offset: i64,
    native: bool,
    trace: &mut Vec<i64>,
) -> Result<(bool, bool), ()> {
    let (exits, divisors) = case.division_exits().unwrap();
    let operand_guards = case.helper_operands().unwrap();
    let parameter_faults = case.parameter_faults().unwrap();
    let mut predicate = |marker, value, target, guard, slot: Option<usize>| {
        let remaining = guard - offset;
        let (value, answer) = if let Some(slot) = slot {
            for (side, operand) in [bits, divisors[slot]].into_iter().enumerate() {
                let fault = operand_guards[slot * 2 + side] - offset;
                if native {
                    trace.extend([609 + (slot * 2 + side) as i64, operand, fault]);
                }
                if fault == 0 {
                    return Err(());
                }
            }
            let value = quotient(case.kind(), bits, divisors[slot]).ok_or(())?;
            for (position, argument) in [target, remaining].into_iter().enumerate() {
                let fault = parameter_faults[slot * 2 + position] - offset;
                if native {
                    trace.extend([613 + (slot * 2 + position) as i64, argument, fault]);
                }
                if fault == 0 {
                    return Err(());
                }
            }
            (
                value,
                super::state_field_exits::matches(case.kind(), value, target),
            )
        } else {
            (value, value == target)
        };
        if native {
            trace.extend([marker, value, target, remaining]);
        }
        if remaining == 0 {
            Err(())
        } else {
            Ok(answer)
        }
    };
    let continuing = predicate(603, index, exits.flat.cut, exits.flat.continue_guard, None)?
        && (predicate(
            605,
            bits,
            exits.flat.permit,
            exits.flat.confirm_guard,
            Some(0),
        )? || predicate(607, index, exits.rescue, exits.rescue_guard, None)?);
    if continuing {
        return Ok((true, false));
    }
    let stopping = (predicate(604, index, exits.flat.stop, exits.flat.break_guard, None)?
        || predicate(
            606,
            index,
            exits.flat.fallback,
            exits.flat.fallback_guard,
            None,
        )?)
        && predicate(608, bits, exits.seal, exits.seal_guard, Some(1))?;
    Ok((false, stopping))
}

pub(super) fn seed(kind: Kind) -> Case {
    promote(super::state_helper_exits::seed(kind))
}

fn promoted_sequence(rows: Vec<Case>) -> Vec<Case> {
    rows.into_iter()
        .map(|row| {
            let operand_guards = row.helper_operands().unwrap();
            let mut promoted = promote(row);
            *faults(&mut promoted) = operand_guards;
            promoted
        })
        .collect()
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut sequences = super::state_helper_exits::sequences(kind)
        .into_iter()
        .map(promoted_sequence)
        .collect::<Vec<_>>();
    for mode in [4294967296, i64::MIN, i64::MAX] {
        let mut row = seed(kind);
        row.child.base.initial = kind.nan();
        controls(&mut row).flat.permit = mode;
        controls(&mut row).seal = mode;
        sequences.push(vec![row, seed(kind), seed(kind)]);
    }
    sequences
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    super::state_helper_exits::cli_sequences(kind)
        .into_iter()
        .map(promoted_sequence)
        .collect()
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_helper_exits::prefix(kind)
        .into_iter()
        .map(promote)
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = super::state_helper_exits::failures(kind)
        .into_iter()
        .map(promote)
        .collect::<Vec<_>>();
    for (i, row) in rows.iter_mut().enumerate() {
        if i < 4 {
            continue;
        }
        let (slot, at) = if i < 14 {
            (
                usize::from(i >= 8),
                if i >= 12 {
                    17
                } else if i % 4 >= 2 {
                    51
                } else {
                    16
                },
            )
        } else if i < 22 {
            ((i - 14) / 4, if i % 2 == 0 { 16 } else { 51 })
        } else {
            (1, 17)
        };
        faults(row)[slot * 2..slot * 2 + 2].fill(at);
    }
    let source_faults = rows.clone();
    for slot in 0..2 {
        for position in 0..2 {
            for late in [false, true] {
                let mut row = source_faults[4 + slot * 4 + usize::from(late) * 2];
                let at = if late { 51 } else { 16 };
                divisors(&mut row)[slot] = super::state_division_exits::number(kind, 1.0);
                *faults(&mut row) = [64; 4];
                faults(&mut row)[slot * 2 + position] = at;
                faults(&mut row)[slot * 2 + 1] = at;
                rows.push(row);
            }
        }
    }
    for position in 0..2 {
        let mut row = source_faults[12];
        divisors(&mut row)[1] = super::state_division_exits::number(kind, 1.0);
        *faults(&mut row) = [64; 4];
        faults(&mut row)[2 + position] = 17;
        faults(&mut row)[3] = 17;
        rows.push(row);
    }
    rows
}

macro_rules! parameter {
    ($name:literal, $marker:literal) => {
        (
            $name,
            concat!(
                "  call void @word_print(i64 ",
                $marker,
                ")\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n"
            ),
        )
    };
}

macro_rules! observers {
    ($old:ident) => {
        &[
            super::state_helper_exits::$old[0],
            super::state_helper_exits::$old[1],
            super::state_helper_exits::$old[2],
            super::state_helper_exits::$old[3],
            super::state_helper_exits::$old[4],
            super::state_helper_exits::$old[5],
            super::state_helper_exits::$old[6],
            super::state_helper_exits::$old[7],
            super::state_helper_exits::$old[8],
            super::state_helper_exits::$old[9],
            super::state_helper_exits::$old[10],
            super::state_helper_exits::$old[11],
            parameter!("continue_mode_value", 613),
            parameter!("continue_guard_value", 614),
            parameter!("break_mode_value", 615),
            parameter!("break_guard_value", 616),
        ]
    };
}

pub(super) const OBSERVERS_F32: &[(&str, &str)] = observers!(OBSERVERS_F32);
pub(super) const OBSERVERS_F64: &[(&str, &str)] = observers!(OBSERVERS_F64);

pub fn matrix(kind: Kind) -> super::Matrix {
    super::Matrix {
        observers: if kind == Kind::F32 {
            OBSERVERS_F32
        } else {
            OBSERVERS_F64
        },
        ..super::STATE_PARAMETER_EXIT
    }
}
