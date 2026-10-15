pub const SOURCE: &str = include_str!("return_stateful_fields.ns");
pub const OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %seen_previous = zext i1 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %seen_previous)\n  call void @nuis_debug_print_i64(i64 %arg2)\n";
pub const MARKERS: [u64; 4] = [0, 0x8000000000000000, 1, 0xfff8000012345678];
pub const CLI_MARKERS: [u64; 4] = [0, 0x8000000000000000, 1, 0xc004000000000000];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub result: bool,
    pub total: i64,
    pub marker: u64,
}

impl State {
    pub fn words(self) -> Vec<i64> {
        vec![self.result.into(), self.total, self.marker as i64]
    }
    pub fn text(self, phase: &str) -> String {
        format!(
            "{phase}:State{{result: {}, total: {}, marker: {}f64}}",
            self.result,
            self.total,
            f64::from_bits(self.marker)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Case {
    pub seed: i64,
    pub gate: bool,
    pub first: bool,
    pub flag: bool,
    pub marker: u64,
}

impl Case {
    pub fn result(self, previous: bool, new_total: i64) -> bool {
        previous
            ^ if self.gate {
                !self.first || new_total % 2 != 0
            } else {
                self.first
            }
    }
    pub fn initial(self) -> State {
        let total = self.seed + 1;
        State {
            result: self.result(self.flag, total),
            total,
            marker: self.marker,
        }
    }
    pub fn step(self, state: State) -> State {
        let total = state.total + 1;
        State {
            result: self.result(state.result, total),
            total,
            marker: state.marker,
        }
    }
    pub fn words(self) -> Vec<i64> {
        vec![
            self.seed,
            self.gate.into(),
            self.first.into(),
            self.flag.into(),
            1,
            self.gate.into(),
            (!self.gate).into(),
            self.gate.into(),
            (!self.gate).into(),
            (self.gate && self.first).into(),
            (!self.gate && self.first).into(),
            self.gate.into(),
            self.marker as i64,
            1,
        ]
    }
    pub fn args(self, bad: Option<usize>) -> String {
        let mut words = self.words();
        if let Some(slot) = bad {
            assert_eq!(words[slot], 1);
            words[slot] = 0;
        }
        words
            .into_iter()
            .enumerate()
            .map(|(i, n)| {
                if (1..=3).contains(&i) {
                    (n != 0).to_string()
                } else if i == 12 {
                    f64::from_bits(n as u64).to_string()
                } else {
                    n.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn trace(self, previous: bool, total: i64, observed: bool) -> Vec<i64> {
        let mut out = vec![5, 10, if self.gate { 20 } else { 30 }];
        if self.gate {
            out.push(50);
        }
        out.push(11);
        if self.first {
            if self.gate {
                if observed {
                    out.extend([total + 1, previous.into(), 1]);
                }
                out.push(40);
            } else {
                out.push(31);
            }
        }
        out
    }
    pub fn before_failure(self, slot: usize) -> Vec<i64> {
        if slot == 13 {
            return vec![];
        }
        let mut out = vec![5];
        if slot == 4 {
            return out;
        }
        out.push(10);
        if matches!(slot, 5 | 6) {
            return out;
        }
        out.push(if self.gate { 20 } else { 30 });
        if slot == 11 {
            return out;
        }
        if self.gate {
            out.push(50);
        }
        if matches!(slot, 7 | 8) {
            return out;
        }
        assert!(matches!(slot, 9 | 10));
        out.push(11);
        out
    }
    pub fn events(self) -> [Self; 5] {
        [toggle(), self, self, toggle(), self].map(|c| Self {
            marker: 123.5f64.to_bits(),
            ..c
        })
    }
}

pub fn toggle() -> Case {
    Case {
        seed: 8,
        gate: true,
        first: false,
        flag: false,
        marker: 0,
    }
}
pub fn identity() -> Case {
    Case {
        seed: 7,
        gate: false,
        first: false,
        flag: false,
        marker: 0,
    }
}
pub fn failure_prefix() -> [Case; 3] {
    [
        Case {
            marker: 0x8000000000000000,
            ..identity()
        },
        toggle(),
        identity(),
    ]
}
pub fn cases() -> Vec<Case> {
    let mut out = vec![];
    for seed in [7, 8] {
        for gate in [false, true] {
            for first in [false, true] {
                for flag in [false, true] {
                    out.push(Case {
                        seed,
                        gate,
                        first,
                        flag,
                        marker: 0,
                    });
                }
            }
        }
    }
    out
}
pub fn failures() -> Vec<(Case, usize)> {
    (4..=11)
        .chain([13])
        .map(|slot| {
            (
                Case {
                    seed: 7,
                    gate: !matches!(slot, 6 | 8 | 10),
                    first: true,
                    flag: false,
                    marker: 123.5f64.to_bits(),
                },
                slot,
            )
        })
        .collect()
}
pub fn source_variants() -> [String; 4] {
    let mut wrapped = SOURCE.to_owned();
    for value in [
        "true",
        "flag",
        "first",
        "flag != true",
        "seal(input, enabled, 1 / sealing)",
        "answer_work(local, input, 1 / answer)",
    ] {
        wrapped = wrapped.replace(
            &format!(": bool = {value};"),
            &format!(": bool = ({value}) && true;"),
        );
    }
    [
        SOURCE.to_owned(),
        mirror(SOURCE),
        wrapped.clone(),
        mirror(&wrapped),
    ]
}
fn mirror(source: &str) -> String {
    let start = source.find("        if choose(gate, 1 / root) {").unwrap();
    let yes = start + "        if choose(gate, 1 / root) {".len();
    let middle = yes + source[yes..].find("\n        } else {").unwrap();
    let no = middle + "\n        } else {".len();
    let end = no + source[no..].find("        }\n    }\n    fn start").unwrap();
    format!(
        "{}        if choose(gate != true, 1 / root) {{{}        }} else {{{}{}",
        &source[..start],
        &source[no..end],
        &source[yes..middle],
        &source[end..]
    )
}
