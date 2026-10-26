#[allow(dead_code)]
#[path = "unary_float_field_epoch_print_cases.rs"]
mod base;
pub use base::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 512;

pub fn source(kind: Kind) -> String {
    include_str!("lazy_call_argument_print_epoch.ns").replace("f64", kind.ty())
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
                2 => current - ready,
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
    ) -> bool {
        let picked = (|| -> Option<(bool, i64)> {
            Some(match role {
                0 => (
                    if flag {
                        tag.checked_div(self.second)? >= 100
                    } else {
                        false
                    },
                    1i64.wrapping_add(tag.wrapping_sub(tag)),
                ),
                1 => (
                    if flag {
                        true
                    } else if tag.checked_div(self.first)? > 0 {
                        true
                    } else {
                        tag.checked_rem(self.first.wrapping_sub(2))? > 0
                    },
                    divisor,
                ),
                2 => (
                    if flag {
                        true
                    } else {
                        tag.checked_rem(self.first)? >= 0
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
    fn evaluate(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
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
            if !self.observe(&mut trace, native, current, tag, 0, 1, flag) {
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
                    )
                {
                    return Err(trace);
                }
                if !self.observe(&mut trace, native, local, tag, 2, 1, flag) {
                    return Err(trace);
                }
                trace.push(tag);
                local ^= self.kind().mask();
                tag = tag.wrapping_add(row.delta.wrapping_mul(2));
                flag = row.selected;
                if !self.observe(&mut trace, native, local, tag, 2, 1, flag) {
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
    let mut altered = prefix(kind)[1];
    altered.first = 2;
    rows.push(altered);
    let mut skipped = altered;
    skipped.base.row.selected = false;
    skipped.base.row.divisor = 0;
    skipped.second = 0;
    skipped.base.row.children = 1;
    rows.push(skipped);
    let mut nochild = altered;
    nochild.first = 0;
    nochild.base.row.children = 0;
    rows.push(nochild);
    let mut short = skipped;
    short.first = 0;
    short.base.row.trips = 1;
    short.base.row.children = 0;
    rows.push(short);
    let mut zero = altered;
    zero.base.row.trips = 0;
    zero.first = 0;
    zero.second = 0;
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
    bad = prefix(kind)[1];
    bad.base.row.delta = i64::MIN.wrapping_sub(100);
    bad.base.row.selected = false;
    bad.base.row.trips = 2;
    bad.base.row.children = 0;
    bad.second = -1;
    rows.push(bad);
    bad.base.row.trips = 1;
    bad.base.row.children = 1;
    bad.base.row.selected = true;
    bad.first = -1;
    bad.second = 1;
    rows.push(bad);
    bad.first = 1;
    rows.push(bad);
    bad.base.row.selected = false;
    bad.first = -1;
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
    for word in [96, 99] {
        let mut raw = prefix(kind);
        raw[0].base.row.initial = word;
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
