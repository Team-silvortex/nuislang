#[allow(dead_code)]
#[path = "state_return_join_exit_argument_print_cases.rs"]
mod previous;
pub use previous::{Kind, State, AFTER_TAG, ENTRY_LIMIT, KINDS, LOOP_LIMIT, TAG};
pub const PREFIX_TAG: i64 = 17179869184;
pub const SOURCE: &str = include_str!("state_return_print_exit_argument_print.ns");

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: previous::Case,
    pub prefix_fault: i64,
    pub skip_prefix_fault: i64,
    pub print_fault: i64,
    pub skip_print_fault: i64,
}
impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind()
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.extend([
            self.prefix_fault,
            self.skip_prefix_fault,
            self.print_fault,
            self.skip_print_fault,
        ]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{},{},{}",
            self.base.args(),
            self.prefix_fault,
            self.skip_prefix_fault,
            self.print_fault,
            self.skip_print_fault
        )
    }
    pub fn execute(
        self,
        prior: Option<State>,
        native: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_prefixed(prior, native, PREFIX_TAG)
    }
    pub fn execute_prefixed(
        self,
        prior: Option<State>,
        native: bool,
        prefix_tag: i64,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.base
            .base
            .execute_then(prior, native, 0, |returned, negative, offset, trace| {
                let input = returned ^ TAG;
                let staged = if input == 0 {
                    input ^ TAG
                } else {
                    let guard = if negative {
                        self.base.negative_finish_fault
                    } else {
                        self.base.nonnegative_finish_fault
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
                    input ^ TAG
                };
                let prefixed = staged ^ prefix_tag;
                trace.push(if input == 0 { 99 } else { 97 });
                let print_guard = if input == 0 {
                    self.skip_print_fault
                } else {
                    self.print_fault
                };
                if print_guard.wrapping_sub(offset) == 0 {
                    return Err(());
                }
                trace.push(prefixed);
                let selected_fault = if input == 0 {
                    self.skip_prefix_fault
                } else {
                    self.prefix_fault
                };
                if selected_fault.wrapping_sub(offset) == 0 {
                    return Err(());
                }
                trace.push(if input == 0 { 100 } else { 98 });
                let encoded = prefixed ^ AFTER_TAG;
                trace.extend([95, encoded]);
                if self.base.joined_fault.wrapping_sub(offset) == 0 {
                    return Err(());
                }
                trace.push(96);
                Ok((encoded ^ AFTER_TAG) ^ prefix_tag)
            })
    }
}
fn promote(base: previous::Case) -> Case {
    Case {
        base,
        prefix_fault: 64,
        skip_prefix_fault: 64,
        print_fault: 64,
        skip_print_fault: 64,
    }
}
pub fn seed(kind: Kind) -> Case {
    promote(previous::seed(kind))
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    previous::prefix(kind).into_iter().map(promote).collect()
}
pub fn skipped_prefix_sequences(kind: Kind) -> Vec<Vec<Case>> {
    [-1, TAG - 2]
        .into_iter()
        .map(|control| {
            let mut row = seed(kind);
            row.base.base.base.base.initial_control = control;
            row.base.base.base.base.arguments.parents = 2;
            row.base.base.base.base.arguments.child.base.trips = 1;
            if control < 0 {
                row.skip_prefix_fault = 16;
            } else {
                row.prefix_fault = 16;
                row.base.nonnegative_finish_fault = 16;
            }
            vec![row, seed(kind)]
        })
        .collect()
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    previous::sequences(kind)
        .into_iter()
        .map(|rows| rows.into_iter().map(promote).collect())
        .chain(skipped_prefix_sequences(kind))
        .chain(skipped_print_sequences(kind))
        .collect()
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    previous::cli_sequences(kind)
        .into_iter()
        .take(2)
        .map(|rows| rows.into_iter().map(promote).collect())
        .chain(skipped_print_sequences(kind))
        .collect()
}
pub fn prefix_failures(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    let mut rows = Vec::new();
    for at in [16, 17, 48, 49] {
        let mut bad = seed(kind);
        bad.prefix_fault = at;
        bad.base.joined_fault = at;
        rows.push((prefix(kind), bad));
    }
    let old = previous::failures(kind);
    for i in [0, 4, 6, 13] {
        let mut bad = promote(old[i]);
        bad.prefix_fault = 16;
        rows.push((prefix(kind), bad));
    }
    let mut skipped = seed(kind);
    skipped.base.nonnegative_finish_fault = 16;
    skipped.prefix_fault = 16;
    skipped.skip_prefix_fault = 16;
    rows.push((
        previous::zero_prefix(kind)
            .into_iter()
            .map(promote)
            .collect(),
        skipped,
    ));
    rows
}
pub fn failure_sequences(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    previous::failure_sequences(kind)
        .into_iter()
        .map(|(rows, bad)| (rows.into_iter().map(promote).collect(), promote(bad)))
        .chain(prefix_failures(kind))
        .chain(print_failures(kind))
        .collect()
}
pub fn delivery_failure_sequences(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    let mut rows = print_failures(kind);
    let old = prefix_failures(kind);
    rows.extend([old[0].clone(), old[8].clone()]);
    let (prefix, bad) = previous::join_failures(kind).remove(0);
    rows.push((prefix.into_iter().map(promote).collect(), promote(bad)));
    rows
}
pub fn skipped_print_sequences(kind: Kind) -> Vec<Vec<Case>> {
    skipped_prefix_sequences(kind)
        .into_iter()
        .map(|mut rows| {
            let row = &mut rows[0];
            if row.base.base.base.base.initial_control < 0 {
                row.skip_prefix_fault = 64;
                row.skip_print_fault = 16;
            } else {
                row.prefix_fault = 64;
                row.print_fault = 16;
            }
            rows
        })
        .collect()
}
pub fn print_failures(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    let mut rows = Vec::new();
    for at in [16, 17, 48, 49] {
        let mut bad = seed(kind);
        bad.print_fault = at;
        bad.prefix_fault = at;
        bad.base.joined_fault = at;
        rows.push((prefix(kind), bad));
    }
    let old = previous::failures(kind);
    for i in [0, 4, 6, 13] {
        let mut bad = promote(old[i]);
        bad.print_fault = 16;
        rows.push((prefix(kind), bad));
    }
    let mut skipped = seed(kind);
    skipped.base.nonnegative_finish_fault = 16;
    skipped.print_fault = 16;
    skipped.skip_print_fault = 16;
    skipped.skip_prefix_fault = 16;
    rows.push((
        previous::zero_prefix(kind)
            .into_iter()
            .map(promote)
            .collect(),
        skipped,
    ));
    rows
}
pub fn observers(kind: Kind) -> Vec<(&'static str, &'static str)> {
    previous::observers(kind)
}
