pub const SOURCE: &str = include_str!("discarded_record_prefix.ns");
pub const LOOP_LIMIT: u64 = 4;
pub const ENTRY_LIMIT: u64 = 64;
pub const LAST_STATE: i64 = -2;
pub const ENTRY_MARKERS: &[i64] = &[94, 96, 97];
pub const OBSERVERS: &[(&str, &str)] = &[
    ("discarded_decide", "  call void @word_print(i64 94)\n"),
    ("discarded_first", "  call void @word_print(i64 96)\n"),
    ("discarded_last", "  call void @word_print(i64 97)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub total: i64,
    pub selected: bool,
    pub trips: i64,
    pub outer: bool,
    pub inner: bool,
    pub condition: i64,
    pub first: i64,
    pub last: i64,
    pub value: i64,
    pub tail: i64,
}
impl Case {
    pub fn words(self) -> Vec<i64> {
        vec![
            self.total,
            self.selected as i64,
            self.trips,
            self.outer as i64,
            self.inner as i64,
            self.condition,
            self.first,
            self.last,
            self.value,
            self.tail,
        ]
    }
    pub fn args(self) -> String {
        self.words()
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                if matches!(i, 1 | 3 | 4) {
                    (v != 0).to_string()
                } else {
                    v.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
    }
    fn evaluate(self, previous: Option<i64>, observe: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let state = previous.unwrap_or(self.total);
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        for _ in 0..self.trips {
            trace.push(71);
            if self.selected {
                let value = state.wrapping_add(self.value);
                0i64.checked_div(self.value).ok_or_else(|| trace.clone())?;
                trace.push(81);
                if observe {
                    trace.push(94);
                }
                let retained = self.inner
                    == (1i64
                        .checked_div(self.condition)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                if self.outer && retained {
                    if observe {
                        trace.push(96);
                    }
                    1i64.checked_div(self.first).ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(97);
                    }
                    1i64.checked_div(self.last).ok_or_else(|| trace.clone())?;
                } else if !self.outer && !retained {
                    if observe {
                        trace.push(97);
                    }
                    1i64.checked_div(self.last).ok_or_else(|| trace.clone())?;
                    if observe {
                        trace.push(96);
                    }
                    1i64.checked_div(self.first).ok_or_else(|| trace.clone())?;
                }
                let picked = if self.outer {
                    value
                } else {
                    0i64.wrapping_sub(value)
                };
                trace.push(82);
                1i64.checked_div(self.tail).ok_or_else(|| trace.clone())?;
                return Ok((picked, trace));
            }
            trace.push(72);
        }
        trace.push(73);
        Ok((state, trace))
    }
}
pub fn prefix() -> Vec<Case> {
    let open = Case {
        total: 64,
        selected: false,
        trips: 0,
        outer: false,
        inner: false,
        condition: 0,
        first: 0,
        last: 0,
        value: 0,
        tail: 0,
    };
    vec![
        open,
        Case {
            total: 999,
            selected: true,
            trips: 2,
            outer: true,
            inner: true,
            condition: 1,
            first: 1,
            last: 1,
            value: 8,
            tail: 1,
        },
        Case {
            total: 999,
            selected: true,
            trips: 2,
            outer: false,
            inner: false,
            condition: 1,
            first: 1,
            last: 1,
            value: -70,
            tail: 1,
        },
    ]
}
pub fn healthy() -> Vec<Case> {
    let mut rows = prefix();
    for base in &prefix()[1..] {
        rows.push(Case {
            inner: !base.inner,
            first: 0,
            last: 0,
            ..*base
        });
        rows.push(Case {
            inner: !base.inner,
            condition: 2,
            ..*base
        });
        rows.push(Case {
            selected: false,
            condition: 0,
            first: 0,
            last: 0,
            value: 0,
            tail: 0,
            ..*base
        });
        rows.push(Case {
            trips: 0,
            condition: 0,
            first: 0,
            last: 0,
            value: 0,
            tail: 0,
            ..*base
        });
        for value in [i64::MIN, i64::MAX] {
            rows.push(Case { value, ..*base });
        }
    }
    rows
}
pub fn failures() -> Vec<Case> {
    let mut rows = Vec::new();
    for base in &prefix()[1..] {
        rows.push(Case {
            condition: 0,
            first: 0,
            last: 0,
            tail: 0,
            ..*base
        });
        for (first, last) in [(0, 0), (0, 1), (1, 0)] {
            rows.push(Case {
                first,
                last,
                tail: 0,
                ..*base
            });
        }
        rows.push(Case { tail: 0, ..*base });
        rows.push(Case {
            value: 0,
            condition: 0,
            first: 0,
            last: 0,
            tail: 0,
            ..*base
        });
    }
    rows
}
pub fn native_failures() -> Vec<Case> {
    failures()
}
