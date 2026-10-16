#[path = "state_loop_cases.rs"]
mod base;
#[allow(unused_imports)]
pub use base::{State, CLI_MARKERS, MARKERS};

pub const SOURCE: &str = include_str!("state_loop_exits.ns");
pub const LOOP_LIMIT: u64 = 3;
pub const WORK_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  call void @nuis_debug_print_i64(i64 %arg2)\n";
pub const INDEX_OBSERVER: &str =
    "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n";
pub const ANSWER_OBSERVER: &str = base::OBSERVER;

#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub leading: bool,
    pub mirrored: bool,
}
impl Variant {
    pub fn source(self) -> String {
        let mut source = SOURCE.to_owned();
        if self.leading {
            assert_eq!(
                source.matches("            index = index + 1;\n").count(),
                2
            );
            source = source.replace("            index = index + 1;\n", "");
            assert_eq!(source.matches("while index < trips {").count(), 1);
            source = source.replace(
                "while index < trips {",
                "while index < trips {\n            index = index + 1;",
            );
        }
        if self.mirrored {
            assert_eq!(source.matches("if index == cut {").count(), 1);
            source = source.replace("if index == cut {", "if index != cut {} else {");
        }
        source
    }
}
pub fn variants() -> [Variant; 4] {
    std::array::from_fn(|n| Variant {
        leading: n >= 2,
        mirrored: n % 2 != 0,
    })
}

#[derive(Clone, Copy)]
pub struct Case {
    pub seed: i64,
    pub gate: bool,
    pub flag: bool,
    pub trips: i64,
    pub cut: i64,
    pub marker: u64,
}
#[derive(Clone, Copy)]
pub struct Invocation {
    pub input: Case,
    pub previous: Option<State>,
    pub leading: bool,
    pub root: i64,
    pub hole: i64,
}
impl Invocation {
    pub fn prior(self) -> bool {
        self.previous.map_or(self.input.flag, |s| s.result)
    }
    pub fn entering(self) -> i64 {
        self.previous.map_or(self.input.seed, |s| s.total)
    }

    // Model source control directly, not generated flags or helper transport.
    pub fn execute(
        self,
        bad: Option<usize>,
        observed: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let mut trace = vec![];
        if bad == Some(9) {
            return Err(trace);
        }
        trace.push(5);
        let mut current = self.entering() + 1;
        let mut index = 0;
        while index < self.input.trips {
            if self.leading {
                index += 1;
            }
            if self.root == 0 || bad == Some(4) {
                return Err(trace);
            }
            trace.push(10);
            current += 1 / self.root;
            if index == self.input.cut {
                trace.push(11);
                if self.input.gate ^ self.prior() {
                    trace.push(12);
                    break;
                }
                trace.push(13);
                if !self.leading {
                    index += 1;
                }
                continue;
            }
            trace.push(19);
            if index == self.hole {
                return Err(trace);
            }
            let proof = 1 / (index - self.hole);
            if observed {
                trace.extend([current, proof, index]);
            }
            current = 2 * current + proof + index;
            trace.push(20);
            if !self.leading {
                index += 1;
            }
        }
        trace.push(31);
        if observed {
            trace.extend([current, index]);
        }
        let value = current + 10 * index;
        if bad == Some(6) {
            return Err(trace);
        }
        trace.push(35);
        let total = value + 3;
        if bad == Some(7) {
            return Err(trace);
        }
        if observed {
            trace.extend([total, self.prior().into(), 1]);
        }
        trace.push(40);
        Ok((
            State {
                total,
                result: self.prior() ^ (total % 2 == 0),
                marker: self.previous.map_or(self.input.marker, |s| s.marker),
            },
            trace,
        ))
    }
    pub fn next(self) -> State {
        self.execute(None, false).unwrap().0
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        self.execute(None, observed).unwrap().1
    }
    pub fn words(self, bad: Option<usize>) -> Vec<i64> {
        let mut words = vec![
            self.input.seed,
            self.input.gate.into(),
            self.input.flag.into(),
            self.input.trips,
            self.root,
            self.hole,
            1,
            1,
            self.input.marker as i64,
            1,
            self.input.cut,
        ];
        if let Some(slot) = bad {
            assert!(matches!(slot, 4 | 6 | 7 | 9));
            assert_eq!(words[slot], 1);
            words[slot] = 0;
        }
        words
    }
    pub fn args(self, bad: Option<usize>) -> String {
        self.words(bad)
            .into_iter()
            .enumerate()
            .map(|(slot, word)| match slot {
                1..=2 => (word != 0).to_string(),
                8 => f64::from_bits(word as u64).to_string(),
                _ => word.to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}
pub fn cases() -> Vec<Case> {
    let mut cases = vec![];
    for seed in [-3, 8] {
        for gate in [false, true] {
            for flag in [false, true] {
                for trips in [-1, 0, 1, 3] {
                    for cut in [0, 1, 2] {
                        cases.push(Case {
                            seed,
                            gate,
                            flag,
                            trips,
                            cut,
                            marker: 0,
                        });
                    }
                }
            }
        }
    }
    cases
}
pub fn invocation(
    input: Case,
    previous: Option<State>,
    leading: bool,
    poisoned: bool,
) -> Invocation {
    Invocation {
        input,
        previous,
        leading,
        root: if poisoned && input.trips <= 0 { 0 } else { 1 },
        hole: if poisoned && (input.cut > 0 || !leading) && input.cut <= input.trips {
            input.cut
        } else {
            -1
        },
    }
}
pub fn sequence(open: Case, leading: bool, poisoned: bool) -> Vec<Invocation> {
    let mut previous = None;
    std::iter::once(open)
        .chain(std::iter::repeat_n(
            Case {
                marker: 123.5f64.to_bits(),
                ..open
            },
            3,
        ))
        .map(|input| {
            let call = invocation(input, previous, leading, poisoned);
            previous = Some(call.next());
            call
        })
        .collect()
}
pub fn failure_prefix(leading: bool) -> Vec<Invocation> {
    sequence(
        Case {
            seed: 8,
            gate: false,
            flag: false,
            trips: 0,
            cut: 1,
            marker: 0x8000000000000000,
        },
        leading,
        true,
    )
    .into_iter()
    .take(3)
    .collect()
}
pub fn failures(leading: bool) -> Vec<(Invocation, Option<usize>)> {
    let previous = failure_prefix(leading).last().unwrap().next();
    let mut failures = vec![];
    for gate in [false, true] {
        let call = invocation(
            Case {
                seed: -3,
                gate,
                flag: true,
                trips: 3,
                cut: 1,
                marker: 123.5f64.to_bits(),
            },
            Some(previous),
            leading,
            false,
        );
        for slot in [4, 6, 7, 9] {
            failures.push((call, Some(slot)));
        }
        for hole in if leading { 1..4 } else { 0..3 } {
            let bad = Invocation { hole, ..call };
            if bad.execute(None, false).is_err() {
                failures.push((bad, None));
            }
        }
    }
    failures
}
