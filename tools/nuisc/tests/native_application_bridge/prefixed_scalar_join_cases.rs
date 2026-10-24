#[allow(dead_code)]
#[path = "logical_value_join_cases.rs"]
mod values;

pub const SOURCE: &str = include_str!("prefixed_scalar_join.ns");
pub const LOOP_LIMIT: u64 = values::LOOP_LIMIT;
pub const ENTRY_LIMIT: u64 = values::ENTRY_LIMIT;
pub const LAST_STATE: i64 = values::LAST_STATE;
pub const OBSERVERS: &[(&str, &str)] = &[
    ("decide", "  call void @word_print(i64 83)\n"),
    ("yes_head", "  call void @word_print(i64 84)\n"),
    ("yes_tail", "  call void @word_print(i64 85)\n"),
    ("no_head", "  call void @word_print(i64 86)\n"),
    ("no_tail", "  call void @word_print(i64 87)\n"),
    ("prefix_seed", "  call void @word_print(i64 88)\n"),
    ("prefix_step", "  call void @word_print(i64 89)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: values::Case,
    pub yes_prefix: i64,
    pub no_prefix: i64,
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.extend([self.yes_prefix, self.no_prefix]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{}",
            self.input.args(),
            self.yes_prefix,
            self.no_prefix
        )
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
    }
    fn evaluate(self, previous: Option<i64>, observe: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let input = self.input;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        let state = previous.unwrap_or(input.total);
        for _ in 0..input.trips {
            trace.push(71);
            if input.selected {
                let value = state.wrapping_add(input.value);
                0i64.checked_div(input.value).ok_or_else(|| trace.clone())?;
                trace.push(81);
                if observe {
                    trace.push(83);
                }
                let condition = input.choice
                    == (1i64
                        .checked_div(input.condition)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                if observe {
                    trace.push(88);
                }
                let seed = 1i64
                    .checked_div(if condition {
                        self.yes_prefix
                    } else {
                        self.no_prefix
                    })
                    .ok_or_else(|| trace.clone())?;
                if observe {
                    trace.push(89);
                }
                let prepared = (if condition { input.yes } else { input.no })
                    .wrapping_add(0i64.checked_div(seed).ok_or_else(|| trace.clone())?);
                if observe {
                    trace.push(if condition { 84 } else { 86 });
                }
                let head = 1i64.checked_div(prepared).ok_or_else(|| trace.clone())? != 0;
                1i64.checked_div(seed.wrapping_add(1))
                    .ok_or_else(|| trace.clone())?;
                let selected = if condition {
                    if head {
                        if observe {
                            trace.push(85);
                        }
                        1i64.checked_div(input.rhs).ok_or_else(|| trace.clone())? != 0
                    } else {
                        false
                    }
                } else if head {
                    true
                } else {
                    if observe {
                        trace.push(87);
                    }
                    1i64.checked_div(input.other).ok_or_else(|| trace.clone())? != 0
                };
                trace.push(82);
                1i64.checked_div(input.tail).ok_or_else(|| trace.clone())?;
                return Ok((if selected { value } else { 0 }, trace));
            }
            trace.push(72);
        }
        trace.push(73);
        Ok((state, trace))
    }
}

fn wrap(input: values::Case) -> Case {
    let selected = if input.selected && input.trips > 0 {
        1i64.checked_div(input.condition)
            .map(|v| input.choice == (v != 0))
    } else {
        None
    };
    Case {
        input,
        yes_prefix: i64::from(selected == Some(true)),
        no_prefix: i64::from(selected == Some(false)),
    }
}

pub fn prefix() -> Vec<Case> {
    values::prefix().into_iter().map(wrap).collect()
}
pub fn healthy() -> Vec<Case> {
    values::healthy().into_iter().map(wrap).collect()
}

pub fn failures() -> Vec<Case> {
    let mut rows = values::failures().into_iter().map(wrap).collect::<Vec<_>>();
    let base = values::prefix()[2];
    let yes = values::Case {
        choice: true,
        value: 8,
        yes: 2,
        rhs: 0,
        no: 0,
        other: 0,
        tail: 0,
        ..base
    };
    let no = values::Case {
        choice: false,
        value: 8,
        yes: 0,
        rhs: 0,
        no: 1,
        other: 0,
        tail: 0,
        ..base
    };
    for divisor in [0, 2, -1] {
        rows.push(Case {
            input: yes,
            yes_prefix: divisor,
            no_prefix: 0,
        });
        rows.push(Case {
            input: no,
            yes_prefix: 0,
            no_prefix: divisor,
        });
    }
    rows
}

pub fn native_failures() -> Vec<Case> {
    let mut rows = failures();
    let base = values::prefix()[2];
    // Unlike the short-circuit prefix faults, these RHS calls would run if
    // the unused initializer were incorrectly moved after the final value.
    rows.extend([
        Case {
            input: values::Case {
                choice: true,
                yes: 1,
                rhs: 1,
                tail: 0,
                ..base
            },
            yes_prefix: -1,
            no_prefix: 0,
        },
        Case {
            input: values::Case {
                choice: false,
                no: 2,
                other: 1,
                tail: 0,
                ..base
            },
            yes_prefix: 0,
            no_prefix: -1,
        },
    ]);
    rows
}
