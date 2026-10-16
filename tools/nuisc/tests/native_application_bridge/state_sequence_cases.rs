#[path = "state_skip_cases.rs"]
mod base;
#[allow(unused_imports)]
pub use base::{State, CLI_MARKERS, MARKERS, OBSERVER};
pub const SECOND_OBSERVER: &str =
    "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n";
pub const SOURCE: &str = include_str!("state_sequence.ns");
pub const EVENTS: usize = 6;

#[derive(Clone, Copy)]
pub struct Case {
    pub seed: i64,
    pub gate: bool,
    pub second: bool,
    pub flag: bool,
    pub marker: u64,
}
#[derive(Clone, Copy)]
pub struct Invocation {
    pub input: Case,
    pub previous: Option<State>,
    pub poisoned: bool,
}
impl Invocation {
    pub fn prior(self) -> bool {
        self.previous.map_or(self.input.flag, |s| s.result)
    }
    pub fn entering(self) -> i64 {
        self.previous.map_or(self.input.seed, |s| s.total)
    }
    pub fn route(self) -> bool {
        self.input.gate ^ self.prior()
    }
    pub fn first(self) -> i64 {
        self.entering() + 1 + if self.route() { 3 } else { 0 }
    }
    pub fn second_route(self) -> bool {
        self.input.second ^ (self.first() % 2 == 0)
    }
    pub fn selected(self) -> i64 {
        if self.second_route() {
            2 * self.first() - 1
        } else {
            self.first()
        }
    }
    pub fn next(self) -> State {
        let total = self.selected() + 3;
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
            self.input.second.into(),
            self.input.flag.into(),
            1,
            (!self.poisoned || self.route()).into(),
            1,
            (!self.poisoned || self.second_route()).into(),
            1,
            1,
            self.input.marker as i64,
            1,
        ];
        if let Some(slot) = bad {
            assert_eq!(words[slot], 1, "failure must target selected work");
            words[slot] = 0;
        }
        words
    }
    pub fn args(self, bad: Option<usize>) -> String {
        self.words(bad)
            .into_iter()
            .enumerate()
            .map(|(slot, word)| match slot {
                1..=3 => (word != 0).to_string(),
                10 => f64::from_bits(word as u64).to_string(),
                _ => word.to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn prefix(self, observed: bool) -> Vec<i64> {
        let mut out = vec![5, 10];
        if self.route() {
            out.push(20);
        }
        out.push(11);
        if self.second_route() {
            if observed {
                out.extend([self.first(), 1]);
            }
            out.push(30);
        }
        out
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        let mut out = self.prefix(observed);
        out.push(35);
        if observed {
            out.extend([self.next().total, self.prior().into(), 1]);
        }
        out.push(40);
        out
    }
    pub fn before_failure(self, slot: usize, observed: bool) -> Vec<i64> {
        if slot == 11 {
            return vec![];
        }
        let mut out = vec![5];
        if slot == 4 {
            return out;
        }
        out.push(10);
        if slot == 5 {
            assert!(self.route());
            return out;
        }
        if self.route() {
            out.push(20);
        }
        if slot == 6 {
            return out;
        }
        out.push(11);
        if slot == 7 {
            assert!(self.second_route());
            return out;
        }
        out = self.prefix(observed);
        if slot == 8 {
            return out;
        }
        assert_eq!(slot, 9);
        out.push(35);
        out
    }
}
pub fn cases() -> Vec<Case> {
    let mut rows = vec![];
    for seed in [-8, -1, 8] {
        for gate in [false, true] {
            for second in [false, true] {
                for flag in [false, true] {
                    rows.push(Case {
                        seed,
                        gate,
                        second,
                        flag,
                        marker: 0,
                    });
                }
            }
        }
    }
    rows
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
            let call = Invocation {
                input,
                previous,
                poisoned,
            };
            previous = Some(call.next());
            call
        })
        .collect()
}
pub fn failure_prefix() -> Vec<Invocation> {
    let open = Case {
        seed: 8,
        gate: false,
        second: false,
        flag: false,
        marker: 0x8000000000000000,
    };
    let mut rows = sequence(open, true);
    rows.truncate(2);
    rows.push(Invocation {
        input: Case {
            gate: true,
            second: true,
            marker: 123.5f64.to_bits(),
            ..open
        },
        previous: Some(rows[1].next()),
        poisoned: true,
    });
    assert_ne!(rows[0].next().result, rows[2].next().result);
    rows
}
pub fn failures() -> Vec<(Invocation, usize)> {
    let previous = failure_prefix()[2].next();
    let mut rows = vec![];
    for slot in [4, 5, 6, 7, 8, 9, 11] {
        for first in [false, true] {
            for second in [false, true] {
                let include = match slot {
                    5 => first && !second,
                    7 => second,
                    8 | 9 => true,
                    _ => !second,
                };
                if !include {
                    continue;
                }
                let current = previous.total + 1 + if first { 3 } else { 0 };
                let input = Case {
                    seed: -8,
                    gate: first ^ previous.result,
                    second: second ^ (current % 2 == 0),
                    flag: true,
                    marker: 123.5f64.to_bits(),
                };
                rows.push((
                    Invocation {
                        input,
                        previous: Some(previous),
                        poisoned: true,
                    },
                    slot,
                ));
            }
        }
    }
    rows
}
pub fn source_variants() -> [String; 4] {
    let wrapped = SOURCE
        .replace("left(current,", "left(current + 0,")
        .replace("refine(current,", "refine(current + 0,")
        .replace("finish(current,", "finish(current + 0,")
        .replace("answer_work(total,", "answer_work(total + 0,");
    [
        SOURCE.to_owned(),
        mirror(SOURCE),
        wrapped.clone(),
        mirror(&wrapped),
    ]
}
fn mirror(source: &str) -> String {
    let mut result = source.to_owned();
    for condition in [
        "choose(gate, 1 / root)",
        "select(second != (current % 2 == 0), 1 / next_root)",
    ] {
        let header = format!("        if {condition} {{");
        let start = result.find(&header).unwrap();
        let body = start + header.len();
        let end = body + result[body..].find("        }\n").unwrap();
        let inverted = if condition.starts_with("choose") {
            "choose(gate != true, 1 / root)"
        } else {
            "select(second == (current % 2 == 0), 1 / next_root)"
        };
        result = format!(
            "{}        if {inverted} {{}} else {{{}{}",
            &result[..start],
            &result[body..end],
            &result[end..]
        );
    }
    result
}
