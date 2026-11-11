#[allow(dead_code)]
#[path = "state_branch_control_exit_argument_print_cases.rs"]
mod previous;
pub use previous::{Kind, State, ENTRY_LIMIT, KINDS, LOOP_LIMIT};

pub const SOURCE: &str = include_str!("state_branch_helper_exit_argument_print.ns");

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: previous::Case,
    pub negative_helper_fault: i64,
    pub nonnegative_helper_fault: i64,
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind()
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.extend([self.negative_helper_fault, self.nonnegative_helper_fault]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{}",
            self.base.args(),
            self.negative_helper_fault,
            self.nonnegative_helper_fault
        )
    }
    pub fn execute(
        self,
        prior: Option<State>,
        native: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_masked(prior, native, 0)
    }
    pub fn execute_masked(
        self,
        prior: Option<State>,
        native: bool,
        mask: i64,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_then(prior, native, mask, |value, _, _, _| Ok(value))
    }
    pub fn execute_then(
        self,
        prior: Option<State>,
        native: bool,
        mask: i64,
        mut after: impl FnMut(i64, bool, i64, &mut Vec<i64>) -> Result<i64, ()>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.base
            .execute_selected(prior, native, |control, offset, trace| {
                let negative = control < 0;
                let input = control ^ mask;
                let guard = if negative {
                    self.negative_helper_fault
                } else {
                    self.nonnegative_helper_fault
                }
                .wrapping_sub(offset);
                if native {
                    trace.extend([if negative { 618 } else { 619 }, input, guard]);
                }
                trace.extend([if negative { 83 } else { 85 }, input]);
                if guard == 0 {
                    return Err(());
                }
                trace.push(if negative { 84 } else { 86 });
                let returned = if negative {
                    (input ^ mask).wrapping_sub(3)
                } else {
                    (input ^ mask).wrapping_add(2)
                };
                after(returned, negative, offset, trace)
            })
    }
}

fn promote(base: previous::Case) -> Case {
    Case {
        base,
        negative_helper_fault: 64,
        nonnegative_helper_fault: 64,
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
                    // Poison the unselected helper at the first child, including wrapping controls.
                    // Later events use accepted State, so their guards must not depend on input decoys.
                    if index == 0 {
                        if base.base.initial_control < 0 {
                            row.nonnegative_helper_fault = 16;
                        } else {
                            row.negative_helper_fault = 16;
                        }
                    }
                    row
                })
                .collect()
        })
        .collect()
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    promote_rows(previous::cli_sequences(kind))
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    promote_rows(previous::sequences(kind))
}

pub fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = previous::failures(kind)
        .into_iter()
        .map(promote)
        .collect::<Vec<_>>();
    for at in [16, 17, 48, 49] {
        let mut row = seed(kind);
        row.negative_helper_fault = at;
        row.nonnegative_helper_fault = at;
        rows.push(row);
    }
    // Caller/leaf collisions must stop before any leaf entry or source print.
    for index in [0, 1, 4] {
        let mut row = rows[index];
        row.negative_helper_fault = row.base.negative_fault;
        row.nonnegative_helper_fault = row.base.nonnegative_fault;
        rows.push(row);
    }
    for row in &mut rows {
        row.base.base.initial_control = -4294967296;
        row.base.base.arguments.child.base.initial = kind.last_state();
    }
    rows
}

pub fn observers(kind: Kind) -> Vec<(&'static str, &'static str)> {
    let mut result = previous::observers(kind);
    result.extend([
        ("negative_update", "  call void @word_print(i64 618)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n"),
        ("nonnegative_update", "  call void @word_print(i64 619)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n"),
    ]);
    result
}
