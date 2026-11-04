#[path = "parent_bound_argument_print_cases.rs"]
mod bounds;
#[allow(dead_code)]
#[path = "epoch_local_argument_print_cases.rs"]
mod child;
#[path = "fallible_bound_argument_print_cases.rs"]
mod fallible;
#[path = "child_progress_argument_print_cases.rs"]
mod progress;
#[path = "state_bound_argument_print_cases.rs"]
mod state_bounds;
#[path = "state_break_argument_print_cases.rs"]
mod state_breaks;
pub use child::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 12;
// Includes outlined YIR entries, not just the source-level helper calls.
pub const ENTRY_LIMIT: u64 = 192;
pub const OBSERVERS: &[(&str, &str)] = child::OBSERVERS;

pub fn source(kind: Kind) -> String {
    include_str!("parent_reentry_argument_print.ns").replace("f64", kind.ty())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardClock {
    Parent,
    ChildProgress,
    ParentBound,
    FallibleBound(i64),
    StateBound(i64),
    StateBreak(i64, i64),
}

#[derive(Clone, Copy)]
pub struct Matrix {
    pub template: &'static str,
    pub sequences: fn(Kind) -> Vec<Vec<Case>>,
    pub cli_sequences: fn(Kind) -> Vec<Vec<Case>>,
    pub prefix: fn(Kind) -> Vec<Case>,
    pub failures: fn(Kind) -> Vec<Case>,
    pub seed: fn(Kind) -> Case,
    pub observers: &'static [(&'static str, &'static str)],
}

pub const PARENT: Matrix = Matrix {
    template: include_str!("parent_reentry_argument_print.ns"),
    sequences,
    cli_sequences,
    prefix,
    failures,
    seed,
    observers: OBSERVERS,
};
pub const CHILD_PROGRESS: Matrix = Matrix {
    template: include_str!("child_progress_argument_print.ns"),
    sequences: progress::sequences,
    cli_sequences: progress::cli_sequences,
    prefix: progress::prefix,
    failures: progress::failures,
    seed: progress::seed,
    observers: OBSERVERS,
};
pub const PARENT_BOUND: Matrix = Matrix {
    template: include_str!("parent_bound_argument_print.ns"),
    sequences: bounds::sequences,
    cli_sequences: bounds::cli_sequences,
    prefix: bounds::prefix,
    failures: bounds::failures,
    seed: bounds::seed,
    observers: &[OBSERVERS[0], ("child_count",
        "  call void @word_print(i64 602)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n")],
};
pub const FALLIBLE_BOUND: Matrix = Matrix {
    template: include_str!("fallible_bound_argument_print.ns"),
    sequences: fallible::sequences,
    cli_sequences: fallible::cli_sequences,
    prefix: fallible::prefix,
    failures: fallible::failures,
    seed: fallible::seed,
    observers: &[OBSERVERS[0], ("child_count",
        "  call void @word_print(i64 602)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n")],
};
pub const STATE_BOUND: Matrix = Matrix {
    template: include_str!("state_bound_argument_print.ns"),
    sequences: state_bounds::sequences,
    cli_sequences: state_bounds::cli_sequences,
    prefix: state_bounds::prefix,
    failures: state_bounds::failures,
    seed: state_bounds::seed,
    observers: &[OBSERVERS[0], ("child_count",
        "  call void @word_print(i64 602)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n  %observed_negative = zext i1 %arg3 to i64\n  call void @word_print(i64 %observed_negative)\n")],
};
pub const STATE_BREAK: Matrix = Matrix {
    template: include_str!("state_break_argument_print.ns"),
    sequences: state_breaks::sequences,
    cli_sequences: state_breaks::cli_sequences,
    prefix: state_breaks::prefix,
    failures: state_breaks::failures,
    seed: state_breaks::seed,
    observers: STATE_BOUND.observers,
};

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub child: child::Case,
    pub parents: i64,
    pub clock: GuardClock,
}

impl Case {
    pub fn child_counts(self) -> Vec<i64> {
        self.child_counts_from(None)
    }
    pub fn child_counts_from(self, previous: Option<i64>) -> Vec<i64> {
        if let GuardClock::StateBreak(_, _) = self.clock {
            return state_breaks::schedule(self, previous)
                .into_iter()
                .map(|work| work.requested)
                .collect();
        }
        if let GuardClock::StateBound(_) = self.clock {
            return state_bounds::counts(self, previous);
        }
        if matches!(
            self.clock,
            GuardClock::ParentBound | GuardClock::FallibleBound(_)
        ) {
            return bounds::counts(self);
        }
        (0..self.parents).map(|_| self.child.base.trips).collect()
    }
    pub fn completed_child_counts_from(self, previous: Option<i64>) -> Vec<i64> {
        if let GuardClock::StateBreak(_, _) = self.clock {
            return state_breaks::schedule(self, previous)
                .into_iter()
                .map(|work| work.completed)
                .collect();
        }
        self.child_counts_from(previous)
    }
    pub fn break_cut(self) -> Option<i64> {
        if let GuardClock::StateBreak(_, cut) = self.clock {
            Some(cut)
        } else {
            None
        }
    }
    pub fn kind(self) -> Kind {
        self.child.kind()
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = self.child.words();
        words.push(self.parents);
        if let GuardClock::FallibleBound(guard)
        | GuardClock::StateBound(guard)
        | GuardClock::StateBreak(guard, _) = self.clock
        {
            words.push(guard);
        }
        if let Some(cut) = self.break_cut() {
            words.push(cut);
        }
        words
    }
    pub fn args(self) -> String {
        let mut args = format!("{},{}", self.child.args(), self.parents);
        if let GuardClock::FallibleBound(guard)
        | GuardClock::StateBound(guard)
        | GuardClock::StateBreak(guard, _) = self.clock
        {
            args.push_str(&format!(",{guard}"));
        }
        if let Some(cut) = self.break_cut() {
            args.push_str(&format!(",{cut}"));
        }
        args
    }
    pub fn execute(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        if matches!(
            self.clock,
            GuardClock::ParentBound
                | GuardClock::FallibleBound(_)
                | GuardClock::StateBound(_)
                | GuardClock::StateBreak(_, _)
        ) {
            return bounds::execute(self, previous, native);
        }
        if self.clock == GuardClock::ChildProgress {
            return progress::execute(self, previous, native);
        }
        let mut current = previous.unwrap_or(self.child.base.initial);
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        for parent in 0..self.parents {
            trace.push(76);
            // Each parent gets a fresh child oracle, including its original left field.
            let mut row = self.child;
            for guard in &mut row.base.guards[..3] {
                *guard -= parent;
            }
            row.base.guards[3] = 1;
            match row.execute(Some(current), native) {
                Ok((next, child_trace)) => {
                    assert_eq!(child_trace.last(), Some(&73));
                    trace.extend_from_slice(&child_trace[2..child_trace.len() - 1]);
                    current = next;
                }
                Err(child_trace) => {
                    trace.extend_from_slice(&child_trace[2..]);
                    return Err(trace);
                }
            }
            trace.push(77);
        }
        trace.push(73);
        if self.child.base.guards[3] == 0 {
            return Err(trace);
        }
        Ok((current, trace))
    }
}

pub fn seed(kind: Kind) -> Case {
    let mut child = child::seed(kind);
    child.base.guards = [4, 4, 4, 1];
    child.base.trips = 2;
    Case {
        child,
        parents: 2,
        clock: GuardClock::Parent,
    }
}

pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    [(0, 2), (3, 0), (2, 1), (3, 2), (2, 3), (3, 1)]
        .into_iter()
        .enumerate()
        .map(|(i, (parents, trips))| {
            let mut row = seed(kind);
            row.parents = parents;
            row.child.base.trips = trips;
            if i < 2 {
                row.child.base.guards = [0, 0, 0, 1];
            }
            if i == 4 {
                row.child.base.selected = false;
                row.child.base.guards[1] = 0;
            }
            if i == 5 {
                row.child.base.mask = i64::from(row.child.base.left) + 4294967296;
            }
            let mut next = seed(kind);
            next.parents = 3;
            next.child.base.trips = 1;
            next.child.base.initial = (kind.initial() as u64 ^ kind.mask()) as i64;
            vec![row, next]
        })
        .collect()
}

pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = cli_sequences(kind);
    rows.extend(child::sequences(kind).into_iter().map(|rows| {
        rows.into_iter()
            .map(|mut child| {
                child.base.guards = [4, 4, 4, 1];
                Case {
                    child,
                    parents: 3,
                    clock: GuardClock::Parent,
                }
            })
            .collect()
    }));
    rows
}

pub fn prefix(kind: Kind) -> Vec<Case> {
    cli_sequences(kind).remove(2)
}

pub fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = Vec::new();
    for (guard, at) in [(0, 1), (1, 1), (2, 1), (0, 2), (3, 0), (1, 1)] {
        let mut row = seed(kind);
        row.parents = 3;
        row.child.base.trips = 1;
        row.child.base.selected = guard != 0 || at != 1;
        row.child.base.guards[guard] = at;
        if rows.len() == 5 {
            row.child.base.mask = i64::from(row.child.base.left) + 4294967296;
        }
        rows.push(row);
    }
    rows
}
