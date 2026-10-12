pub const SOURCE: &str = include_str!("return_private_fields.ns");
pub const STAGES: [i64; 11] = [90, 87, 88, 91, 92, 95, 94, 93, 96, 83, 97];
pub const FIELDS: [&str; 13] = [
    "first",
    "second",
    "outer",
    "init",
    "small_init",
    "float_init",
    "cond",
    "parent_rhs",
    "small_rhs",
    "float_rhs",
    "after",
    "unused",
    "answer",
];
#[derive(Clone)]
pub struct Case {
    retain_owner: bool,
    pub ty: &'static str,
    pub seed: i64,
    pub small_seed: i32,
    pub float_seed: i64,
    pub first: bool,
    pub second: bool,
    pub owner: i64,
    pub fields: Option<(i64, i64, i64)>,
    pub result: bool,
    pub trace: Vec<i64>,
}
impl Case {
    // Independent i64/i32 complements and raw IEEE sign bits, not a compiler carrier.
    pub fn new(
        ty: &'static str,
        seed: i64,
        small_seed: i32,
        float_seed: i64,
        first: bool,
        second: bool,
    ) -> Self {
        Self::calculate(ty, seed, small_seed, float_seed, first, second, false)
    }
    #[allow(dead_code)]
    pub fn retained(
        ty: &'static str,
        seed: i64,
        small_seed: i32,
        float_seed: i64,
        first: bool,
        second: bool,
    ) -> Self {
        Self::calculate(ty, seed, small_seed, float_seed, first, second, true)
    }
    fn calculate(
        ty: &'static str,
        seed: i64,
        small_seed: i32,
        float_seed: i64,
        first: bool,
        second: bool,
        retain_owner: bool,
    ) -> Self {
        let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
        let negative = |bits| {
            if ty == "f32" {
                f32::from_bits(bits as u32) < 0.0
            } else {
                f64::from_bits(bits as u64) < 0.0
            }
        };
        let parent = |value: i64, small: i32, bits| {
            if (small < 0) != negative(bits) {
                !value
            } else {
                value
            }
        };
        let mut local = seed;
        let mut fields = None;
        let mut trace = vec![99, 90, 87];
        if first && second {
            let mut a = small_seed;
            let mut b = float_seed;
            trace.extend([88, 91, 92]);
            if negative(b) {
                if !retain_owner {
                    local = parent(local, a, b);
                    trace.push(95);
                }
                a = !a;
                trace.push(94);
                b ^= sign;
                trace.push(93);
                if a == 0 {
                    a = !a;
                    trace.push(94);
                    b ^= sign;
                    trace.push(93);
                    if !retain_owner {
                        local = parent(local, a, b);
                        trace.push(95);
                    }
                }
            } else {
                b ^= sign;
                trace.push(93);
                a = !a;
                trace.push(94);
                if !retain_owner {
                    local = parent(local, a, b);
                    trace.push(95);
                }
                if a == -1 {
                    if !retain_owner {
                        local = parent(local, a, b);
                        trace.push(95);
                    }
                    b ^= sign;
                    trace.push(93);
                    a = !a;
                    trace.push(94);
                }
            }
            fields = Some((local, i64::from(a), b));
            trace.push(96);
            if retain_owner {
                local = parent(local, a, b);
                trace.push(95);
            }
            if a < 0 {
                local = parent(local, a, b);
                trace.push(95);
            }
            trace.push(83);
        }
        trace.extend([97, 79]);
        let equal = local == seed;
        Self {
            retain_owner,
            ty,
            seed,
            small_seed,
            float_seed,
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
        Self::calculate(
            self.ty,
            self.seed,
            self.small_seed,
            self.float_seed,
            !self.first,
            !self.second,
            self.retain_owner,
        )
    }
    pub fn values(&self) -> Vec<String> {
        let mut a = vec![self.first.to_string(), self.second.to_string()];
        a.extend(STAGES.map(|s| if self.trace.contains(&s) { "1" } else { "0" }.into()));
        a
    }
    pub fn words(&self) -> Vec<i64> {
        let mut a = vec![i64::from(self.first), i64::from(self.second)];
        a.extend(STAGES.map(|s| i64::from(self.trace.contains(&s))));
        a
    }
    pub fn open_words(&self) -> Vec<i64> {
        [self.seed, i64::from(self.small_seed), self.float_seed]
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
        let mut f = vec![
            format!("seed: {}", self.seed),
            format!("small_seed: {}i32", self.small_seed),
            format!("float_seed: {}{}", self.state_decimal(), self.ty),
        ];
        f.extend(
            FIELDS
                .into_iter()
                .zip(self.values())
                .map(|(n, v)| format!("{n}: {v}")),
        );
        f.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", f.join(", "))
    }
    pub fn decimal(&self) -> String {
        if self.ty == "f32" {
            f32::from_bits(self.float_seed as u32).to_string()
        } else {
            f64::from_bits(self.float_seed as u64).to_string()
        }
    }
    pub fn state_decimal(&self) -> String {
        if self.ty == "f32" {
            f64::from(f32::from_bits(self.float_seed as u32)).to_string()
        } else {
            f64::from_bits(self.float_seed as u64).to_string()
        }
    }
    pub fn open_args(&self) -> String {
        format!(
            "{},{},{},{}",
            self.seed,
            self.small_seed,
            self.decimal(),
            self.values().join(",")
        )
    }
    pub fn bad_values(&self, index: usize) -> Vec<String> {
        let mut v = self.values();
        assert_eq!(v[index + 2], "1");
        v[index + 2] = "0".into();
        v
    }
    fn observe_until(&self, count: usize) -> Vec<i64> {
        // Source-event positions remain distinct from observed field words and sentinels.
        let mut out = vec![];
        for event in self.trace.iter().take(count) {
            if *event == 96 {
                let (owner, a, b) = self.fields.unwrap();
                out.extend([owner, a, b]);
            } else if *event == 97 {
                out.push(self.owner);
            }
            out.push(*event);
        }
        out
    }
    pub fn observed(&self) -> Vec<i64> {
        self.observe_until(self.trace.len())
    }
    pub fn before_failure(&self, index: usize) -> Vec<i64> {
        self.observe_until(self.trace.iter().position(|v| *v == STAGES[index]).unwrap())
    }
}

pub fn words(ty: &str) -> Vec<i64> {
    if ty == "f32" {
        vec![
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            0x007f_ffff,
            0x807f_ffff,
            0x3fc0_0000,
            0xc020_0000,
            0x7f80_0000,
            0xff80_0000,
            0x7fc0_1234,
            0xff80_0001,
        ]
    } else {
        vec![
            0,
            i64::MIN,
            1,
            0x8000_0000_0000_0001_u64 as i64,
            0x000f_ffff_ffff_ffff,
            0x800f_ffff_ffff_ffff_u64 as i64,
            0x3ff8_0000_0000_0000,
            0xc004_0000_0000_0000_u64 as i64,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000_u64 as i64,
            0x7ff8_1234_5678_abcd,
            0xfff0_0000_0000_0001_u64 as i64,
        ]
    }
}

#[allow(dead_code)]
pub fn cases(ty: &'static str) -> Vec<Case> {
    let mut out = vec![];
    for seed in [i64::MIN, i64::MAX, -(1_i64 << 40) - 96] {
        for small in [i32::MIN, i32::MAX, -1, 0] {
            for word in words(ty) {
                for first in [false, true] {
                    for second in [false, true] {
                        out.push(Case::new(ty, seed, small, word, first, second));
                    }
                }
            }
        }
    }
    out
}
pub fn cli_cases(ty: &'static str) -> Vec<Case> {
    let mut out = vec![];
    for (seed, small, value) in [
        (i64::MIN, i32::MIN, -2.5),
        (i64::MAX, i32::MAX, 1.5),
        (-(1_i64 << 40) - 96, -1, -0.0),
        ((1_i64 << 40) + 96, 0, 0.0),
        (70001, 70001, -2.5),
        (-70001, i32::MIN, 1.5),
    ] {
        let word = if ty == "f32" {
            i64::from((value as f32).to_bits())
        } else {
            f64::to_bits(value) as i64
        };
        for first in [false, true] {
            for second in [false, true] {
                out.push(Case::new(ty, seed, small, word, first, second));
            }
        }
    }
    out
}
pub fn selected(ty: &'static str) -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|stage| {
            if stage == 83 || stage == 97 {
                let word = if ty == "f32" {
                    i64::from((-2.5_f32).to_bits())
                } else {
                    (-2.5_f64).to_bits() as i64
                };
                Case::new(ty, 70001, 70001, word, true, true)
            } else {
                cli_cases(ty)
                    .into_iter()
                    .find(|c| c.trace.contains(&stage))
                    .unwrap()
            }
        })
        .collect()
}
#[allow(dead_code)]
pub fn native_selected(ty: &'static str) -> Vec<Case> {
    let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
    let mut out = selected(ty);
    for index in [9, 10] {
        out[index] = Case::new(ty, 70001, 70001, sign | 96, true, true);
    }
    out
}
