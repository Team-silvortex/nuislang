#[allow(dead_code)]
#[path = "guarded_scalar_join_cases.rs"]
mod guarded;

pub const SOURCE: &str = include_str!("computed_scalar_join.ns");
pub const LOOP_LIMIT: u64 = guarded::LOOP_LIMIT;
pub const ENTRY_LIMIT: u64 = guarded::ENTRY_LIMIT;
pub const DECISION_OBSERVER: &str = "  call void @word_print(i64 83)\n";

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: guarded::Case,
    pub condition: i64,
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.insert(4, self.condition);
        words
    }
    pub fn args(self) -> String {
        let mut args = self
            .input
            .args()
            .split(',')
            .map(str::to_owned)
            .collect::<Vec<_>>();
        args.insert(4, self.condition.to_string());
        args.join(",")
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
    }
    fn evaluate(self, previous: Option<i64>, observe: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        let input = self.input;
        let seed = previous.unwrap_or(input.total);
        for _ in 0..input.trips {
            trace.push(71);
            if input.selected {
                let value = seed.wrapping_add(input.value);
                0i64.checked_div(input.value).ok_or_else(|| trace.clone())?;
                trace.push(81);
                if observe {
                    trace.push(83);
                }
                let proof = 1i64
                    .checked_div(self.condition)
                    .ok_or_else(|| trace.clone())?;
                let decision = input.choice == (proof != 0);
                let value = if decision {
                    value.checked_div(input.yes)
                } else {
                    value.checked_rem(input.no)
                }
                .ok_or_else(|| trace.clone())?;
                trace.push(82);
                1i64.checked_div(input.tail).ok_or_else(|| trace.clone())?;
                return Ok((value, trace));
            }
            trace.push(72);
        }
        trace.push(73);
        Ok((seed, trace))
    }
}

pub fn prefix() -> Vec<Case> {
    guarded::prefix()
        .into_iter()
        .map(|input| Case {
            input,
            condition: 1,
        })
        .collect()
}

pub fn healthy() -> Vec<Case> {
    let mut rows = guarded::healthy()
        .into_iter()
        .map(|input| Case {
            condition: if input.selected && input.trips > 0 {
                1
            } else {
                0
            },
            input,
        })
        .collect::<Vec<_>>();
    let base = guarded::prefix()[2];
    rows.extend([
        Case {
            input: guarded::Case {
                choice: false,
                value: 8,
                yes: 4,
                no: 0,
                ..base
            },
            condition: 2,
        },
        Case {
            input: guarded::Case {
                choice: false,
                value: 5,
                yes: 0,
                no: 2,
                ..base
            },
            condition: -1,
        },
        Case {
            input: guarded::Case {
                choice: false,
                value: 8,
                yes: 4,
                no: 0,
                ..base
            },
            condition: i64::MIN,
        },
    ]);
    rows
}

pub fn failures() -> Vec<Case> {
    let mut rows = guarded::failures()
        .into_iter()
        .map(|input| Case {
            input,
            condition: 1,
        })
        .collect::<Vec<_>>();
    let base = guarded::prefix()[2];
    rows.extend([
        Case {
            input: guarded::Case {
                value: 8,
                yes: 0,
                no: 0,
                tail: 0,
                ..base
            },
            condition: 0,
        },
        Case {
            input: guarded::Case {
                value: 8,
                yes: 0,
                no: 1,
                ..base
            },
            condition: 2,
        },
    ]);
    rows
}
