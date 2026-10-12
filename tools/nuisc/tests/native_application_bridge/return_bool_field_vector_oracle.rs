#[allow(dead_code)]
#[path = "return_private_fields_oracle.rs"]
mod inputs;
pub const SOURCE: &str = include_str!("return_bool_field_vector.ns");
pub const STAGES: [i64; 13] = [90, 87, 88, 91, 82, 92, 95, 94, 93, 81, 96, 83, 97];
const FIELDS: [&str; 15] = [
    "first",
    "second",
    "outer",
    "init",
    "small_init",
    "float_init",
    "bool_init",
    "cond",
    "parent_rhs",
    "small_rhs",
    "float_rhs",
    "bool_rhs",
    "after",
    "unused",
    "answer",
];
#[derive(Clone)]
pub struct Case {
    pub ty: &'static str,
    pub seed: i64,
    pub small_seed: i32,
    pub float_seed: i64,
    pub bool_seed: bool,
    pub first: bool,
    pub second: bool,
    pub owner: i64,
    pub fields: Option<(i64, i64, i64, i64)>,
    pub result: bool,
    pub trace: Vec<i64>,
}
impl Case {
    pub fn new(
        ty: &'static str,
        seed: i64,
        small_seed: i32,
        float_seed: i64,
        bool_seed: bool,
        first: bool,
        second: bool,
    ) -> Self {
        // Independent signed complements, IEEE sign XORs and boolean logic, not a compiler carrier.
        let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
        let negative = |bits| {
            if ty == "f32" {
                f32::from_bits(bits as u32) < 0.0
            } else {
                f64::from_bits(bits as u64) < 0.0
            }
        };
        let parent = |v: i64, a: i32, b, c: bool| {
            if ((a < 0) != negative(b)) != c {
                !v
            } else {
                v
            }
        };
        let mut local = seed;
        let mut fields = None;
        let mut trace = vec![99, 90, 87];
        if first && second {
            let mut a = small_seed;
            let mut b = float_seed;
            let mut c = bool_seed;
            trace.extend([88, 91, 82, 92]);
            if negative(b) {
                a = !a;
                trace.push(94);
                b ^= sign;
                trace.push(93);
                if c {
                    trace.push(81);
                }
                if a == 0 {
                    a = !a;
                    trace.push(94);
                    c = !c;
                    trace.push(81);
                }
            } else {
                if !c {
                    trace.push(81);
                }
                b ^= sign;
                trace.push(93);
                a = !a;
                trace.push(94);
                if a == -1 {
                    b ^= sign;
                    trace.push(93);
                    c = !c;
                    trace.push(81);
                }
            }
            fields = Some((local, i64::from(a), b, i64::from(c)));
            trace.push(96);
            local = parent(local, a, b, c);
            trace.push(95);
            if a < 0 {
                c = !c;
                trace.push(81);
                local = parent(local, a, b, c);
                trace.push(95);
            }
            trace.push(83);
        }
        trace.extend([97, 79]);
        let equal = local == seed;
        Self {
            ty,
            seed,
            small_seed,
            float_seed,
            bool_seed,
            first,
            second,
            owner: local,
            fields,
            result: if first {
                equal && first
            } else {
                equal || second
            },
            trace,
        }
    }
    pub fn previous(&self) -> Self {
        Self::new(
            self.ty,
            self.seed,
            self.small_seed,
            self.float_seed,
            self.bool_seed,
            !self.first,
            !self.second,
        )
    }
    pub fn values(&self) -> Vec<String> {
        let mut out = vec![self.first.to_string(), self.second.to_string()];
        out.extend(STAGES.map(|s| if self.trace.contains(&s) { "1" } else { "0" }.into()));
        out
    }
    pub fn words(&self) -> Vec<i64> {
        let mut out = vec![i64::from(self.first), i64::from(self.second)];
        out.extend(STAGES.map(|s| i64::from(self.trace.contains(&s))));
        out
    }
    pub fn open_words(&self) -> Vec<i64> {
        [
            self.seed,
            i64::from(self.small_seed),
            self.float_seed,
            i64::from(self.bool_seed),
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
    pub fn decimal(&self) -> String {
        if self.ty == "f32" {
            f32::from_bits(self.float_seed as u32).to_string()
        } else {
            f64::from_bits(self.float_seed as u64).to_string()
        }
    }
    fn state_decimal(&self) -> String {
        if self.ty == "f32" {
            f64::from(f32::from_bits(self.float_seed as u32)).to_string()
        } else {
            self.decimal()
        }
    }
    pub fn state_text(&self, phase: &str) -> String {
        let mut out = vec![
            format!("seed: {}", self.seed),
            format!("small_seed: {}i32", self.small_seed),
            format!("float_seed: {}{}", self.state_decimal(), self.ty),
            format!("bool_seed: {}", self.bool_seed),
        ];
        out.extend(
            FIELDS
                .into_iter()
                .zip(self.values())
                .map(|(n, v)| format!("{n}: {v}")),
        );
        out.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", out.join(", "))
    }
    pub fn open_args(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.seed,
            self.small_seed,
            self.decimal(),
            self.bool_seed,
            self.values().join(",")
        )
    }
    pub fn bad_values(&self, index: usize) -> Vec<String> {
        let mut out = self.values();
        assert_eq!(out[index + 2], "1");
        out[index + 2] = "0".into();
        out
    }
    fn observe_until(&self, count: usize) -> Vec<i64> {
        let mut out = vec![];
        // Source-event positions disambiguate sentinels and actual four field words.
        for &event in self.trace.iter().take(count) {
            if event == 96 {
                let (v, a, b, c) = self.fields.unwrap();
                out.extend([v, a, b, c]);
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
#[allow(dead_code)]
pub fn cases(ty: &'static str) -> Vec<Case> {
    let mut out = vec![];
    for seed in [i64::MIN, i64::MAX, -(1_i64 << 40) - 96] {
        for small in [i32::MIN, i32::MAX, -1, 0] {
            for bits in inputs::words(ty) {
                for flag in [false, true] {
                    for first in [false, true] {
                        for second in [false, true] {
                            out.push(Case::new(ty, seed, small, bits, flag, first, second));
                        }
                    }
                }
            }
        }
    }
    out
}
pub fn cli_cases(ty: &'static str) -> Vec<Case> {
    let mut out = vec![];
    for (seed, small, value, flag) in [
        (i64::MIN, i32::MIN, -2.5, false),
        (i64::MAX, i32::MAX, 1.5, false),
        (-(1_i64 << 40) - 96, -1, -0.0, false),
        ((1_i64 << 40) + 96, -1, 0.0, true),
        (70001, 70001, -2.5, false),
        (-70001, i32::MIN, 1.5, true),
    ] {
        let bits = if ty == "f32" {
            i64::from((value as f32).to_bits())
        } else {
            f64::to_bits(value) as i64
        };
        for first in [false, true] {
            for second in [false, true] {
                out.push(Case::new(ty, seed, small, bits, flag, first, second));
            }
        }
    }
    out
}
pub fn selected(ty: &'static str) -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|s| {
            if s == 83 || s == 97 {
                let bits = if ty == "f32" {
                    i64::from((-2.5_f32).to_bits())
                } else {
                    (-2.5_f64).to_bits() as i64
                };
                Case::new(ty, 70001, 70001, bits, true, true, true)
            } else {
                cli_cases(ty)
                    .into_iter()
                    .find(|c| c.trace.contains(&s))
                    .unwrap()
            }
        })
        .collect()
}
#[allow(dead_code)]
pub fn native_selected(ty: &'static str) -> Vec<Case> {
    let mut out = selected(ty);
    let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
    for index in [11, 12] {
        out[index] = Case::new(ty, 70001, 70001, sign | 96, true, true, true);
    }
    out
}
