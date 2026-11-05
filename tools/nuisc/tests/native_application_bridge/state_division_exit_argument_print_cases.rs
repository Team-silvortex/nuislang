pub use super::state_field_exits::Plan;
use super::{Case, GroupedExits, GuardClock, Kind};

pub fn number(kind: Kind, value: f64) -> i64 {
    match kind {
        Kind::F32 => (value as f32).to_bits() as i64,
        Kind::F64 => value.to_bits() as i64,
    }
}

pub fn quotient(kind: Kind, bits: i64, divisor: i64) -> Option<i64> {
    if divisor as u64 & (kind.mask() - 1) == 0 {
        return None;
    }
    let result = match kind {
        Kind::F32 => {
            (f32::from_bits(bits as u32) / f32::from_bits(divisor as u32)).to_bits() as i64
        }
        Kind::F64 => {
            (f64::from_bits(bits as u64) / f64::from_bits(divisor as u64)).to_bits() as i64
        }
    };
    Some(if super::state_field_exits::matches(kind, result, 2) {
        if kind == Kind::F32 {
            0x7fc00000
        } else {
            0x7ff8000000000000
        }
    } else {
        result
    })
}

pub fn controls(row: &mut Case) -> &mut GroupedExits {
    let GuardClock::StateDivisionExit { exits, .. } = &mut row.clock else {
        panic!("division-exit profile");
    };
    exits
}

pub fn divisors(row: &mut Case) -> &mut [i64; 2] {
    let GuardClock::StateDivisionExit { divisors, .. } = &mut row.clock else {
        panic!("division-exit profile");
    };
    divisors
}

pub fn field(mut row: Case) -> Case {
    let GuardClock::StateDivisionExit {
        guard,
        suffix_guard,
        exits,
        ..
    } = row.clock
    else {
        panic!("division-exit profile");
    };
    row.clock = GuardClock::StateFieldExit {
        guard,
        suffix_guard,
        exits,
    };
    row
}

fn promote(mut row: Case) -> Case {
    let GuardClock::StateFieldExit {
        guard,
        suffix_guard,
        exits,
    } = row.clock
    else {
        panic!("field-exit predecessor");
    };
    row.clock = GuardClock::StateDivisionExit {
        guard,
        suffix_guard,
        exits,
        divisors: [number(row.kind(), 1.0); 2],
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
                continuing =
                    super::state_field_exits::matches(case.kind(), value, exits.flat.permit);
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
                if super::state_field_exits::matches(case.kind(), value, exits.seal) {
                    break;
                }
            }
            result.suffixes += 1;
            result.indices[slot] += 1;
        }
    }
    result
}

pub fn evaluate(
    exits: GroupedExits,
    divisors: [i64; 2],
    kind: Kind,
    bits: i64,
    index: i64,
    offset: i64,
    native: bool,
    trace: &mut Vec<i64>,
) -> Result<(bool, bool), ()> {
    let mut predicate = |marker, value, target, guard, slot: Option<usize>| {
        let (value, answer) = if let Some(slot) = slot {
            let value = quotient(kind, bits, divisors[slot]).ok_or(())?;
            (
                value,
                super::state_field_exits::matches(kind, value, target),
            )
        } else {
            (value, value == target)
        };
        let guard = guard - offset;
        if native {
            trace.extend([marker, value, target, guard]);
        }
        if guard == 0 {
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
    promote(super::state_field_exits::seed(kind))
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    (0..10)
        .map(|i| {
            let mut row = seed(kind);
            match i {
                1 => row.child.base.initial = kind.last_state(),
                2 => divisors(&mut row)[0] = number(kind, -1.0),
                3 => divisors(&mut row)[1] = number(kind, -1.0),
                4 | 8 => {
                    row.child.base.initial = if i == 4 { 0 } else { kind.mask() as i64 };
                    *divisors(&mut row) = [number(kind, 2.0), number(kind, -2.0)];
                    let exits = controls(&mut row);
                    exits.flat.permit = 1;
                    exits.seal = 1;
                    exits.flat.stop = 1;
                }
                5 => {
                    row.child.base.trips = 0;
                    row.child.base.guards = [0, 0, 0, 1];
                    *divisors(&mut row) = [0, kind.mask() as i64];
                }
                6 => {
                    controls(&mut row).flat.cut = 99;
                    divisors(&mut row)[0] = 0;
                }
                7 => {
                    controls(&mut row).flat.stop = 99;
                    controls(&mut row).flat.fallback = 99;
                    divisors(&mut row)[1] = kind.mask() as i64;
                }
                9 => {
                    row.child.base.selected = false;
                    row.child.base.guards[1] = 16;
                    *divisors(&mut row) = [number(kind, 2.0), number(kind, -2.0)];
                }
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
    for (magnitude, divisor, mode) in [
        (1, number(kind, 2.0), 1),
        (min_normal, number(kind, 2.0), 0),
        (infinity - 1, number(kind, 0.5), 0),
        (infinity, infinity, 2),
        (kind.initial(), infinity, 1),
        (kind.nan(), number(kind, 2.0), 2),
        (kind.initial(), kind.nan(), 2),
        (number(kind, 1.0), number(kind, 3.0), 0),
        (0, number(kind, -1.0), 1),
    ] {
        for sign in [0, kind.mask()] {
            let mut row = seed(kind);
            row.child.base.initial = (magnitude as u64 | sign) as i64;
            *divisors(&mut row) = [divisor; 2];
            controls(&mut row).flat.permit = mode;
            controls(&mut row).seal = mode;
            rows.push(vec![row, seed(kind), seed(kind)]);
        }
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

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let preceding = super::state_field_exits::failures(kind);
    let mut rows = [0, 2, 3, 4, 6, 8, 10, 16, 18]
        .into_iter()
        .map(|i| promote(preceding[i]))
        .collect::<Vec<_>>();
    for slot in 0..2 {
        for late in [false, true] {
            for sign in [0, kind.mask()] {
                let mut row = seed(kind);
                divisors(&mut row)[slot] = sign as i64;
                let exits = controls(&mut row);
                exits.flat.cut = if slot == 0 {
                    if late {
                        3
                    } else {
                        0
                    }
                } else {
                    99
                };
                exits.flat.stop = if slot == 1 {
                    if late {
                        3
                    } else {
                        0
                    }
                } else {
                    99
                };
                exits.flat.fallback = 99;
                if slot == 0 {
                    exits.flat.confirm_guard = if late { 51 } else { 16 };
                } else {
                    exits.seal_guard = if late { 51 } else { 16 };
                }
                rows.push(row);
            }
        }
    }
    for sign in [0, kind.mask()] {
        let mut row = seed(kind);
        divisors(&mut row)[1] = sign as i64;
        let exits = controls(&mut row);
        exits.flat.cut = 99;
        exits.flat.stop = 99;
        exits.flat.fallback = 1;
        exits.seal_guard = 17;
        rows.push(row);
    }
    for row in &mut rows {
        row.child.base.initial = kind.last_state();
    }
    rows
}

// Arithmetic NaNs are normalized only in observations, never in production or State.
pub(super) const OBSERVERS_F32: &[(&str, &str)] = &[
    super::STATE_FIELD_EXIT.observers[0], super::STATE_FIELD_EXIT.observers[1],
    super::STATE_FIELD_EXIT.observers[2], super::STATE_FIELD_EXIT.observers[3],
    ("confirm_continue", "  call void @word_print(i64 605)\n  %division_nan = fcmp uno float %arg0, %arg0\n  %field_bits = bitcast float %arg0 to i32\n  %field_word = zext i32 %field_bits to i64\n  %observed_word = select i1 %division_nan, i64 2143289344, i64 %field_word\n  call void @word_print(i64 %observed_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
    super::STATE_FIELD_EXIT.observers[5], super::STATE_FIELD_EXIT.observers[6],
    ("confirm_break", "  call void @word_print(i64 608)\n  %division_nan = fcmp uno float %arg0, %arg0\n  %field_bits = bitcast float %arg0 to i32\n  %field_word = zext i32 %field_bits to i64\n  %observed_word = select i1 %division_nan, i64 2143289344, i64 %field_word\n  call void @word_print(i64 %observed_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
];

pub(super) const OBSERVERS_F64: &[(&str, &str)] = &[
    super::STATE_FIELD_EXIT.observers[0], super::STATE_FIELD_EXIT.observers[1],
    super::STATE_FIELD_EXIT.observers[2], super::STATE_FIELD_EXIT.observers[3],
    ("confirm_continue", "  call void @word_print(i64 605)\n  %division_nan = fcmp uno double %arg0, %arg0\n  %field_word = bitcast double %arg0 to i64\n  %observed_word = select i1 %division_nan, i64 9221120237041090560, i64 %field_word\n  call void @word_print(i64 %observed_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
    super::STATE_FIELD_EXIT.observers[5], super::STATE_FIELD_EXIT.observers[6],
    ("confirm_break", "  call void @word_print(i64 608)\n  %division_nan = fcmp uno double %arg0, %arg0\n  %field_word = bitcast double %arg0 to i64\n  %observed_word = select i1 %division_nan, i64 9221120237041090560, i64 %field_word\n  call void @word_print(i64 %observed_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
];

pub fn matrix(kind: Kind) -> super::Matrix {
    super::Matrix {
        observers: if kind == Kind::F32 {
            OBSERVERS_F32
        } else {
            OBSERVERS_F64
        },
        ..super::STATE_DIVISION_EXIT
    }
}
