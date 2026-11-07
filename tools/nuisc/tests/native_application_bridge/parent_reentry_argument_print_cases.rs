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
#[path = "state_continue_argument_print_cases.rs"]
mod state_continues;
#[path = "state_grouped_argument_print_cases.rs"]
pub mod state_grouped;
#[path = "state_mixed_argument_print_cases.rs"]
mod state_mixed;
#[path = "state_predicate_argument_print_cases.rs"]
mod state_predicates;
#[path = "state_short_circuit_argument_print_cases.rs"]
mod state_short_circuit;
pub use child::{Kind, KINDS};
pub use state_grouped::Exits as GroupedExits;
#[path = "state_division_exit_argument_print_cases.rs"]
pub mod state_division_exits;
#[path = "state_field_exit_argument_print_cases.rs"]
pub mod state_field_exits;
#[path = "state_helper_exit_argument_print_cases.rs"]
pub mod state_helper_exits;
#[path = "state_parameter_exit_argument_print_cases.rs"]
pub mod state_parameter_exits;
#[path = "state_record_parameter_exit_argument_print_cases.rs"]
pub mod state_record_parameters;
pub use state_short_circuit::Exits as ShortCircuitExits;
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
    StateContinue(i64, i64, i64),
    StateMixed {
        guard: i64,
        cut: i64,
        suffix_guard: i64,
        stop: i64,
    },
    StatePredicate {
        guard: i64,
        cut: i64,
        suffix_guard: i64,
        stop: i64,
        continue_guard: i64,
        break_guard: i64,
    },
    StateFieldExit {
        guard: i64,
        suffix_guard: i64,
        exits: GroupedExits,
    },
    StateDivisionExit {
        guard: i64,
        suffix_guard: i64,
        exits: GroupedExits,
        divisors: [i64; 2],
    },
    StateHelperExit {
        guard: i64,
        suffix_guard: i64,
        exits: GroupedExits,
        divisors: [i64; 2],
        operand_guards: [i64; 4],
    },
    StateParameterExit {
        guard: i64,
        suffix_guard: i64,
        exits: GroupedExits,
        divisors: [i64; 2],
        operand_guards: [i64; 4],
        parameter_faults: [i64; 4],
    },
    StateRecordParameterExit {
        guard: i64,
        suffix_guard: i64,
        exits: GroupedExits,
        divisors: [i64; 2],
        operand_guards: [i64; 4],
        parameter_faults: [i64; 4],
    },
    StateGrouped {
        guard: i64,
        suffix_guard: i64,
        exits: GroupedExits,
    },
    StateShortCircuit {
        guard: i64,
        suffix_guard: i64,
        exits: ShortCircuitExits,
    },
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
pub const STATE_CONTINUE: Matrix = Matrix {
    template: include_str!("state_continue_argument_print.ns"),
    sequences: state_continues::sequences,
    cli_sequences: state_continues::cli_sequences,
    prefix: state_continues::prefix,
    failures: state_continues::failures,
    seed: state_continues::seed,
    observers: STATE_BOUND.observers,
};
pub const STATE_MIXED: Matrix = Matrix {
    template: include_str!("state_mixed_argument_print.ns"),
    sequences: state_mixed::sequences,
    cli_sequences: state_mixed::cli_sequences,
    prefix: state_mixed::prefix,
    failures: state_mixed::failures,
    seed: state_mixed::seed,
    observers: STATE_BOUND.observers,
};
pub const STATE_PREDICATE: Matrix = Matrix {
    template: include_str!("state_predicate_argument_print.ns"),
    sequences: state_predicates::sequences,
    cli_sequences: state_predicates::cli_sequences,
    prefix: state_predicates::prefix,
    failures: state_predicates::failures,
    seed: state_predicates::seed,
    observers: &[
        STATE_BOUND.observers[0], STATE_BOUND.observers[1],
        ("continue_at", "  call void @word_print(i64 603)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
        ("break_at", "  call void @word_print(i64 604)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
    ],
};
pub const STATE_SHORT_CIRCUIT: Matrix = Matrix {
    template: include_str!("state_short_circuit_argument_print.ns"),
    sequences: state_short_circuit::sequences,
    cli_sequences: state_short_circuit::cli_sequences,
    prefix: state_short_circuit::prefix,
    failures: state_short_circuit::failures,
    seed: state_short_circuit::seed,
    observers: &[
        STATE_PREDICATE.observers[0], STATE_PREDICATE.observers[1],
        STATE_PREDICATE.observers[2], STATE_PREDICATE.observers[3],
        ("confirm_continue", "  call void @word_print(i64 605)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
        ("fallback_break", "  call void @word_print(i64 606)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
    ],
};

pub const STATE_GROUPED: Matrix = Matrix {
    template: include_str!("state_grouped_argument_print.ns"),
    sequences: state_grouped::sequences,
    cli_sequences: state_grouped::cli_sequences,
    prefix: state_grouped::prefix,
    failures: state_grouped::failures,
    seed: state_grouped::seed,
    observers: &[
        STATE_SHORT_CIRCUIT.observers[0], STATE_SHORT_CIRCUIT.observers[1],
        STATE_SHORT_CIRCUIT.observers[2], STATE_SHORT_CIRCUIT.observers[3],
        STATE_SHORT_CIRCUIT.observers[4], STATE_SHORT_CIRCUIT.observers[5],
        ("rescue_continue", "  call void @word_print(i64 607)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
        ("confirm_break", "  call void @word_print(i64 608)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n  call void @word_print(i64 %arg2)\n"),
    ],
};

pub const STATE_FIELD_EXIT: Matrix = Matrix {
    template: include_str!("state_field_exit_argument_print.ns"),
    sequences: state_field_exits::sequences,
    cli_sequences: state_field_exits::cli_sequences,
    prefix: state_field_exits::prefix,
    failures: state_field_exits::failures,
    seed: state_field_exits::seed,
    observers: state_field_exits::OBSERVERS_F64,
};

pub const STATE_DIVISION_EXIT: Matrix = Matrix {
    template: include_str!("state_division_exit_argument_print.ns"),
    sequences: state_division_exits::sequences,
    cli_sequences: state_division_exits::cli_sequences,
    prefix: state_division_exits::prefix,
    failures: state_division_exits::failures,
    seed: state_division_exits::seed,
    observers: state_division_exits::OBSERVERS_F64,
};

pub const STATE_HELPER_EXIT: Matrix = Matrix {
    template: include_str!("state_helper_exit_argument_print.ns"),
    sequences: state_helper_exits::sequences,
    cli_sequences: state_helper_exits::cli_sequences,
    prefix: state_helper_exits::prefix,
    failures: state_helper_exits::failures,
    seed: state_helper_exits::seed,
    observers: state_helper_exits::OBSERVERS_F64,
};

pub const STATE_PARAMETER_EXIT: Matrix = Matrix {
    template: include_str!("state_parameter_exit_argument_print.ns"),
    sequences: state_parameter_exits::sequences,
    cli_sequences: state_parameter_exits::cli_sequences,
    prefix: state_parameter_exits::prefix,
    failures: state_parameter_exits::failures,
    seed: state_parameter_exits::seed,
    observers: state_parameter_exits::OBSERVERS_F64,
};

pub const STATE_RECORD_PARAMETER_EXIT: Matrix = Matrix {
    template: include_str!("state_record_parameter_exit_argument_print.ns"),
    sequences: state_record_parameters::sequences,
    cli_sequences: state_record_parameters::cli_sequences,
    prefix: state_record_parameters::prefix,
    failures: state_record_parameters::failures,
    seed: state_record_parameters::seed,
    observers: state_record_parameters::OBSERVERS_F64,
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
        if self.has_record_parameters() {
            return state_record_parameters::plan(self, previous).requested;
        }
        if self.division_exits().is_some() {
            return state_division_exits::plan(self, previous).requested;
        }
        if self.field_exits().is_some() {
            return state_field_exits::plan(self, previous).requested;
        }
        if self.break_cut().is_some() {
            return state_breaks::schedule(self, previous)
                .into_iter()
                .map(|work| work.requested)
                .collect();
        }
        if matches!(
            self.clock,
            GuardClock::StateBound(_) | GuardClock::StateContinue(_, _, _)
        ) {
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
        if self.has_record_parameters() {
            return state_record_parameters::plan(self, previous).completed;
        }
        if self.division_exits().is_some() {
            return state_division_exits::plan(self, previous).completed;
        }
        if self.field_exits().is_some() {
            return state_field_exits::plan(self, previous).completed;
        }
        if self.break_cut().is_some() {
            return state_breaks::schedule(self, previous)
                .into_iter()
                .map(|work| work.completed)
                .collect();
        }
        self.child_counts_from(previous)
    }
    pub fn break_cut(self) -> Option<i64> {
        match self.clock {
            GuardClock::StateFieldExit { .. }
            | GuardClock::StateDivisionExit { .. }
            | GuardClock::StateHelperExit { .. }
            | GuardClock::StateParameterExit { .. }
            | GuardClock::StateRecordParameterExit { .. } => Some(-1),
            GuardClock::StateGrouped { exits, .. } => Some(exits.first_break()),
            GuardClock::StateShortCircuit { exits, .. } => Some(exits.first_break()),
            GuardClock::StateBreak(_, cut) => Some(cut),
            // The earlier continue consumes a colliding break predicate.
            GuardClock::StateMixed { stop, cut, .. }
            | GuardClock::StatePredicate { stop, cut, .. } => {
                Some(if stop == cut { -1 } else { stop })
            }
            _ => None,
        }
    }
    pub fn continue_control(self) -> Option<(i64, i64)> {
        match self.clock {
            GuardClock::StateFieldExit { suffix_guard, .. }
            | GuardClock::StateDivisionExit { suffix_guard, .. }
            | GuardClock::StateHelperExit { suffix_guard, .. }
            | GuardClock::StateParameterExit { suffix_guard, .. }
            | GuardClock::StateRecordParameterExit { suffix_guard, .. } => Some((-1, suffix_guard)),
            GuardClock::StateGrouped {
                exits,
                suffix_guard,
                ..
            } => Some((exits.continue_index(), suffix_guard)),
            GuardClock::StateShortCircuit {
                exits,
                suffix_guard,
                ..
            } => Some((exits.continue_index(), suffix_guard)),
            GuardClock::StateContinue(_, cut, guard) => Some((cut, guard)),
            GuardClock::StateMixed {
                cut, suffix_guard, ..
            }
            | GuardClock::StatePredicate {
                cut, suffix_guard, ..
            } => Some((cut, suffix_guard)),
            _ => None,
        }
    }
    pub fn has_record_parameters(self) -> bool {
        matches!(self.clock, GuardClock::StateRecordParameterExit { .. })
    }
    pub fn parameter_faults(self) -> Option<[i64; 4]> {
        if let GuardClock::StateParameterExit {
            parameter_faults, ..
        }
        | GuardClock::StateRecordParameterExit {
            parameter_faults, ..
        } = self.clock
        {
            Some(parameter_faults)
        } else {
            None
        }
    }
    pub fn helper_operands(self) -> Option<[i64; 4]> {
        if let GuardClock::StateHelperExit { operand_guards, .. }
        | GuardClock::StateParameterExit { operand_guards, .. }
        | GuardClock::StateRecordParameterExit { operand_guards, .. } = self.clock
        {
            Some(operand_guards)
        } else {
            None
        }
    }
    pub fn division_exits(self) -> Option<(GroupedExits, [i64; 2])> {
        if let GuardClock::StateDivisionExit {
            exits, divisors, ..
        }
        | GuardClock::StateHelperExit {
            exits, divisors, ..
        }
        | GuardClock::StateParameterExit {
            exits, divisors, ..
        }
        | GuardClock::StateRecordParameterExit {
            exits, divisors, ..
        } = self.clock
        {
            Some((exits, divisors))
        } else {
            None
        }
    }
    pub fn field_exits(self) -> Option<GroupedExits> {
        if let GuardClock::StateFieldExit { exits, .. } = self.clock {
            Some(exits)
        } else {
            None
        }
    }
    fn field_predecessor(self) -> Option<Case> {
        let GuardClock::StateFieldExit {
            guard,
            suffix_guard,
            exits,
        } = self.clock
        else {
            return None;
        };
        let mut predecessor = self;
        predecessor.clock = GuardClock::StateGrouped {
            guard,
            suffix_guard,
            exits,
        };
        Some(predecessor)
    }
    pub fn grouped(self) -> Option<GroupedExits> {
        if let GuardClock::StateGrouped { exits, .. } = self.clock {
            Some(exits)
        } else {
            None
        }
    }
    fn grouped_predecessor(self) -> Option<(Case, GroupedExits)> {
        let GuardClock::StateGrouped {
            guard,
            suffix_guard,
            exits,
        } = self.clock
        else {
            return None;
        };
        let mut predecessor = self;
        predecessor.clock = GuardClock::StateShortCircuit {
            guard,
            suffix_guard,
            exits: exits.flat,
        };
        Some((predecessor, exits))
    }
    pub fn grouped_calls_from(self, previous: Option<i64>) -> [usize; 6] {
        if self.has_record_parameters() {
            return state_record_parameters::plan(self, previous).calls;
        }
        if self.division_exits().is_some() {
            return state_division_exits::plan(self, previous).calls;
        }
        if self.field_exits().is_some() {
            return state_field_exits::plan(self, previous).calls;
        }
        self.grouped().map_or([0; 6], |exits| {
            exits.call_counts(&self.completed_child_counts_from(previous))
        })
    }
    pub fn short_circuit(self) -> Option<ShortCircuitExits> {
        if let GuardClock::StateShortCircuit { exits, .. } = self.clock {
            Some(exits)
        } else {
            None
        }
    }
    fn short_predecessor(self) -> Option<(Case, ShortCircuitExits)> {
        let GuardClock::StateShortCircuit {
            guard,
            suffix_guard,
            exits,
        } = self.clock
        else {
            return None;
        };
        let mut predecessor = self;
        predecessor.clock = GuardClock::StatePredicate {
            guard,
            suffix_guard,
            cut: exits.cut,
            stop: exits.stop,
            continue_guard: exits.continue_guard,
            break_guard: exits.break_guard,
        };
        Some((predecessor, exits))
    }
    pub fn predicate_guards(self) -> Option<(i64, i64, i64)> {
        if let GuardClock::StatePredicate {
            continue_guard,
            break_guard,
            stop,
            ..
        } = self.clock
        {
            Some((continue_guard, break_guard, stop))
        } else {
            None
        }
    }
    pub fn predicate_calls_from(self, previous: Option<i64>) -> (usize, usize) {
        if self.grouped().is_some()
            || self.field_exits().is_some()
            || self.division_exits().is_some()
        {
            let counts = self.grouped_calls_from(previous);
            return (counts[0], counts[2]);
        }
        if let Some(exits) = self.short_circuit() {
            let counts = exits.call_counts(&self.completed_child_counts_from(previous));
            return (counts[0], counts[2]);
        }
        if self.predicate_guards().is_none() {
            return (0, 0);
        }
        let cut = self.continue_control().unwrap().0;
        self.completed_child_counts_from(previous).into_iter().fold(
            (0, 0),
            |(first, second), count| {
                (
                    first + count as usize,
                    second + (count - i64::from(cut >= 0 && cut < count)) as usize,
                )
            },
        )
    }
    pub fn right_predicate_calls_from(self, previous: Option<i64>) -> (usize, usize) {
        if self.grouped().is_some()
            || self.field_exits().is_some()
            || self.division_exits().is_some()
        {
            let counts = self.grouped_calls_from(previous);
            return (counts[1], counts[3]);
        }
        self.short_circuit().map_or((0, 0), |exits| {
            let counts = exits.call_counts(&self.completed_child_counts_from(previous));
            (counts[1], counts[3])
        })
    }
    pub fn suffix_calls_from(self, previous: Option<i64>) -> usize {
        if self.has_record_parameters() {
            return state_record_parameters::plan(self, previous).suffixes;
        }
        if self.division_exits().is_some() {
            return state_division_exits::plan(self, previous).suffixes;
        }
        if self.field_exits().is_some() {
            return state_field_exits::plan(self, previous).suffixes;
        }
        self.continue_control().map_or(0, |(cut, _)| {
            self.completed_child_counts_from(previous)
                .into_iter()
                .map(|count| {
                    let stopped = self
                        .break_cut()
                        .is_some_and(|stop| stop >= 0 && stop < count);
                    (count - i64::from(cut >= 0 && cut < count) - i64::from(stopped)) as usize
                })
                .sum()
        })
    }
    pub fn kind(self) -> Kind {
        self.child.kind()
    }
    pub fn words(self) -> Vec<i64> {
        if self.has_record_parameters() {
            return state_record_parameters::parameter(self).words();
        }
        if let Some(faults) = self.parameter_faults() {
            let mut words = state_parameter_exits::helper(self).words();
            words.extend(faults);
            return words;
        }
        if let Some(guards) = self.helper_operands() {
            let mut words = state_helper_exits::division(self).words();
            words.extend(guards);
            return words;
        }
        if let Some((_, divisors)) = self.division_exits() {
            let mut words = state_division_exits::field(self).words();
            words.extend(divisors);
            return words;
        }
        if let Some(predecessor) = self.field_predecessor() {
            return predecessor.words();
        }
        if let Some((predecessor, exits)) = self.grouped_predecessor() {
            let mut words = predecessor.words();
            words.extend([
                exits.rescue,
                exits.rescue_guard,
                exits.seal,
                exits.seal_guard,
            ]);
            return words;
        }
        if let Some((predecessor, exits)) = self.short_predecessor() {
            let mut words = predecessor.words();
            words.extend([
                exits.permit,
                exits.confirm_guard,
                exits.fallback,
                exits.fallback_guard,
            ]);
            return words;
        }
        let mut words = self.child.words();
        words.push(self.parents);
        if let GuardClock::FallibleBound(guard)
        | GuardClock::StateBound(guard)
        | GuardClock::StateBreak(guard, _)
        | GuardClock::StateContinue(guard, _, _)
        | GuardClock::StateMixed { guard, .. }
        | GuardClock::StatePredicate { guard, .. } = self.clock
        {
            words.push(guard);
        }
        if let GuardClock::StateBreak(_, cut) = self.clock {
            words.push(cut);
        }
        if let Some((cut, guard)) = self.continue_control() {
            words.extend([cut, guard]);
        }
        if let GuardClock::StateMixed { stop, .. } | GuardClock::StatePredicate { stop, .. } =
            self.clock
        {
            words.push(stop);
        }
        if let Some((first, second, _)) = self.predicate_guards() {
            words.extend([first, second]);
        }
        words
    }
    pub fn args(self) -> String {
        if self.has_record_parameters() {
            return state_record_parameters::parameter(self).args();
        }
        if let Some(faults) = self.parameter_faults() {
            return format!(
                "{},{},{},{},{}",
                state_parameter_exits::helper(self).args(),
                faults[0],
                faults[1],
                faults[2],
                faults[3]
            );
        }
        if let Some(guards) = self.helper_operands() {
            return format!(
                "{},{},{},{},{}",
                state_helper_exits::division(self).args(),
                guards[0],
                guards[1],
                guards[2],
                guards[3]
            );
        }
        if let Some((_, divisors)) = self.division_exits() {
            return format!(
                "{},{},{}",
                state_division_exits::field(self).args(),
                self.kind().decimal(divisors[0]),
                self.kind().decimal(divisors[1])
            );
        }
        if let Some(predecessor) = self.field_predecessor() {
            return predecessor.args();
        }
        if let Some((predecessor, exits)) = self.grouped_predecessor() {
            return format!(
                "{},{},{},{},{}",
                predecessor.args(),
                exits.rescue,
                exits.rescue_guard,
                exits.seal,
                exits.seal_guard
            );
        }
        if let Some((predecessor, exits)) = self.short_predecessor() {
            return format!(
                "{},{},{},{},{}",
                predecessor.args(),
                exits.permit,
                exits.confirm_guard,
                exits.fallback,
                exits.fallback_guard
            );
        }
        let mut args = format!("{},{}", self.child.args(), self.parents);
        if let GuardClock::FallibleBound(guard)
        | GuardClock::StateBound(guard)
        | GuardClock::StateBreak(guard, _)
        | GuardClock::StateContinue(guard, _, _)
        | GuardClock::StateMixed { guard, .. }
        | GuardClock::StatePredicate { guard, .. } = self.clock
        {
            args.push_str(&format!(",{guard}"));
        }
        if let GuardClock::StateBreak(_, cut) = self.clock {
            args.push_str(&format!(",{cut}"));
        }
        if let Some((cut, guard)) = self.continue_control() {
            args.push_str(&format!(",{cut},{guard}"));
        }
        if let GuardClock::StateMixed { stop, .. } | GuardClock::StatePredicate { stop, .. } =
            self.clock
        {
            args.push_str(&format!(",{stop}"));
        }
        if let Some((first, second, _)) = self.predicate_guards() {
            args.push_str(&format!(",{first},{second}"));
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
                | GuardClock::StateContinue(_, _, _)
                | GuardClock::StateMixed { .. }
                | GuardClock::StatePredicate { .. }
                | GuardClock::StateShortCircuit { .. }
                | GuardClock::StateGrouped { .. }
                | GuardClock::StateFieldExit { .. }
                | GuardClock::StateDivisionExit { .. }
                | GuardClock::StateHelperExit { .. }
                | GuardClock::StateParameterExit { .. }
                | GuardClock::StateRecordParameterExit { .. }
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
