pub const SOURCE: &str = include_str!("bool_field_epoch_print.ns");
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 128;
pub const LAST_STATE: i64 = -2;
pub const ENTRY_MARKERS: &[i64] = &[94, 95];
pub const OBSERVERS: &[(&str, &str)] = &[
    ("observe", "  call void @word_print(i64 94)\n"),
    ("finish", "  call void @word_print(i64 95)\n"),
];

fn observed(trace: &mut Vec<i64>, native: bool, flag: bool, value: i64, divisor: i64) -> bool {
    if native {
        trace.push(94);
    }
    let Some(checked) = 0i64.checked_div(divisor) else {
        return false;
    };
    trace.push(
        value
            .wrapping_add(if flag { 0 } else { 10 })
            .wrapping_add(checked),
    );
    true
}

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub initial: i64,
    pub selected: bool,
    pub trips: i64,
    pub children: i64,
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
            self.children,
            self.delta,
            self.divisor,
            self.tail,
        ]
    }

    pub fn args(self) -> String {
        format!(
            "{},{},{},{},{},{},{}",
            self.initial,
            self.selected,
            self.trips,
            self.children,
            self.delta,
            self.divisor,
            self.tail
        )
    }

    fn evaluate(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let mut current = previous.unwrap_or(self.initial);
        let mut tag = 100i64;
        let mut flag = self.selected;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        assert!(self.trips <= 3 && self.children <= 3);
        for index in 0..self.trips {
            trace.push(71);
            assert!(observed(&mut trace, native, flag, current, 1));
            trace.push(tag);
            current = current.wrapping_add(self.delta);
            tag = tag.wrapping_add(1);
            flag = !flag;
            let mut local = current;
            for child in 0..self.children {
                trace.push(72);
                if self.selected {
                    let divisor = self.divisor.wrapping_sub(index).wrapping_sub(child);
                    if !observed(&mut trace, native, flag, local, divisor) {
                        return Err(trace);
                    }
                }
                assert!(observed(&mut trace, native, flag, local, 1));
                trace.push(tag);
                local = local.wrapping_add(self.delta);
                tag = tag.wrapping_add(2);
                flag = !flag;
                assert!(observed(&mut trace, native, flag, local, 1));
                trace.push(tag);
            }
            trace.push(74);
            current = local;
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
        children: 2,
        delta: 1,
        divisor: 0,
        tail: 1,
    }
}

pub fn prefix() -> Vec<Case> {
    vec![
        base(),
        Case {
            selected: true,
            trips: 1,
            children: 1,
            delta: 4,
            divisor: 3,
            ..base()
        },
        Case {
            selected: true,
            trips: 1,
            children: 1,
            delta: -37,
            divisor: -1,
            ..base()
        },
    ]
}

pub fn healthy() -> Vec<Case> {
    let mut rows = prefix();
    rows.extend([
        Case {
            selected: true,
            trips: 2,
            children: 2,
            divisor: 9,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            children: 0,
            delta: 2,
            ..base()
        },
        Case {
            trips: 2,
            children: 2,
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
            selected: true,
            trips: 3,
            children: 3,
            divisor: 9,
            ..base()
        },
        Case {
            trips: 2,
            children: -1,
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
            children: 1,
            ..base()
        },
        Case {
            selected: true,
            trips: 1,
            children: 2,
            divisor: 1,
            ..base()
        },
        Case {
            selected: true,
            trips: 1,
            children: 3,
            divisor: 2,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            children: 1,
            divisor: 1,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            children: 2,
            divisor: 2,
            ..base()
        },
        Case { tail: 0, ..base() },
        Case {
            trips: 2,
            children: 0,
            tail: 0,
            ..base()
        },
        Case {
            trips: 2,
            children: 2,
            tail: 0,
            ..base()
        },
        Case {
            selected: true,
            trips: 2,
            children: 2,
            divisor: 9,
            tail: 0,
            ..base()
        },
    ]
}

pub fn native_failures() -> Vec<Case> {
    failures()
}
