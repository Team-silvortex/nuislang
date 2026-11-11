#[allow(dead_code)]
#[path = "state_control_exit_argument_print_cases.rs"]
mod previous;
pub use previous::{Kind, State, ENTRY_LIMIT, KINDS, LOOP_LIMIT};

pub const SOURCE: &str = include_str!("state_checked_control_exit_argument_print.ns");

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: previous::Case,
    pub control_update_fault: i64,
}

// Payload words may equal markers; advance by each observation's declared width.
pub fn records(trace: &[i64], native: bool) -> Vec<(usize, i64)> {
    let mut records = Vec::new();
    let mut at = 0;
    while at < trace.len() {
        let marker = trace[at];
        let len = match marker {
            601 | 609..=612 | 617 if native => 3,
            602 if native => 5,
            603..=608 | 613..=616 if native => 4,
            80 => 2,
            _ => 1,
        };
        assert!(at + len <= trace.len());
        records.push((at, marker));
        at += len;
    }
    records
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind()
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.push(self.control_update_fault);
        words
    }
    pub fn args(self) -> String {
        format!("{},{}", self.base.args(), self.control_update_fault)
    }
    pub fn plan(self, prior: Option<State>) -> previous::Plan {
        self.base.plan(prior)
    }
    pub fn execute(
        self,
        prior: Option<State>,
        native: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let outcome = self.base.execute(prior, native);
        let (state, source) = match outcome {
            Ok((state, trace)) => (Some(state), trace),
            Err(trace) => (None, trace),
        };
        let mut trace = Vec::new();
        let mut parent = -1;
        let mut child = -1;
        let mut control = prior.unwrap_or(self.base.initial())[1];
        let boundaries = records(&source, native);
        for (position, &(at, marker)) in boundaries.iter().enumerate() {
            let end = boundaries
                .get(position + 1)
                .map_or(source.len(), |row| row.0);
            if marker == 76 {
                parent += 1;
                child = -1;
            }
            if marker == 71 {
                child += 1;
            }
            if marker == 75 {
                let fault = self.control_update_fault.wrapping_sub(parent * 16 + child);
                if native {
                    trace.extend([617, control, fault]);
                }
                if fault == 0 {
                    return Err(trace);
                }
                control = control.wrapping_add(1);
            }
            trace.extend_from_slice(&source[at..end]);
        }
        if let Some(state) = state {
            assert_eq!(state[1], control);
            Ok((state, trace))
        } else {
            Err(trace)
        }
    }
}

pub fn seed(kind: Kind) -> Case {
    Case {
        base: previous::seed(kind),
        control_update_fault: 64,
    }
}

fn promoted(rows: Vec<previous::Case>) -> Vec<Case> {
    rows.into_iter()
        .map(|base| Case {
            base,
            control_update_fault: 64,
        })
        .collect()
}

pub fn prefix(kind: Kind) -> Vec<Case> {
    promoted(previous::prefix(kind))
}

pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = previous::cli_sequences(kind)
        .into_iter()
        .map(promoted)
        .collect::<Vec<_>>();
    for i in [5, 6] {
        rows[i][0].control_update_fault = 0;
    }
    let mut early = promoted(previous::early_sequences(kind).remove(0));
    early[0].control_update_fault = 17;
    rows.push(early);
    rows
}

pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = previous::sequences(kind)
        .into_iter()
        .map(promoted)
        .collect::<Vec<_>>();
    for i in [5, 6] {
        rows[i][0].control_update_fault = 0;
    }
    rows.last_mut().unwrap()[0].control_update_fault = 17;
    for control in [71, 75, 76, 80, 617] {
        let mut row = seed(kind);
        row.base.initial_control = control;
        rows.push(vec![row, seed(kind)]);
    }
    for fault in [i64::MIN, i64::MAX] {
        let mut row = seed(kind);
        row.control_update_fault = fault;
        rows.push(vec![row, seed(kind)]);
    }
    for trips in [i64::MIN, i64::MAX] {
        let mut row = seed(kind);
        row.base.arguments.parents = 1;
        row.base.arguments.child.base.trips = trips;
        row.control_update_fault = 0;
        rows.push(vec![row, seed(kind)]);
    }
    rows
}

pub fn failures(kind: Kind) -> Vec<Case> {
    let previous = previous::failures(kind);
    let mut rows = [0, 1, 2, 3, 9, 15, 21, 27]
        .into_iter()
        .map(|i| Case {
            base: previous[i],
            control_update_fault: 64,
        })
        .collect::<Vec<_>>();
    for at in [16, 17, 18, 48, 49, 50, 51] {
        let mut row = seed(kind);
        row.control_update_fault = at;
        row.base.initial_control = 4294967296;
        row.base.arguments.child.base.initial = kind.last_state();
        rows.push(row);
    }
    for at in [16, 48] {
        let mut row = seed(kind);
        row.control_update_fault = at;
        row.base.arguments.child.base.guards[2] = at;
        row.base.initial_control = 4294967296;
        row.base.arguments.child.base.initial = kind.last_state();
        rows.push(row);
    }
    for (i, at) in [(9, 16), (15, 51)] {
        rows.push(Case {
            base: previous[i],
            control_update_fault: at,
        });
    }
    rows
}

pub fn observers(kind: Kind) -> Vec<(&'static str, &'static str)> {
    let mut observers = previous::observers(kind).to_vec();
    observers.push(("advance_control", "  call void @word_print(i64 617)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n"));
    observers
}
