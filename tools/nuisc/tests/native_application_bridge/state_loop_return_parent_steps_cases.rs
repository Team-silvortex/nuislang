#[path = "state_loop_return_selected_cases.rs"]
mod selected;
#[allow(unused_imports)]
pub use selected::{variants, Kind, State, Variant, BOUND_OBSERVER, CLI_WORDS, OBSERVER, WORDS};
pub const SOURCE: &str = include_str!("state_loop_return_parent_steps.ns");
pub const LOOP_LIMIT: u64 = 9;

pub fn source(variant: Variant) -> String {
    selected::source_from(variant, SOURCE)
}
pub fn cli_variants() -> Vec<Variant> {
    selected::cli_variants()
        .into_iter()
        .filter(|v| v.leading == matches!(v.kind, Kind::F32 | Kind::Record))
        .collect()
}
#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub inner: selected::Call,
}
impl std::ops::Deref for Call {
    type Target = selected::Call;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl std::ops::DerefMut for Call {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl Call {
    pub fn execute(
        self,
        observed: bool,
        budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.inner.execute_with_parent_step(observed, budget, false)
    }
    pub fn next(self) -> State {
        self.execute(false, None).unwrap().0
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        self.execute(observed, None).unwrap().1
    }
    pub fn first_selected(self) -> bool {
        let prior = self.prior();
        (self.gate ^ prior.result) ^ ((prior.total + 1) % 2 != 0)
    }
    pub fn select_first(&mut self, selected: bool) {
        let prior = self.prior();
        self.gate = selected ^ ((prior.total + 1) % 2 != 0) ^ prior.result;
    }
}
pub fn base(variant: Variant) -> Call {
    let mut call = Call {
        inner: selected::base(variant),
    };
    call.input.total = 3;
    call.select_first(true);
    call
}
pub fn cases(variant: Variant) -> Vec<Call> {
    let mut rows = vec![];
    for (gain, marker) in WORDS {
        for result in [false, true] {
            for (chosen, trips, parent) in [
                (true, 3, -1),
                (false, 3, 0),
                (true, 0, 0),
                (true, -1, 0),
                (false, i64::MAX, 0),
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
                if !chosen {
                    call.divisor = 0;
                }
                call.select_first(chosen);
                rows.push(call);
            }
        }
    }
    rows
}
pub fn sequence(open: Call) -> Vec<Call> {
    let mut rows = vec![open];
    for (chosen, trips) in [(false, i64::MAX), (true, 0), (true, 3)] {
        let mut call = base(open.variant);
        call.previous = Some(rows.last().unwrap().next());
        call.input = State {
            result: !open.input.result,
            total: -99,
            gain: 9.25f32.to_bits(),
            marker: 123.5f64.to_bits(),
        };
        call.trips = trips;
        call.parent = 0;
        call.divisor = if chosen { 1 } else { 0 };
        call.select_first(chosen);
        rows.push(call);
    }
    rows
}
pub fn inactive_sequence(variant: Variant, divisor: i64, trips: i64) -> Vec<Call> {
    let mut rows = vec![];
    for _ in 0..4 {
        let mut call = base(variant);
        call.previous = rows.last().map(|c: &Call| c.next());
        call.rounds = 2;
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
    call.input.total = 4;
    call.after = 2;
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
    let chosen = call.first_selected();
    call.previous = Some(prefix(call.variant).last().unwrap().next());
    call.select_first(chosen);
    call
}
pub fn failures(variant: Variant) -> Vec<Call> {
    let mut rows = vec![];
    for (divisor, trips) in [(0, 3), (-1, i64::MIN)] {
        let mut call = base(variant);
        call.divisor = divisor;
        call.trips = trips;
        rows.push(call);
    }
    for root in [0, 2] {
        let mut call = base(variant);
        call.root = root;
        call.parent = 0;
        rows.push(call);
    }
    let mut body = base(variant);
    body.cut = 99;
    body.hole = i64::from(variant.leading);
    rows.push(body);
    let mut suffix = base(variant);
    suffix.trips = 0;
    suffix.after = 0;
    rows.push(suffix);
    let mut tail = base(variant);
    tail.tail = 0;
    rows.push(tail);
    rows
}
