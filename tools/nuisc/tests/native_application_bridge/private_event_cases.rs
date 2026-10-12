//! Shared test inputs and an independent state/order oracle, not source authority.
#[path = "return_bool_pair_oracle.rs"]
mod bool_oracle;
#[path = "return_float_pair_oracle.rs"]
#[allow(dead_code)] // Raw-word probe helpers are used by the dedicated float tests.
mod float_oracle;
#[path = "return_i32_pair_oracle.rs"]
mod i32_oracle;
#[path = "return_i64_pair_oracle.rs"]
mod i64_oracle;

#[derive(Clone, Copy, Debug)]
pub enum Family {
    Bool,
    I32,
    I64,
    F32,
    F64,
}

const NUMERIC_FIELDS: &[&str] = &[
    "first",
    "second",
    "outer",
    "init",
    "child_init",
    "cond",
    "child_rhs",
    "parent_rhs",
    "after",
    "unused",
    "answer",
];
const BOOL_FIELDS: &[&str] = &[
    "first",
    "second",
    "reply",
    "outer",
    "init",
    "child_init",
    "cond",
    "child_rhs",
    "parent_rhs",
    "after",
    "unused",
    "answer",
    "inner_cond",
    "inner_child",
    "inner_parent",
];
const NUMERIC_STAGES: &[i32] = &[90, 87, 91, 92, 93, 95, 96, 83, 97];

#[derive(Clone, Debug)]
pub struct Case {
    pub family: Family,
    pub seed_word: u64,
    pub seed: String,
    pub flags: Vec<bool>,
    pub values: Vec<String>,
    pub result: bool,
    pub trace: Vec<i32>,
}

impl Family {
    pub fn source(self) -> String {
        match self {
            Self::Bool => include_str!(
                "return_nested_sequential_mixed_logical_effectful_scalar_selections.ns"
            )
            .into(),
            Self::I32 => {
                include_str!("return_i32_pair_mixed_logical_effectful_scalar_selections.ns").into()
            }
            Self::I64 => {
                include_str!("return_i64_pair_mixed_logical_effectful_scalar_selections.ns").into()
            }
            Self::F32 | Self::F64 => {
                include_str!("return_float_pair_mixed_logical_effectful_scalar_selections.ns")
                    .replace("f64", self.name())
            }
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }

    pub fn stages(self) -> &'static [i32] {
        if matches!(self, Self::Bool) {
            &bool_oracle::STAGES
        } else {
            NUMERIC_STAGES
        }
    }

    fn seeds(self) -> Vec<u64> {
        match self {
            Self::Bool => vec![0, 1],
            Self::I32 => [i32::MIN as i64, -1, 0, 1, i32::MAX as i64]
                .map(|v| v as u64)
                .to_vec(),
            Self::I64 => [i64::MIN, -4_294_967_297, -1, 0, 1, 4_294_967_296, i64::MAX]
                .map(|v| v as u64)
                .to_vec(),
            Self::F32 => [-2.5_f32, -0.0, 0.0, 1.5]
                .map(|v| u64::from(v.to_bits()))
                .to_vec(),
            Self::F64 => [-2.5_f64, -0.0, 0.0, 1.5].map(f64::to_bits).to_vec(),
        }
    }

    fn case(self, seed_word: u64, flags: Vec<bool>) -> Case {
        let (seed, result, trace) = match self {
            Self::Bool => {
                let (result, trace) =
                    bool_oracle::oracle(seed_word != 0, flags[0], flags[1], flags[2]);
                ((seed_word != 0).to_string(), result, trace)
            }
            Self::I32 | Self::I64 => {
                let (result, trace) = if matches!(self, Self::I32) {
                    i32_oracle::oracle(seed_word as i64, flags[0], flags[1])
                } else {
                    i64_oracle::oracle(seed_word as i64, flags[0], flags[1])
                };
                ((seed_word as i64).to_string(), result, trace)
            }
            Self::F32 | Self::F64 => {
                let outcome = float_oracle::oracle(self.name(), seed_word, flags[0], flags[1]);
                // Decimal cases are exact at both widths; raw NaN payload coverage
                // remains in return_float_pair, not this CLI parser proof.
                let _ = (outcome.child, float_oracle::STAGES);
                let seed = if matches!(self, Self::F32) {
                    f32::from_bits(seed_word as u32).to_string()
                } else {
                    f64::from_bits(seed_word).to_string()
                };
                (seed, outcome.result, outcome.trace)
            }
        };
        let mut values = flags.iter().map(bool::to_string).collect::<Vec<_>>();
        values.extend(
            self.stages()
                .iter()
                .map(|stage| if trace.contains(stage) { "1" } else { "0" }.into()),
        );
        Case {
            family: self,
            seed_word,
            seed,
            flags,
            values,
            result,
            trace,
        }
    }

    pub fn cases(self) -> Vec<Case> {
        let flags = if matches!(self, Self::Bool) { 3 } else { 2 };
        self.seeds()
            .into_iter()
            .flat_map(|seed| {
                (0..(1 << flags)).map(move |bits| {
                    self.case(seed, (0..flags).map(|bit| bits & (1 << bit) != 0).collect())
                })
            })
            .collect()
    }

    pub fn selected(self) -> Vec<Case> {
        let cases = self.cases();
        self.stages()
            .iter()
            .map(|stage| {
                cases
                    .iter()
                    .find(|case| case.trace.contains(stage))
                    .unwrap()
                    .clone()
            })
            .collect()
    }
}

impl Case {
    pub fn previous(&self) -> Self {
        self.family
            .case(self.seed_word, self.flags.iter().map(|v| !v).collect())
    }

    pub fn bad_values(&self, stage_index: usize) -> Vec<String> {
        let mut values = self.values.clone();
        let slot = self.flags.len() + stage_index;
        assert_eq!(values[slot], "1", "selected stage must actually execute");
        values[slot] = "0".into();
        values
    }

    pub fn open_args(&self) -> String {
        format!("{},{}", self.seed, self.values.join(","))
    }

    pub fn event_words(&self, values: &[String]) -> Vec<u64> {
        values
            .iter()
            .enumerate()
            .map(|(slot, value)| {
                if slot < self.flags.len() {
                    u64::from(value == "true")
                } else {
                    value.parse::<i64>().unwrap() as u64
                }
            })
            .collect()
    }

    pub fn open_words(&self) -> Vec<u64> {
        std::iter::once(self.seed_word)
            .chain(self.event_words(&self.values))
            .collect()
    }

    pub fn state_words(&self) -> Vec<u64> {
        self.open_words()
            .into_iter()
            .chain([u64::from(self.result)])
            .collect()
    }

    pub fn state_text(&self, phase: &str) -> String {
        let seed = match self.family {
            Family::F32 | Family::F64 => format!("{}{}", self.seed, self.family.name()),
            _ => self.seed.clone(),
        };
        let fields = if matches!(self.family, Family::Bool) {
            BOOL_FIELDS
        } else {
            NUMERIC_FIELDS
        };
        let mut values = vec![format!("seed: {seed}")];
        values.extend(
            fields
                .iter()
                .zip(&self.values)
                .map(|(name, value)| format!("{name}: {value}")),
        );
        values.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", values.join(", "))
    }
}
