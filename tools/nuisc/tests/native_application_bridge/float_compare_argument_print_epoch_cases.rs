#[allow(dead_code)]
#[path = "unary_float_field_epoch_print_cases.rs"]
mod base;
pub use base::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 512;

pub fn source(kind: Kind) -> String {
    include_str!("float_compare_argument_print_epoch.ns").replace("f64", kind.ty())
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
    for (name, marker) in [("quotient", 101), ("positive", 102)] {
        bodies.push((name,format!("  call void @word_print(i64 {marker})\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n")));
    }
    for (name, marker) in [("accept", 103), ("weight", 104)] {
        bodies.push((name, format!("  call void @word_print(i64 {marker})\n  %pick_word = zext i1 %arg0 to i64\n  call void @word_print(i64 %pick_word)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n")));
    }
    let mut sample = baseline[1].1.replace("i64 95", "i64 105");
    sample.push_str("  call void @word_print(i64 %arg1)\n");
    bodies.push(("sample", sample));
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
}

fn quotient(
    value: i64,
    divisor: i64,
    trace: &mut Vec<i64>,
    native: bool,
    counts: &mut [usize; 5],
) -> Option<i64> {
    counts[0] += 1;
    if native {
        trace.extend([101, value, divisor]);
    }
    value.checked_div(divisor)
}
fn positive(
    value: i64,
    divisor: i64,
    trace: &mut Vec<i64>,
    native: bool,
    counts: &mut [usize; 5],
) -> Option<bool> {
    counts[1] += 1;
    if native {
        trace.extend([102, value, divisor]);
    }
    Some(quotient(value, divisor, trace, native, counts)? > 0)
}

fn accept(
    pick: bool,
    value: i64,
    divisor: i64,
    trace: &mut Vec<i64>,
    native: bool,
    counts: &mut [usize; 5],
) -> Option<bool> {
    counts[2] += 1;
    if native {
        trace.extend([103, i64::from(pick), value, divisor]);
    }
    Some(pick == (quotient(value, divisor, trace, native, counts)? > 0))
}
fn weight(
    pick: bool,
    value: i64,
    divisor: i64,
    trace: &mut Vec<i64>,
    native: bool,
    counts: &mut [usize; 5],
) -> Option<i64> {
    counts[3] += 1;
    if native {
        trace.extend([104, i64::from(pick), value, divisor]);
    }
    let checked = quotient(value, divisor, trace, native, counts)?;
    Some(if pick { checked } else { 0 })
}

#[derive(Clone, Copy, Debug)]
pub enum Comparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
pub const COMPARISONS: [Comparison; 6] = [
    Comparison::Eq,
    Comparison::Ne,
    Comparison::Lt,
    Comparison::Le,
    Comparison::Gt,
    Comparison::Ge,
];

pub fn ieee_compare(kind: Kind, left: u64, right: u64, op: Comparison) -> bool {
    let sign = kind.mask();
    let mask = if kind == Kind::F32 {
        0xffff_ffff
    } else {
        u64::MAX
    };
    let exponent = if kind == Kind::F32 {
        0x7f80_0000
    } else {
        0x7ff0_0000_0000_0000
    };
    let fraction = if kind == Kind::F32 {
        0x007f_ffff
    } else {
        0x000f_ffff_ffff_ffff
    };
    let left = left & mask;
    let right = right & mask;
    let nan = |v: u64| v & exponent == exponent && v & fraction != 0;
    let unordered = nan(left) || nan(right);
    let equal = !unordered && (left == right || (left & !sign == 0 && right & !sign == 0));
    let key = |v: u64| if v & sign != 0 { !v & mask } else { v ^ sign };
    let less = !unordered && !equal && key(left) < key(right);
    match op {
        Comparison::Eq => equal,
        Comparison::Ne => !equal,
        Comparison::Lt => less,
        Comparison::Le => equal || less,
        Comparison::Gt => !unordered && !equal && !less,
        Comparison::Ge => !unordered && !less,
    }
}

pub fn comparison_pairs(kind: Kind) -> Vec<(u64, u64)> {
    let sign = kind.mask();
    let finite = kind.initial() as u64;
    let infinity = if kind == Kind::F32 {
        0x7f80_0000
    } else {
        0x7ff0_0000_0000_0000
    };
    let quiet = kind.nan() as u64;
    let signaling = infinity | 1;
    vec![
        (0, sign),
        (sign, 0),
        (1, 0),
        (0, 1),
        (1, sign | 1),
        (sign | 1, 1),
        (finite + 1, finite),
        (finite, finite + 1),
        (infinity, infinity),
        (infinity | sign, infinity | sign),
        (infinity, infinity | sign),
        (infinity | sign, infinity),
        (quiet, 0),
        (0, quiet),
        (signaling, quiet),
        (quiet, signaling),
        (quiet, quiet),
        (signaling | sign, signaling | sign),
    ]
}

fn sample(
    kind: Kind,
    value: u64,
    divisor: i64,
    trace: &mut Vec<i64>,
    native: bool,
    counts: &mut [usize; 5],
) -> Option<u64> {
    counts[4] += 1;
    if native {
        trace.extend([105, value as i64, divisor]);
    }
    0i64.checked_div(divisor)?;
    Some(
        value
            & if kind == Kind::F32 {
                0xffff_ffff
            } else {
                u64::MAX
            },
    )
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.extend([self.operand, self.first, self.second]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{},{}",
            self.base.args(),
            self.kind().decimal(self.operand),
            self.first,
            self.second
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
        counts: &mut [usize; 5],
    ) -> bool {
        let picked = (|| -> Option<(bool, i64)> {
            Some(match role {
                0 => (
                    if flag {
                        let left = sample(self.kind(), current, self.first, trace, native, counts)?;
                        let right = sample(
                            self.kind(),
                            self.operand as u64,
                            self.second,
                            trace,
                            native,
                            counts,
                        )?;
                        let pick = ieee_compare(self.kind(), left, right, Comparison::Eq);
                        accept(pick, tag, self.first, trace, native, counts)?
                    } else {
                        false
                    },
                    1i64.wrapping_add(tag.wrapping_sub(tag)),
                ),
                1 => (
                    if flag {
                        true
                    } else {
                        let left = sample(
                            self.kind(),
                            self.operand as u64,
                            self.first,
                            trace,
                            native,
                            counts,
                        )?;
                        let pick = if ieee_compare(self.kind(), left, current, Comparison::Lt) {
                            true
                        } else {
                            let right = sample(
                                self.kind(),
                                self.operand as u64,
                                self.second,
                                trace,
                                native,
                                counts,
                            )?;
                            if ieee_compare(self.kind(), current, right, Comparison::Le) {
                                let v = quotient(tag, self.first, trace, native, counts)?;
                                positive(v, self.second, trace, native, counts)?
                                    || tag.checked_rem(self.first.wrapping_sub(2))? > 0
                            } else {
                                false
                            }
                        };
                        accept(pick, tag, self.second, trace, native, counts)?
                    },
                    divisor,
                ),
                2 => (
                    if flag {
                        true
                    } else {
                        let right = sample(
                            self.kind(),
                            self.operand as u64,
                            self.first,
                            trace,
                            native,
                            counts,
                        )?;
                        let pick = if ieee_compare(self.kind(), current, right, Comparison::Gt) {
                            true
                        } else {
                            let left =
                                sample(self.kind(), current, self.second, trace, native, counts)?;
                            ieee_compare(self.kind(), left, self.operand as u64, Comparison::Ge)
                                || tag.checked_rem(self.first)? > 0
                        };
                        let value = weight(pick, tag, self.second, trace, native, counts)?;
                        positive(value, self.second, trace, native, counts)?
                    },
                    1i64.wrapping_add(0i64.wrapping_mul(tag)),
                ),
                3 => (
                    if flag {
                        true
                    } else {
                        let left = sample(self.kind(), current, self.first, trace, native, counts)?;
                        let right = sample(
                            self.kind(),
                            self.operand as u64,
                            self.second,
                            trace,
                            native,
                            counts,
                        )?;
                        let pick = ieee_compare(self.kind(), left, right, Comparison::Ne);
                        let value = weight(pick, tag, self.second, trace, native, counts)?;
                        positive(value, self.second, trace, native, counts)?
                    },
                    1i64.wrapping_add(0i64.wrapping_mul(tag)),
                ),
                _ => unreachable!(),
            })
        })();
        let Some((pick, guard)) = picked else {
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
    ) -> Result<(i64, Vec<i64>, [usize; 5]), Vec<i64>> {
        let mut counts = [0usize; 5];
        let row = self.base.row;
        let mut current = previous.unwrap_or(row.initial) as u64;
        let mut tag = 100i64;
        let mut flag = row.selected;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }];
        if native {
            trace.extend([96, self.operand, self.first, self.second]);
        }
        trace.push(70);
        assert!(row.trips <= 3 && row.children <= 3);
        for index in 0..row.trips {
            trace.push(71);
            if !self.observe(&mut trace, native, current, tag, 0, 1, flag, &mut counts) {
                return Err(trace);
            }
            trace.push(tag);
            current ^= self.kind().mask();
            tag = tag.wrapping_add(row.delta);
            flag = !row.selected;
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
                        &mut counts,
                    )
                {
                    return Err(trace);
                }
                if !self.observe(&mut trace, native, local, tag, 2, 1, flag, &mut counts) {
                    return Err(trace);
                }
                trace.push(tag);
                local ^= self.kind().mask();
                tag = tag.wrapping_add(row.delta.wrapping_mul(2));
                flag = row.selected;
                if !self.observe(&mut trace, native, local, tag, 3, 1, flag, &mut counts) {
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
    pub fn helper_counts(self, previous: Option<i64>) -> [usize; 5] {
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
        })
        .collect()
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    wrap(base::prefix(kind))
}
pub fn healthy(kind: Kind) -> Vec<Case> {
    let mut rows = wrap(base::healthy(kind));
    let mut varied = prefix(kind)[1];
    varied.first = 2;
    rows.push(varied);
    let mut skipped = varied;
    skipped.base.row.selected = false;
    skipped.first = 0;
    skipped.second = 0;
    skipped.base.row.trips = 1;
    skipped.base.row.children = 0;
    rows.push(skipped);
    let mut zero = skipped;
    zero.base.row.selected = true;
    zero.base.row.trips = 0;
    rows.push(zero);
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
    rows.push(bad);
    bad.first = 2;
    bad.base.row.trips = 2;
    bad.base.row.children = 0;
    rows.push(bad);
    bad.first = 0;
    bad.second = 1;
    rows.push(bad);
    bad = prefix(kind)[1];
    bad.base.row.delta = i64::MIN.wrapping_sub(100);
    bad.base.row.selected = false;
    bad.base.row.trips = 2;
    bad.base.row.children = 0;
    bad.second = -1;
    let mut child_overflow = bad;
    child_overflow.base.row.trips = 1;
    child_overflow.base.row.children = 1;
    child_overflow.base.row.delta = i64::MIN
        .wrapping_sub(100)
        .wrapping_mul(0xaaaaaaaaaaaaaaabu64 as i64);
    rows.push(child_overflow);
    bad.second = 1;
    bad.first = -1;
    rows.push(bad);
    bad.base.row.trips = 1;
    bad.base.row.children = 1;
    bad.base.row.selected = true;
    bad.first = -1;
    bad.second = 1;
    rows.push(bad);
    bad.first = 1;
    bad.second = 2;
    rows.push(bad);
    bad.base.row.selected = true;
    bad.base.row.trips = 2;
    bad.first = -1;
    bad.second = 1;
    bad.base.row.delta = i64::MIN
        .wrapping_sub(100)
        .wrapping_mul(0xaaaaaaaaaaaaaaabu64 as i64);
    rows.push(bad);
    rows
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::sequences(kind)
        .into_iter()
        .map(wrap)
        .collect::<Vec<_>>();
    rows[0] = healthy(kind);
    for word in [96, 99, 101, 102, 103, 104, 105] {
        let mut raw = prefix(kind);
        raw[0].base.row.initial = word;
        rows.push(raw);
    }
    for (left, right) in comparison_pairs(kind) {
        let mut raw = prefix(kind);
        raw[0].base.row.initial = left as i64;
        for row in &mut raw {
            row.operand = right as i64;
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
    rows
}
