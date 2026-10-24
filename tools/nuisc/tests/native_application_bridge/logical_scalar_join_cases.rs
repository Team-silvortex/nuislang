#[allow(dead_code)]
#[path = "guarded_scalar_join_cases.rs"]
mod guarded;

pub const SOURCE: &str = include_str!("logical_scalar_join.ns");
pub const LOOP_LIMIT: u64 = guarded::LOOP_LIMIT;
pub const ENTRY_LIMIT: u64 = guarded::ENTRY_LIMIT;
pub const OBSERVERS: &[(&str, &str)] = &[
    ("decide", "  call void @word_print(i64 83)\n"),
    ("right", "  call void @word_print(i64 84)\n"),
    ("fallback", "  call void @word_print(i64 85)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: guarded::Case,
    pub condition: i64,
    pub rhs: i64,
    pub other: i64,
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.splice(4..4, [self.condition, self.rhs, self.other]);
        words
    }
    pub fn args(self) -> String {
        let mut args = self
            .input
            .args()
            .split(',')
            .map(str::to_owned)
            .collect::<Vec<_>>();
        args.splice(
            4..4,
            [self.condition, self.rhs, self.other].map(|v| v.to_string()),
        );
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
                let lhs = input.choice
                    == (1i64
                        .checked_div(self.condition)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                let conjunction = if lhs {
                    if observe {
                        trace.push(84);
                    }
                    1i64.checked_div(self.rhs).ok_or_else(|| trace.clone())? != 0
                } else {
                    false
                };
                let decision = if conjunction {
                    true
                } else {
                    if observe {
                        trace.push(85);
                    }
                    1i64.checked_div(self.other).ok_or_else(|| trace.clone())? != 0
                };
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

fn wrap(input: guarded::Case) -> Case {
    Case {
        condition: if input.selected && input.trips > 0 {
            1
        } else {
            0
        },
        rhs: if input.choice { 1 } else { 0 },
        other: if input.choice { 0 } else { 2 },
        input,
    }
}

pub fn prefix() -> Vec<Case> {
    guarded::prefix().into_iter().map(wrap).collect()
}

pub fn healthy() -> Vec<Case> {
    let mut rows = guarded::healthy().into_iter().map(wrap).collect::<Vec<_>>();
    let base = guarded::prefix()[2];
    rows.extend([
        Case {
            input: guarded::Case {
                choice: true,
                value: 8,
                yes: 4,
                no: 0,
                ..base
            },
            condition: 1,
            rhs: 2,
            other: 1,
        },
        Case {
            input: guarded::Case {
                choice: true,
                value: 5,
                yes: 0,
                no: 2,
                ..base
            },
            condition: 1,
            rhs: i64::MIN,
            other: 2,
        },
        Case {
            input: guarded::Case {
                choice: false,
                value: 8,
                yes: 4,
                no: 0,
                ..base
            },
            condition: 2,
            rhs: -1,
            other: 0,
        },
        Case {
            input: guarded::Case {
                choice: false,
                value: 6,
                yes: 3,
                no: 0,
                ..base
            },
            condition: -1,
            rhs: 0,
            other: -1,
        },
    ]);
    rows
}

pub fn failures() -> Vec<Case> {
    let mut rows = guarded::failures()
        .into_iter()
        .map(wrap)
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
            rhs: 0,
            other: 0,
        },
        Case {
            input: guarded::Case {
                choice: true,
                value: 8,
                yes: 0,
                no: 0,
                tail: 0,
                ..base
            },
            condition: 1,
            rhs: 0,
            other: 0,
        },
        Case {
            input: guarded::Case {
                value: 8,
                yes: 0,
                no: 0,
                tail: 0,
                ..base
            },
            condition: 1,
            rhs: 0,
            other: 0,
        },
        Case {
            input: guarded::Case {
                choice: true,
                value: 8,
                yes: 0,
                no: 0,
                tail: 0,
                ..base
            },
            condition: 1,
            rhs: 2,
            other: 0,
        },
        Case {
            input: guarded::Case {
                value: 8,
                yes: 0,
                ..base
            },
            condition: 1,
            rhs: 0,
            other: 1,
        },
    ]);
    rows
}
