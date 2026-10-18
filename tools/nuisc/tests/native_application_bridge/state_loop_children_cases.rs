#[path = "return_stateful_fields_cases.rs"]
mod base;
#[allow(unused_imports)]
pub use base::{State, CLI_MARKERS, MARKERS};

pub const SOURCE: &str = include_str!("state_loop_children.ns");
pub const LOOP_LIMIT: u64 = 8;
pub const WORK_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  call void @nuis_debug_print_i64(i64 %arg2)\n";
pub const COMBINE_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  call void @nuis_debug_print_i64(i64 %arg2)\n  call void @nuis_debug_print_i64(i64 %arg3)\n";
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
            assert_eq!(source.matches("index = index + 1;").count(), 2);
            source = source
                .replace("                    index = index + 1;\n", "")
                .replace("                index = index + 1;\n", "");
            assert_eq!(source.matches("while index < trips {").count(), 1);
            source = source.replace(
                "while index < trips {",
                "while index < trips {\n                index = index + 1;",
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
    pub rounds: i64,
    pub marker: u64,
}
#[derive(Clone, Copy)]
pub struct Invocation {
    pub input: Case,
    pub previous: Option<State>,
    pub leading: bool,
    pub root: i64,
    pub hole: i64,
    pub suffix: i64,
}

// Reserve a declared source bound, independently of normalized flag transport.
fn reserve(bound: i64, remaining: &mut Option<u64>) -> bool {
    let work = bound.max(0) as u64;
    if let Some(left) = remaining {
        if work > *left {
            return false;
        }
        *left -= work;
    }
    true
}
impl Invocation {
    pub fn prior(self) -> bool {
        self.previous.map_or(self.input.flag, |s| s.result)
    }
    pub fn entering(self) -> i64 {
        self.previous.map_or(self.input.seed, |s| s.total)
    }
    pub fn execute(
        self,
        bad: Option<usize>,
        observed: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_with_budget(bad, observed, None)
    }
    pub fn execute_with_budget(
        self,
        bad: Option<usize>,
        observed: bool,
        mut remaining: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let mut trace = vec![];
        if bad == Some(9) {
            return Err(trace);
        }
        trace.push(5);
        let mut current = self.entering() + 1;
        let mut outer = 0;
        if !reserve(self.input.rounds, &mut remaining) {
            return Err(trace);
        }
        while outer < self.input.rounds {
            outer += 1;
            trace.push(8);
            current += outer;
            let local_gate = self.input.gate ^ self.prior() ^ (outer % 2 == 0);
            let mut index = 0;
            if !reserve(self.input.trips, &mut remaining) {
                return Err(trace);
            }
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
                    if local_gate {
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
            trace.push(21);
            if outer == self.suffix {
                return Err(trace);
            }
            let proof = 1 / (outer - self.suffix);
            if observed {
                trace.extend([current, index, outer, proof]);
            }
            current += 10 * index + 100 * outer + proof;
            trace.push(22);
        }
        trace.push(31);
        if observed {
            trace.extend([current, outer]);
        }
        let value = current + 10 * outer;
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
                result: self.prior() ^ (total % 2 == 0),
                total,
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
            self.input.rounds,
            self.suffix,
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
                    for rounds in [0, 1, 2] {
                        cases.push(Case {
                            seed,
                            gate,
                            flag,
                            trips,
                            cut: 1,
                            rounds,
                            marker: 0,
                        });
                    }
                }
            }
        }
    }
    for gate in [false, true] {
        for (rounds, cut) in [(-1, 1), (2, 2)] {
            cases.push(Case {
                seed: 8,
                gate,
                flag: false,
                trips: 3,
                cut,
                rounds,
                marker: 0,
            });
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
        root: if poisoned && (input.rounds <= 0 || input.trips <= 0) {
            0
        } else {
            1
        },
        hole: if poisoned { input.cut } else { -1 },
        suffix: if poisoned && input.rounds <= 0 { 1 } else { -1 },
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
            rounds: 0,
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
                rounds: 2,
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
        for suffix in [1, 2] {
            failures.push((Invocation { suffix, ..call }, None));
        }
    }
    failures
}
