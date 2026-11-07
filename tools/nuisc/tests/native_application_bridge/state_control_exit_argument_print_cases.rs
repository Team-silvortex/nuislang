#[allow(dead_code)]
#[path = "parent_reentry_argument_print_cases.rs"]
mod previous;
use previous::state_record_parameters as record;
pub use previous::{Kind, ENTRY_LIMIT, KINDS, LOOP_LIMIT};

pub const SOURCE: &str = include_str!("state_control_exit_argument_print.ns");
pub type State = [i64; 2];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub arguments: previous::Case,
    pub initial_control: i64,
    pub control_tail_guard: i64,
}

pub struct Plan {
    pub requested: Vec<i64>,
    pub completed: Vec<i64>,
    pub indices: Vec<i64>,
    pub calls: [usize; 6],
    pub suffixes: usize,
    pub state: State,
}

pub fn controls(row: &mut Case) -> &mut previous::GroupedExits {
    record::controls(&mut row.arguments)
}

pub fn effective(row: Case, control: i64) -> previous::Case {
    let mut args = row.arguments;
    let exits = record::controls(&mut args);
    exits.flat.permit ^= control;
    exits.seal ^= control;
    exits.flat.confirm_guard = exits.flat.confirm_guard.wrapping_add(control);
    exits.seal_guard = exits.seal_guard.wrapping_add(control);
    args
}

pub fn observers(kind: Kind) -> &'static [(&'static str, &'static str)] {
    record::matrix(kind).observers
}

impl Case {
    pub fn kind(self) -> Kind {
        self.arguments.kind()
    }
    pub fn initial(self) -> State {
        [self.arguments.child.base.initial, self.initial_control]
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.arguments.words();
        words.extend([self.initial_control, self.control_tail_guard]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{}",
            self.arguments.args(),
            self.initial_control,
            self.control_tail_guard
        )
    }
    pub fn plan(self, previous: Option<State>) -> Plan {
        let mut result = Plan {
            requested: Vec::new(),
            completed: Vec::new(),
            indices: Vec::new(),
            calls: [0; 6],
            suffixes: 0,
            state: previous.unwrap_or(self.initial()),
        };
        let exits = self.arguments.division_exits().unwrap().0;
        let divisors = self.arguments.division_exits().unwrap().1;
        for parent in 0..self.arguments.parents {
            let count = if parent % 2 == 0 {
                0
            } else {
                self.arguments.child.base.trips
                    + i64::from(record::guard(self.kind(), result.state[0], 0) == -1)
            };
            result.requested.push(count);
            let slot = parent as usize;
            result.completed.push(0);
            result.indices.push(0);
            for index in 0..count {
                result.state[0] = (result.state[0] as u64 ^ self.kind().mask()) as i64;
                result.state[1] = result.state[1].wrapping_add(1);
                result.completed[slot] += 1;
                result.indices[slot] = index;
                result.calls[0] += 1;
                let mut continuing = false;
                if index == exits.flat.cut {
                    let Some(value) = record::quotient(self.kind(), result.state[0], divisors[0])
                    else {
                        return result;
                    };
                    result.calls[1] += 1;
                    let mode = record::mode(
                        self.kind(),
                        result.state[0],
                        exits.flat.permit ^ result.state[1],
                    );
                    continuing = previous::state_field_exits::matches(self.kind(), value, mode);
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
                    let Some(value) = record::quotient(self.kind(), result.state[0], divisors[1])
                    else {
                        return result;
                    };
                    result.calls[5] += 1;
                    let mode =
                        record::mode(self.kind(), result.state[0], exits.seal ^ result.state[1]);
                    if previous::state_field_exits::matches(self.kind(), value, mode) {
                        break;
                    }
                }
                result.suffixes += 1;
                result.indices[slot] += 1;
            }
        }
        result
    }
    pub fn execute(
        self,
        previous: Option<State>,
        native: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let args = self.arguments;
        let mut state = previous.unwrap_or(self.initial());
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        let previous::GuardClock::StateRecordParameterExit {
            guard,
            suffix_guard,
            ..
        } = args.clock
        else {
            unreachable!()
        };
        for parent in 0..args.parents {
            trace.push(76);
            let negative = previous::state_field_exits::matches(self.kind(), state[0], 0);
            let remaining = guard - parent;
            if native {
                trace.extend([
                    602,
                    args.child.base.trips,
                    parent,
                    remaining,
                    i64::from(negative),
                ]);
            }
            if remaining == 0 {
                return Err(trace);
            }
            let trips = if parent % 2 == 0 {
                0
            } else {
                args.child.base.trips + i64::from(negative)
            };
            let mut code = args.child.base.left;
            let mut observed = 0;
            for index in 0..trips {
                let offset = parent * 16 + index;
                let mut child = args.child;
                child.base.left = code;
                child.base.trips = 1;
                for guard in &mut child.base.guards[..3] {
                    *guard -= offset;
                }
                child.base.guards[3] = 1;
                match child.execute(Some(state[0]), native) {
                    Ok((total, child_trace)) => {
                        trace.extend_from_slice(&child_trace[2..child_trace.len() - 1]);
                        state = [total, state[1].wrapping_add(1)];
                        code = args.child.base.right;
                    }
                    Err(child_trace) => {
                        trace.extend_from_slice(&child_trace[2..]);
                        return Err(trace);
                    }
                }
                let decisions = record::evaluate(
                    effective(self, state[1]),
                    state[0],
                    index,
                    offset,
                    native,
                    &mut trace,
                );
                let (continuing, stopping) = match decisions {
                    Ok(value) => value,
                    Err(()) => return Err(trace),
                };
                if continuing {
                    trace.push(78);
                    observed = index + 1;
                    continue;
                }
                if stopping {
                    trace.push(81);
                    observed = index;
                    break;
                }
                trace.push(79);
                let remaining = suffix_guard - offset;
                if native {
                    trace.extend([601, 0, remaining]);
                }
                if remaining == 0 {
                    return Err(trace);
                }
                trace.push(0);
                observed = index + 1;
            }
            trace.extend([80, observed, 77]);
        }
        trace.push(73);
        if args.child.base.guards[3] == 0 {
            return Err(trace);
        }
        trace.push(82);
        if self.control_tail_guard == 0 {
            return Err(trace);
        }
        Ok((state, trace))
    }
}

pub fn seed(kind: Kind) -> Case {
    Case {
        arguments: (previous::STATE_RECORD_PARAMETER_EXIT.seed)(kind),
        initial_control: -1,
        control_tail_guard: 1,
    }
}

pub fn prefix(kind: Kind) -> Vec<Case> {
    let mut first = seed(kind);
    first.initial_control = -2;
    first.arguments.parents = 2;
    first.arguments.child.base.trips = 1;
    let mut second = seed(kind);
    second.arguments.parents = 2;
    second.arguments.child.base.trips = 0;
    second.arguments.child.base.initial = kind.last_state();
    second.initial_control = 4294967296;
    vec![first, second]
}

pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    (0..10)
        .map(|i| {
            let mut row = seed(kind);
            match i {
                1 => row.initial_control = 0,
                2 => row.initial_control = 4294967296,
                3 => row.initial_control = i64::MAX - 3,
                4 => row.initial_control = -4,
                5 => {
                    row.arguments.parents = 0;
                    row.initial_control = i64::MIN;
                }
                6 => {
                    row.arguments.child.base.trips = 0;
                    row.initial_control = i64::MAX;
                }
                7 => row.arguments.child.base.initial = 0,
                8 => row.arguments.child.base.initial = kind.mask() as i64,
                9 => {
                    row.arguments.child.base.initial = kind.last_state();
                    row.arguments.child.base.selected = false;
                    row.arguments.child.base.guards[1] = 16;
                }
                _ => {}
            }
            let mut next = seed(kind);
            next.arguments.child.base.initial = kind.last_state();
            next.initial_control = i64::MIN + 7;
            let mut last = seed(kind);
            last.initial_control = 4294967296;
            vec![row, next, last]
        })
        .collect()
}

pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    let infinity = record::number(kind, f64::INFINITY);
    for magnitude in [1, infinity - 1, infinity, kind.nan()] {
        for sign in [0, kind.mask()] {
            let mut row = seed(kind);
            row.arguments.child.base.initial = (magnitude as u64 | sign) as i64;
            rows.push(vec![row, seed(kind), seed(kind)]);
        }
    }
    for control in [i64::MIN, i64::MAX, -4294967296, 4294967297] {
        let mut row = seed(kind);
        row.initial_control = control;
        rows.push(vec![row, seed(kind), seed(kind)]);
    }
    rows.push(prefix(kind));
    rows.extend(early_sequences(kind));
    rows
}

pub fn early_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut row = seed(kind);
    controls(&mut row).flat.cut = 99;
    controls(&mut row).flat.stop = 0;
    controls(&mut row).flat.fallback = 99;
    let mut next = row;
    next.arguments.child.base.initial = kind.initial();
    next.initial_control = -1;
    vec![vec![row, next, next]]
}

pub fn failures(kind: Kind) -> Vec<Case> {
    let mut bound = seed(kind);
    if let previous::GuardClock::StateRecordParameterExit { guard, .. } = &mut bound.arguments.clock
    {
        *guard = 3;
    }
    let mut update = seed(kind);
    update.arguments.child.base.guards[2] = 16;
    let mut float_tail = seed(kind);
    float_tail.arguments.child.base.guards[3] = 0;
    let mut control_tail = seed(kind);
    control_tail.control_tail_guard = 0;
    let mut rows = vec![bound, update, float_tail, control_tail];
    for slot in 0..2 {
        for late in [false, true] {
            for stage in 0..6 {
                let mut row = seed(kind);
                let index = if late { 3 } else { 0 };
                let at = if late { 51 } else { 16 };
                let completed = if late { 7 } else { 1 };
                let exits = controls(&mut row);
                exits.flat.cut = if slot == 0 { index } else { 99 };
                exits.flat.stop = if slot == 1 { index } else { 99 };
                exits.flat.fallback = 99;
                if slot == 0 {
                    exits.flat.confirm_guard = at + 1 - completed;
                } else {
                    exits.seal_guard = at + 1 - completed;
                }
                if stage <= 2 {
                    record::divisors(&mut row.arguments)[slot] = 0;
                }
                if stage == 0 {
                    record::operands(&mut row.arguments)[slot * 2] = at;
                }
                if stage <= 1 {
                    record::operands(&mut row.arguments)[slot * 2 + 1] = at;
                }
                if stage <= 3 {
                    record::faults(&mut row.arguments)[slot * 2] = at;
                }
                if stage <= 4 {
                    record::faults(&mut row.arguments)[slot * 2 + 1] = at;
                }
                rows.push(row);
            }
        }
    }
    for slot in 0..2 {
        let mut row = rows[6 + slot * 12];
        record::divisors(&mut row.arguments)[slot] = kind.mask() as i64;
        rows.push(row);
    }
    for row in &mut rows {
        row.initial_control = 4294967296;
        row.arguments.child.base.initial = kind.last_state();
    }
    rows
}
