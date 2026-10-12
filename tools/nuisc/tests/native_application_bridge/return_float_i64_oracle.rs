pub const SOURCE: &str = include_str!("return_float_i64_private_pair.ns");
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
    pub first: bool,
    pub second: bool,
    pub owner: i64,
    pub pair: Option<(i64, i64)>,
    pub result: bool,
    pub trace: Vec<i64>,
}

impl Case {
    // Independent IEEE owner bits and full-width integer child updates never use the compiler carrier.
    pub fn new(ty: &'static str, seed: i64, first: bool, second: bool) -> Self {
        let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
        let negative = |bits| {
            if ty == "f32" {
                f32::from_bits(bits as u32) < 0.0
            } else {
                f64::from_bits(bits as u64) < 0.0
            }
        };
        let zero = |bits| {
            if ty == "f32" {
                f32::from_bits(bits as u32) == 0.0
            } else {
                f64::from_bits(bits as u64) == 0.0
            }
        };
        let initial_child = if zero(seed) {
            96
        } else if negative(seed) {
            i64::MIN
        } else {
            i64::MAX
        };
        let parent = |bits, child: i64| if child < 0 { bits ^ sign } else { bits };
        let mut trace = vec![99, 90, 87];
        let mut local = seed;
        let mut pair = None;
        if first && second {
            let mut child = initial_child;
            trace.extend([91, 92]);
            if child < 0 {
                local = parent(local, child);
                trace.push(95);
                child = !child;
                trace.push(93);
                if child == 96 {
                    child = !child;
                    trace.push(93);
                    local = parent(local, child);
                    trace.push(95);
                }
            } else {
                child = !child;
                trace.push(93);
                local = parent(local, child);
                trace.push(95);
                if child == -97 {
                    local = parent(local, child);
                    trace.push(95);
                    child = !child;
                    trace.push(93);
                }
            }
            pair = Some((local, child));
            trace.push(96);
            if child < 0 {
                local = parent(local, child);
                trace.push(95);
            }
            let _seal = child;
            trace.push(83);
        }
        trace.extend([97, 79]);
        let equal = if ty == "f32" {
            f32::from_bits(local as u32) == f32::from_bits(seed as u32)
        } else {
            f64::from_bits(local as u64) == f64::from_bits(seed as u64)
        };
        Self {
            ty,
            seed,
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

    pub fn previous(&self) -> Self {
        Self::new(self.ty, self.seed, !self.first, !self.second)
    }
    pub fn values(&self) -> Vec<String> {
        let mut args = vec![self.first.to_string(), self.second.to_string()];
        args.extend(STAGES.map(|stage| {
            if self.trace.contains(&stage) {
                "1"
            } else {
                "0"
            }
            .into()
        }));
        args
    }
    pub fn words(&self) -> Vec<i64> {
        let mut args = vec![i64::from(self.first), i64::from(self.second)];
        args.extend(STAGES.map(|stage| i64::from(self.trace.contains(&stage))));
        args
    }
    pub fn state_words(&self) -> Vec<i64> {
        std::iter::once(i64::from(self.seed))
            .chain(self.words())
            .chain([i64::from(self.result)])
            .collect()
    }
    pub fn state_text(&self, phase: &str) -> String {
        let mut fields = vec![format!("seed: {}{}", self.decimal(), self.ty)];
        fields.extend(
            FIELDS
                .into_iter()
                .zip(self.values())
                .map(|(name, value)| format!("{name}: {value}")),
        );
        fields.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", fields.join(", "))
    }
    pub fn decimal(&self) -> String {
        if self.ty == "f32" {
            f32::from_bits(self.seed as u32).to_string()
        } else {
            f64::from_bits(self.seed as u64).to_string()
        }
    }
    pub fn open_args(&self) -> String {
        format!("{},{}", self.decimal(), self.values().join(","))
    }
    pub fn bad_values(&self, stage: usize) -> Vec<String> {
        let mut values = self.values();
        assert_eq!(values[stage + 2], "1");
        values[stage + 2] = "0".into();
        values
    }
    fn observe_until(&self, count: usize) -> Vec<i64> {
        // Source-event positions cannot be confused with observed field values.
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

pub fn cases(ty: &'static str) -> Vec<Case> {
    words(ty)
        .into_iter()
        .flat_map(|seed| {
            [false, true].into_iter().flat_map(move |first| {
                [false, true]
                    .into_iter()
                    .map(move |second| Case::new(ty, seed, first, second))
            })
        })
        .collect()
}
pub fn cli_cases(ty: &'static str) -> Vec<Case> {
    cases(ty)
        .into_iter()
        .filter(|case| {
            [-2.5, -0.0, 0.0, 1.5].into_iter().any(|value: f64| {
                let bits = if ty == "f32" {
                    i64::from((value as f32).to_bits())
                } else {
                    value.to_bits() as i64
                };
                bits == case.seed
            })
        })
        .collect()
}
pub fn selected(ty: &'static str) -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|stage| {
            // Zero seeds carry private integer marker 96 into real late failures.
            if stage == 83 || stage == 97 {
                Case::new(ty, 0, true, true)
            } else {
                cli_cases(ty)
                    .into_iter()
                    .find(|case| case.trace.contains(&stage))
                    .unwrap()
            }
        })
        .collect()
}
