#[path = "state_join_cases.rs"]
mod base;
#[allow(unused_imports)]
pub use base::{State, CLI_MARKERS, MARKERS, OBSERVER};
pub const SOURCE: &str = include_str!("state_loop.ns");
pub const EVENTS: usize = 3;
pub const LOOP_LIMIT: u64 = 3;
pub const WORK_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  call void @nuis_debug_print_i64(i64 %arg2)\n  call void @nuis_debug_print_i64(i64 20)\n";
pub const CONDITION_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 10)\n";

#[derive(Clone, Copy)]
pub struct Case {
    pub seed: i64,
    pub gate: bool,
    pub flag: bool,
    pub trips: i64,
    pub marker: u64,
}
#[derive(Clone, Copy)]
pub struct Invocation {
    pub input: Case,
    pub previous: Option<State>,
    pub root: i64,
    pub hole: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct Iteration {
    pub index: i64,
    pub entering: i64,
    pub selected: bool,
    pub proof: i64,
    pub current: i64,
}
impl Invocation {
    pub fn prior(self) -> bool {
        self.previous.map_or(self.input.flag, |s| s.result)
    }
    pub fn entering(self) -> i64 {
        self.previous.map_or(self.input.seed, |s| s.total)
    }
    pub fn iterations(self) -> Result<Vec<Iteration>, Vec<Iteration>> {
        let mut rows = vec![];
        let mut current = self.entering() + 1;
        for index in 0..self.input.trips.max(0) {
            if self.root == 0 {
                return Err(rows);
            }
            let selected = self.input.gate ^ self.prior() ^ (current % 2 == 0);
            if selected && index == self.hole {
                return Err(rows);
            }
            let proof = if selected { 1 / (index - self.hole) } else { 0 };
            let entering = current;
            current = if selected {
                2 * current + proof + index
            } else {
                current
            } + 1;
            rows.push(Iteration {
                index,
                entering,
                selected,
                proof,
                current,
            });
        }
        Ok(rows)
    }
    pub fn current(self) -> i64 {
        self.iterations()
            .unwrap()
            .last()
            .map_or(self.entering() + 1, |r| r.current)
    }
    pub fn next(self) -> State {
        let total = self.current() + 3;
        State {
            result: self.prior() ^ (total % 2 == 0),
            total,
            marker: self.previous.map_or(self.input.marker, |s| s.marker),
        }
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
    fn iteration_trace(rows: &[Iteration], observed: bool) -> Vec<i64> {
        let mut trace = vec![];
        for row in rows {
            if observed {
                trace.push(10);
            }
            if row.selected {
                if observed {
                    trace.extend([row.entering, row.proof, row.index]);
                }
                if observed {
                    trace.push(20);
                }
            }
        }
        trace
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        let mut trace = vec![5];
        trace.extend(Self::iteration_trace(&self.iterations().unwrap(), observed));
        trace.push(35);
        if observed {
            trace.extend([self.next().total, self.prior().into(), 1]);
        }
        trace.push(40);
        trace
    }
    pub fn before_failure(self, bad: Option<usize>, observed: bool) -> Vec<i64> {
        if bad == Some(9) {
            return vec![];
        }
        let mut trace = vec![5];
        if bad == Some(4) {
            return trace;
        }
        match self.iterations() {
            Err(rows) => {
                trace.extend(Self::iteration_trace(&rows, observed));
                if self.root != 0 && observed {
                    trace.push(10);
                }
            }
            Ok(rows) => {
                trace.extend(Self::iteration_trace(&rows, observed));
                if bad == Some(7) {
                    trace.push(35);
                } else {
                    assert_eq!(bad, Some(6));
                }
            }
        }
        trace
    }
}
pub fn cases() -> Vec<Case> {
    let mut rows = vec![];
    for seed in [-8, -1, 8] {
        for gate in [false, true] {
            for flag in [false, true] {
                for trips in [-1, 0, 1, 3] {
                    rows.push(Case {
                        seed,
                        gate,
                        flag,
                        trips,
                        marker: 0,
                    });
                }
            }
        }
    }
    rows
}
pub fn invocation(input: Case, previous: Option<State>, poisoned: bool) -> Invocation {
    let mut call = Invocation {
        input,
        previous,
        root: 1,
        hole: -1,
    };
    if poisoned {
        if input.trips <= 0 {
            call.root = 0;
            call.hole = 0;
        } else if let Some(hole) = (0..input.trips)
            .map(|offset| (offset + call.entering().rem_euclid(input.trips)) % input.trips)
            .find(|hole| {
                let test = Invocation {
                    hole: *hole,
                    ..call
                };
                test.iterations()
                    .is_ok_and(|rows| !rows[*hole as usize].selected)
            })
        {
            call.hole = hole;
        }
    }
    call
}
pub fn sequence(open: Case, poisoned: bool) -> Vec<Invocation> {
    let mut previous = None;
    std::iter::once(open)
        .chain(std::iter::repeat_n(
            Case {
                marker: 123.5f64.to_bits(),
                ..open
            },
            EVENTS,
        ))
        .map(|input| {
            let call = invocation(input, previous, poisoned);
            previous = Some(call.next());
            call
        })
        .collect()
}
pub fn failure_prefix() -> Vec<Invocation> {
    let open = Case {
        seed: 8,
        gate: false,
        flag: false,
        trips: 0,
        marker: 0x8000000000000000,
    };
    let a = invocation(open, None, true);
    let b = invocation(
        Case {
            marker: 123.5f64.to_bits(),
            ..open
        },
        Some(a.next()),
        true,
    );
    let c = invocation(
        Case {
            trips: 1,
            marker: 123.5f64.to_bits(),
            ..open
        },
        Some(b.next()),
        false,
    );
    assert_ne!(a.next().result, c.next().result);
    vec![a, b, c]
}
pub fn failures() -> Vec<(Invocation, Option<usize>)> {
    let previous = failure_prefix()[2].next();
    let mut failures = vec![];
    for gate in [false, true] {
        let call = invocation(
            Case {
                seed: -8,
                gate,
                flag: true,
                trips: 3,
                marker: 123.5f64.to_bits(),
            },
            Some(previous),
            false,
        );
        for slot in [4, 6, 7, 9] {
            failures.push((call, Some(slot)));
        }
        for hole in 0..3 {
            let bad = Invocation { hole, ..call };
            if bad.iterations().is_err() {
                failures.push((bad, None));
            }
        }
    }
    failures
}
pub fn source_variants() -> [String; 4] {
    let wrapped = SOURCE
        .replace("update(current,", "update(current + 0,")
        .replace("refine(value,", "refine(value + 0,")
        .replace("answer_work(total,", "answer_work(total + 0,");
    let mirror = |source: &str| {
        source.replace(
            "if choose(gate != (value % 2 == 0), 1 / root) {",
            "if choose(gate == (value % 2 == 0), 1 / root) {} else {",
        )
    };
    [
        SOURCE.to_owned(),
        mirror(SOURCE),
        wrapped.clone(),
        mirror(&wrapped),
    ]
}
