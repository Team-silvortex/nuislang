#[allow(dead_code)]
#[path = "f32_field_epoch_print_cases.rs"]
mod narrow;
#[allow(dead_code)]
#[path = "f64_field_epoch_print_cases.rs"]
mod schedule;

pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    F32,
    F64,
}
pub const KINDS: [Kind; 2] = [Kind::F32, Kind::F64];

impl Kind {
    pub fn ty(self) -> &'static str {
        if self == Self::F32 {
            "f32"
        } else {
            "f64"
        }
    }
    pub fn mask(self) -> u64 {
        if self == Self::F32 {
            1 << 31
        } else {
            1 << 63
        }
    }
    pub fn initial(self) -> i64 {
        if self == Self::F32 {
            0x3fc00000
        } else {
            0x3ff8000000000000
        }
    }
    pub fn last_state(self) -> i64 {
        (self.initial() as u64 ^ self.mask()) as i64
    }
    pub fn nan(self) -> i64 {
        if self == Self::F32 {
            0x7fc12345
        } else {
            0x7ff812345678abcd
        }
    }
    pub fn decimal(self, bits: i64) -> String {
        if self == Self::F32 {
            f64::from(f32::from_bits(bits as u32)).to_string()
        } else {
            f64::from_bits(bits as u64).to_string()
        }
    }
    pub fn source(self) -> String {
        self.source_with_negations(1)
    }
    pub fn source_with_negations(self, count: usize) -> String {
        assert!(count > 0);
        let mut source = include_str!("unary_float_field_epoch_print.ns").replace("f64", self.ty());
        for field in ["current.inner.total", "local.inner.total"] {
            let mut value = field.to_owned();
            for _ in 0..count {
                value = format!("-({value})");
            }
            source = source.replace(&format!("observe(-{field},"), &format!("observe({value},"));
        }
        source
    }
    pub fn observers(self) -> &'static [(&'static str, &'static str)] {
        if self == Self::F32 {
            narrow::OBSERVERS
        } else {
            schedule::OBSERVERS
        }
    }
    fn words(self) -> &'static [u64] {
        if self == Self::F32 {
            &[
                0, 0x80000000, 0x3fc00000, 0xc0200000, 1, 0x807fffff, 0x7f800000, 0xff800000,
                0x7fc12345, 0xffc56789, 0x7f800001, 0xff800001, 94, 95, 0x3f80005e, 0x3f80005f,
            ]
        } else {
            &[
                0,
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
            ]
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub kind: Kind,
    pub negations: usize,
    pub row: schedule::Case,
}

impl Case {
    pub fn with_negations(mut self, negations: usize) -> Self {
        self.negations = negations;
        self
    }
    pub fn words(self) -> Vec<i64> {
        self.row.words()
    }
    pub fn args(self) -> String {
        let row = self.row;
        format!(
            "{},{},{},{},{},{},{}",
            self.kind.decimal(row.initial),
            row.selected,
            row.trips,
            row.children,
            row.delta,
            row.divisor,
            row.tail
        )
    }
    pub fn observer_counts(self) -> [usize; 2] {
        self.row.observer_counts()
    }

    fn observe(self, trace: &mut Vec<i64>, native: bool, current: u64, divisor: i64) -> bool {
        // This unsigned oracle models the source argument, not the callee body
        // or NIR shape. The persistent carry itself is not changed by this read.
        let negated = if self.negations % 2 == 0 {
            current
        } else {
            current ^ self.kind.mask()
        };
        if native {
            trace.extend([94, negated as i64]);
        }
        let Some(checked) = 0i64.checked_div(divisor) else {
            return false;
        };
        let value = if self.kind == Kind::F32 {
            f64::from(f32::from_bits(negated as u32))
        } else {
            f64::from_bits(negated)
        };
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

    fn evaluate(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let row = self.row;
        let mut current = previous.unwrap_or(row.initial) as u64;
        let mut tag = 100i64;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        assert!(row.trips <= 3 && row.children <= 3);
        for index in 0..row.trips {
            trace.push(71);
            assert!(self.observe(&mut trace, native, current, 1));
            trace.push(tag);
            current ^= self.kind.mask();
            tag = tag.wrapping_add(row.delta);
            let mut local = current;
            for child in 0..row.children {
                trace.push(72);
                if row.selected
                    && !self.observe(
                        &mut trace,
                        native,
                        local,
                        row.divisor.wrapping_sub(index).wrapping_sub(child),
                    )
                {
                    return Err(trace);
                }
                assert!(self.observe(&mut trace, native, local, 1));
                trace.push(tag);
                local ^= self.kind.mask();
                tag = tag.wrapping_add(row.delta.wrapping_mul(2));
                assert!(self.observe(&mut trace, native, local, 1));
                trace.push(tag);
            }
            trace.push(74);
            current = local;
        }
        trace.push(73);
        if native {
            trace.extend([95, current as i64]);
        }
        let Some(_) = 0i64.checked_div(row.tail) else {
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

fn rows(kind: Kind, rows: Vec<schedule::Case>) -> Vec<Case> {
    rows.into_iter()
        .map(|row| Case {
            kind,
            negations: 1,
            row: schedule::Case {
                initial: kind.initial(),
                ..row
            },
        })
        .collect()
}

pub fn with_negations(rows: Vec<Case>, count: usize) -> Vec<Case> {
    rows.into_iter()
        .map(|row| row.with_negations(count))
        .collect()
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    rows(kind, schedule::prefix())
}
pub fn failures(kind: Kind) -> Vec<Case> {
    rows(kind, schedule::failures())
}
pub fn healthy(kind: Kind) -> Vec<Case> {
    rows(kind, schedule::healthy())
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut sequences = vec![healthy(kind)];
    for &word in kind.words() {
        let mut sequence = prefix(kind);
        sequence[0].row.initial = word as i64;
        sequences.push(sequence);
    }
    sequences
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let words = kind.words();
    sequences(kind)
        .into_iter()
        .filter(|rows| {
            rows.len() > 3
                || [words[0], words[1], words[3], words[4], words[5]]
                    .contains(&(rows[0].row.initial as u64))
        })
        .collect()
}
