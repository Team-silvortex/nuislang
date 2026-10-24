#[allow(dead_code)]
#[path = "nested_prefix_join_cases.rs"]
mod nested;

pub const SOURCE: &str = include_str!("record_prefix_join.ns");
pub const LOOP_LIMIT: u64 = nested::LOOP_LIMIT;
pub const ENTRY_LIMIT: u64 = nested::ENTRY_LIMIT;
pub const LAST_STATE: i64 = nested::LAST_STATE;
pub const ENTRY_MARKERS: &[i64] = &[83, 84, 85, 86, 87, 88, 89, 94, 95, 96, 97, 98];
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
    ("record_word", "  call void @word_print(i64 96)\n"),
    ("record_unused", "  call void @word_print(i64 97)\n"),
    ("record_seed", "  call void @word_print(i64 98)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: nested::Case,
    pub record_guard: i64,
}
impl Case {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.push(self.record_guard);
        words
    }
    pub fn args(self) -> String {
        format!("{},{}", self.input.args(), self.record_guard)
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
    }
    fn evaluate(self, previous: Option<i64>, observe: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let input = self.input.input.input;
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
                let child = self.input.inner
                    == (1i64
                        .checked_div(self.input.inner_div)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                if observe {
                    trace.push(if child { 88 } else { 95 });
                }
                let temp = 1i64
                    .checked_div(if condition {
                        self.input.input.yes_prefix
                    } else {
                        self.input.input.no_prefix
                    })
                    .ok_or_else(|| trace.clone())?;
                let guard = self.record_guard;
                // Construction order is source order, not canonical field order.
                let word = if child {
                    if observe {
                        trace.push(96);
                    }
                    0i64.checked_div(guard).ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(97);
                    }
                    1i64.checked_div(guard.wrapping_add(1))
                        .ok_or_else(|| trace.clone())?;
                    temp
                } else {
                    if observe {
                        trace.push(97);
                    }
                    1i64.checked_div(guard.wrapping_add(1))
                        .ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(96);
                    }
                    0i64.checked_div(guard).ok_or_else(|| trace.clone())?;
                    0i64.wrapping_sub(temp)
                };
                if observe {
                    trace.push(98);
                }
                let seed = (if child { word } else { 0i64.wrapping_sub(word) })
                    .wrapping_add(guard)
                    .wrapping_sub(1);
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
                let selected = selected && seed == temp.wrapping_add(guard).wrapping_sub(1);
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
fn wrap(input: nested::Case) -> Case {
    Case {
        input,
        record_guard: 1,
    }
}
pub fn prefix() -> Vec<Case> {
    nested::prefix().into_iter().map(wrap).collect()
}
pub fn healthy() -> Vec<Case> {
    let mut rows = nested::healthy().into_iter().map(wrap).collect::<Vec<_>>();
    for input in &nested::prefix()[1..] {
        for inner in [false, true] {
            for record_guard in [2, i64::MIN, i64::MAX] {
                rows.push(Case {
                    input: nested::Case {
                        inner,
                        inner_div: 1,
                        ..*input
                    },
                    record_guard,
                });
            }
        }
    }
    rows
}
pub fn failures() -> Vec<Case> {
    let mut rows = nested::failures().into_iter().map(wrap).collect::<Vec<_>>();
    for input in &nested::prefix()[1..] {
        for inner in [false, true] {
            for record_guard in [0, -1] {
                rows.push(Case {
                    input: nested::Case {
                        inner,
                        inner_div: 1,
                        ..*input
                    },
                    record_guard,
                });
            }
        }
    }
    rows
}
pub fn native_failures() -> Vec<Case> {
    let mut rows = failures();
    rows.extend(
        nested::native_failures()
            .into_iter()
            .skip(nested::failures().len())
            .map(wrap),
    );
    rows
}
