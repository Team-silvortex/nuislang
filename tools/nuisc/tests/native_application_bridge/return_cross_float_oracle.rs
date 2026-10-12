pub const SOURCE: &str = include_str!("return_cross_float_private_pair.ns");
pub fn child_type(ty: &str) -> &'static str {
    match ty {
        "f32" => "f64",
        "f64" => "f32",
        _ => panic!("invalid floating kind"),
    }
}
pub fn source(ty: &str) -> String {
    SOURCE
        .replace("f32", "OWNER")
        .replace("f64", child_type(ty))
        .replace("OWNER", ty)
}
pub const STAGES: [i64; 9] = [90, 87, 91, 92, 95, 93, 96, 83, 97];
pub const FIELDS: [&str; 11] = [
    "first",
    "second",
    "outer",
    "init",
    "child_init",
    "cond",
    "parent_rhs",
    "child_rhs",
    "after",
    "unused",
    "answer",
];

#[derive(Clone)]
pub struct Case {
    pub ty: &'static str,
    pub seed: i64,
    pub child_seed: i64,
    pub first: bool,
    pub second: bool,
    pub owner: i64,
    pub pair: Option<(i64, i64)>,
    pub result: bool,
    pub trace: Vec<i64>,
}

impl Case {
    // Independent mixed-width input words retain their own sign/payload bits, without compiler carrier logic.
    pub fn new(ty: &'static str, seed: i64, child_seed: i64, first: bool, second: bool) -> Self {
        let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
        let child_ty = child_type(ty);
        let child_sign = if child_ty == "f32" { 1 << 31 } else { i64::MIN };
        let number = |bits, ty| {
            if ty == "f32" {
                f64::from(f32::from_bits(bits as u32))
            } else {
                f64::from_bits(bits as u64)
            }
        };
        let parent = |bits, child| {
            if number(child, child_ty) < 0.0 {
                bits ^ sign
            } else {
                bits
            }
        };
        let mut trace = vec![99, 90, 87];
        let mut local = seed;
        let mut pair = None;
        if first && second {
            let mut child = child_seed;
            trace.extend([91, 92]);
            if number(child, child_ty) < 0.0 {
                local = parent(local, child);
                trace.push(95);
                child ^= child_sign;
                trace.push(93);
                if number(child, child_ty) == 0.0 {
                    child ^= child_sign;
                    trace.push(93);
                    local = parent(local, child);
                    trace.push(95);
                }
            } else {
                child ^= child_sign;
                trace.push(93);
                local = parent(local, child);
                trace.push(95);
                if number(child, child_ty) == 0.0 {
                    local = parent(local, child);
                    trace.push(95);
                    child ^= child_sign;
                    trace.push(93);
                }
            }
            pair = Some((local, child));
            trace.push(96);
            if number(child, child_ty) < 0.0 {
                local = parent(local, child);
                trace.push(95);
            }
            let _seal = child;
            trace.push(83);
        }
        trace.extend([97, 79]);
        let equal = number(local, ty) == number(seed, ty);
        Self {
            ty,
            seed,
            child_seed,
            first,
            second,
            owner: local,
            pair,
            result: if first {
                equal && first
            } else {
                equal || second
            },
            trace,
        }
    }

    pub fn child_ty(&self) -> &'static str {
        child_type(self.ty)
    }

    pub fn previous(&self) -> Self {
        Self::new(
            self.ty,
            self.seed,
            self.child_seed,
            !self.first,
            !self.second,
        )
    }
    pub fn values(&self) -> Vec<String> {
        let mut values = vec![self.first.to_string(), self.second.to_string()];
        values.extend(STAGES.map(|stage| {
            if self.trace.contains(&stage) {
                "1"
            } else {
                "0"
            }
            .into()
        }));
        values
    }
    pub fn words(&self) -> Vec<i64> {
        let mut values = vec![i64::from(self.first), i64::from(self.second)];
        values.extend(STAGES.map(|stage| i64::from(self.trace.contains(&stage))));
        values
    }
    pub fn open_words(&self) -> Vec<i64> {
        [self.seed, self.child_seed]
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
    fn decimal(&self, bits: i64, ty: &str) -> String {
        if ty == "f32" {
            f32::from_bits(bits as u32).to_string()
        } else {
            f64::from_bits(bits as u64).to_string()
        }
    }
    pub fn state_text(&self, phase: &str) -> String {
        let mut fields = vec![
            format!("seed: {}{}", self.decimal(self.seed, self.ty), self.ty),
            format!(
                "child_seed: {}{}",
                self.decimal(self.child_seed, self.child_ty()),
                self.child_ty()
            ),
        ];
        fields.extend(
            FIELDS
                .into_iter()
                .zip(self.values())
                .map(|(n, v)| format!("{n}: {v}")),
        );
        fields.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", fields.join(", "))
    }
    pub fn open_args(&self) -> String {
        format!(
            "{},{},{}",
            self.decimal(self.seed, self.ty),
            self.decimal(self.child_seed, self.child_ty()),
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
        // Field words may equal event markers; boundaries are original event positions.
        let mut output = Vec::new();
        for stage in self.trace.iter().take(count) {
            if *stage == 96 {
                let (owner, child) = self.pair.unwrap();
                output.extend([owner, child]);
            } else if *stage == 97 {
                output.push(self.owner);
            }
            output.push(*stage);
        }
        output
    }
    pub fn observed(&self) -> Vec<i64> {
        self.observe_until(self.trace.len())
    }
    pub fn before_failure(&self, index: usize) -> Vec<i64> {
        let position = self.trace.iter().position(|v| *v == STAGES[index]).unwrap();
        self.observe_until(position)
    }
}

#[allow(dead_code)] // Native-only raw IEEE inventory; the CLI uses finite inputs.
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
    let mut cases = Vec::new();
    for seed in words(ty) {
        for child_seed in words(child_type(ty)) {
            for first in [false, true] {
                for second in [false, true] {
                    cases.push(Case::new(ty, seed, child_seed, first, second));
                }
            }
        }
    }
    cases
}
pub fn cli_cases(ty: &'static str) -> Vec<Case> {
    let bits = |value: f64, kind: &str| {
        if kind == "f32" {
            i64::from((value as f32).to_bits())
        } else {
            value.to_bits() as i64
        }
    };
    let mut cases = Vec::new();
    for (seed, child_seed) in [(1.5, -2.5), (-2.5, 1.5), (0.0, -0.0), (-0.0, 0.0)] {
        for first in [false, true] {
            for second in [false, true] {
                cases.push(Case::new(
                    ty,
                    bits(seed, ty),
                    bits(child_seed, child_type(ty)),
                    first,
                    second,
                ));
            }
        }
    }
    cases
}
pub fn selected(ty: &'static str) -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|stage| {
            cli_cases(ty)
                .into_iter()
                .find(|case| case.trace.contains(&stage))
                .unwrap()
        })
        .collect()
}
#[allow(dead_code)]
pub fn native_selected(ty: &'static str) -> Vec<Case> {
    let mut selected = selected(ty);
    let child_sign = if child_type(ty) == "f32" {
        1 << 31
    } else {
        i64::MIN
    };
    // Negation produces actual floating child bits 96 at late fault boundaries.
    for index in [7, 8] {
        selected[index] = Case::new(ty, 0, child_sign | 96, true, true);
    }
    selected
}
