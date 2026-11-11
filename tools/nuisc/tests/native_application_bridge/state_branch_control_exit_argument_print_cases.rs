#[allow(dead_code)]
#[path = "state_control_exit_argument_print_cases.rs"]
mod previous;
pub use previous::{Kind, State, ENTRY_LIMIT, KINDS, LOOP_LIMIT};

pub const SOURCE: &str = include_str!("state_branch_control_exit_argument_print.ns");

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: previous::Case,
    pub negative_fault: i64,
    pub nonnegative_fault: i64,
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind()
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.base.words();
        words.extend([self.negative_fault, self.nonnegative_fault]);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{}",
            self.base.args(),
            self.negative_fault,
            self.nonnegative_fault
        )
    }
    pub fn execute(
        self,
        prior: Option<State>,
        native: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_selected(prior, native, |control, _, _| {
            Ok(if control < 0 {
                control.wrapping_sub(3)
            } else {
                control.wrapping_add(2)
            })
        })
    }
    pub fn execute_selected(
        self,
        prior: Option<State>,
        native: bool,
        mut selected: impl FnMut(i64, i64, &mut Vec<i64>) -> Result<i64, ()>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.base
            .execute_control(prior, native, |control, offset, trace| {
                let negative = self.negative_fault.wrapping_sub(offset);
                let nonnegative = self.nonnegative_fault.wrapping_sub(offset);
                if native {
                    trace.extend([617, control, negative, nonnegative]);
                }
                let guard = if control < 0 { negative } else { nonnegative };
                if guard == 0 {
                    return Err(());
                }
                selected(control, offset, trace)
            })
    }
}

pub fn seed(kind: Kind) -> Case {
    let mut base = previous::seed(kind);
    // Checked branches have their own helper entries; keep the existing ceiling.
    base.arguments.child.base.trips = 2;
    let exits = previous::controls(&mut base);
    exits.flat.confirm_guard = 1000;
    exits.seal_guard = 1000;
    Case {
        base,
        negative_fault: 64,
        nonnegative_fault: 64,
    }
}

pub fn prefix(kind: Kind) -> Vec<Case> {
    let mut row = seed(kind);
    row.base.initial_control = i64::MIN + 3;
    row.base.arguments.parents = 2;
    row.base.arguments.child.base.trips = 1;
    let mut next = row;
    next.base.initial_control = 4294967296;
    next.base.arguments.child.base.initial = kind.last_state();
    vec![row, next]
}

pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = vec![prefix(kind)];
    for control in [-1, 0, i64::MIN, i64::MAX, 4294967296] {
        let mut first = seed(kind);
        first.base.initial_control = control;
        // A zero guard belongs only to the skipped branch at the first child.
        if control < 0 {
            first.nonnegative_fault = 16;
        } else {
            first.negative_fault = 16;
        }
        let mut next = seed(kind);
        next.base.initial_control = control.wrapping_add(123);
        next.base.arguments.child.base.initial = kind.last_state();
        rows.push(vec![first, next]);
    }
    for parents in [0, 1] {
        let mut empty = seed(kind);
        empty.base.arguments.parents = parents;
        empty.negative_fault = 0;
        empty.nonnegative_fault = 0;
        rows.push(vec![empty, seed(kind)]);
    }
    rows
}

pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    for control in [i64::MIN + 3, i64::MAX - 2, -4294967296, 617, 75] {
        let mut row = seed(kind);
        row.base.initial_control = control;
        rows.push(vec![row, seed(kind), row]);
    }
    for bits in [0, kind.mask() as i64, kind.nan()] {
        let mut row = seed(kind);
        row.base.arguments.child.base.initial = bits;
        rows.push(vec![row, seed(kind)]);
    }
    rows
}

pub fn failures(kind: Kind) -> Vec<Case> {
    // The accepted prefix leaves MAX: the first two selected branches differ.
    let mut rows = Vec::new();
    for at in [16, 17, 48, 49] {
        let mut row = seed(kind);
        row.negative_fault = at;
        row.nonnegative_fault = at;
        rows.push(row);
    }
    let mut collision = rows[0];
    collision.base.arguments.child.base.guards[2] = 16;
    rows.push(collision);
    let mut tail = seed(kind);
    tail.base.control_tail_guard = 0;
    rows.push(tail);
    for row in &mut rows {
        row.base.initial_control = -4294967296;
        row.base.arguments.child.base.initial = kind.last_state();
    }
    rows
}

pub fn observers(kind: Kind) -> Vec<(&'static str, &'static str)> {
    let mut observers = previous::observers(kind).to_vec();
    observers.push(("advance_control", "  call void @word_print(i64 617)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"));
    observers
}
