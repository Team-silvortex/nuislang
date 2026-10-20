#[path = "state_loop_return_parent_continue_cases.rs"]
mod continuing;
#[allow(unused_imports)]
pub use continuing::{
    budget_variants, cli_variants, variants, Kind, State, Variant, BOUND_OBSERVER, CLI_WORDS,
    OBSERVER, WORDS,
};
pub const SOURCE: &str = include_str!("state_loop_return_parent_break.ns");
pub const LOOP_LIMIT: u64 = 9;
pub const FALLBACK_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n";

pub fn source(variant: Variant) -> String {
    assert_eq!(SOURCE.matches("outer = outer + 1;").count(), 1);
    continuing::source_from(variant, SOURCE)
}
pub fn source_from(variant: Variant, template: &str) -> String {
    continuing::source_from(variant, template)
}
#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub inner: continuing::Call,
    pub finish: i64,
}
impl std::ops::Deref for Call {
    type Target = continuing::Call;
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
    pub fn words(self) -> Vec<i64> {
        let mut words = self.inner.words();
        words.push(self.finish);
        words
    }
    pub fn args(self) -> String {
        format!("{},{}", self.inner.args(), self.finish)
    }
    pub fn execute(
        self,
        observed: bool,
        budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.inner.inner.execute_with_parent_exits(
            observed,
            budget,
            self.variant.parent_leading,
            None,
            Some(self.skip),
            Some(self.finish),
        )
    }
    pub fn next(self) -> State {
        self.execute(false, None).unwrap().0
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        self.execute(observed, None).unwrap().1
    }
}
pub fn base(variant: Variant) -> Call {
    Call {
        inner: continuing::base(variant),
        finish: 0,
    }
}
pub fn skipped(variant: Variant, divisor: i64, trips: i64) -> Call {
    let mut call = base(variant);
    call.parent = call.skip;
    call.root = 0;
    call.after = 0;
    call.divisor = divisor;
    call.trips = trips;
    call.finish = 1;
    call.select_first(false);
    call
}
pub fn completed(variant: Variant) -> Call {
    let mut call = base(variant);
    call.cut = 99;
    call.finish = 1;
    call.select_first(true);
    call
}
pub fn child_continued(variant: Variant) -> Call {
    let mut call = base(variant);
    call.input.total += 1;
    call.finish = 1;
    call.select_first(true);
    call
}
pub fn late(variant: Variant) -> Call {
    let mut call = base(variant);
    call.input.total += 1;
    call.skip += 1;
    call.after = if variant.parent_leading { 1 } else { 2 };
    call.select_first(false);
    call
}
pub fn cases(variant: Variant) -> Vec<Call> {
    let mut rows = vec![];
    for (gain, marker) in WORDS {
        for result in [false, true] {
            for chosen in [false, true] {
                let mut call = if chosen {
                    base(variant)
                } else {
                    skipped(variant, 0, i64::MAX)
                };
                call.input = State {
                    result,
                    gain,
                    marker,
                    ..call.input
                };
                call.select_first(chosen);
                rows.push(call);
            }
        }
    }
    rows
}
pub fn sequence(open: Call) -> Vec<Call> {
    let mut rows = vec![open];
    for profile in 0..3 {
        let mut call = match profile {
            0 => skipped(open.variant, 0, i64::MAX),
            1 => completed(open.variant),
            _ => {
                let mut c = base(open.variant);
                c.finish = 1;
                c
            }
        };
        call.previous = Some(rows.last().unwrap().next());
        call.input = State {
            result: !open.input.result,
            total: -99,
            gain: 9.25f32.to_bits(),
            marker: 123.5f64.to_bits(),
        };
        call.select_first(profile != 0);
        rows.push(call);
    }
    rows
}
pub fn inactive_sequence(variant: Variant, divisor: i64, trips: i64) -> Vec<Call> {
    let mut rows = vec![];
    for _ in 0..4 {
        let mut call = skipped(variant, divisor, trips);
        call.previous = rows.last().map(|c: &Call| c.next());
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
pub fn sequences(variant: Variant) -> Vec<Vec<Call>> {
    let mut rows = cases(variant).into_iter().map(sequence).collect::<Vec<_>>();
    rows.push(inactive_sequence(variant, -1, i64::MIN));
    rows.push(sequence(completed(variant)));
    rows.push(sequence(child_continued(variant)));
    rows.push(sequence(late(variant)));
    for trips in [0, -1] {
        let mut call = base(variant);
        call.trips = trips;
        call.finish = 2;
        rows.push(sequence(call));
    }
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
        let mut call = late(variant);
        call.divisor = divisor;
        call.trips = trips;
        call.finish = 1;
        rows.push(call);
    }
    for root in [0, 2] {
        let mut call = late(variant);
        call.root = root;
        call.finish = 0;
        call.parent = call.skip;
        rows.push(call);
    }
    let mut body = base(variant);
    body.cut = 99;
    body.hole = i64::from(variant.leading);
    rows.push(body);
    let mut fallback = skipped(variant, 0, i64::MAX);
    fallback.finish = 0;
    rows.push(fallback);
    let mut suffix = base(variant);
    suffix.skip = -1;
    suffix.trips = 0;
    suffix.after = 0;
    rows.push(suffix);
    let mut tail = skipped(variant, 0, i64::MAX);
    tail.tail = 0;
    rows.push(tail);
    rows
}
