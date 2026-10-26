#[allow(dead_code)]
#[path = "binary_float_field_epoch_print_cases.rs"]
mod base;
pub use base::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 512;

pub fn source(kind: Kind) -> String {
    include_str!("float_call_field_epoch_print.ns").replace("f64", kind.ty())
}

pub fn observers(kind: Kind) -> Vec<(&'static str, String)> {
    let mut bodies = base::observers(kind);
    bodies[2]
        .1
        .push_str("  call void @word_print(i64 %arg8)\n  call void @word_print(i64 %arg9)\n");
    let (ty, nan) = if kind == Kind::F32 {
        ("float", 0x7fc00000i64)
    } else {
        ("double", 0x7ff8000000000000i64)
    };
    for (name, marker, count) in [("identity", 97, 1), ("scale", 98, 2)] {
        let mut body = format!("  call void @word_print(i64 {marker})\n");
        for arg in 0..count {
            if kind == Kind::F32 {
                body.push_str(&format!("  %probe_bits{arg} = bitcast float %arg{arg} to i32\n  %probe_word{arg} = zext i32 %probe_bits{arg} to i64\n"));
            } else {
                body.push_str(&format!(
                    "  %probe_word{arg} = bitcast double %arg{arg} to i64\n"
                ));
            }
            body.push_str(&format!("  %probe_nan{arg} = fcmp uno {ty} %arg{arg}, %arg{arg}\n  %probe_value{arg} = select i1 %probe_nan{arg}, i64 {nan}, i64 %probe_word{arg}\n  call void @word_print(i64 %probe_value{arg})\n"));
        }
        body.push_str(&format!("  call void @word_print(i64 %arg{count})\n"));
        bodies.push((name, body));
    }
    bodies
}

macro_rules! calculation {
    ($name:ident, $ty:ty, $nan:expr) => {
        fn $name(
            current: u64,
            ready: u64,
            role: usize,
            first: i64,
            second: i64,
            trace: &mut Vec<i64>,
            native: bool,
        ) -> Result<u64, ()> {
            let current = <$ty>::from_bits(current as _);
            let ready = <$ty>::from_bits(ready as _);
            let word = |v: $ty| {
                if v.is_nan() {
                    $nan as i64
                } else {
                    v.to_bits() as i64
                }
            };
            let identity = |v: $ty, guard: i64, trace: &mut Vec<i64>| {
                if native {
                    trace.extend([97, word(v), guard]);
                }
                0i64.checked_div(guard).map(|_| v).ok_or(())
            };
            let scale = |v: $ty, factor: $ty, guard: i64, trace: &mut Vec<i64>| {
                if native {
                    trace.extend([98, word(v), word(factor), guard]);
                }
                0i64.checked_div(guard).map(|_| v * factor).ok_or(())
            };
            let divide = |left: $ty, right: $ty| {
                if right == 0.0 {
                    Err(())
                } else {
                    Ok(left / right)
                }
            };
            let result = match role {
                0 => {
                    let left = identity(current + ready, 1, trace)? - (-ready);
                    let numerator = -scale(left, ready, 1, trace)?;
                    identity(divide(numerator, ready)?, 1, trace)?
                }
                1 => {
                    let left = identity(ready - current, first, trace)?;
                    let right = identity(current + ready, second, trace)?;
                    let numerator = scale(left, right, 1, trace)?;
                    let denominator = identity(ready - current, 1, trace)?;
                    identity(divide(numerator, denominator)?, 1, trace)?
                }
                2 => {
                    let numerator = identity(current - ready, 1, trace)?;
                    let left = identity(-divide(numerator, ready)?, 1, trace)?;
                    let right = scale(current, ready, 1, trace)?;
                    left + right
                }
                _ => unreachable!(),
            };
            // Computed NaN payloads may vary; carried State words remain exact.
            Ok(word(result) as u64)
        }
    };
}
calculation!(calculate_f32, f32, 0x7fc0_0000u64);
calculation!(calculate_f64, f64, 0x7ff8_0000_0000_0000u64);

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: base::Case,
    pub first: i64,
    pub second: i64,
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind()
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.extend([self.first, self.second]);
        words
    }
    pub fn args(self) -> String {
        format!("{},{},{}", self.base.args(), self.first, self.second)
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
            calculate_f32(
                current,
                self.base.operand as u64,
                role,
                self.first,
                self.second,
                trace,
                native,
            )
        } else {
            calculate_f64(
                current,
                self.base.operand as u64,
                role,
                self.first,
                self.second,
                trace,
                native,
            )
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
        let row = self.base.base.row;
        let mut current = previous.unwrap_or(row.initial) as u64;
        let mut tag = 100i64;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }];
        if native {
            trace.extend([96, self.base.operand, self.first, self.second]);
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
        .map(|base| Case {
            base,
            first: 1,
            second: 1,
        })
        .collect()
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    wrap(base::prefix(kind))
}
pub fn healthy(kind: Kind) -> Vec<Case> {
    let mut rows = wrap(base::healthy(kind));
    let mut skipped = prefix(kind)[1];
    skipped.base.base.row.selected = false;
    skipped.first = 0;
    skipped.second = 0;
    rows.push(skipped);
    let mut zero = skipped;
    zero.base.base.row.selected = true;
    zero.base.base.row.trips = 0;
    rows.push(zero);
    rows
}
pub fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = wrap(base::failures(kind));
    let mut bad = prefix(kind)[1];
    bad.base.base.row.divisor = 9;
    bad.base.base.row.trips = 1;
    bad.base.base.row.children = 1;
    for (first, second) in [(0, 1), (1, 0), (0, 0)] {
        bad.first = first;
        bad.second = second;
        rows.push(bad);
    }
    rows
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::sequences(kind)
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows[0] = healthy(kind);
    rows
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::cli_sequences(kind)
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows[0] = healthy(kind);
    rows
}
