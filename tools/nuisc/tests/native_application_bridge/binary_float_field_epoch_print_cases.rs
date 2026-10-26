#[allow(dead_code)]
#[path = "unary_float_field_epoch_print_cases.rs"]
mod base;
pub use base::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 256;

pub fn source(kind: Kind) -> String {
    include_str!("binary_float_field_epoch_print.ns").replace("f64", kind.ty())
}

pub fn observers(kind: Kind) -> Vec<(&'static str, String)> {
    let baseline = kind.observers();
    let (word, ty, canonical) = if kind == Kind::F32 {
        ("%observed_f32_word", "float", 0x7fc0_0000i64)
    } else {
        ("%observed_f64_word", "double", 0x7ff8_0000_0000_0000i64)
    };
    let print = format!("  %argument_nan = fcmp uno {ty} %arg0, %arg0\n  %argument_word = select i1 %argument_nan, i64 {canonical}, i64 {word}\n  call void @word_print(i64 %argument_word)\n");
    let input = if kind == Kind::F32 {
        "  %ready_bits = bitcast float %arg7 to i32\n  %ready_word = zext i32 %ready_bits to i64\n"
    } else {
        "  %ready_word = bitcast double %arg7 to i64\n"
    };
    vec![
        (
            "observe",
            baseline[0]
                .1
                .replace(&format!("  call void @word_print(i64 {word})\n"), &print),
        ),
        ("finish", baseline[1].1.to_owned()),
        ("work", format!("  call void @word_print(i64 96)\n{input}  call void @word_print(i64 %ready_word)\n")),
    ]
}

macro_rules! calculation {
    ($name:ident, $ty:ty, $nan:expr) => {
        fn $name(current: u64, ready: u64, role: usize) -> Result<u64, ()> {
            let current = <$ty>::from_bits(current as _);
            let ready = <$ty>::from_bits(ready as _);
            let denominator = if role == 1 { ready - current } else { ready };
            if denominator == 0.0 {
                return Err(());
            }
            let result = match role {
                0 => -(((current + ready) - (-ready)) * ready / ready),
                1 => (ready - current) * (current + ready) / (ready - current),
                2 => -((current - ready) / ready) + current * ready,
                _ => unreachable!(),
            };
            // Arithmetic NaN payloads are not a cross-backend contract. Only
            // NaNs normalize here; finite, infinite and signed-zero bits stay exact.
            Ok(if result.is_nan() {
                $nan
            } else {
                result.to_bits() as u64
            })
        }
    };
}
calculation!(calculate_f32, f32, 0x7fc0_0000u64);
calculation!(calculate_f64, f64, 0x7ff8_0000_0000_0000u64);

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: base::Case,
    pub operand: i64,
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.push(self.operand);
        words
    }
    pub fn args(self) -> String {
        format!("{},{}", self.base.args(), self.kind().decimal(self.operand))
    }
    pub fn observer_counts(self) -> [usize; 2] {
        self.base.observer_counts()
    }
    fn observe(
        self,
        trace: &mut Vec<i64>,
        native: bool,
        current: u64,
        role: usize,
        divisor: i64,
    ) -> bool {
        let result = if self.kind() == Kind::F32 {
            calculate_f32(current, self.operand as u64, role)
        } else {
            calculate_f64(current, self.operand as u64, role)
        };
        let Ok(bits) = result else {
            return false;
        };
        if native {
            trace.extend([94, bits as i64]);
        }
        let Some(checked) = 0i64.checked_div(divisor) else {
            return false;
        };
        let value = if self.kind() == Kind::F32 {
            f64::from(f32::from_bits(bits as u32))
        } else {
            f64::from_bits(bits)
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
        let row = self.base.row;
        let mut current = previous.unwrap_or(row.initial) as u64;
        let mut tag = 100i64;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }];
        if native {
            trace.extend([96, self.operand]);
        }
        trace.push(70);
        assert!(row.trips <= 3 && row.children <= 3);
        for index in 0..row.trips {
            trace.push(71);
            if !self.observe(&mut trace, native, current, 0, 1) {
                return Err(trace);
            }
            trace.push(tag);
            current ^= self.kind().mask();
            tag = tag.wrapping_add(row.delta);
            let mut local = current;
            for child in 0..row.children {
                trace.push(72);
                if row.selected
                    && !self.observe(
                        &mut trace,
                        native,
                        local,
                        1,
                        row.divisor.wrapping_sub(index).wrapping_sub(child),
                    )
                {
                    return Err(trace);
                }
                if !self.observe(&mut trace, native, local, 2, 1) {
                    return Err(trace);
                }
                trace.push(tag);
                local ^= self.kind().mask();
                tag = tag.wrapping_add(row.delta.wrapping_mul(2));
                if !self.observe(&mut trace, native, local, 2, 1) {
                    return Err(trace);
                }
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

fn wrap(rows: Vec<base::Case>) -> Vec<Case> {
    rows.into_iter()
        .map(|row| Case {
            operand: if row.kind == Kind::F32 {
                0x4000_0000
            } else {
                0x4000_0000_0000_0000
            },
            base: row,
        })
        .collect()
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    wrap(base::prefix(kind))
}
pub fn healthy(kind: Kind) -> Vec<Case> {
    let mut rows = wrap(base::healthy(kind));
    let mut skipped = rows[0];
    skipped.base.row.trips = 1;
    skipped.base.row.children = 1;
    skipped.operand = kind.initial();
    rows.push(skipped);
    rows
}
pub fn failures(kind: Kind) -> Vec<Case> {
    let mut failures = wrap(base::failures(kind));
    let mut bad = prefix(kind)[1];
    bad.base.row.divisor = 9;
    for operand in [0, kind.mask() as i64, kind.initial(), kind.last_state()] {
        bad.operand = operand;
        bad.base.row.children = 2;
        failures.push(bad);
    }
    failures
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::sequences(kind)
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows[0] = healthy(kind);
    for (initial, operand) in if kind == Kind::F32 {
        [
            (0x7f7fffffu64, 0x40400000u64),
            (1, 0x00800000),
            (0x3f800001, 0xbfa00000),
            (0x3f000000, 0x7f7fffff),
        ]
    } else {
        [
            (0x7fefffffffffffff, 0x4008000000000000),
            (1, 0x0010000000000000),
            (0x3ff0000000000001, 0xbff4000000000000),
            (0x3fe0000000000000, 0x7fefffffffffffff),
        ]
    } {
        let mut sequence = prefix(kind);
        sequence[0].base.row.initial = initial as i64;
        for row in &mut sequence {
            row.operand = operand as i64;
        }
        rows.push(sequence);
    }
    rows
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::cli_sequences(kind)
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows[0] = healthy(kind);
    let fractional = if kind == Kind::F32 {
        0xbfa0_0000i64
    } else {
        0xbff4_0000_0000_0000u64 as i64
    };
    rows.push(
        sequences(kind)
            .into_iter()
            .find(|rows| rows[0].operand == fractional)
            .unwrap(),
    );
    rows
}
