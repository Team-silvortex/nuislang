#[allow(dead_code)]
#[path = "joined_record_return_cases.rs"]
mod prior;
pub const SOURCE: &str = include_str!("local_record_return.ns");
pub const LOOP_LIMIT: u64 = prior::LOOP_LIMIT;
pub const ENTRY_LIMIT: u64 = prior::ENTRY_LIMIT;
pub const LAST_STATE: i64 = prior::LAST_STATE;
pub const ENTRY_MARKERS: &[i64] = &[94, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107];
pub const OBSERVERS: &[(&str, &str)] = &[
    ("discarded_decide", "  call void @word_print(i64 94)\n"),
    ("discarded_first", "  call void @word_print(i64 96)\n"),
    ("discarded_last", "  call void @word_print(i64 97)\n"),
    ("outer_word", "  call void @word_print(i64 98)\n"),
    ("outer_unused", "  call void @word_print(i64 99)\n"),
    ("outer_result", "  call void @word_print(i64 100)\n"),
    ("field_head", "  call void @word_print(i64 101)\n"),
    ("field_tail", "  call void @word_print(i64 102)\n"),
    ("field_spare", "  call void @word_print(i64 103)\n"),
    ("return_prefix", "  call void @word_print(i64 104)\n"),
    ("join_decide", "  call void @word_print(i64 105)\n"),
    ("join_unused", "  call void @word_print(i64 106)\n"),
    ("local_after", "  call void @word_print(i64 107)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: prior::Case,
    pub post_div: i64,
}
impl Case {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.push(self.post_div);
        words
    }
    pub fn args(self) -> String {
        format!("{},{}", self.input.args(), self.post_div)
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
    }
    fn bit(self, trace: &mut Vec<i64>, observe: bool) -> Result<bool, Vec<i64>> {
        if observe {
            trace.push(101);
        }
        let head = 1i64
            .checked_div(self.input.input.input.head)
            .ok_or_else(|| trace.clone())?
            != 0;
        if (self.input.input.input.input.input.outer && !head)
            || (!self.input.input.input.input.input.outer && head)
        {
            return Ok(head);
        }
        if observe {
            trace.push(102);
        }
        Ok(1i64
            .checked_div(self.input.input.input.rhs)
            .ok_or_else(|| trace.clone())?
            != 0)
    }
    fn spare(self, trace: &mut Vec<i64>, observe: bool) -> Result<(), Vec<i64>> {
        if self.input.input.input.spare {
            if observe {
                trace.push(103);
            }
            1i64.checked_div(self.input.input.input.spare_div)
                .ok_or_else(|| trace.clone())?;
        }
        Ok(())
    }
    fn evaluate(self, previous: Option<i64>, observe: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let input = self.input.input.input.input.input;
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
                trace.push(86);
                if observe {
                    trace.push(105);
                }
                let joined = input.outer
                    == (1i64
                        .checked_div(self.input.join_div)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                if observe {
                    trace.push(104);
                }
                0i64.checked_div(self.input.input.prefix_div)
                    .ok_or_else(|| trace.clone())?;
                if observe {
                    trace.push(106);
                }
                let divisor = if joined {
                    self.input.join_tail
                } else {
                    self.input.join_tail.wrapping_add(1)
                };
                1i64.checked_div(divisor).ok_or_else(|| trace.clone())?;
                trace.push(89);
                trace.push(if input.outer { 87 } else { 88 });
                let guard = self.input.input.input.input.record_guard;
                let bit = if input.outer {
                    if observe {
                        trace.push(98);
                    }
                    0i64.checked_div(guard).ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(99);
                    }
                    1i64.checked_div(guard.wrapping_add(1))
                        .ok_or_else(|| trace.clone())?;
                    let bit = self.bit(&mut trace, observe)?;
                    self.spare(&mut trace, observe)?;
                    bit
                } else {
                    self.spare(&mut trace, observe)?;
                    let bit = self.bit(&mut trace, observe)?;
                    if observe {
                        trace.push(99);
                    }
                    1i64.checked_div(guard.wrapping_add(1))
                        .ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(98);
                    }
                    0i64.checked_div(guard).ok_or_else(|| trace.clone())?;
                    bit
                };
                trace.push(83);
                if observe {
                    trace.push(107);
                }
                1i64.checked_div(self.post_div)
                    .ok_or_else(|| trace.clone())?;
                trace.push(84);
                let picked = if bit != input.outer {
                    0
                } else if input.outer {
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
    Case { input, post_div: 1 }
}
pub fn prefix() -> Vec<Case> {
    prior::prefix().into_iter().map(wrap).collect()
}
pub fn healthy() -> Vec<Case> {
    let mut rows = prior::healthy().into_iter().map(wrap).collect::<Vec<_>>();
    for input in &prior::prefix()[1..] {
        for post_div in [i64::MIN, i64::MAX, -1] {
            rows.push(Case {
                input: *input,
                post_div,
            });
        }
    }
    rows.push(Case {
        input: prior::prefix()[0],
        post_div: 0,
    });
    rows
}
fn post_failures() -> Vec<Case> {
    prior::prefix()[1..]
        .iter()
        .map(|base| Case {
            input: *base,
            post_div: 0,
        })
        .collect()
}
pub fn failures() -> Vec<Case> {
    let mut rows = prior::failures().into_iter().map(wrap).collect::<Vec<_>>();
    rows.extend(post_failures());
    rows
}
pub fn native_failures() -> Vec<Case> {
    let mut rows = prior::native_failures()
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows.extend(post_failures());
    rows
}
