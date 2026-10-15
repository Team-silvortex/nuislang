pub const SOURCE: &str = include_str!("return_plain_const.ns");
pub const OBSERVER: &str =
    "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n";

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub seed: i64,
    pub gate: bool,
    pub first: bool,
    pub flag: bool,
}

impl Case {
    pub fn result(self) -> i64 {
        if self.gate {
            if self.first {
                (self.seed + 1 == 9).into()
            } else {
                (!self.flag).into()
            }
        } else {
            self.first.into()
        }
    }

    pub fn next(self) -> Self {
        Self {
            seed: 17 - self.seed,
            gate: !self.gate,
            first: !self.first,
            flag: !self.flag,
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
        ]
    }

    pub fn trace(self, observed: bool) -> Vec<i64> {
        let mut out = vec![10, if self.gate { 20 } else { 30 }];
        if self.gate {
            out.push(50);
        }
        out.push(11);
        if self.first {
            if self.gate {
                if observed {
                    out.extend([self.seed + 1, 1]);
                }
                out.push(40);
            } else {
                out.push(31);
            }
        }
        out
    }

    pub fn before_failure(self, slot: usize) -> Vec<i64> {
        let mut out = vec![];
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

    pub fn args(self, bad: Option<usize>) -> String {
        let mut words = self.words();
        if let Some(slot) = bad {
            assert_eq!(words[slot], 1);
            words[slot] = 0;
        }
        words
            .into_iter()
            .enumerate()
            .map(|(i, w)| {
                if (1..=3).contains(&i) {
                    (w != 0).to_string()
                } else {
                    w.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn state(self, phase: &str) -> String {
        format!("{phase}:State{{result: {}}}", self.result() != 0)
    }
}

pub fn cases() -> Vec<Case> {
    let mut out = vec![];
    for seed in [8, 9] {
        for gate in [false, true] {
            for first in [false, true] {
                for flag in [false, true] {
                    out.push(Case {
                        seed,
                        gate,
                        first,
                        flag,
                    });
                }
            }
        }
    }
    out
}

pub fn failures() -> Vec<(Case, usize)> {
    (4..=11)
        .map(|slot| {
            let gate = !matches!(slot, 6 | 8 | 10);
            (
                Case {
                    seed: 9,
                    gate,
                    first: true,
                    flag: false,
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
        "false",
        "first",
        "flag",
        "seal(input, enabled, 1 / sealing)",
        "answer_work(local, 1 / answer)",
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
