#[allow(dead_code)]
#[path = "state_branch_helper_exit_argument_print_cases.rs"]
mod previous;
pub use previous::{Kind, State, ENTRY_LIMIT, KINDS, LOOP_LIMIT};

pub const SOURCE: &str = include_str!("state_return_gate_exit_argument_print.ns");
pub const TAG: i64 = 4294967296;

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: previous::Case,
    pub negative_finish_fault: i64,
    pub nonnegative_finish_fault: i64,
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind()
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.extend([self.negative_finish_fault, self.nonnegative_finish_fault]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{}",
            self.base.args(),
            self.negative_finish_fault,
            self.nonnegative_finish_fault
        )
    }
    pub fn execute(
        self,
        prior: Option<State>,
        native: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_tagged(prior, native, TAG)
    }
    pub fn execute_tagged(
        self,
        prior: Option<State>,
        native: bool,
        tag: i64,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.base
            .execute_then(prior, native, 0, |returned, negative, offset, trace| {
                // The branch is selected by the original control, not by this intermediate.
                let input = returned ^ tag;
                if input == 0 {
                    return Ok(input ^ tag);
                }
                let guard = if negative {
                    self.negative_finish_fault
                } else {
                    self.nonnegative_finish_fault
                }
                .wrapping_sub(offset);
                if native {
                    trace.extend([if negative { 620 } else { 621 }, input, guard]);
                }
                trace.extend([if negative { 87 } else { 93 }, input]);
                if guard == 0 {
                    return Err(());
                }
                trace.push(if negative { 88 } else { 94 });
                Ok(input ^ tag)
            })
    }
}

fn promote(base: previous::Case) -> Case {
    Case {
        base,
        negative_finish_fault: 64,
        nonnegative_finish_fault: 64,
    }
}
pub fn seed(kind: Kind) -> Case {
    promote(previous::seed(kind))
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    previous::prefix(kind).into_iter().map(promote).collect()
}
fn promote_rows(rows: Vec<Vec<previous::Case>>) -> Vec<Vec<Case>> {
    rows.into_iter()
        .map(|rows| {
            rows.into_iter()
                .enumerate()
                .map(|(index, base)| {
                    let mut row = promote(base);
                    if index == 0 {
                        if base.base.base.initial_control < 0 {
                            row.nonnegative_finish_fault = 16;
                        } else {
                            row.negative_finish_fault = 16;
                        }
                    }
                    row
                })
                .collect()
        })
        .collect()
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let rows = previous::cli_sequences(kind);
    let mut selected = promote_rows([0, 1, 2, 6].map(|i| rows[i].clone()).to_vec());
    selected.extend(skip_sequences(kind));
    selected
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = promote_rows(previous::sequences(kind));
    rows.extend(skip_sequences(kind));
    rows
}

pub fn zero_prefix(kind: Kind) -> Vec<Case> {
    let mut row = seed(kind);
    row.base.base.base.initial_control = TAG - 2;
    row.base.base.base.arguments.parents = 0;
    vec![row]
}

pub fn skip_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = Vec::new();
    for (ordinal, at) in [16, 17, 48, 49].into_iter().enumerate() {
        let mut first = seed(kind);
        first.base.base.base.initial_control = TAG - 2 * (ordinal as i64 + 1);
        first.base.base.base.arguments.parents = 4;
        first.nonnegative_finish_fault = at;
        rows.push(vec![first, seed(kind)]);
    }
    let mut event = seed(kind);
    event.base.base.base.initial_control = i64::MIN;
    event.nonnegative_finish_fault = 16;
    let mut accepted = zero_prefix(kind);
    accepted.push(event);
    rows.push(accepted);
    rows
}

pub fn zero_gate_failures(kind: Kind) -> Vec<Case> {
    (0..3)
        .map(|stage| {
            let mut bad = seed(kind);
            bad.base.base.base.initial_control = i64::MIN;
            bad.nonnegative_finish_fault = 16;
            match stage {
                0 => bad.base.base.base.arguments.child.base.guards[2] = 16,
                1 => bad.base.base.nonnegative_fault = 16,
                _ => bad.base.nonnegative_helper_fault = 16,
            }
            bad
        })
        .collect()
}

pub fn failure_sequences(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    failures(kind)
        .into_iter()
        .map(|bad| (prefix(kind), bad))
        .chain(
            zero_gate_failures(kind)
                .into_iter()
                .map(|bad| (zero_prefix(kind), bad)),
        )
        .collect()
}

pub fn delivery_failure_sequences(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    delivery_failures(kind)
        .into_iter()
        .map(|bad| (prefix(kind), bad))
        .chain(
            zero_gate_failures(kind)
                .into_iter()
                .map(|bad| (zero_prefix(kind), bad)),
        )
        .collect()
}

pub fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = previous::failures(kind)
        .into_iter()
        .map(promote)
        .collect::<Vec<_>>();
    for at in [16, 17, 48, 49] {
        let mut row = seed(kind);
        row.negative_finish_fault = at;
        row.nonnegative_finish_fault = at;
        rows.push(row);
    }
    // First leaf, caller, and float-check collisions must all precede the second leaf.
    for (index, at) in [(6, 16), (7, 17), (0, 16), (4, 16)] {
        let mut row = rows[index];
        row.negative_finish_fault = at;
        row.nonnegative_finish_fault = at;
        rows.push(row);
    }
    for row in &mut rows {
        row.base.base.base.initial_control = -4294967296;
        row.base.base.base.arguments.child.base.initial = kind.last_state();
    }
    rows
}

pub fn delivery_failures(kind: Kind) -> Vec<Case> {
    let rows = failures(kind);
    [0, 5, 6]
        .into_iter()
        .chain(13..21)
        .map(|i| rows[i])
        .collect()
}

pub fn observers(kind: Kind) -> Vec<(&'static str, &'static str)> {
    let mut result = previous::observers(kind);
    result.extend([
        ("negative_finish", "  call void @word_print(i64 620)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n"),
        ("nonnegative_finish", "  call void @word_print(i64 621)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n"),
    ]);
    result
}
