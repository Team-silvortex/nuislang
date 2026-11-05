use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_short_circuit_argument_print.ns",
        required_patterns: &[
            "&& confirm_continue(child, permit, confirm_guard - (outer * 16 + child))",
            "|| fallback_break(child, fallback_at, fallback_guard - (outer * 16 + child))",
            "fn confirm_continue(index: i64, target: i64, guard: i64) -> bool",
            "fn fallback_break(index: i64, target: i64, guard: i64) -> bool",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_short_circuit_argument_print_accepts_ordered_checked_operands",
            "state_short_circuit_argument_print_keeps_step_type_and_scope_refusals",
            "function `confirm_continue` argument 1 expects `i64`",
            "function `fallback_break` argument 2 expects `i64`",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-effect-model",
        path: "tools/nuisc/tests/native_application_bridge/state_short_circuit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn first_break",
            "pub fn call_counts",
            "call(603, self.cut, self.continue_guard)?",
            "&& call(605, self.permit, self.confirm_guard)?",
            "|| call(606, self.fallback, self.fallback_guard)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-ordered-dispatch",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "exits.evaluate(",
            "Err(()) => return Err(trace)",
            "decisions.map_or(child_index == cut, |(continuing, _)| continuing)",
            "decisions.map_or(child_index == cut, |(_, stopping)| stopping)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_short_circuit_argument_print.rs",
        required_patterns: &[
            "state_short_circuit_argument_print_native_observes_swapped_operand_faults",
            "state_short_circuit_argument_print_native_keeps_changed_work_and_fault_priority",
            "state_short_circuit_argument_print_oracle_pins_lazy_rhs_and_changed_state",
            "(matrix.failures)(kind).len(), 38",
            "row.words().len(), 21",
            "(fallback, 10, 0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-helper-counts",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "row.right_predicate_calls_from(previous)",
            "calls(\"] confirm_continue(\")",
            "calls(\"] fallback_break(\")",
            "cases::GuardClock::StateShortCircuit { .. }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_short_circuit_argument_print_fresh_cache_and_source_free_delivery_preserve_lazy_rhs",
            "cases::STATE_SHORT_CIRCUIT",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_short_circuit_argument_print;",
            "pub(super) const STATE_SHORT_CIRCUIT_ARGUMENT_PRINT",
            "state_short_circuit_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-receipt",
        path: "docs/reference/nuis-native-state-short-circuit-argument-print-v1.md",
        required_patterns: &[
            "Short-Circuit Exit Conditions",
            "35 raw sequences",
            "sixteen finite sequences",
            "38 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-short-circuit-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded short-circuit-exit witnesses:",
            "Previous state-predicate checkpoint:",
            "Previous state-predicate task checkpoint:",
            "Previous state-predicate blocker checkpoint:",
            "Previous state-predicate action checkpoint:",
            "Previous state-predicate validation checkpoint:",
            "Previous state-predicate artifact checkpoint:",
        ],
    },
];
