pub const SOURCE: &str = include_str!("return_typed_exit_prefix.ns");
pub const PLAIN_SOURCE: &str = include_str!("return_plain_exit_prefix.ns");
pub const STAGES: [i64; 12] = [11, 14, 20, 30, 21, 22, 23, 24, 27, 29, 28, 25];
pub const OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %small_word = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %small_word)\n  %narrow_word = bitcast float %arg2 to i32\n  %narrow_raw = zext i32 %narrow_word to i64\n  call void @nuis_debug_print_i64(i64 %narrow_raw)\n  %wide_raw = bitcast double %arg3 to i64\n  call void @nuis_debug_print_i64(i64 %wide_raw)\n  %flag_word = zext i1 %arg4 to i64\n  call void @nuis_debug_print_i64(i64 %flag_word)\n  %peer_word = zext i1 %arg5 to i64\n  call void @nuis_debug_print_i64(i64 %peer_word)\n";

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub seed: i64,
    pub small: i32,
    pub narrow: u32,
    pub wide: u64,
    pub flag: bool,
    pub peer: bool,
    pub pick: bool,
    pub take: bool,
    pub inner: bool,
}

pub fn answer([owner, a, b, c, d, e]: [i64; 6]) -> bool {
    owner == 9
        && ((a < 0)
            ^ (d != 0)
            ^ (f32::from_bits(b as u32) < 0.0)
            ^ (f64::from_bits(c as u64) < 0.0)
            ^ (e != 0))
}

impl Case {
    pub fn before(self) -> [i64; 6] {
        [
            self.seed,
            i64::from(!self.small),
            i64::from(self.narrow ^ (1 << 31)),
            (self.wide ^ (1 << 63)) as i64,
            (!self.flag).into(),
            (!self.peer).into(),
        ]
    }

    pub fn fields(self) -> [i64; 6] {
        let [mut owner, mut a, mut b, mut c, mut d, mut e] = self.before();
        owner += if self.take && !self.pick { 2 } else { 1 };
        if self.take && self.inner {
            if !self.pick {
                if f32::from_bits(b as u32) < 0.0 {
                    c ^= i64::MIN;
                }
                d ^= i64::from(f64::from_bits(c as u64) < 0.0);
            }
            a = i64::from(!(a as i32));
            if a >= 0 {
                b ^= 1 << 31;
            }
            if self.pick {
                if f32::from_bits(b as u32) < 0.0 {
                    c ^= i64::MIN;
                }
                d ^= i64::from(f64::from_bits(c as u64) < 0.0);
            }
            e ^= d;
        }
        [owner, a, b, c, d, e]
    }

    pub fn result(self) -> i64 {
        answer(self.fields()).into()
    }

    pub fn next(self) -> Self {
        Self {
            pick: !self.pick,
            take: !self.take,
            inner: !self.inner,
            ..self
        }
    }

    pub fn trace(self, observed: bool) -> Vec<i64> {
        let mut trace = vec![10, 11];
        trace.extend(if self.pick {
            [12, 13, 15, 16, 17]
        } else {
            [17, 16, 15, 13, 12]
        });
        trace.push(14);
        if self.take {
            trace.extend([if self.pick { 20 } else { 30 }, 21]);
            if self.inner {
                trace.extend(if self.pick {
                    [22, 23, 24, 27, 29]
                } else {
                    [24, 27, 22, 23, 29]
                });
            }
        } else {
            trace.extend([25, 26]);
        }
        if observed {
            trace.extend(self.fields());
        }
        trace.push(28);
        trace
    }

    pub fn words(self) -> Vec<i64> {
        [
            self.seed,
            i64::from(self.small),
            self.narrow.into(),
            self.wide as i64,
            self.flag.into(),
            self.peer.into(),
            self.pick.into(),
            self.take.into(),
            self.inner.into(),
        ]
        .into_iter()
        .chain(STAGES.map(|stage| i64::from(self.trace(false).contains(&stage))))
        .collect()
    }

    pub fn args(self, bad: Option<usize>) -> String {
        let mut words = self.words();
        if let Some(slot) = bad {
            assert_eq!(words[slot], 1);
            words[slot] = 0;
        }
        words
            .iter()
            .enumerate()
            .map(|(i, n)| match i {
                2 => f32::from_bits(*n as u32).to_string(),
                3 => f64::from_bits(*n as u64).to_string(),
                4..=8 => (*n != 0).to_string(),
                _ => n.to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn state(self, phase: &str) -> String {
        format!("{phase}:State{{result: {}}}", self.result() != 0)
    }

    pub fn before_failure(self, stage: i64) -> Vec<i64> {
        let source = self.trace(false);
        let cut = source.iter().position(|&v| v == stage).unwrap();
        // Every poisoned operand is evaluated before the answer-callee probe.
        source[..cut].to_vec()
    }
}

pub fn mirror_source() -> String {
    mirror(SOURCE)
}

pub fn source_variants() -> [String; 4] {
    [
        SOURCE.to_owned(),
        mirror_source(),
        PLAIN_SOURCE.to_owned(),
        mirror(PLAIN_SOURCE),
    ]
}

fn mirror(source: &str) -> String {
    let start = source
        .find("            if select(pick, 1 / branch) {")
        .unwrap();
    let yes = start + "            if select(pick, 1 / branch) {".len();
    let middle = yes + source[yes..].find("            } else {").unwrap();
    let no = middle + "            } else {".len();
    let end = no
        + source[no..]
            .find("            }\n            let finished")
            .unwrap();
    format!(
        "{}            if select(pick != true, 1 / branch) {{{}            }} else {{{}{}",
        &source[..start],
        &source[no..end],
        &source[yes..middle],
        &source[end..]
    )
}

fn base(pick: bool, take: bool, inner: bool, flag: bool, peer: bool) -> Case {
    Case {
        seed: if pick { 8 } else { 7 },
        small: -2,
        narrow: 1.5_f32.to_bits(),
        wide: 2.5_f64.to_bits(),
        flag,
        peer,
        pick,
        take,
        inner,
    }
}

pub fn cli_cases() -> Vec<Case> {
    let mut out = Vec::new();
    for pick in [false, true] {
        for take in [false, true] {
            for inner in [false, true] {
                for peer in [false, true] {
                    out.push(base(pick, take, inner, false, peer));
                }
            }
        }
    }
    out
}

pub fn witnesses() -> Vec<Case> {
    let mut out = Vec::new();
    for pick in [false, true] {
        for bits in 0..32 {
            let mut c = base(pick, true, true, bits & 8 != 0, bits & 16 != 0);
            c.small = if bits & 1 == 0 { -2 } else { 2 };
            c.narrow = (if bits & 2 == 0 { 1.5_f32 } else { -1.5_f32 }).to_bits();
            c.wide = (if bits & 4 == 0 { 2.5_f64 } else { -2.5_f64 }).to_bits();
            out.push(c);
        }
    }
    out
}

pub fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for seed in [7, 8] {
        for pick in [false, true] {
            for take in [false, true] {
                for inner in [false, true] {
                    for flag in [false, true] {
                        for peer in [false, true] {
                            let mut c = base(pick, take, inner, flag, peer);
                            c.seed = seed;
                            c.small = if flag { i32::MIN } else { i32::MAX };
                            out.push(c);
                        }
                    }
                }
            }
        }
    }
    out.extend(witnesses());
    let narrow = [
        0, 0x80000000, 1, 0x80000001, 0x7f800000, 0xff800000, 0x7fc01234, 0xff801234,
    ];
    let wide = [
        0,
        0x8000000000000000,
        1,
        0x8000000000000001,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff8000000001234,
        0xfff0000000001234,
    ];
    for pick in [false, true] {
        for inner in [false, true] {
            for i in 0..8 {
                let mut c = base(pick, true, inner, i % 2 == 0, i % 3 == 0);
                c.narrow = narrow[i];
                c.wide = wide[(i + 3) % 8];
                out.push(c);
            }
        }
    }
    out
}

pub fn delivery_cases() -> Vec<Case> {
    let mut rows = cli_cases();
    for field in 0..6 {
        rows.push(
            witnesses()
                .into_iter()
                .find(|c| {
                    let current = c.fields();
                    let mut stale = current;
                    stale[field] = c.before()[field];
                    answer(current) != answer(stale)
                })
                .unwrap(),
        );
    }
    rows
}

pub fn failures() -> Vec<(Case, usize, i64)> {
    STAGES
        .into_iter()
        .enumerate()
        .map(|(i, stage)| {
            let pick = if stage == 20 {
                true
            } else if stage == 30 {
                false
            } else {
                i % 2 == 0
            };
            (base(pick, stage != 25, true, false, true), 9 + i, stage)
        })
        .collect()
}
