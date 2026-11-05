use super::{Case, GroupedExits, GuardClock, Kind};

pub struct Plan {
    pub requested: Vec<i64>,
    pub completed: Vec<i64>,
    pub indices: Vec<i64>,
    pub calls: [usize; 6],
    pub suffixes: usize,
    pub state: i64,
}

pub fn matches(kind: Kind, bits: i64, mode: i64) -> bool {
    let magnitude = bits as u64 & (kind.mask() - 1);
    let infinity = if kind == Kind::F32 {
        0x7f800000
    } else {
        0x7ff0000000000000
    };
    match mode {
        0 => bits as u64 & kind.mask() != 0 && magnitude > 0 && magnitude <= infinity,
        1 => magnitude == 0,
        _ => magnitude > infinity,
    }
}

pub fn controls(row: &mut Case) -> &mut GroupedExits {
    let GuardClock::StateFieldExit { exits, .. } = &mut row.clock else {
        panic!("field-exit profile");
    };
    exits
}

pub fn plan(case: Case, previous: Option<i64>) -> Plan {
    let exits = case.field_exits().unwrap();
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
            case.child.base.trips + i64::from(matches(case.kind(), result.state, 0))
        };
        result.requested.push(count);
        let mut completed = 0;
        let mut observed = 0;
        for index in 0..count {
            result.state = (result.state as u64 ^ case.kind().mask()) as i64;
            completed += 1;
            observed = index;
            result.calls[0] += 1;
            let mut continuing = false;
            if index == exits.flat.cut {
                result.calls[1] += 1;
                continuing = matches(case.kind(), result.state, exits.flat.permit);
                if !continuing {
                    result.calls[4] += 1;
                    continuing = index == exits.rescue;
                }
            }
            if continuing {
                observed += 1;
                continue;
            }
            result.calls[2] += 1;
            let mut candidate = index == exits.flat.stop;
            if !candidate {
                result.calls[3] += 1;
                candidate = index == exits.flat.fallback;
            }
            if candidate {
                result.calls[5] += 1;
                if matches(case.kind(), result.state, exits.seal) {
                    break;
                }
            }
            result.suffixes += 1;
            observed += 1;
        }
        result.completed.push(completed);
        result.indices.push(observed);
    }
    result
}

pub fn evaluate(
    exits: GroupedExits,
    kind: Kind,
    bits: i64,
    index: i64,
    offset: i64,
    native: bool,
    trace: &mut Vec<i64>,
) -> Result<(bool, bool), ()> {
    let mut call = |marker, value, target, guard, answer| {
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
    let continuing = call(
        603,
        index,
        exits.flat.cut,
        exits.flat.continue_guard,
        index == exits.flat.cut,
    )? && (call(
        605,
        bits,
        exits.flat.permit,
        exits.flat.confirm_guard,
        matches(kind, bits, exits.flat.permit),
    )? || call(
        607,
        index,
        exits.rescue,
        exits.rescue_guard,
        index == exits.rescue,
    )?);
    if continuing {
        return Ok((true, false));
    }
    let stopping = (call(
        604,
        index,
        exits.flat.stop,
        exits.flat.break_guard,
        index == exits.flat.stop,
    )? || call(
        606,
        index,
        exits.flat.fallback,
        exits.flat.fallback_guard,
        index == exits.flat.fallback,
    )?) && call(
        608,
        bits,
        exits.seal,
        exits.seal_guard,
        matches(kind, bits, exits.seal),
    )?;
    Ok((false, stopping))
}

pub(super) fn seed(kind: Kind) -> Case {
    let mut row = super::state_grouped::seed(kind);
    let GuardClock::StateGrouped {
        guard,
        suffix_guard,
        mut exits,
    } = row.clock
    else {
        unreachable!()
    };
    exits.flat.permit = 0;
    exits.seal = 0;
    row.clock = GuardClock::StateFieldExit {
        guard,
        suffix_guard,
        exits,
    };
    row
}

pub(super) fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    (0..12)
        .map(|i| {
            let mut row = seed(kind);
            match i {
                0 => {
                    row.parents = 0;
                    row.child.base.guards = [0, 0, 0, 1];
                }
                1 => {
                    row.child.base.trips = 0;
                    row.child.base.guards = [0, 0, 0, 1];
                }
                3 => row.child.base.initial = kind.last_state(),
                4 | 5 => {
                    row.child.base.initial = if i == 4 { 0 } else { kind.mask() as i64 };
                    controls(&mut row).flat.permit = 1;
                    controls(&mut row).seal = 1;
                    controls(&mut row).flat.stop = 1;
                }
                6 => controls(&mut row).seal = 2,
                7 => {
                    let exits = controls(&mut row);
                    exits.flat.permit = 2;
                    exits.rescue = 0;
                    exits.flat.stop = 99;
                    exits.flat.fallback = 1;
                }
                8 => {
                    let exits = controls(&mut row);
                    exits.flat.cut = 99;
                    exits.flat.confirm_guard = 16;
                    exits.rescue_guard = 16;
                    exits.seal_guard = 16;
                }
                9 => {
                    row.child.base.selected = false;
                    row.child.base.guards[1] = 16;
                }
                10 => row.child.base.trips = 1,
                11 => {
                    controls(&mut row).flat.cut = 99;
                    controls(&mut row).flat.stop = 0;
                }
                _ => {}
            }
            vec![row, seed(kind), seed(kind)]
        })
        .collect()
}

pub(super) fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    let infinity = if kind == Kind::F32 {
        0x7f800000
    } else {
        0x7ff0000000000000
    };
    for magnitude in [0, 1, infinity - 1, infinity, kind.nan() as u64] {
        for sign in [0, kind.mask()] {
            for mode in 0..3 {
                let mut row = seed(kind);
                row.child.base.initial = (magnitude | sign) as i64;
                controls(&mut row).flat.permit = mode;
                controls(&mut row).seal = mode;
                rows.push(vec![row, seed(kind), seed(kind)]);
            }
        }
    }
    rows
}

pub(super) fn prefix(kind: Kind) -> Vec<Case> {
    vec![seed(kind), seed(kind)]
}

pub(super) fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = Vec::new();
    for guard in [0, 1, 3] {
        let mut row = seed(kind);
        if let GuardClock::StateFieldExit { guard: slot, .. } = &mut row.clock {
            *slot = guard;
        }
        rows.push(row);
    }
    for (slot, at) in [(3, 0), (0, 16), (1, 17), (2, 16), (2, 48)] {
        let mut row = seed(kind);
        row.child.base.guards[slot] = at;
        rows.push(row);
    }
    for (slot, at) in [
        (0, 16),
        (0, 48),
        (1, 16),
        (1, 48),
        (2, 48),
        (3, 17),
        (3, 48),
        (4, 17),
        (5, 18),
        (5, 50),
    ] {
        let mut row = seed(kind);
        let exits = controls(&mut row);
        let guard = match slot {
            0 => &mut exits.flat.continue_guard,
            1 => &mut exits.flat.confirm_guard,
            2 => &mut exits.rescue_guard,
            3 => &mut exits.flat.break_guard,
            4 => &mut exits.flat.fallback_guard,
            _ => &mut exits.seal_guard,
        };
        *guard = at;
        rows.push(row);
    }
    for at in [17, 48] {
        let mut row = seed(kind);
        if let GuardClock::StateFieldExit { suffix_guard, .. } = &mut row.clock {
            *suffix_guard = at;
        }
        rows.push(row);
    }
    let mut continuing = seed(kind);
    controls(&mut continuing).flat.cut = 1;
    controls(&mut continuing).flat.confirm_guard = 17;
    rows.push(continuing);
    let mut stopping = seed(kind);
    controls(&mut stopping).flat.cut = 99;
    controls(&mut stopping).flat.stop = 1;
    controls(&mut stopping).seal_guard = 17;
    rows.push(stopping);
    rows
}

pub(super) const OBSERVERS_F32: &[(&str, &str)] = &[
    super::STATE_GROUPED.observers[0],
    super::STATE_GROUPED.observers[1],
    super::STATE_GROUPED.observers[2],
    super::STATE_GROUPED.observers[3],
    ("confirm_continue", "  call void @word_print(i64 605)\n  %field_bits = bitcast float %arg0 to i32\n  %field_word = zext i32 %field_bits to i64\n  call void @word_print(i64 %field_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
    super::STATE_GROUPED.observers[5],
    super::STATE_GROUPED.observers[6],
    ("confirm_break", "  call void @word_print(i64 608)\n  %field_bits = bitcast float %arg0 to i32\n  %field_word = zext i32 %field_bits to i64\n  call void @word_print(i64 %field_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
];

pub(super) const OBSERVERS_F64: &[(&str, &str)] = &[
    super::STATE_GROUPED.observers[0],
    super::STATE_GROUPED.observers[1],
    super::STATE_GROUPED.observers[2],
    super::STATE_GROUPED.observers[3],
    ("confirm_continue", "  call void @word_print(i64 605)\n  %field_word = bitcast double %arg0 to i64\n  call void @word_print(i64 %field_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
    super::STATE_GROUPED.observers[5],
    super::STATE_GROUPED.observers[6],
    ("confirm_break", "  call void @word_print(i64 608)\n  %field_word = bitcast double %arg0 to i64\n  call void @word_print(i64 %field_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
];

pub fn matrix(kind: Kind) -> super::Matrix {
    super::Matrix {
        observers: if kind == Kind::F32 {
            OBSERVERS_F32
        } else {
            OBSERVERS_F64
        },
        ..super::STATE_FIELD_EXIT
    }
}
