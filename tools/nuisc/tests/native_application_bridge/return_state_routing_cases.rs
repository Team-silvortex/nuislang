#[path = "return_stateful_fields_cases.rs"]
mod base;
#[allow(unused_imports)]
pub use base::{Case, State, CLI_MARKERS, MARKERS, OBSERVER};
pub const SOURCE: &str = include_str!("return_state_routing.ns");
pub const EVENTS: usize = 6;

// Independent expected state supplies masks only; the native probe carries real outputs.
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
    pub fn total(self) -> i64 {
        self.previous.map_or(self.input.seed, |s| s.total) + 1
    }
    pub fn route(self) -> Case {
        Case {
            gate: self.input.gate ^ self.prior(),
            first: self.input.first ^ (self.total() % 2 == 0),
            flag: self.prior(),
            ..self.input
        }
    }
    pub fn next(self) -> State {
        State {
            result: self.route().result(self.prior(), self.total()),
            total: self.total(),
            marker: self.previous.map_or(self.input.marker, |s| s.marker),
        }
    }
    pub fn words(self, bad: Option<usize>) -> Vec<i64> {
        let mut words = self.route().words();
        words[1] = self.input.gate.into();
        words[2] = self.input.first.into();
        words[3] = self.input.flag.into();
        if !self.poisoned {
            for slot in (4..=11).chain([13]) {
                words[slot] = 1;
            }
        }
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
                12 => f64::from_bits(word as u64).to_string(),
                _ => word.to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn trace(self, observed: bool) -> Vec<i64> {
        self.route().trace(self.prior(), self.total(), observed)
    }
    pub fn before_failure(self, slot: usize) -> Vec<i64> {
        self.route().before_failure(slot)
    }
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
pub fn cases() -> Vec<Case> {
    base::cases()
}
pub fn failure_prefix() -> Vec<Invocation> {
    let open = Case {
        marker: 0x8000000000000000,
        ..base::identity()
    };
    let mut rows = sequence(open, true);
    rows.truncate(2);
    let previous = Some(rows[1].next());
    rows.push(Invocation {
        input: Case {
            gate: true,
            marker: 123.5f64.to_bits(),
            ..open
        },
        previous,
        poisoned: true,
    });
    assert_ne!(rows[0].next().result, rows[2].next().result);
    rows
}
pub fn failures() -> Vec<(Invocation, usize)> {
    let last = failure_prefix().last().unwrap().next();
    (4..=11)
        .chain([13])
        .map(|slot| {
            let gate = !matches!(slot, 6 | 8 | 10);
            let input = Case {
                gate: gate ^ last.result,
                first: true ^ ((last.total + 1) % 2 == 0),
                ..base::identity()
            };
            (
                Invocation {
                    input,
                    previous: Some(last),
                    poisoned: true,
                },
                slot,
            )
        })
        .collect()
}
pub fn source_variants() -> [String; 4] {
    let rows = base::source_variants().map(|source| {
        source
            .replace(
                "observe(total, gate, first, flag,",
                "observe(total, gate != flag, first != (total % 2 == 0), flag,",
            )
            .replace(
                "observe(total, gate, first, state.result,",
                "observe(total, gate != state.result, first != (total % 2 == 0), state.result,",
            )
    });
    assert_eq!(rows[0], SOURCE);
    rows
}
