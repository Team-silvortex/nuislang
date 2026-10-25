pub const SOURCE: &str = include_str!("f64_field_epoch_print.ns");
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 256;
pub const LAST_STATE: i64 = 0xbff8_0000_0000_0000_u64 as i64;
pub const OBSERVERS: &[(&str, &str)] = &[
    ("observe", "  call void @word_print(i64 94)\n  %observed_f64_word = bitcast double %arg0 to i64\n  call void @word_print(i64 %observed_f64_word)\n"),
    ("finish", "  call void @word_print(i64 95)\n  %observed_f64_word = bitcast double %arg0 to i64\n  call void @word_print(i64 %observed_f64_word)\n"),
];

fn observed(trace: &mut Vec<i64>, native: bool, bits: u64, divisor: i64) -> bool {
    if native {
        trace.extend([94, bits as i64]);
    }
    let Some(checked) = 0i64.checked_div(divisor) else {
        return false;
    };
    let value = f64::from_bits(bits);
    let class = if value < 0.0 {
        -1
    } else if value == 0.0 {
        0
    } else if value.is_nan() {
        2
    } else {
        1
    };
    trace.push(class + checked);
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
            f64::from_bits(self.initial as u64),
            self.selected,
            self.trips,
            self.children,
            self.delta,
            self.divisor,
            self.tail
        )
    }

    fn evaluate(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let mut current = previous.unwrap_or(self.initial) as u64;
        let mut tag = 100i64;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        assert!(self.trips <= 3 && self.children <= 3);
        for index in 0..self.trips {
            trace.push(71);
            assert!(observed(&mut trace, native, current, 1));
            trace.push(tag);
            current ^= 0x8000_0000_0000_0000;
            tag = tag.wrapping_add(self.delta);
            let mut local = current;
            for child in 0..self.children {
                trace.push(72);
                if self.selected {
                    let divisor = self.divisor.wrapping_sub(index).wrapping_sub(child);
                    if !observed(&mut trace, native, local, divisor) {
                        return Err(trace);
                    }
                }
                assert!(observed(&mut trace, native, local, 1));
                trace.push(tag);
                local ^= 0x8000_0000_0000_0000;
                tag = tag.wrapping_add(self.delta.wrapping_mul(2));
                assert!(observed(&mut trace, native, local, 1));
                trace.push(tag);
            }
            trace.push(74);
            current = local;
        }
        trace.push(73);
        if native {
            trace.extend([95, current as i64]);
        }
        let Some(_) = 0i64.checked_div(self.tail) else {
            return Err(trace);
        };
        Ok((current as i64, trace))
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
        initial: 0x3ff8_0000_0000_0000,
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
            children: 2,
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

pub fn sequences() -> Vec<Vec<Case>> {
    let mut rows = vec![healthy()];
    for word in [
        0u64,
        0x8000000000000000,
        0x3ff8000000000000,
        0xc004000000000000,
        1,
        0x800fffffffffffff,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff812345678abcd,
        0xfff856789abcdef0,
        0x7ff0000000000001,
        0xfff0000000000001,
        94,
        95,
        0x3ff000000000005e,
        0x3ff000000000005f,
    ] {
        let initial = word as i64;
        rows.push(vec![
            Case { initial, ..base() },
            Case {
                selected: true,
                trips: 1,
                children: 2,
                divisor: 9,
                ..base()
            },
            Case {
                trips: 1,
                children: 0,
                delta: -2,
                ..base()
            },
        ]);
    }
    rows
}

pub fn cli_sequences() -> Vec<Vec<Case>> {
    sequences()
        .into_iter()
        .filter(|rows| {
            rows.len() > 3
                || [
                    0u64,
                    0x8000000000000000,
                    0xc004000000000000,
                    1,
                    0x800fffffffffffff,
                ]
                .contains(&(rows[0].initial as u64))
        })
        .collect()
}

pub fn decimal(bits: i64) -> String {
    f64::from_bits(bits as u64).to_string()
}

impl Case {
    pub fn observer_counts(self) -> [usize; 2] {
        let trips = self.trips.max(0) as usize;
        let children = self.children.max(0) as usize;
        [trips * (1 + children * (2 + usize::from(self.selected))), 1]
    }
}
