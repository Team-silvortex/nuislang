#[allow(dead_code)]
#[path = "unary_float_field_epoch_print_cases.rs"]
mod base;
pub use base::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 512;

pub fn source(kind: Kind) -> String {
    include_str!("i32_negation_argument_print_epoch.ns").replace("f64", kind.ty())
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
    let mut bodies = vec![
        (
            "observe",
            baseline[0]
                .1
                .replace(&format!("  call void @word_print(i64 {word})\n"), &print),
        ),
        ("finish", baseline[1].1.to_owned()),
        ("work", format!("  call void @word_print(i64 96)\n{input}  call void @word_print(i64 %ready_word)\n")),
    ];
    bodies[2]
        .1
        .push_str("  call void @word_print(i64 %arg8)\n  call void @word_print(i64 %arg9)\n");
    let mut choose = bodies[0].1.replace("i64 94", "i64 99");
    choose.push_str("  %choice_word = zext i1 %arg1 to i64\n  call void @word_print(i64 %choice_word)\n  call void @word_print(i64 %arg2)\n");
    bodies.push(("choose", choose));
    bodies[2].1.push_str("  %left_word = sext i32 %arg10 to i64\n  %right_word = sext i32 %arg11 to i64\n  call void @word_print(i64 %left_word)\n  call void @word_print(i64 %right_word)\n");
    for (name, marker) in [("checked32", 106), ("relay32", 107)] {
        bodies.push((name,format!("  call void @word_print(i64 {marker})\n  %code_word = sext i32 %arg0 to i64\n  call void @word_print(i64 %code_word)\n  call void @word_print(i64 %arg1)\n")));
    }
    bodies
}

macro_rules! calculation {
    ($name:ident, $ty:ty, $nan:expr) => {
        fn $name(
            current: u64,
            ready: u64,
            role: usize,
            pick: bool,
            guard: i64,
            trace: &mut Vec<i64>,
            native: bool,
        ) -> Result<u64, ()> {
            let current = <$ty>::from_bits(current as _);
            let ready = <$ty>::from_bits(ready as _);
            let value = match role {
                0 => current + ready,
                1 => ready - current,
                2 | 3 => current - ready,
                _ => unreachable!(),
            };
            let word = |v: $ty| if v.is_nan() { $nan } else { v.to_bits() as u64 };
            if native {
                trace.extend([99, word(value) as i64, i64::from(pick), guard]);
            }
            0i64.checked_div(guard).ok_or(())?;
            Ok(word(-(if pick { value } else { -value })))
        }
    };
}
calculation!(calculate_f32, f32, 0x7fc0_0000u64);
calculation!(calculate_f64, f64, 0x7ff8_0000_0000_0000u64);

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: base::Case,
    pub operand: i64,
    pub first: i64,
    pub second: i64,
    pub left: i32,
    pub right: i32,
}

fn checked32(
    value: i32,
    guard: i64,
    trace: &mut Vec<i64>,
    native: bool,
    counts: &mut [usize; 2],
) -> Option<i32> {
    counts[0] += 1;
    if native {
        trace.extend([106, i64::from(value), guard]);
    }
    0i64.checked_div(guard)?;
    Some(value)
}
fn relay32(
    value: i32,
    guard: i64,
    trace: &mut Vec<i64>,
    native: bool,
    counts: &mut [usize; 2],
) -> Option<i32> {
    counts[1] += 1;
    if native {
        trace.extend([107, i64::from(value), guard]);
    }
    checked32(value, guard, trace, native, counts)
}

// Independent signed arithmetic oracle; do not use a wide quotient followed by truncation.
fn checked_word(left: i32, right: i32, remainder: bool) -> Option<i32> {
    if right == 0 || (left == i32::MIN && right == -1) {
        return None;
    }
    let left = i64::from(left);
    let right = i64::from(right);
    i32::try_from(if remainder {
        left % right
    } else {
        left / right
    })
    .ok()
}

// Restore signed modulo 2^32 independently of the compiler's cast/operation tree.
fn wrapped_word(value: i64) -> i32 {
    let word = value & 0xffff_ffff;
    (if word >= 0x8000_0000 {
        word - 0x1_0000_0000
    } else {
        word
    }) as i32
}

fn neg_word(value: i32) -> i32 {
    wrapped_word(-i64::from(value))
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.extend([
            self.operand,
            self.first,
            self.second,
            i64::from(self.left),
            i64::from(self.right),
        ]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.base.args(),
            self.kind().decimal(self.operand),
            self.first,
            self.second,
            self.left,
            self.right
        )
    }
    pub fn observer_counts(self) -> [usize; 2] {
        self.base.observer_counts()
    }
    fn observe(
        self,
        trace: &mut Vec<i64>,
        native: bool,
        current: u64,
        tag: i64,
        role: usize,
        divisor: i64,
        flag: bool,
        code: i32,
        counts: &mut [usize; 2],
    ) -> bool {
        let selected = (|| -> Option<(bool, i64)> {
            Some(match role {
                0 => (
                    if flag {
                        let value = checked32(code, self.first, trace, native, counts)?;
                        i64::from(relay32(
                            neg_word(neg_word(neg_word(wrapped_word(
                                i64::from(checked_word(value, self.right, false)?) + 1,
                            )))),
                            self.second,
                            trace,
                            native,
                            counts,
                        )?) < 0
                    } else {
                        false
                    },
                    1i64.wrapping_add(tag.wrapping_sub(tag)),
                ),
                1 => (
                    if flag {
                        true
                    } else {
                        i64::from(neg_word(checked32(
                            code, self.first, trace, native, counts,
                        )?)) <= i64::from(i32::MAX)
                            && i64::from(relay32(self.right, self.second, trace, native, counts)?)
                                >= i64::from(self.left)
                    },
                    divisor,
                ),
                2 => (
                    if flag {
                        true
                    } else {
                        i64::from(checked32(code, self.first, trace, native, counts)?)
                            > i64::from(self.left)
                            || relay32(neg_word(107), self.second, trace, native, counts)? == code
                    },
                    1i64.wrapping_add(0i64.wrapping_mul(tag)),
                ),
                3 => (
                    if flag {
                        true
                    } else {
                        let value = checked32(code, self.second, trace, native, counts)?;
                        relay32(
                            neg_word(wrapped_word(
                                i64::from(checked_word(value, self.right, true)?) + 1,
                            )),
                            self.first,
                            trace,
                            native,
                            counts,
                        )? != i32::MIN
                    },
                    1i64.wrapping_add(0i64.wrapping_mul(tag)),
                ),
                _ => unreachable!(),
            })
        })();
        let Some((pick, guard)) = selected else {
            return false;
        };
        let result = if self.kind() == Kind::F32 {
            calculate_f32(
                current,
                self.operand as u64,
                role,
                pick,
                guard,
                trace,
                native,
            )
        } else {
            calculate_f64(
                current,
                self.operand as u64,
                role,
                pick,
                guard,
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
        let value = if self.kind() == Kind::F32 {
            f64::from(f32::from_bits(bits as u32))
        } else {
            f64::from_bits(bits)
        };
        trace.push(if value < 0.0 {
            -1
        } else if value == 0.0 {
            0
        } else if value.is_nan() {
            2
        } else {
            1
        });
        true
    }
    fn evaluate(
        self,
        previous: Option<i64>,
        native: bool,
    ) -> Result<(i64, Vec<i64>, [usize; 2]), Vec<i64>> {
        let mut counts = [0usize; 2];
        let row = self.base.row;
        let mut current = previous.unwrap_or(row.initial) as u64;
        let mut tag = 100i64;
        let mut flag = row.selected;
        let mut code = self.left;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }];
        if native {
            trace.extend([
                96,
                self.operand,
                self.first,
                self.second,
                i64::from(self.left),
                i64::from(self.right),
            ]);
        }
        trace.push(70);
        assert!(row.trips <= 3 && row.children <= 3);
        for index in 0..row.trips {
            trace.push(71);
            if !self.observe(
                &mut trace,
                native,
                current,
                tag,
                0,
                1,
                flag,
                code,
                &mut counts,
            ) {
                return Err(trace);
            }
            trace.push(tag);
            current ^= self.kind().mask();
            tag = tag.wrapping_add(row.delta);
            flag = !row.selected;
            code = self.right;
            let mut local = current;
            for child in 0..row.children {
                trace.push(72);
                if row.selected
                    && !self.observe(
                        &mut trace,
                        native,
                        local,
                        tag,
                        1,
                        row.divisor.wrapping_sub(index).wrapping_sub(child),
                        flag,
                        code,
                        &mut counts,
                    )
                {
                    return Err(trace);
                }
                if !self.observe(
                    &mut trace,
                    native,
                    local,
                    tag,
                    2,
                    1,
                    flag,
                    code,
                    &mut counts,
                ) {
                    return Err(trace);
                }
                trace.push(tag);
                local ^= self.kind().mask();
                tag = tag.wrapping_add(row.delta.wrapping_mul(2));
                flag = row.selected;
                code = self.left;
                if !self.observe(
                    &mut trace,
                    native,
                    local,
                    tag,
                    3,
                    1,
                    flag,
                    code,
                    &mut counts,
                ) {
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
        Ok((current as i64, trace, counts))
    }
    pub fn helper_counts(self, previous: Option<i64>) -> [usize; 2] {
        self.evaluate(previous, false).unwrap().2
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, false)
            .map(|(state, trace, _)| (state, trace))
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.evaluate(previous, true)
            .map(|(state, trace, _)| (state, trace))
    }
}

fn wrap(rows: Vec<base::Case>) -> Vec<Case> {
    rows.into_iter()
        .map(|base| Case {
            operand: if base.kind == Kind::F32 {
                0x40000000
            } else {
                0x4000000000000000
            },
            base,
            first: 1,
            second: 1,
            left: -1,
            right: 1,
        })
        .collect()
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    wrap(base::prefix(kind))
}
pub fn healthy(kind: Kind) -> Vec<Case> {
    let mut rows = wrap(base::healthy(kind));
    let mut altered = prefix(kind)[1];
    altered.first = 2;
    rows.push(altered);
    let mut skipped = altered;
    skipped.base.row.selected = false;
    skipped.base.row.divisor = 0;
    rows.push(skipped);
    let mut zero = altered;
    zero.base.row.trips = 0;
    zero.first = 0;
    zero.second = 0;
    rows.push(zero);
    for (left, right) in [(7, 0), (i32::MIN, -1)] {
        let mut skipped = altered;
        skipped.base.row.trips = 1;
        skipped.base.row.children = 0;
        skipped.base.row.selected = false;
        skipped.left = left;
        skipped.right = right;
        rows.push(skipped);
    }
    rows
}
pub fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = wrap(base::failures(kind));
    let mut bad = prefix(kind)[1];
    bad.base.row.trips = 1;
    bad.base.row.children = 1;
    for (first, second) in [(0, 1), (1, 0), (0, 0)] {
        bad.first = first;
        bad.second = second;
        rows.push(bad);
    }
    bad.base.row.selected = false;
    for (first, second) in [(0, 1), (1, 0)] {
        bad.first = first;
        bad.second = second;
        rows.push(bad);
    }
    bad.base.row.trips = 2;
    bad.base.row.children = 0;
    for (first, second) in [(0, 1), (1, 0)] {
        bad.first = first;
        bad.second = second;
        rows.push(bad);
    }
    for (left, right) in [(7, 0), (i32::MIN, -1)] {
        for selected in [false, true] {
            let mut arithmetic = prefix(kind)[1];
            arithmetic.base.row.trips = 1;
            arithmetic.base.row.children = if selected { 0 } else { 1 };
            arithmetic.base.row.selected = selected;
            arithmetic.left = left;
            arithmetic.right = right;
            rows.push(arithmetic);
        }
    }
    rows
}
pub fn signed_pairs() -> [(i32, i32); 12] {
    [
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
        (-7, 3),
        (0, -1),
        (7, -3),
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
        (1, 1),
        (106, 107),
        (107, 106),
        (i32::MIN + 1, i32::MIN),
        (i32::MAX - 1, i32::MAX),
    ]
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::sequences(kind)
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows[0] = healthy(kind);
    for word in [96, 99, 106, 107] {
        let mut raw = prefix(kind);
        raw[0].base.row.initial = word;
        rows.push(raw);
    }
    for (left, right) in signed_pairs() {
        let mut raw = prefix(kind);
        for row in &mut raw {
            row.left = left;
            row.right = right;
        }
        rows.push(raw);
    }
    rows
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::cli_sequences(kind)
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows[0] = healthy(kind);
    for (index, (left, right)) in signed_pairs().into_iter().enumerate() {
        let mut row = prefix(kind)[1];
        row.left = left;
        row.right = right;
        let slot = index % rows.len();
        rows[slot].push(row);
    }
    rows
}
