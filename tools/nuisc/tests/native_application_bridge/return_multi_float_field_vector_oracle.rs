#[allow(dead_code)]
#[path = "return_private_fields_oracle.rs"]
mod inputs;
pub const SOURCE: &str = include_str!("return_multi_float_field_vector.ns");
pub const STAGES: [i64; 17] = [
    90, 87, 88, 91, 73, 82, 84, 92, 95, 94, 93, 74, 81, 78, 96, 83, 97,
];
const FIELDS: [&str; 19] = [
    "first",
    "second",
    "outer",
    "init",
    "small_init",
    "narrow_init",
    "wide_init",
    "bool_init",
    "peer_init",
    "cond",
    "parent_rhs",
    "small_rhs",
    "narrow_rhs",
    "wide_rhs",
    "bool_rhs",
    "peer_rhs",
    "after",
    "unused",
    "answer",
];
#[derive(Clone)]
pub struct Case {
    pub seed: i64,
    pub small_seed: i32,
    pub narrow_seed: i64,
    pub wide_seed: i64,
    pub bool_seed: bool,
    pub peer_seed: bool,
    pub first: bool,
    pub second: bool,
    pub owner: i64,
    pub fields: Option<(i64, i64, i64, i64, i64, i64)>,
    pub result: bool,
    pub trace: Vec<i64>,
}
fn negative32(bits: i64) -> bool {
    f32::from_bits(bits as u32) < 0.0
}
fn negative64(bits: i64) -> bool {
    f64::from_bits(bits as u64) < 0.0
}
impl Case {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seed: i64,
        small_seed: i32,
        narrow_seed: i64,
        wide_seed: i64,
        bool_seed: bool,
        peer_seed: bool,
        first: bool,
        second: bool,
    ) -> Self {
        Self::evaluate(
            seed,
            small_seed,
            narrow_seed,
            wide_seed,
            bool_seed,
            peer_seed,
            first,
            second,
            false,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn evaluate(
        seed: i64,
        small_seed: i32,
        narrow_seed: i64,
        wide_seed: i64,
        bool_seed: bool,
        peer_seed: bool,
        first: bool,
        second: bool,
        stale: bool,
    ) -> Self {
        // Original-width sign XORs and signed complements are independent of compiler transport.
        let wide = |value, b, flag| {
            if negative32(if stale { narrow_seed } else { b }) != flag {
                value ^ i64::MIN
            } else {
                value
            }
        };
        let parent = |value: i64, a: i32, b, c, d, e| {
            if ((((a < 0) != negative32(b)) != negative64(c)) != d) != e {
                !value
            } else {
                value
            }
        };
        let mut local = seed;
        let mut fields = None;
        let mut trace = vec![99, 90, 87];
        if first && second {
            let (mut a, mut b, mut c, mut d, mut e) =
                (small_seed, narrow_seed, wide_seed, bool_seed, peer_seed);
            trace.extend([88, 91, 73, 82, 84, 92]);
            if negative32(b) {
                a = !a;
                trace.push(94);
                b ^= 1 << 31;
                trace.push(93);
                d = !d;
                trace.push(81);
                if e {
                    trace.push(78);
                }
                e = e && d;
                c = wide(c, b, d);
                trace.push(74);
                if a == 0 {
                    a = !a;
                    trace.push(94);
                    d = !d;
                    trace.push(81);
                    c = wide(c, b, d);
                    trace.push(74);
                }
            } else {
                e = !e;
                trace.push(78);
                if !d {
                    trace.push(81);
                }
                d = d || e;
                b ^= 1 << 31;
                trace.push(93);
                a = !a;
                trace.push(94);
                c = wide(c, b, d);
                trace.push(74);
                if a == -1 {
                    b ^= 1 << 31;
                    trace.push(93);
                    e = !e;
                    trace.push(78);
                    c = wide(c, b, d);
                    trace.push(74);
                }
            }
            fields = Some((local, i64::from(a), b, c, i64::from(d), i64::from(e)));
            trace.push(96);
            local = parent(local, a, b, c, d, e);
            trace.push(95);
            if a < 0 {
                e = !e;
                trace.push(78);
                c = wide(c, b, e);
                trace.push(74);
                local = parent(local, a, b, c, d, e);
                trace.push(95);
            }
            trace.push(83);
        }
        trace.extend([97, 79]);
        Self {
            seed,
            small_seed,
            narrow_seed,
            wide_seed,
            bool_seed,
            peer_seed,
            first,
            second,
            owner: local,
            fields,
            result: if first {
                local == seed && first
            } else {
                local == seed || second
            },
            trace,
        }
    }
    pub fn stale_narrow(&self) -> Self {
        Self::evaluate(
            self.seed,
            self.small_seed,
            self.narrow_seed,
            self.wide_seed,
            self.bool_seed,
            self.peer_seed,
            self.first,
            self.second,
            true,
        )
    }
    pub fn previous(&self) -> Self {
        Self::new(
            self.seed,
            self.small_seed,
            self.narrow_seed,
            self.wide_seed,
            self.bool_seed,
            self.peer_seed,
            !self.first,
            !self.second,
        )
    }
    pub fn values(&self) -> Vec<String> {
        self.words()
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                if index < 2 {
                    (value != 0).to_string()
                } else {
                    value.to_string()
                }
            })
            .collect()
    }
    pub fn words(&self) -> Vec<i64> {
        [i64::from(self.first), i64::from(self.second)]
            .into_iter()
            .chain(STAGES.map(|stage| i64::from(self.trace.contains(&stage))))
            .collect()
    }
    pub fn open_words(&self) -> Vec<i64> {
        [
            self.seed,
            i64::from(self.small_seed),
            self.narrow_seed,
            self.wide_seed,
            i64::from(self.bool_seed),
            i64::from(self.peer_seed),
        ]
        .into_iter()
        .chain(self.words())
        .collect()
    }
    pub fn state_words(&self) -> Vec<i64> {
        self.open_words()
            .into_iter()
            .chain([i64::from(self.result)])
            .collect()
    }
    pub fn state_text(&self, phase: &str) -> String {
        let mut out = vec![
            format!("seed: {}", self.seed),
            format!("small_seed: {}i32", self.small_seed),
            format!(
                "narrow_seed: {}f32",
                f64::from(f32::from_bits(self.narrow_seed as u32))
            ),
            format!("wide_seed: {}f64", f64::from_bits(self.wide_seed as u64)),
            format!("bool_seed: {}", self.bool_seed),
            format!("peer_seed: {}", self.peer_seed),
        ];
        out.extend(
            FIELDS
                .into_iter()
                .zip(self.values())
                .map(|(name, value)| format!("{name}: {value}")),
        );
        out.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", out.join(", "))
    }
    pub fn open_args(&self) -> String {
        format!(
            "{},{},{},{},{},{},{}",
            self.seed,
            self.small_seed,
            f32::from_bits(self.narrow_seed as u32),
            f64::from_bits(self.wide_seed as u64),
            self.bool_seed,
            self.peer_seed,
            self.values().join(",")
        )
    }
    pub fn bad_values(&self, index: usize) -> Vec<String> {
        let mut values = self.values();
        assert_eq!(values[index + 2], "1");
        values[index + 2] = "0".into();
        values
    }
    fn observe_until(&self, count: usize) -> Vec<i64> {
        let mut out = vec![];
        // Cut by source-event positions, not potentially colliding floating payload words.
        for &event in self.trace.iter().take(count) {
            if event == 96 {
                let (v, a, b, c, d, e) = self.fields.unwrap();
                out.extend([v, a, b, c, d, e]);
            } else if event == 97 {
                out.push(self.owner);
            }
            out.push(event);
        }
        out
    }
    pub fn observed(&self) -> Vec<i64> {
        self.observe_until(self.trace.len())
    }
    pub fn before_failure(&self, index: usize) -> Vec<i64> {
        self.observe_until(self.trace.iter().position(|&v| v == STAGES[index]).unwrap())
    }
}
pub fn narrow_bits(value: f32) -> i64 {
    i64::from(value.to_bits())
}
pub fn wide_bits(value: f64) -> i64 {
    value.to_bits() as i64
}
#[allow(dead_code)]
pub fn cases() -> Vec<Case> {
    let mut out = vec![];
    let narrow = inputs::words("f32");
    let wide = inputs::words("f64");
    // Two companion wide profiles per narrow word, not a full IEEE Cartesian product.
    for (index, b) in narrow.into_iter().enumerate() {
        for offset in [1, 5] {
            for seed in [i64::MIN, i64::MAX] {
                for small in [i32::MIN, i32::MAX] {
                    for d in [false, true] {
                        for e in [false, true] {
                            for first in [false, true] {
                                for second in [false, true] {
                                    out.push(Case::new(
                                        seed,
                                        small,
                                        b,
                                        wide[(index + offset) % wide.len()],
                                        d,
                                        e,
                                        first,
                                        second,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out.extend(masked_cases());
    for seed in [i64::MIN, i64::MAX] {
        out.push(Case::new(
            seed,
            -1,
            narrow_bits(-2.5),
            wide_bits(4.75),
            false,
            true,
            true,
            true,
        ));
        out.push(Case::new(
            seed,
            0,
            narrow_bits(1.5),
            wide_bits(-7.25),
            false,
            false,
            true,
            true,
        ));
    }
    out
}
pub fn masked_cases() -> Vec<Case> {
    let words = inputs::words("f64");
    [0, i64::MIN, words[10], words[11]]
        .into_iter()
        .flat_map(|c| {
            [-2.5, 1.5].map(|b| {
                Case::new(
                    i64::MAX,
                    i32::MIN,
                    narrow_bits(b),
                    c,
                    false,
                    true,
                    true,
                    true,
                )
            })
        })
        .collect()
}
pub fn cli_cases() -> Vec<Case> {
    let mut out = vec![];
    for (seed, b, c, d, e) in [
        (i64::MIN, -2.5, 4.75, false, true),
        (i64::MAX, 1.5, -7.25, false, true),
        (70001, -2.5, -0.0, true, false),
        (-70001, 1.5, 0.0, true, true),
    ] {
        for first in [false, true] {
            for second in [false, true] {
                out.push(Case::new(
                    seed,
                    i32::MIN,
                    narrow_bits(b),
                    wide_bits(c),
                    d,
                    e,
                    first,
                    second,
                ));
            }
        }
    }
    out.push(Case::new(
        70001,
        -1,
        narrow_bits(-2.5),
        wide_bits(4.75),
        false,
        true,
        true,
        true,
    ));
    out.push(Case::new(
        -70001,
        0,
        narrow_bits(1.5),
        wide_bits(-7.25),
        false,
        false,
        true,
        true,
    ));
    out
}
pub fn selected() -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|stage| {
            if stage == 83 || stage == 97 {
                Case::new(
                    70001,
                    70001,
                    narrow_bits(-2.5),
                    wide_bits(-7.25),
                    false,
                    true,
                    true,
                    true,
                )
            } else {
                cli_cases()
                    .into_iter()
                    .find(|case| case.trace.contains(&stage))
                    .unwrap()
            }
        })
        .collect()
}
#[allow(dead_code)]
pub fn native_selected() -> Vec<Case> {
    let mut out = selected();
    for index in [15, 16] {
        out[index] = Case::new(
            70001,
            70001,
            (1 << 31) | 93,
            i64::MIN | 96,
            false,
            true,
            true,
            true,
        );
    }
    out
}
