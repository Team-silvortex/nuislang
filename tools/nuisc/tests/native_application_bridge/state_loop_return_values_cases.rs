pub const SOURCE: &str = include_str!("state_loop_return_values.ns");
pub const LOOP_LIMIT: u64 = 9;
pub const OBSERVER: &str = "  %seen_flag = zext i1 %arg0 to i64\n  call void @nuis_debug_print_i64(i64 %seen_flag)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  %gain_bits = bitcast float %arg2 to i32\n  %gain_word = zext i32 %gain_bits to i64\n  call void @nuis_debug_print_i64(i64 %gain_word)\n  %marker_word = bitcast double %arg3 to i64\n  call void @nuis_debug_print_i64(i64 %marker_word)\n  call void @nuis_debug_print_i64(i64 %arg4)\n  call void @nuis_debug_print_i64(i64 %arg5)\n  call void @nuis_debug_print_i64(i64 %arg6)\n";
pub const WORDS: [(u32, u64); 6] = [
    (0, 0),
    (0x80000000, 0x8000000000000000),
    (1, 1),
    (0x80000001, 0x8000000000000001),
    (0x7fc01234, 0x7ff8000012345678),
    (0xff800123, 0xfff0000012345678),
];
pub const CLI_WORDS: [(u32, u64); 2] = [(0x80000000, 0x8000000000000000), (1, 1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bool,
    F32,
    F64,
    Record,
}
pub const KINDS: [Kind; 4] = [Kind::Bool, Kind::F32, Kind::F64, Kind::Record];
impl Kind {
    pub fn ty(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Record => "State",
        }
    }
    fn selected(self) -> &'static str {
        match self {
            Self::Bool => "flag",
            Self::F32 => "gain",
            Self::F64 => "marker",
            Self::Record => unreachable!(),
        }
    }
    fn fallback(self) -> &'static str {
        match self {
            Self::Bool => "flag != true",
            Self::F32 => "-gain",
            Self::F64 => "-marker",
            Self::Record => unreachable!(),
        }
    }
    fn publish(self, event: bool) -> String {
        let prefix = if event { "state." } else { "" };
        let result = if self == Self::Bool {
            "value".into()
        } else {
            format!("{prefix}{}", if event { "result" } else { "flag" })
        };
        let gain = if self == Self::F32 {
            "value".into()
        } else {
            format!("{prefix}gain")
        };
        let marker = if self == Self::F64 {
            "value".into()
        } else {
            format!("{prefix}marker")
        };
        format!("return State {{ result: {result}, total: {prefix}total + check, gain: {gain}, marker: {marker} }};")
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub kind: Kind,
    pub leading: bool,
    pub mirrored: bool,
}
fn replace(source: String, old: &str, new: &str, count: usize) -> String {
    assert_eq!(source.matches(old).count(), count, "{old}");
    source.replace(old, new)
}
impl Variant {
    pub fn source(self) -> String {
        self.source_from(SOURCE, "trips")
    }
    pub fn source_from(self, template: &str, child_bound: &str) -> String {
        let mut source = template.to_owned();
        if self.kind != Kind::Record {
            source = replace(
                source,
                "proof: i64) -> State",
                &format!("proof: i64) -> {}", self.kind.ty()),
                1,
            );
            source = replace(
                source,
                "after: i64) -> State",
                &format!("after: i64) -> {}", self.kind.ty()),
                1,
            );
            source = replace(source, "return State { result: flag, total: total + 10 * index + 100 * outer + checked, gain: gain, marker: marker };", &format!("return {};", self.kind.selected()), 1);
            source = replace(source, "return State { result: flag != true, total: total - 7, gain: -gain, marker: -marker };", &format!("return {};", self.kind.fallback()), 1);
            let original = "return State { result: value.result, total: value.total + check, gain: value.gain, marker: value.marker };";
            assert_eq!(source.matches(original).count(), 2);
            source = source.replacen(original, &self.kind.publish(false), 1);
            source = source.replacen(original, &self.kind.publish(true), 1);
        }
        if self.leading {
            assert_eq!(source.matches("index = index + 1;").count(), 2);
            source = source
                .lines()
                .filter(|l| l.trim() != "index = index + 1;")
                .collect::<Vec<_>>()
                .join("\n");
            source = replace(
                source,
                &format!("while index < {child_bound} {{"),
                &format!("while index < {child_bound} {{\n                index = index + 1;"),
                1,
            );
        }
        if self.mirrored {
            source = replace(source, "if index == cut {", "if index != cut {} else {", 1);
            source = replace(
                source,
                "if gate { print(12);",
                "if gate != true {} else { print(12);",
                1,
            );
            source = replace(
                source,
                "if outer == parent {",
                "if outer != parent {} else {",
                1,
            );
        }
        source
    }
}
pub fn variants() -> Vec<Variant> {
    KINDS
        .into_iter()
        .flat_map(|kind| {
            [false, true].into_iter().flat_map(move |leading| {
                [false, true].map(move |mirrored| Variant {
                    kind,
                    leading,
                    mirrored,
                })
            })
        })
        .collect()
}
pub fn cli_variants() -> Vec<Variant> {
    variants()
        .into_iter()
        .filter(|v| v.leading == v.mirrored)
        .collect()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub result: bool,
    pub total: i64,
    pub gain: u32,
    pub marker: u64,
}
impl State {
    pub fn words(self) -> Vec<i64> {
        vec![
            self.result.into(),
            self.total,
            self.gain.into(),
            self.marker as i64,
        ]
    }
    pub fn text(self, phase: &str) -> String {
        format!(
            "{phase}:State{{result: {}, total: {}, gain: {}f32, marker: {}f64}}",
            self.result,
            self.total,
            // Match Value::F32's established widened CLI display; word checks
            // independently verify the original f32 precision and sign bits.
            f64::from(f32::from_bits(self.gain)),
            f64::from_bits(self.marker)
        )
    }
    fn flipped(self) -> Self {
        Self {
            result: !self.result,
            gain: self.gain ^ (1 << 31),
            marker: self.marker ^ (1 << 63),
            ..self
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub input: State,
    pub previous: Option<State>,
    pub gate: bool,
    pub trips: i64,
    pub cut: i64,
    pub rounds: i64,
    pub parent: i64,
    pub root: i64,
    pub hole: i64,
    pub after: i64,
    pub tail: i64,
    pub variant: Variant,
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
            i64::from(state.result),
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
    pub fn prior(self) -> State {
        self.previous.unwrap_or(self.input)
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.extend([
            i64::from(self.gate),
            self.trips,
            self.cut,
            self.rounds,
            self.parent,
            self.root,
            self.hole,
            self.after,
            self.tail,
        ]);
        words
    }
    pub fn args(self) -> String {
        self.words()
            .into_iter()
            .enumerate()
            .map(|(slot, word)| match slot {
                0 | 4 => (word != 0).to_string(),
                2 => f32::from_bits(word as u32).to_string(),
                3 => f64::from_bits(word as u64).to_string(),
                _ => word.to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    // Model actual source exits, never the generated return signal or its seed.
    pub fn execute(
        self,
        observed: bool,
        mut budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let prior = self.prior();
        let mut trace = vec![5];
        let mut current = State {
            total: prior.total + 1,
            ..prior
        };
        let gate = self.gate ^ prior.result;
        let result = (|| -> Option<State> {
            let mut outer = 0;
            if !reserve(self.rounds, &mut budget) {
                return None;
            }
            while outer < self.rounds {
                outer += 1;
                trace.push(8);
                current = current.flipped();
                current.total += outer;
                let mut index = 0;
                if !reserve(self.trips, &mut budget) {
                    return None;
                }
                while index < self.trips {
                    if self.variant.leading {
                        index += 1;
                    }
                    trace.push(10);
                    current = current.flipped();
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
            }
            trace.push(31);
            Some(State {
                total: prior.total + 1 - 7,
                ..prior.flipped()
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
}
pub fn base(variant: Variant) -> Call {
    Call {
        input: State {
            result: false,
            total: 4,
            gain: 0x80000000,
            marker: 0x8000000000000000,
        },
        previous: None,
        gate: true,
        trips: 3,
        cut: i64::from(variant.leading),
        rounds: 2,
        parent: -1,
        root: 1,
        hole: -1,
        after: 1,
        tail: 1,
        variant,
    }
}
pub fn cases(variant: Variant) -> Vec<Call> {
    let mut calls = vec![];
    for (gain, marker) in WORDS {
        for result in [false, true] {
            for (gate, trips, rounds, parent) in [
                (true, 3, 2, -1),
                (false, 2, 2, 1),
                (true, 0, 2, 1),
                (false, 2, 2, -1),
                (true, i64::MAX, 0, -1),
            ] {
                let mut call = base(variant);
                call.input = State {
                    result,
                    gain,
                    marker,
                    ..call.input
                };
                call.gate = gate ^ result;
                call.trips = trips;
                call.rounds = rounds;
                call.parent = parent;
                if rounds == 0 {
                    call.root = 0;
                }
                calls.push(call);
            }
        }
    }
    calls
}
pub fn sequence(open: Call) -> Vec<Call> {
    let mut rows = vec![open];
    for (gate, trips, parent) in [(true, 0, 1), (false, 2, -1), (true, 3, -1)] {
        let previous = Some(rows.last().unwrap().next());
        rows.push(Call {
            previous,
            input: State {
                result: !open.input.result,
                total: -99,
                gain: 9.25f32.to_bits(),
                marker: 123.5f64.to_bits(),
            },
            gate: gate ^ previous.unwrap().result,
            trips,
            parent,
            root: 1,
            after: if gate && parent == -1 { 0 } else { 1 },
            rounds: 2,
            ..open
        });
    }
    rows
}
pub fn prefix(variant: Variant) -> Vec<Call> {
    let mut rows = vec![];
    for _ in 0..3 {
        let previous = rows.last().map(|c: &Call| c.next());
        rows.push(Call {
            previous,
            rounds: 0,
            trips: i64::MAX,
            root: 0,
            ..base(variant)
        });
    }
    rows
}
pub fn after_prefix(mut call: Call) -> Call {
    let prior = prefix(call.variant).last().unwrap().next();
    let gate = call.gate ^ call.input.result;
    call.previous = Some(prior);
    call.gate = gate ^ prior.result;
    call
}
pub fn failures(variant: Variant) -> Vec<Call> {
    let base = base(variant);
    let mut rows = vec![];
    for parent in [-1, 1] {
        for root in [0, 2] {
            rows.push(Call {
                root,
                trips: if parent == 1 { 0 } else { 3 },
                parent,
                ..base
            });
        }
    }
    rows.push(Call {
        cut: 99,
        hole: i64::from(variant.leading),
        ..base
    });
    rows.push(Call {
        gate: false,
        trips: 0,
        parent: -1,
        after: 0,
        ..base
    });
    rows.push(Call { tail: 0, ..base });
    rows
}
