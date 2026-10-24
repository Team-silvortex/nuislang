pub const SOURCE: &str = include_str!("logical_value_join.ns");
pub const LOOP_LIMIT: u64 = 4;
pub const ENTRY_LIMIT: u64 = 64;
pub const LAST_STATE: i64 = -2;
pub const OBSERVERS: &[(&str, &str)] = &[
    ("decide", "  call void @word_print(i64 83)\n"),
    ("yes_head", "  call void @word_print(i64 84)\n"),
    ("yes_tail", "  call void @word_print(i64 85)\n"),
    ("no_head", "  call void @word_print(i64 86)\n"),
    ("no_tail", "  call void @word_print(i64 87)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub total: i64,
    pub selected: bool,
    pub trips: i64,
    pub choice: bool,
    pub condition: i64,
    pub rhs: i64,
    pub other: i64,
    pub value: i64,
    pub yes: i64,
    pub no: i64,
    pub tail: i64,
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        vec![
            self.total,
            self.selected as i64,
            self.trips,
            self.choice as i64,
            self.condition,
            self.rhs,
            self.other,
            self.value,
            self.yes,
            self.no,
            self.tail,
        ]
    }
    pub fn args(self) -> String {
        self.words()
            .into_iter()
            .enumerate()
            .map(|(slot, word)| {
                if matches!(slot, 1 | 3) {
                    (word != 0).to_string()
                } else {
                    word.to_string()
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
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        let seed = previous.unwrap_or(self.total);
        for _ in 0..self.trips {
            trace.push(71);
            if self.selected {
                let value = seed.wrapping_add(self.value);
                0i64.checked_div(self.value).ok_or_else(|| trace.clone())?;
                trace.push(81);
                if observe {
                    trace.push(83);
                }
                let condition = self.choice
                    == (1i64
                        .checked_div(self.condition)
                        .ok_or_else(|| trace.clone())?
                        != 0);
                let selected = if condition {
                    if observe {
                        trace.push(84);
                    }
                    let head = 1i64.checked_div(self.yes).ok_or_else(|| trace.clone())? != 0;
                    if head {
                        if observe {
                            trace.push(85);
                        }
                        1i64.checked_div(self.rhs).ok_or_else(|| trace.clone())? != 0
                    } else {
                        false
                    }
                } else {
                    if observe {
                        trace.push(86);
                    }
                    let head = 1i64.checked_div(self.no).ok_or_else(|| trace.clone())? != 0;
                    if head {
                        true
                    } else {
                        if observe {
                            trace.push(87);
                        }
                        1i64.checked_div(self.other).ok_or_else(|| trace.clone())? != 0
                    }
                };
                trace.push(82);
                1i64.checked_div(self.tail).ok_or_else(|| trace.clone())?;
                return Ok((if selected { value } else { 0 }, trace));
            }
            trace.push(72);
        }
        trace.push(73);
        Ok((seed, trace))
    }
}

pub fn prefix() -> Vec<Case> {
    let open = Case {
        total: 64,
        selected: false,
        trips: 0,
        choice: false,
        condition: 0,
        rhs: 0,
        other: 0,
        value: 0,
        yes: 0,
        no: 0,
        tail: 0,
    };
    vec![
        open,
        Case {
            total: 999,
            selected: true,
            trips: 2,
            choice: true,
            condition: 1,
            rhs: 1,
            value: 8,
            yes: 1,
            tail: 1,
            ..open
        },
        Case {
            total: 999,
            selected: true,
            trips: 2,
            choice: false,
            condition: 1,
            other: 1,
            value: -74,
            no: 2,
            tail: 1,
            ..open
        },
    ]
}

pub fn healthy() -> Vec<Case> {
    let mut rows = prefix();
    let base = rows[2];
    rows.extend([
        Case {
            selected: false,
            condition: 0,
            no: 0,
            other: 0,
            tail: 0,
            ..base
        },
        Case {
            trips: 0,
            condition: 0,
            no: 0,
            other: 0,
            tail: 0,
            ..base
        },
        Case {
            no: 2,
            other: 2,
            ..base
        },
        Case {
            choice: true,
            yes: i64::MIN,
            rhs: 0,
            no: 0,
            other: 0,
            ..base
        },
        Case {
            choice: true,
            yes: -1,
            rhs: 2,
            no: 0,
            other: 0,
            ..base
        },
        Case {
            choice: false,
            no: -1,
            other: 0,
            value: i64::MIN,
            ..base
        },
        Case {
            choice: false,
            no: 2,
            other: 1,
            value: i64::MAX,
            ..base
        },
        Case {
            choice: false,
            condition: 2,
            yes: 1,
            rhs: -1,
            no: 0,
            other: 0,
            value: -1,
            ..base
        },
        Case {
            choice: true,
            condition: 2,
            no: 2,
            other: -1,
            ..base
        },
    ]);
    rows
}

pub fn failures() -> Vec<Case> {
    let base = prefix()[2];
    vec![
        Case {
            value: 0,
            condition: 0,
            ..base
        },
        Case {
            condition: 0,
            yes: 0,
            rhs: 0,
            no: 0,
            other: 0,
            tail: 0,
            ..base
        },
        Case {
            choice: true,
            yes: 0,
            rhs: 0,
            no: 0,
            other: 0,
            tail: 0,
            ..base
        },
        Case {
            choice: true,
            yes: 1,
            rhs: 0,
            no: 0,
            other: 0,
            tail: 0,
            ..base
        },
        Case {
            no: 0,
            other: 0,
            yes: 0,
            rhs: 0,
            tail: 0,
            ..base
        },
        Case {
            no: 2,
            other: 0,
            yes: 0,
            rhs: 0,
            tail: 0,
            ..base
        },
        Case {
            choice: true,
            yes: 1,
            rhs: 1,
            tail: 0,
            ..base
        },
        Case {
            choice: true,
            yes: 2,
            rhs: 0,
            tail: 0,
            ..base
        },
        Case {
            no: 1,
            other: 0,
            tail: 0,
            ..base
        },
        Case {
            no: 2,
            other: 2,
            tail: 0,
            ..base
        },
    ]
}
