#[allow(dead_code)]
#[path = "prefixed_scalar_join_cases.rs"]
mod prefix;

pub const SOURCE: &str = include_str!("nested_prefix_join.ns");
pub const LOOP_LIMIT: u64 = prefix::LOOP_LIMIT;
pub const ENTRY_LIMIT: u64 = prefix::ENTRY_LIMIT;
pub const LAST_STATE: i64 = prefix::LAST_STATE;
pub const ENTRY_MARKERS: &[i64] = &[83, 84, 85, 86, 87, 88, 89, 94, 95];
pub const OBSERVERS: &[(&str, &str)] = &[
    ("decide", "  call void @word_print(i64 83)\n"),
    ("yes_head", "  call void @word_print(i64 84)\n"),
    ("yes_tail", "  call void @word_print(i64 85)\n"),
    ("no_head", "  call void @word_print(i64 86)\n"),
    ("no_tail", "  call void @word_print(i64 87)\n"),
    ("prefix_seed", "  call void @word_print(i64 88)\n"),
    ("prefix_step", "  call void @word_print(i64 89)\n"),
    ("nested_decide", "  call void @word_print(i64 94)\n"),
    ("prefix_alt", "  call void @word_print(i64 95)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: prefix::Case,
    pub inner: bool,
    pub inner_div: i64,
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.extend([self.inner as i64, self.inner_div]);
        words
    }
    pub fn args(self) -> String {
        format!("{},{},{}", self.input.args(), self.inner, self.inner_div)
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
    }
    fn evaluate(self, previous: Option<i64>, observe: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let input = self.input.input;
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
                    trace.push(94);
                }
                let nested = self.inner
                    == (1i64
                        .checked_div(self.inner_div)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                if observe {
                    trace.push(if nested { 88 } else { 95 });
                }
                let seed = 1i64
                    .checked_div(if condition {
                        self.input.yes_prefix
                    } else {
                        self.input.no_prefix
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

fn wrap(input: prefix::Case) -> Case {
    let reached = input.input.selected && input.input.trips > 0;
    Case {
        input,
        inner: true,
        inner_div: if reached { 1 } else { 0 },
    }
}
pub fn prefix() -> Vec<Case> {
    prefix::prefix().into_iter().map(wrap).collect()
}
pub fn healthy() -> Vec<Case> {
    let mut rows = prefix::healthy().into_iter().map(wrap).collect::<Vec<_>>();
    let base = prefix::prefix();
    for input in [base[1], base[2]] {
        rows.push(Case {
            input,
            inner: false,
            inner_div: 1,
        });
        rows.push(Case {
            input,
            inner: true,
            inner_div: 2,
        });
    }
    rows
}
pub fn failures() -> Vec<Case> {
    let prior = prefix::failures();
    let mut rows = prior.iter().copied().map(wrap).collect::<Vec<_>>();
    for input in &prefix::prefix()[1..] {
        rows.push(Case {
            input: *input,
            inner: true,
            inner_div: 0,
        });
    }
    rows.extend(prior[10..].iter().map(|input| Case {
        input: *input,
        inner: false,
        inner_div: 1,
    }));
    rows
}
pub fn native_failures() -> Vec<Case> {
    let mut rows = failures();
    for input in prefix::native_failures()
        .into_iter()
        .skip(prefix::failures().len())
    {
        rows.push(wrap(input));
        rows.push(Case {
            input,
            inner: false,
            inner_div: 1,
        });
    }
    rows
}
