#[path = "return_stateful_fields_cases.rs"]
mod base;
#[allow(unused_imports)]
pub use base::{State, CLI_MARKERS, MARKERS, OBSERVER};
pub const SOURCE: &str = include_str!("state_join.ns");
pub const EVENTS: usize = 6;

#[derive(Clone, Copy)]
pub struct Case {
    pub seed: i64,
    pub gate: bool,
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
    pub fn selected(self) -> i64 {
        self.entering() + if self.route() { 4 } else { -1 }
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
            self.input.flag.into(),
            1,
            (!self.poisoned || self.route()).into(),
            (!self.poisoned || !self.route()).into(),
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
                1..=2 => (word != 0).to_string(),
                8 => f64::from_bits(word as u64).to_string(),
                _ => word.to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        let mut out = vec![5, 10, if self.route() { 20 } else { 30 }, 35];
        if observed {
            out.extend([self.next().total, self.prior().into(), 1]);
        }
        out.push(40);
        out
    }
    pub fn before_failure(self, slot: usize) -> Vec<i64> {
        if slot == 9 {
            return vec![];
        }
        let mut out = vec![5];
        if slot == 3 {
            return out;
        }
        out.push(10);
        if matches!(slot, 4 | 5) {
            return out;
        }
        out.push(if self.route() { 20 } else { 30 });
        if slot == 6 {
            return out;
        }
        assert_eq!(slot, 7);
        out.push(35);
        out
    }
}
pub fn cases() -> Vec<Case> {
    let mut rows = vec![];
    for seed in [7, 8] {
        for gate in [false, true] {
            for flag in [false, true] {
                rows.push(Case {
                    seed,
                    gate,
                    flag,
                    marker: 0,
                });
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
        flag: false,
        marker: 0x8000000000000000,
    };
    let mut rows = sequence(open, true);
    rows.truncate(3);
    assert_ne!(rows[0].next().result, rows[2].next().result);
    rows
}
pub fn failures() -> Vec<(Invocation, usize)> {
    let previous = failure_prefix()[2].next();
    [3, 4, 5, 6, 7, 9]
        .into_iter()
        .flat_map(|slot| {
            [false, true]
                .into_iter()
                .filter(move |route| match slot {
                    4 => *route,
                    5 => !route,
                    _ => true,
                })
                .map(move |route| {
                    let input = Case {
                        seed: 7,
                        gate: route ^ previous.result,
                        flag: true,
                        marker: 123.5f64.to_bits(),
                    };
                    (
                        Invocation {
                            input,
                            previous: Some(previous),
                            poisoned: true,
                        },
                        slot,
                    )
                })
        })
        .collect()
}
pub fn source_variants() -> [String; 4] {
    let wrapped = SOURCE
        .replace("left(current,", "left(current + 0,")
        .replace("right(current,", "right(current + 0,")
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
    let start = source.find("        if choose(gate, 1 / root) {").unwrap();
    let yes = start + "        if choose(gate, 1 / root) {".len();
    let middle = yes + source[yes..].find("\n        } else {").unwrap();
    let no = middle + "\n        } else {".len();
    let end = no
        + source[no..]
            .find("        }\n        return finish")
            .unwrap();
    format!(
        "{}        if choose(gate != true, 1 / root) {{{}        }} else {{{}{}",
        &source[..start],
        &source[no..end],
        &source[yes..middle],
        &source[end..]
    )
}
