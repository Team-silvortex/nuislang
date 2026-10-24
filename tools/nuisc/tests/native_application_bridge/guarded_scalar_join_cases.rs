pub const SOURCE: &str = include_str!("guarded_scalar_join.ns");
pub const LOOP_LIMIT: u64 = 4;
pub const ENTRY_LIMIT: u64 = 64;

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub total: i64,
    pub selected: bool,
    pub trips: i64,
    pub choice: bool,
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
            self.value,
            self.yes,
            self.no,
            self.tail,
        ]
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{},{},{},{},{},{}",
            self.total,
            self.selected,
            self.trips,
            self.choice,
            self.value,
            self.yes,
            self.no,
            self.tail
        )
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        let seed = previous.unwrap_or(self.total);
        for _ in 0..self.trips {
            trace.push(71);
            if self.selected {
                let input = seed.wrapping_add(self.value);
                0i64.checked_div(self.value).ok_or_else(|| trace.clone())?;
                trace.push(81);
                let value = if self.choice {
                    input.checked_div(self.yes)
                } else {
                    input.checked_rem(self.no)
                }
                .ok_or_else(|| trace.clone())?;
                trace.push(82);
                1i64.checked_div(self.tail).ok_or_else(|| trace.clone())?;
                return Ok((value, trace));
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
            value: 8,
            yes: 4,
            no: 0,
            tail: 1,
        },
        Case {
            total: 999,
            selected: true,
            trips: 2,
            choice: false,
            value: -23,
            yes: 0,
            no: 3,
            tail: 1,
        },
    ]
}

pub fn healthy() -> Vec<Case> {
    let mut rows = prefix();
    let base = rows[2];
    rows.extend([
        Case {
            selected: false,
            trips: 2,
            yes: 0,
            no: 0,
            tail: 0,
            ..base
        },
        Case {
            trips: 0,
            yes: 0,
            no: 0,
            tail: 0,
            ..base
        },
        Case {
            choice: true,
            value: 10,
            yes: 2,
            no: 0,
            ..base
        },
    ]);
    rows
}

pub fn failures() -> Vec<Case> {
    let base = prefix()[2];
    vec![
        Case {
            choice: true,
            value: 8,
            yes: 0,
            ..base
        },
        Case {
            value: 8,
            no: 0,
            ..base
        },
        Case {
            choice: true,
            value: i64::MIN + 2,
            yes: -1,
            ..base
        },
        Case {
            value: i64::MIN + 2,
            no: -1,
            ..base
        },
        Case {
            value: 5,
            tail: 0,
            ..base
        },
        Case { value: 0, ..base },
    ]
}
