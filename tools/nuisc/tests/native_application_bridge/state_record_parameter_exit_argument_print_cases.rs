pub use super::state_division_exits::{number, quotient};
pub use super::state_field_exits::Plan;
use super::{Case, GroupedExits, GuardClock, Kind};

pub fn mode(kind: Kind, bits: i64, base: i64) -> i64 {
    base ^ i64::from(super::state_field_exits::matches(kind, bits, 0))
}

pub fn guard(kind: Kind, bits: i64, base: i64) -> i64 {
    base.wrapping_sub(i64::from(super::state_field_exits::matches(kind, bits, 0)))
}

pub fn controls(row: &mut Case) -> &mut GroupedExits {
    let GuardClock::StateRecordParameterExit { exits, .. } = &mut row.clock else {
        panic!("record-parameter profile");
    };
    exits
}

pub fn divisors(row: &mut Case) -> &mut [i64; 2] {
    let GuardClock::StateRecordParameterExit { divisors, .. } = &mut row.clock else {
        panic!("record-parameter profile");
    };
    divisors
}

pub fn faults(row: &mut Case) -> &mut [i64; 4] {
    let GuardClock::StateRecordParameterExit {
        parameter_faults, ..
    } = &mut row.clock
    else {
        panic!("record-parameter profile");
    };
    parameter_faults
}

pub fn operands(row: &mut Case) -> &mut [i64; 4] {
    let GuardClock::StateRecordParameterExit { operand_guards, .. } = &mut row.clock else {
        panic!("record-parameter profile");
    };
    operand_guards
}

pub fn parameter(mut row: Case) -> Case {
    let GuardClock::StateRecordParameterExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
        parameter_faults,
    } = row.clock
    else {
        panic!("record-parameter profile");
    };
    row.clock = GuardClock::StateParameterExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
        parameter_faults,
    };
    row
}

pub fn promote(mut row: Case) -> Case {
    let GuardClock::StateParameterExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
        parameter_faults,
    } = row.clock
    else {
        panic!("parameter predecessor");
    };
    row.clock = GuardClock::StateRecordParameterExit {
        guard,
        suffix_guard,
        exits,
        divisors,
        operand_guards,
        parameter_faults,
    };
    row
}

pub fn plan(case: Case, previous: Option<i64>) -> Plan {
    let (exits, divisors) = case.division_exits().unwrap();
    let mut result = Plan {
        requested: Vec::new(),
        completed: Vec::new(),
        indices: Vec::new(),
        calls: [0; 6],
        suffixes: 0,
        state: previous.unwrap_or(case.child.base.initial),
    };
    for parent in 0..case.parents {
        let count = if parent % 2 == 0 {
            0
        } else {
            case.child.base.trips
                + i64::from(super::state_field_exits::matches(
                    case.kind(),
                    result.state,
                    0,
                ))
        };
        result.requested.push(count);
        result.completed.push(0);
        result.indices.push(0);
        let slot = parent as usize;
        for index in 0..count {
            result.state = (result.state as u64 ^ case.kind().mask()) as i64;
            result.completed[slot] += 1;
            result.indices[slot] = index;
            result.calls[0] += 1;
            let mut continuing = false;
            if index == exits.flat.cut {
                let Some(value) = quotient(case.kind(), result.state, divisors[0]) else {
                    return result;
                };
                result.calls[1] += 1;
                continuing = super::state_field_exits::matches(
                    case.kind(),
                    value,
                    mode(case.kind(), result.state, exits.flat.permit),
                );
                if !continuing {
                    result.calls[4] += 1;
                    continuing = index == exits.rescue;
                }
            }
            if continuing {
                result.indices[slot] += 1;
                continue;
            }
            result.calls[2] += 1;
            let mut candidate = index == exits.flat.stop;
            if !candidate {
                result.calls[3] += 1;
                candidate = index == exits.flat.fallback;
            }
            if candidate {
                let Some(value) = quotient(case.kind(), result.state, divisors[1]) else {
                    return result;
                };
                result.calls[5] += 1;
                if super::state_field_exits::matches(
                    case.kind(),
                    value,
                    mode(case.kind(), result.state, exits.seal),
                ) {
                    break;
                }
            }
            result.suffixes += 1;
            result.indices[slot] += 1;
        }
    }
    result
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
    let mut predicate = |marker, value, target, check: i64, slot: Option<usize>| {
        let mut remaining = check.wrapping_sub(offset);
        let mut target = target;
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
            let mode_fault = parameter_faults[slot * 2] - offset;
            if native {
                trace.extend([613 + slot as i64 * 2, bits, target, mode_fault]);
            }
            if mode_fault == 0 {
                return Err(());
            }
            target = mode(case.kind(), bits, target);
            let guard_fault = parameter_faults[slot * 2 + 1] - offset;
            if native {
                trace.extend([614 + slot as i64 * 2, bits, remaining, guard_fault]);
            }
            if guard_fault == 0 {
                return Err(());
            }
            remaining = guard(case.kind(), bits, remaining);
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
    let mut row = promote(super::state_parameter_exits::seed(kind));
    controls(&mut row).flat.permit = 1;
    controls(&mut row).seal = 1;
    row
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    (0..10)
        .map(|i| {
            let mut row = seed(kind);
            match i {
                1 => {
                    controls(&mut row).flat.confirm_guard = 16;
                    controls(&mut row).seal_guard = 18;
                }
                2 => controls(&mut row).flat.permit = 0,
                3 => controls(&mut row).seal = 0,
                4 | 5 => row.child.base.initial = if i == 4 { 0 } else { kind.mask() as i64 },
                6 => {
                    row.child.base.trips = 0;
                    row.child.base.guards = [0, 0, 0, 1];
                    *divisors(&mut row) = [0; 2];
                    *operands(&mut row) = [0; 4];
                    *faults(&mut row) = [0; 4];
                }
                7 => {
                    controls(&mut row).flat.cut = 99;
                    operands(&mut row)[..2].fill(16);
                    faults(&mut row)[..2].fill(16);
                }
                8 => {
                    controls(&mut row).flat.stop = 99;
                    controls(&mut row).flat.fallback = 99;
                    operands(&mut row)[2..].fill(17);
                    faults(&mut row)[2..].fill(17);
                }
                9 => *divisors(&mut row) = [number(kind, -1.0); 2],
                _ => {}
            }
            vec![row, seed(kind), seed(kind)]
        })
        .collect()
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    let infinity = number(kind, f64::INFINITY);
    let min_normal = if kind == Kind::F32 {
        0x00800000
    } else {
        0x0010000000000000
    };
    for (magnitude, divisor, base) in [
        (1, number(kind, 2.0), 0),
        (min_normal, number(kind, 2.0), 1),
        (infinity - 1, number(kind, 0.5), 1),
        (infinity, infinity, 2),
        (kind.initial(), infinity, 0),
        (kind.nan(), number(kind, 2.0), 2),
        (kind.initial(), kind.nan(), 2),
        (number(kind, 1.0), number(kind, 3.0), 1),
        (0, number(kind, -1.0), 1),
    ] {
        for sign in [0, kind.mask()] {
            let mut row = seed(kind);
            row.child.base.initial = (magnitude as u64 | sign) as i64;
            *divisors(&mut row) = [divisor; 2];
            controls(&mut row).flat.permit = base;
            controls(&mut row).seal = base;
            rows.push(vec![row, seed(kind), seed(kind)]);
        }
    }
    for base in [4294967296, i64::MIN, i64::MAX] {
        let mut row = seed(kind);
        controls(&mut row).flat.permit = base;
        controls(&mut row).seal = base;
        rows.push(vec![row, seed(kind), seed(kind)]);
    }
    for (continuing, stopping) in [
        (i64::MIN, i64::MAX),
        (i64::MIN + 16, i64::MIN + 18),
        (4294967296, -4294967296),
    ] {
        let mut row = seed(kind);
        controls(&mut row).flat.confirm_guard = continuing;
        controls(&mut row).seal_guard = stopping;
        rows.push(vec![row, seed(kind), seed(kind)]);
    }
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    let mut next = seed(kind);
    next.parents = 2;
    next.child.base.trips = 2;
    next.child.base.initial = kind.last_state();
    vec![seed(kind), next]
}

fn fault_row(kind: Kind, slot: usize, late: bool, stage: usize, fallback: bool) -> Case {
    let mut row = seed(kind);
    row.child.base.initial = kind.last_state();
    let at = if fallback {
        17
    } else if late {
        51
    } else {
        16
    };
    let target = if late { 3 } else { 0 };
    let negative = i64::from(!fallback);
    let exits = controls(&mut row);
    exits.flat.cut = if slot == 0 { target } else { 99 };
    exits.flat.stop = if slot == 1 && !fallback { target } else { 99 };
    exits.flat.fallback = if fallback { 1 } else { 99 };
    if slot == 0 {
        exits.flat.confirm_guard = at + negative;
    } else {
        exits.seal_guard = at + negative;
    }
    if stage <= 2 {
        divisors(&mut row)[slot] = 0;
    }
    if stage == 0 {
        operands(&mut row)[slot * 2] = at;
    }
    if stage <= 1 {
        operands(&mut row)[slot * 2 + 1] = at;
    }
    if stage <= 3 {
        faults(&mut row)[slot * 2] = at;
    }
    if stage <= 4 {
        faults(&mut row)[slot * 2 + 1] = at;
    }
    row
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut bound = seed(kind);
    if let GuardClock::StateRecordParameterExit { guard, .. } = &mut bound.clock {
        *guard = 3;
    }
    let mut update = seed(kind);
    update.child.base.guards[2] = 16;
    let mut tail = seed(kind);
    tail.child.base.guards[3] = 0;
    let mut rows = vec![bound, update, tail];
    for slot in 0..2 {
        for late in [false, true] {
            for stage in 0..6 {
                rows.push(fault_row(kind, slot, late, stage, false));
            }
        }
    }
    for slot in 0..2 {
        for late in [false, true] {
            let mut row = fault_row(kind, slot, late, 2, false);
            divisors(&mut row)[slot] = kind.mask() as i64;
            rows.push(row);
        }
    }
    for stage in 3..6 {
        rows.push(fault_row(kind, 1, false, stage, true));
    }
    for row in &mut rows {
        row.child.base.initial = kind.last_state();
    }
    rows
}

macro_rules! field32 {
    ($name:literal, $marker:literal) => {
        ($name, concat!("  call void @word_print(i64 ", $marker, ")\n  %record_bits = bitcast float %arg0 to i32\n  %record_word = zext i32 %record_bits to i64\n  call void @word_print(i64 %record_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"))
    };
}
macro_rules! field64 {
    ($name:literal, $marker:literal) => {
        ($name, concat!("  call void @word_print(i64 ", $marker, ")\n  %record_word = bitcast double %arg0 to i64\n  call void @word_print(i64 %record_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"))
    };
}
macro_rules! observers {
    ($old:ident, $field:ident) => {
        &[
            super::state_parameter_exits::$old[0],
            super::state_parameter_exits::$old[1],
            super::state_parameter_exits::$old[2],
            super::state_parameter_exits::$old[3],
            super::state_parameter_exits::$old[4],
            super::state_parameter_exits::$old[5],
            super::state_parameter_exits::$old[6],
            super::state_parameter_exits::$old[7],
            super::state_parameter_exits::$old[8],
            super::state_parameter_exits::$old[9],
            super::state_parameter_exits::$old[10],
            super::state_parameter_exits::$old[11],
            $field!("continue_mode_value", 613),
            $field!("continue_guard_value", 614),
            $field!("break_mode_value", 615),
            $field!("break_guard_value", 616),
        ]
    };
}
pub(super) const OBSERVERS_F32: &[(&str, &str)] = observers!(OBSERVERS_F32, field32);
pub(super) const OBSERVERS_F64: &[(&str, &str)] = observers!(OBSERVERS_F64, field64);

pub fn matrix(kind: Kind) -> super::Matrix {
    super::Matrix {
        observers: if kind == Kind::F32 {
            OBSERVERS_F32
        } else {
            OBSERVERS_F64
        },
        ..super::STATE_RECORD_PARAMETER_EXIT
    }
}
