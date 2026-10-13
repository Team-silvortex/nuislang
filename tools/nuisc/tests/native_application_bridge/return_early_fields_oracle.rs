pub const SOURCE: &str = include_str!("return_early_fields.ns");
pub const STAGES: [i64; 12] = [90, 87, 92, 94, 93, 74, 81, 78, 95, 76, 96, 97];
const FIELDS: [&str; 14] = [
    "first",
    "second",
    "outer",
    "init",
    "cond",
    "small_rhs",
    "narrow_rhs",
    "wide_rhs",
    "bool_rhs",
    "peer_rhs",
    "parent_rhs",
    "exit_guard",
    "after",
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
    pub exited: bool,
    pub result: bool,
    pub trace: Vec<i64>,
}
fn parent(value: i64, a: i32, b: i64, c: i64, d: bool, e: bool) -> i64 {
    let parity =
        (a < 0) ^ (f32::from_bits(b as u32) < 0.0) ^ (f64::from_bits(c as u64) < 0.0) ^ d ^ e;
    if parity {
        !value
    } else {
        value
    }
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
        let mut trace = vec![99, 90];
        let (mut owner, mut a, mut b, mut c, mut d, mut e) = (
            seed,
            small_seed,
            narrow_seed,
            wide_seed,
            bool_seed,
            peer_seed,
        );
        let mut fields = None;
        let mut exited = false;
        let result;
        if first {
            trace.extend([87, 92]);
            if second {
                trace.extend([94, 93, 74, 81]);
                a = !a;
                b ^= 1 << 31;
                if (f32::from_bits(b as u32) < 0.0) ^ d {
                    c ^= i64::MIN;
                }
                d = !d;
                if e {
                    trace.push(78);
                    e = d;
                }
                trace.push(95);
                owner = parent(owner, a, b, c, d, e);
            }
            fields = Some((owner, i64::from(a), b, c, i64::from(d), i64::from(e)));
            trace.push(76);
            exited = d != e;
            if exited {
                if d {
                    trace.push(97);
                }
                result = d && owner == seed;
            } else {
                trace.extend([96, 95, 97]);
                owner = parent(owner, a, b, c, d, e);
                result = owner == seed || e;
            }
        } else {
            trace.push(97);
            result = second;
        }
        trace.push(79);
        Self {
            seed,
            small_seed,
            narrow_seed,
            wide_seed,
            bool_seed,
            peer_seed,
            first,
            second,
            owner,
            fields,
            exited,
            result,
            trace,
        }
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
    pub fn words(&self) -> Vec<i64> {
        [i64::from(self.first), i64::from(self.second)]
            .into_iter()
            .chain(STAGES.map(|stage| i64::from(self.trace.contains(&stage))))
            .collect()
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
        for &event in self.trace.iter().take(count) {
            if event == 76 {
                let (v, a, b, c, d, e) = self.fields.unwrap();
                out.extend([v, a, b, c, d, e]);
            }
            if event == 97 {
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
pub fn cli_cases() -> Vec<Case> {
    let mut cases = vec![];
    for first in [false, true] {
        for second in [false, true] {
            for d in [false, true] {
                for e in [false, true] {
                    cases.push(Case::new(
                        i64::MAX,
                        i32::MIN,
                        i64::from((-2.5f32).to_bits()),
                        4.75f64.to_bits() as i64,
                        d,
                        e,
                        first,
                        second,
                    ));
                }
            }
        }
    }
    cases
}
pub fn cases() -> Vec<Case> {
    let mut cases = cli_cases();
    for (b, c) in [
        (0, 0),
        (1 << 31, i64::MIN),
        (1, 1),
        ((1 << 31) | 1, i64::MIN | 1),
        (0x7f800000, 0x7ff0000000000000),
        (0xff800000, 0xfff0000000000000u64 as i64),
        (0x7fc01234, 0x7ff8000000001234),
        (0xff801234, 0xfff0000000001234u64 as i64),
    ] {
        for d in [false, true] {
            for e in [false, true] {
                for second in [false, true] {
                    cases.push(Case::new(i64::MIN, i32::MAX, b, c, d, e, true, second));
                }
            }
        }
    }
    cases
}
pub fn selected() -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|stage| {
            cases()
                .into_iter()
                .find(|c| c.trace.contains(&stage))
                .unwrap()
        })
        .collect()
}
