pub const SOURCE: &str = include_str!("carry_epoch_print.ns");
pub const LOOP_LIMIT: u64 = 4;
pub const ENTRY_LIMIT: u64 = 64;
pub const LAST_STATE: i64 = -2;
pub const ENTRY_MARKERS: &[i64] = &[94, 95];
pub const OBSERVERS: &[(&str, &str)] = &[
    ("observe", "  call void @word_print(i64 94)\n"),
    ("finish", "  call void @word_print(i64 95)\n"),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub initial: i64,
    pub selected: bool,
    pub trips: i64,
    pub delta: i64,
    pub divisor: i64,
    pub tail: i64,
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        vec![
            self.initial,
            i64::from(self.selected),
            self.trips,
            self.delta,
            self.divisor,
            self.tail,
        ]
    }

    pub fn args(self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.initial, self.selected, self.trips, self.delta, self.divisor, self.tail
        )
    }

    fn evaluate(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let mut current = previous.unwrap_or(self.initial);
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        assert!(self.trips <= LOOP_LIMIT as i64);
        for index in 0..self.trips {
            trace.push(71);
            if self.selected {
                if native {
                    trace.push(94);
                }
                let Some(checked) = 0i64.checked_div(self.divisor.wrapping_sub(index)) else {
                    return Err(trace);
                };
                trace.push(current.wrapping_add(checked));
            }
            trace.push(current);
            current = current.wrapping_add(self.delta);
            trace.push(current);
        }
        trace.push(73);
        if native {
            trace.push(95);
        }
        let Some(checked) = 0i64.checked_div(self.tail) else {
            return Err(trace);
        };
        Ok((current.wrapping_add(checked), trace))
    }

    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
    }

    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
    }
}

fn base() -> Case {
    Case {
        initial: 64,
        selected: false,
        trips: 0,
        delta: 4,
        divisor: 0,
        tail: 1,
    }
}

pub fn prefix() -> Vec<Case> {
    vec![
        base(),
        Case {
            selected: true,
            trips: 2,
            divisor: 3,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            delta: -37,
            divisor: -1,
            ..base()
        },
    ]
}

pub fn healthy() -> Vec<Case> {
    let mut rows = prefix();
    rows.extend([
        Case { trips: 3, ..base() },
        Case {
            selected: true,
            trips: 0,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            divisor: i64::MIN,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            divisor: i64::MAX,
            ..base()
        },
        Case {
            trips: -1,
            ..base()
        },
    ]);
    rows
}

pub fn failures() -> Vec<Case> {
    vec![
        Case {
            selected: true,
            trips: 1,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            divisor: 1,
            ..base()
        },
        Case {
            selected: true,
            trips: 3,
            divisor: 2,
            ..base()
        },
        Case {
            trips: 0,
            tail: 0,
            ..base()
        },
        Case {
            trips: 2,
            tail: 0,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            divisor: 3,
            tail: 0,
            ..base()
        },
    ]
}

pub fn native_failures() -> Vec<Case> {
    failures()
}
