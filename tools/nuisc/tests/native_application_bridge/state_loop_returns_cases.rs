#[path = "return_stateful_fields_cases.rs"]
mod base;
#[allow(unused_imports)]
pub use base::{State, CLI_MARKERS, MARKERS};
pub const SOURCE: &str = include_str!("state_loop_returns.ns");
pub const LOOP_LIMIT: u64 = 9;
pub const RETURN_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  call void @nuis_debug_print_i64(i64 %arg2)\n  call void @nuis_debug_print_i64(i64 %arg3)\n";
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
                .lines()
                .filter(|l| l.trim() != "index = index + 1;")
                .collect::<Vec<_>>()
                .join("\n");
            source = source.replace(
                "while index < trips {",
                "while index < trips {\n                index = index + 1;",
            );
        }
        if self.mirrored {
            source = source
                .replace("if index == cut {", "if index != cut {} else {")
                .replace(
                    "if gate { print(12);",
                    "if gate != true {} else { print(12);",
                )
                .replace("if outer == parent {", "if outer != parent {} else {");
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

#[derive(Clone, Copy, Debug)]
pub struct Invocation {
    pub seed: i64,
    pub gate: bool,
    pub flag: bool,
    pub trips: i64,
    pub root: i64,
    pub hole: i64,
    pub marker: u64,
    pub cut: i64,
    pub rounds: i64,
    pub parent: i64,
    pub leading: bool,
    pub previous: Option<State>,
}
fn reserve(bound: i64, remaining: &mut Option<u64>) -> bool {
    let cost = bound.max(0) as u64;
    if let Some(left) = remaining {
        if cost > *left {
            return false;
        }
        *left -= cost;
    }
    true
}
impl Invocation {
    pub fn words(self, bad: Option<usize>) -> Vec<i64> {
        let mut words = vec![
            self.seed,
            self.gate.into(),
            self.flag.into(),
            self.trips,
            self.root,
            self.hole,
            1,
            1,
            self.marker as i64,
            1,
            self.cut,
            self.rounds,
            self.parent,
        ];
        if let Some(slot) = bad {
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
    // Source-level oracle: return leaves this Rust closure immediately. It does
    // not model the compiler's pending bit, generated breaks or result storage.
    pub fn execute(
        self,
        bad: Option<usize>,
        observed: bool,
        mut budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let mut trace = vec![];
        if bad == Some(9) {
            return Err(trace);
        }
        trace.push(5);
        let prior = self.previous.map_or(self.flag, |s| s.result);
        let mut current = self.previous.map_or(self.seed, |s| s.total) + 1;
        let gate = self.gate ^ prior;
        let walk = (|| -> Option<i64> {
            let mut outer = 0;
            if !reserve(self.rounds, &mut budget) {
                return None;
            }
            while outer < self.rounds {
                outer += 1;
                trace.push(8);
                current += outer;
                let mut index = 0;
                if !reserve(self.trips, &mut budget) {
                    return None;
                }
                while index < self.trips {
                    if self.leading {
                        index += 1;
                    }
                    trace.push(10);
                    current += 1;
                    if index == self.cut {
                        trace.push(11);
                        if gate {
                            trace.push(12);
                            return returned(
                                current, index, outer, self.root, observed, &mut trace,
                            );
                        }
                        trace.push(13);
                        if !self.leading {
                            index += 1;
                        }
                        continue;
                    }
                    trace.push(19);
                    if index == self.hole {
                        return None;
                    }
                    current = 2 * current + 1 / (index - self.hole) + index;
                    trace.push(20);
                    if !self.leading {
                        index += 1;
                    }
                }
                trace.push(23);
                if outer == self.parent {
                    trace.push(24);
                    return returned(current, index, outer, self.root, observed, &mut trace);
                }
                trace.push(21);
                current += 3;
                trace.push(22);
            }
            trace.push(31);
            Some(current + 10 * outer)
        })();
        let Some(current) = walk else {
            return Err(trace);
        };
        if bad == Some(6) {
            return Err(trace);
        }
        trace.push(35);
        let total = current + 3;
        if bad == Some(7) {
            return Err(trace);
        }
        if observed {
            trace.extend([total, prior.into(), 1]);
        }
        trace.push(40);
        Ok((
            State {
                result: prior ^ (total % 2 == 0),
                total,
                marker: self.previous.map_or(self.marker, |s| s.marker),
            },
            trace,
        ))
    }
    pub fn next(self) -> State {
        self.execute(None, false, None).unwrap().0
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        self.execute(None, observed, None).unwrap().1
    }
}
fn returned(
    value: i64,
    index: i64,
    outer: i64,
    root: i64,
    observed: bool,
    trace: &mut Vec<i64>,
) -> Option<i64> {
    if root == 0 {
        return None;
    }
    let proof = 1 / root;
    if observed {
        trace.extend([value, index, outer, proof]);
    }
    trace.push(30);
    if proof == 0 {
        return None;
    }
    Some(value + 10 * index + 100 * outer + 1 / proof)
}
pub fn base(leading: bool) -> Invocation {
    Invocation {
        seed: 4,
        gate: true,
        flag: false,
        trips: 3,
        root: 1,
        hole: -1,
        marker: 0x8000000000000000,
        cut: if leading { 1 } else { 0 },
        rounds: 2,
        parent: -1,
        leading,
        previous: None,
    }
}
pub fn sequence(open: Invocation) -> Vec<Invocation> {
    let mut rows = vec![open];
    for (gate, trips, parent) in [(false, 2, 1), (true, 0, 1), (false, 2, -1)] {
        let previous = Some(rows.last().unwrap().next());
        rows.push(Invocation {
            previous,
            seed: -99,
            flag: !open.flag,
            gate,
            trips,
            parent,
            marker: 123.5f64.to_bits(),
            ..open
        });
    }
    rows
}
pub fn cases(leading: bool) -> Vec<Invocation> {
    let mut rows = vec![];
    for seed in [-3, 4] {
        for gate in [false, true] {
            for flag in [false, true] {
                for trips in [-1, 0, 1, 3] {
                    for parent in [-1, 1] {
                        rows.push(Invocation {
                            seed,
                            gate,
                            flag,
                            trips,
                            parent,
                            ..base(leading)
                        });
                    }
                }
            }
        }
    }
    rows
}
pub fn failures(leading: bool) -> Vec<(Invocation, Option<usize>)> {
    let base = base(leading);
    let mut rows = vec![];
    for parent in [-1, 1] {
        for root in [0, 2] {
            rows.push((
                Invocation {
                    root,
                    trips: if parent == 1 { 0 } else { 3 },
                    parent,
                    ..base
                },
                None,
            ));
        }
    }
    rows.push((
        Invocation {
            hole: if leading { 1 } else { 0 },
            cut: 99,
            ..base
        },
        None,
    ));
    for slot in [6, 7, 9] {
        rows.push((base, Some(slot)));
    }
    rows
}

pub fn failure_prefix(leading: bool) -> Vec<Invocation> {
    let mut rows = vec![];
    for _ in 0..3 {
        let previous = rows.last().map(|call: &Invocation| call.next());
        rows.push(Invocation {
            rounds: 0,
            trips: i64::MAX,
            root: 0,
            previous,
            ..base(leading)
        });
    }
    rows
}

pub fn after_prefix(mut call: Invocation) -> Invocation {
    let last = failure_prefix(call.leading).last().unwrap().next();
    call.gate = call.gate ^ call.flag ^ last.result;
    call.previous = Some(last);
    call
}
