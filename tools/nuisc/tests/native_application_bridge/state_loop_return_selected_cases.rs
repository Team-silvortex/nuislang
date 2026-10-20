#[path = "state_loop_return_values_cases.rs"]
mod base_values;
#[allow(unused_imports)]
pub use base_values::{cli_variants, variants, Kind, State, Variant, CLI_WORDS, OBSERVER, WORDS};
pub const SOURCE: &str = include_str!("state_loop_return_selected.ns");
pub const LOOP_LIMIT: u64 = 9;
pub const BOUND_OBSERVER: &str =
    "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n";
const INACTIVE: &str =
    "print(9); current = current + 2; choice = choice != true; narrow = -narrow; wide = -wide;";

pub fn source(variant: Variant) -> String {
    source_from(variant, SOURCE)
}
pub fn source_from(variant: Variant, template: &str) -> String {
    let source = variant.source_from(template, "width");
    if !variant.mirrored {
        return source;
    }
    let inactive = format!("}} else {{ {INACTIVE} }}");
    assert_eq!(source.matches(&inactive).count(), 1);
    let source = source.replace(&inactive, "}");
    assert_eq!(source.matches("if selected {").count(), 1);
    source.replace(
        "if selected {",
        &format!("if selected != true {{ {INACTIVE} }} else {{"),
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub inner: base_values::Call,
    pub divisor: i64,
}
impl std::ops::Deref for Call {
    type Target = base_values::Call;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl std::ops::DerefMut for Call {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
fn flip(state: State) -> State {
    State {
        result: !state.result,
        gain: state.gain ^ (1 << 31),
        marker: state.marker ^ (1 << 63),
        ..state
    }
}
fn reserve(bound: i64, budget: &mut Option<u64>) -> bool {
    let cost = bound.max(0) as u64;
    if let Some(left) = budget {
        if cost > *left {
            return false;
        }
        *left -= cost;
    }
    true
}
fn returned(
    mut state: State,
    index: i64,
    outer: i64,
    root: i64,
    observed: bool,
    trace: &mut Vec<i64>,
) -> Option<State> {
    if root == 0 {
        return None;
    }
    let proof = 1 / root;
    if observed {
        trace.extend([
            state.result.into(),
            state.total,
            state.gain.into(),
            state.marker as i64,
            index,
            outer,
            proof,
        ]);
    }
    trace.push(30);
    if proof == 0 {
        return None;
    }
    state.total += 10 * index + 100 * outer + 1 / proof;
    Some(state)
}
impl Call {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.inner.words();
        words.push(self.divisor);
        words
    }
    pub fn args(self) -> String {
        format!("{},{}", self.inner.args(), self.divisor)
    }
    // Source-level selection and immediate returns, independent of generated
    // guards/signals. A skipped invocation does not even divide its bound.
    pub fn execute(
        self,
        observed: bool,
        budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_with_parent_step(observed, budget, true)
    }
    pub fn execute_with_parent_step(
        self,
        observed: bool,
        budget: Option<u64>,
        leading_parent: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_with_parent_flow(observed, budget, leading_parent, None)
    }
    pub fn execute_with_parent_flow(
        self,
        observed: bool,
        budget: Option<u64>,
        leading_parent: bool,
        parent_continue: Option<i64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_with_parent_exits(
            observed,
            budget,
            leading_parent,
            parent_continue,
            None,
            None,
        )
    }
    pub fn execute_with_parent_exits(
        self,
        observed: bool,
        mut budget: Option<u64>,
        leading_parent: bool,
        parent_continue: Option<i64>,
        parent_break: Option<i64>,
        fallback_divisor: Option<i64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let prior = self.prior();
        let mut current = State {
            total: prior.total + 1,
            ..prior
        };
        let gate = self.gate ^ prior.result;
        let mut trace = vec![5];
        let result = (|| -> Option<State> {
            let mut outer = 0;
            if !reserve(self.rounds, &mut budget) {
                return None;
            }
            while outer < self.rounds {
                if leading_parent {
                    outer += 1;
                }
                trace.push(8);
                current = flip(current);
                current.total += outer;
                let selected = gate ^ (current.total % 2 != 0);
                let mut index = 0;
                if selected {
                    trace.push(6);
                    if observed {
                        trace.extend([self.trips, self.divisor]);
                    }
                    trace.push(16);
                    let width = self.trips.checked_div(self.divisor)?;
                    trace.push(7);
                    if !reserve(width, &mut budget) {
                        return None;
                    }
                    while index < width {
                        if self.variant.leading {
                            index += 1;
                        }
                        trace.push(10);
                        current = flip(current);
                        current.total += index + 1;
                        if index == self.cut {
                            trace.push(11);
                            if gate {
                                trace.push(12);
                                return returned(
                                    current, index, outer, self.root, observed, &mut trace,
                                );
                            }
                            trace.push(13);
                            if !self.variant.leading {
                                index += 1;
                            }
                            continue;
                        }
                        trace.push(19);
                        if index == self.hole {
                            return None;
                        }
                        current.total += 1 / (index - self.hole);
                        trace.push(20);
                        if !self.variant.leading {
                            index += 1;
                        }
                    }
                } else {
                    trace.push(9);
                    current = flip(current);
                    current.total += 2;
                }
                if parent_break == Some(outer) {
                    trace.push(25);
                    current.total += outer;
                    break;
                }
                if parent_continue == Some(outer) {
                    trace.push(25);
                    current.total += outer;
                    if !leading_parent {
                        outer += 1;
                    }
                    continue;
                }
                trace.push(23);
                if outer == self.parent {
                    trace.push(24);
                    return returned(current, index, outer, self.root, observed, &mut trace);
                }
                if self.after == 0 {
                    return None;
                }
                trace.push(21);
                current.total += 1 / self.after;
                trace.push(22);
                if !leading_parent {
                    trace.push(26);
                    outer += 1;
                }
            }
            trace.push(31);
            if let Some(divisor) = fallback_divisor {
                if observed {
                    trace.push(outer);
                }
                trace.push(33);
                let _checked = 1i64.checked_div(divisor)?;
                trace.push(32);
            }
            Some(State {
                total: prior.total + 1 - 7,
                ..flip(prior)
            })
        })();
        let Some(result) = result else {
            return Err(trace);
        };
        trace.push(35);
        if self.tail == 0 {
            return Err(trace);
        }
        let check = 1 / self.tail;
        let state = match self.variant.kind {
            Kind::Record => State {
                total: result.total + check,
                ..result
            },
            Kind::Bool => State {
                result: result.result,
                total: prior.total + check,
                ..prior
            },
            Kind::F32 => State {
                gain: result.gain,
                total: prior.total + check,
                ..prior
            },
            Kind::F64 => State {
                marker: result.marker,
                total: prior.total + check,
                ..prior
            },
        };
        Ok((state, trace))
    }
    pub fn next(self) -> State {
        self.execute(false, None).unwrap().0
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        self.execute(observed, None).unwrap().1
    }
    pub fn first_selected(self) -> bool {
        let prior = self.prior();
        (self.gate ^ prior.result) ^ ((prior.total + 2) % 2 != 0)
    }
    pub fn select_first(&mut self, selected: bool) {
        let prior = self.prior();
        self.gate = selected ^ ((prior.total + 2) % 2 != 0) ^ prior.result;
    }
}
pub fn base(variant: Variant) -> Call {
    Call {
        inner: base_values::base(variant),
        divisor: 1,
    }
}
pub fn cases(variant: Variant) -> Vec<Call> {
    let mut rows = vec![];
    for (gain, marker) in WORDS {
        for result in [false, true] {
            for (selected, trips, parent) in [
                (true, 3, -1),
                (false, 3, 1),
                (true, 0, 1),
                (true, -1, 1),
                (false, i64::MAX, 1),
            ] {
                let mut call = base(variant);
                call.input = State {
                    result,
                    gain,
                    marker,
                    ..call.input
                };
                call.trips = trips;
                call.parent = parent;
                call.select_first(selected);
                if !selected {
                    call.divisor = 0;
                }
                rows.push(call);
            }
        }
    }
    rows
}
pub fn sequence(open: Call) -> Vec<Call> {
    let mut rows = vec![open];
    for (selected, trips, parent) in [(false, i64::MAX, 1), (true, 0, 1), (true, 3, 1)] {
        let previous = Some(rows.last().unwrap().next());
        let mut call = base(open.variant);
        call.previous = previous;
        call.input = State {
            result: !open.input.result,
            total: -99,
            gain: 9.25f32.to_bits(),
            marker: 123.5f64.to_bits(),
        };
        call.trips = trips;
        call.parent = parent;
        call.divisor = if selected { 1 } else { 0 };
        call.select_first(selected);
        rows.push(call);
    }
    rows
}
pub fn inactive_sequence(variant: Variant, divisor: i64, trips: i64) -> Vec<Call> {
    let mut rows = vec![];
    for _ in 0..4 {
        let mut call = base(variant);
        call.previous = rows.last().map(|c: &Call| c.next());
        call.rounds = 1;
        call.parent = -1;
        call.root = 0;
        call.divisor = divisor;
        call.trips = trips;
        call.select_first(false);
        rows.push(call);
    }
    rows
}
pub fn prefix(variant: Variant) -> Vec<Call> {
    inactive_sequence(variant, 0, i64::MAX)
        .into_iter()
        .take(3)
        .collect()
}
pub fn late(variant: Variant) -> Call {
    let mut call = base(variant);
    call.input.total = 3;
    call.select_first(false);
    call
}
pub fn sequences(variant: Variant) -> Vec<Vec<Call>> {
    let mut rows = cases(variant).into_iter().map(sequence).collect::<Vec<_>>();
    rows.push(inactive_sequence(variant, 0, i64::MAX));
    rows.push(inactive_sequence(variant, -1, i64::MIN));
    rows.push(sequence(late(variant)));
    rows
}
pub fn after_prefix(mut call: Call) -> Call {
    let selected = call.first_selected();
    call.previous = Some(prefix(call.variant).last().unwrap().next());
    call.select_first(selected);
    call
}
pub fn failures(variant: Variant) -> Vec<Call> {
    let mut rows = vec![];
    for (divisor, trips) in [(0, 3), (-1, i64::MIN)] {
        let mut c = base(variant);
        c.divisor = divisor;
        c.trips = trips;
        rows.push(c);
    }
    for root in [0, 2] {
        let mut c = base(variant);
        c.root = root;
        c.parent = 1;
        rows.push(c);
    }
    let mut work = base(variant);
    work.cut = 99;
    work.hole = i64::from(variant.leading);
    rows.push(work);
    let mut suffix = base(variant);
    suffix.trips = 0;
    suffix.parent = -1;
    suffix.after = 0;
    rows.push(suffix);
    let mut tail = base(variant);
    tail.tail = 0;
    rows.push(tail);
    rows
}
