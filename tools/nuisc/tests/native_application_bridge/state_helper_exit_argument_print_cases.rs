pub use super::state_division_exits::{plan, quotient};
use super::{Case, GroupedExits, GuardClock, Kind};

pub fn controls(row: &mut Case) -> &mut GroupedExits {
    let GuardClock::StateHelperExit { exits, .. } = &mut row.clock else {
        panic!("helper-exit profile");
    };
    exits
}

pub fn divisors(row: &mut Case) -> &mut [i64; 2] {
    let GuardClock::StateHelperExit { divisors, .. } = &mut row.clock else {
        panic!("helper-exit profile");
    };
    divisors
}

pub fn guards(row: &mut Case) -> &mut [i64; 4] {
    let GuardClock::StateHelperExit { operand_guards, .. } = &mut row.clock else {
        panic!("helper-exit profile");
    };
    operand_guards
}

pub fn division(mut row: Case) -> Case {
    let GuardClock::StateHelperExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        ..
    } = row.clock
    else {
        panic!("helper-exit profile");
    };
    row.clock = GuardClock::StateDivisionExit {
        guard,
        suffix_guard,
        exits,
        divisors,
    };
    row
}

pub fn promote(mut row: Case) -> Case {
    let GuardClock::StateDivisionExit {
        guard,
        suffix_guard,
        exits,
        divisors,
    } = row.clock
    else {
        panic!("division-exit predecessor");
    };
    row.clock = GuardClock::StateHelperExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards: [64; 4],
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
    let mut predicate = |marker, value, target, guard, slot: Option<usize>| {
        let (value, answer) = if let Some(slot) = slot {
            for (side, operand) in [bits, divisors[slot]].into_iter().enumerate() {
                let remaining = operand_guards[slot * 2 + side] - offset;
                if native {
                    trace.extend([609 + (slot * 2 + side) as i64, operand, remaining]);
                }
                if remaining == 0 {
                    return Err(());
                }
            }
            let value = quotient(case.kind(), bits, divisors[slot]).ok_or(())?;
            (
                value,
                super::state_field_exits::matches(case.kind(), value, target),
            )
        } else {
            (value, value == target)
        };
        let remaining = guard - offset;
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
    promote(super::state_division_exits::seed(kind))
}

fn promoted_sequence(index: usize, rows: Vec<Case>) -> Vec<Case> {
    let mut rows = rows.into_iter().map(promote).collect::<Vec<_>>();
    match index {
        5 => *guards(&mut rows[0]) = [0; 4],
        6 => guards(&mut rows[0])[..2].fill(16),
        7 => guards(&mut rows[0])[2..].fill(17),
        _ => {}
    }
    rows
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    super::state_division_exits::sequences(kind)
        .into_iter()
        .enumerate()
        .map(|(i, rows)| promoted_sequence(i, rows))
        .collect()
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    super::state_division_exits::cli_sequences(kind)
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !matches!(*i, 2 | 3))
        .map(|(i, rows)| promoted_sequence(i, rows))
        .collect()
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    super::state_division_exits::prefix(kind)
        .into_iter()
        .map(promote)
        .collect()
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let preceding = super::state_division_exits::failures(kind);
    let mut rows = [1, 4, 6, 7]
        .into_iter()
        .map(|i| promote(preceding[i]))
        .collect::<Vec<_>>();
    rows.extend(preceding[9..].iter().copied().map(promote));
    for slot in 0..2 {
        for side in 0..2 {
            for late in [false, true] {
                let index = 9 + slot * 4 + usize::from(late) * 2;
                let mut row = promote(preceding[index]);
                let at = if late { 51 } else { 16 };
                guards(&mut row)[slot * 2 + side] = at;
                guards(&mut row)[slot * 2 + 1] = at;
                rows.push(row);
            }
        }
    }
    for side in 0..2 {
        let mut row = promote(preceding[17]);
        guards(&mut row)[2 + side] = 17;
        guards(&mut row)[3] = 17;
        rows.push(row);
    }
    rows
}

macro_rules! producer32 {
    ($name:literal, $marker:literal) => {
        ($name, concat!("  call void @word_print(i64 ", $marker, ")\n  %producer_bits = bitcast float %arg0 to i32\n  %producer_word = zext i32 %producer_bits to i64\n  call void @word_print(i64 %producer_word)\n  call void @word_print(i64 %arg1)\n"))
    };
}

macro_rules! producer64 {
    ($name:literal, $marker:literal) => {
        ($name, concat!("  call void @word_print(i64 ", $marker, ")\n  %producer_word = bitcast double %arg0 to i64\n  call void @word_print(i64 %producer_word)\n  call void @word_print(i64 %arg1)\n"))
    };
}

pub(super) const OBSERVERS_F32: &[(&str, &str)] = &[
    super::state_division_exits::OBSERVERS_F32[0],
    super::state_division_exits::OBSERVERS_F32[1],
    super::state_division_exits::OBSERVERS_F32[2],
    super::state_division_exits::OBSERVERS_F32[3],
    super::state_division_exits::OBSERVERS_F32[4],
    super::state_division_exits::OBSERVERS_F32[5],
    super::state_division_exits::OBSERVERS_F32[6],
    super::state_division_exits::OBSERVERS_F32[7],
    producer32!("continue_lhs", 609),
    producer32!("continue_rhs", 610),
    producer32!("break_lhs", 611),
    producer32!("break_rhs", 612),
];

pub(super) const OBSERVERS_F64: &[(&str, &str)] = &[
    super::state_division_exits::OBSERVERS_F64[0],
    super::state_division_exits::OBSERVERS_F64[1],
    super::state_division_exits::OBSERVERS_F64[2],
    super::state_division_exits::OBSERVERS_F64[3],
    super::state_division_exits::OBSERVERS_F64[4],
    super::state_division_exits::OBSERVERS_F64[5],
    super::state_division_exits::OBSERVERS_F64[6],
    super::state_division_exits::OBSERVERS_F64[7],
    producer64!("continue_lhs", 609),
    producer64!("continue_rhs", 610),
    producer64!("break_lhs", 611),
    producer64!("break_rhs", 612),
];

pub fn matrix(kind: Kind) -> super::Matrix {
    super::Matrix {
        observers: if kind == Kind::F32 {
            OBSERVERS_F32
        } else {
            OBSERVERS_F64
        },
        ..super::STATE_HELPER_EXIT
    }
}
