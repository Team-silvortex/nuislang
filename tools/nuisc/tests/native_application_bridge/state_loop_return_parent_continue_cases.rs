#[path = "state_loop_return_selected_cases.rs"]
mod selected;
#[allow(unused_imports)]
pub use selected::{Kind, State, BOUND_OBSERVER, CLI_WORDS, OBSERVER, WORDS};
pub const SOURCE: &str = include_str!("state_loop_return_parent_continue.ns");
pub const LOOP_LIMIT: u64 = 9;

#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub inner: selected::Variant,
    pub parent_leading: bool,
}
impl std::ops::Deref for Variant {
    type Target = selected::Variant;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
pub fn variants() -> Vec<Variant> {
    selected::variants()
        .into_iter()
        .flat_map(|inner| {
            [false, true].map(move |parent_leading| Variant {
                inner,
                parent_leading,
            })
        })
        .collect()
}
pub fn cli_variants() -> Vec<Variant> {
    variants()
        .into_iter()
        .filter(|v| {
            v.leading == v.mirrored
                && v.leading == matches!(v.kind, Kind::F32 | Kind::Record)
                && v.parent_leading == matches!(v.kind, Kind::Bool | Kind::Record)
        })
        .collect()
}
pub fn budget_variants() -> Vec<Variant> {
    cli_variants()
        .into_iter()
        .flat_map(|v| {
            [
                v,
                Variant {
                    parent_leading: !v.parent_leading,
                    ..v
                },
            ]
        })
        .collect()
}
pub fn source(variant: Variant) -> String {
    assert_eq!(SOURCE.matches("outer = outer + 1;").count(), 2);
    source_from(variant, SOURCE)
}
pub fn source_from(variant: Variant, template: &str) -> String {
    let mut source = selected::source_from(variant.inner, template);
    if variant.parent_leading {
        source = source
            .lines()
            .filter(|l| l.trim() != "outer = outer + 1;" && l.trim() != "print(26);")
            .collect::<Vec<_>>()
            .join("\n");
        source = source.replace(
            "while outer < rounds {",
            "while outer < rounds {\n            outer = outer + 1;",
        );
    }
    if variant.mirrored {
        assert_eq!(source.matches("if outer == skip {").count(), 1);
        source = source.replace("if outer == skip {", "if outer != skip {} else {");
    }
    source
}

#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub inner: selected::Call,
    pub variant: Variant,
    pub skip: i64,
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
    pub fn words(self) -> Vec<i64> {
        let mut words = self.inner.words();
        words.push(self.skip);
        words
    }
    pub fn args(self) -> String {
        format!("{},{}", self.inner.args(), self.skip)
    }
    pub fn execute(
        self,
        observed: bool,
        budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.inner.execute_with_parent_flow(
            observed,
            budget,
            self.variant.parent_leading,
            Some(self.skip),
        )
    }
    pub fn next(self) -> State {
        self.execute(false, None).unwrap().0
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        self.execute(observed, None).unwrap().1
    }
    pub fn first_outer(self) -> i64 {
        i64::from(self.variant.parent_leading)
    }
    pub fn first_selected(self) -> bool {
        let prior = self.prior();
        (self.gate ^ prior.result) ^ ((prior.total + 1 + self.first_outer()) % 2 != 0)
    }
    pub fn select_first(&mut self, selected: bool) {
        let prior = self.prior();
        self.gate = selected ^ ((prior.total + 1 + self.first_outer()) % 2 != 0) ^ prior.result;
    }
}
pub fn base(variant: Variant) -> Call {
    let mut call = Call {
        inner: selected::base(variant.inner),
        variant,
        skip: i64::from(variant.parent_leading),
    };
    call.input.total = 3 - call.first_outer();
    call.parent = call.first_outer() + 1;
    call.select_first(true);
    call
}
pub fn late(variant: Variant) -> Call {
    let mut call = base(variant);
    call.input.total += 1;
    call.after = 0;
    call.select_first(false);
    call
}
pub fn skipped(variant: Variant, divisor: i64, trips: i64) -> Call {
    let mut call = base(variant);
    call.rounds = 1;
    call.parent = call.skip;
    call.root = 0;
    call.after = 0;
    call.divisor = divisor;
    call.trips = trips;
    call.select_first(false);
    call
}
pub fn completed(variant: Variant) -> Call {
    let mut call = base(variant);
    call.cut = 99;
    call.select_first(true);
    call
}
pub fn child_continued(variant: Variant) -> Call {
    let mut call = base(variant);
    call.input.total += 1;
    call.select_first(true);
    call
}
pub fn cases(variant: Variant) -> Vec<Call> {
    let mut rows = vec![];
    for (gain, marker) in WORDS {
        for result in [false, true] {
            for profile in 0..3 {
                let mut call = if profile == 2 {
                    late(variant)
                } else {
                    base(variant)
                };
                call.input = State {
                    result,
                    gain,
                    marker,
                    ..call.input
                };
                if profile == 1 {
                    call.trips = 0;
                }
                call.select_first(profile != 2);
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
            _ => base(open.variant),
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
    rows.push(inactive_sequence(variant, 0, i64::MAX));
    rows.push(inactive_sequence(variant, -1, i64::MIN));
    rows.push(sequence(completed(variant)));
    rows.push(sequence(child_continued(variant)));
    let mut empty = base(variant);
    empty.trips = -1;
    rows.push(sequence(empty));
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
    // Reach failures only after a completed parent continue; an inactive first
    // child must not evaluate or reserve its poisoned bound.
    for (divisor, trips) in [(0, 3), (-1, i64::MIN)] {
        let mut call = late(variant);
        call.divisor = divisor;
        call.trips = trips;
        rows.push(call);
    }
    for root in [0, 2] {
        let mut call = late(variant);
        call.root = root;
        call.after = 1;
        rows.push(call);
    }
    let mut body = late(variant);
    body.cut = 99;
    body.hole = i64::from(variant.leading);
    rows.push(body);
    let mut suffix = base(variant);
    suffix.skip = -1;
    suffix.trips = 0;
    suffix.after = 0;
    rows.push(suffix);
    let mut tail = base(variant);
    tail.tail = 0;
    rows.push(tail);
    rows
}
