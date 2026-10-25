#[allow(dead_code)]
#[path = "discarded_record_prefix_cases.rs"]
mod prior;
pub const SOURCE: &str = include_str!("outer_record_prefix.ns");
pub const LOOP_LIMIT: u64 = prior::LOOP_LIMIT;
pub const ENTRY_LIMIT: u64 = prior::ENTRY_LIMIT;
pub const LAST_STATE: i64 = prior::LAST_STATE;
pub const ENTRY_MARKERS: &[i64] = &[94, 96, 97, 98, 99, 100];
pub const OBSERVERS: &[(&str, &str)] = &[
    ("discarded_decide", "  call void @word_print(i64 94)\n"),
    ("discarded_first", "  call void @word_print(i64 96)\n"),
    ("discarded_last", "  call void @word_print(i64 97)\n"),
    ("outer_word", "  call void @word_print(i64 98)\n"),
    ("outer_unused", "  call void @word_print(i64 99)\n"),
    ("outer_result", "  call void @word_print(i64 100)\n"),
];
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: prior::Case,
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
        let input = self.input;
        let state = previous.unwrap_or(input.total);
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        for _ in 0..input.trips {
            trace.push(71);
            if input.selected {
                let value = state.wrapping_add(input.value);
                0i64.checked_div(input.value).ok_or_else(|| trace.clone())?;
                trace.push(81);
                if observe {
                    trace.push(94);
                }
                let retained = input.inner
                    == (1i64
                        .checked_div(input.condition)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                if input.outer && retained {
                    if observe {
                        trace.push(96);
                    }
                    1i64.checked_div(input.first).ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(97);
                    }
                    1i64.checked_div(input.last).ok_or_else(|| trace.clone())?;
                } else if !input.outer && !retained {
                    if observe {
                        trace.push(97);
                    }
                    1i64.checked_div(input.last).ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(96);
                    }
                    1i64.checked_div(input.first).ok_or_else(|| trace.clone())?;
                }
                let guard = self.record_guard;
                if input.outer {
                    if observe {
                        trace.push(98);
                    }
                    0i64.checked_div(guard).ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(99);
                    }
                    1i64.checked_div(guard.wrapping_add(1))
                        .ok_or_else(|| trace.clone())?;
                } else {
                    if observe {
                        trace.push(99);
                    }
                    1i64.checked_div(guard.wrapping_add(1))
                        .ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(98);
                    }
                    0i64.checked_div(guard).ok_or_else(|| trace.clone())?;
                }
                let picked = if input.outer {
                    value
                } else {
                    0i64.wrapping_sub(value)
                };
                trace.push(82);
                if observe {
                    trace.push(100);
                }
                1i64.checked_div(input.tail).ok_or_else(|| trace.clone())?;
                return Ok((picked, trace));
            }
            trace.push(72);
        }
        trace.push(73);
        Ok((state, trace))
    }
}
fn wrap(input: prior::Case) -> Case {
    Case {
        input,
        record_guard: 1,
    }
}
pub fn prefix() -> Vec<Case> {
    prior::prefix().into_iter().map(wrap).collect()
}
pub fn healthy() -> Vec<Case> {
    let mut rows = prior::healthy().into_iter().map(wrap).collect::<Vec<_>>();
    for input in &prior::prefix()[1..] {
        for record_guard in [2, i64::MIN, i64::MAX] {
            rows.push(Case {
                input: *input,
                record_guard,
            });
        }
    }
    for record_guard in [0, -1] {
        rows.push(Case {
            input: prior::Case {
                selected: false,
                condition: 0,
                first: 0,
                last: 0,
                value: 0,
                tail: 0,
                ..prior::prefix()[1]
            },
            record_guard,
        });
    }
    rows
}
pub fn failures() -> Vec<Case> {
    let mut rows = prior::failures().into_iter().map(wrap).collect::<Vec<_>>();
    for input in &prior::prefix()[1..] {
        for record_guard in [0, -1] {
            rows.push(Case {
                input: *input,
                record_guard,
            });
        }
    }
    rows
}
pub fn native_failures() -> Vec<Case> {
    failures()
}
