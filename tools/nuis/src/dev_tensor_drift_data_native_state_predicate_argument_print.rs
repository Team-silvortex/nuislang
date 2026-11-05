use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_predicate_argument_print.ns",
        required_patterns: &[
            "fn continue_at(index: i64, target: i64, guard: i64) -> bool",
            "fn break_at(index: i64, target: i64, guard: i64) -> bool",
            "if continue_at(child, cut, continue_guard - (outer * 16 + child))",
            "if break_at(child, stop_at, break_guard - (outer * 16 + child))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_predicate_argument_print_accepts_ordered_checked_exits",
            "state_predicate_argument_print_keeps_step_type_and_scope_refusals",
            "function `continue_at` argument 1 expects `i64`",
            "function `break_at` argument 2 expects `i64`",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-effect-order",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.predicate_guards()",
            "trace.extend([603, child_index, case.continue_control().unwrap().0, guard])",
            "trace.extend([604, child_index, stop, guard])",
            "observed_index = child_index + 1;",
            "if guard == 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-profile",
        path: "tools/nuisc/tests/native_application_bridge/state_predicate_argument_print_cases.rs",
        required_patterns: &[
            "super::state_mixed::failures(kind)",
            "3 | 6 => (64, 48)",
            "4 | 7 => (64, 49)",
            "row.child.base.guards[2] = 16",
            "*stop = 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_predicate_argument_print.rs",
        required_patterns: &[
            "state_predicate_argument_print_native_reverses_collision_and_fault_priority",
            "state_predicate_argument_print_native_keeps_reservation_and_entry_boundaries",
            "state_predicate_argument_print_oracle_pins_reached_checks_and_fault_order",
            "row.trace.splice(exit..exit, [604, 0, 0, 48])",
            "(matrix.failures)(kind).len(), 30",
            "row.words().len(), 17",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-helper-counts",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "row.predicate_calls_from(previous)",
            "calls(\"] continue_at(\")",
            "calls(\"] break_at(\")",
            "cases::GuardClock::StatePredicate { .. }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_predicate_argument_print_fresh_cache_and_source_free_delivery_preserve_checks",
            "cases::STATE_PREDICATE",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_predicate_argument_print;",
            "pub(super) const STATE_PREDICATE_ARGUMENT_PRINT",
            "state_predicate_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-receipt",
        path: "docs/reference/nuis-native-state-predicate-argument-print-v1.md",
        required_patterns: &[
            "Checked Exit Predicates",
            "31 raw sequences",
            "twelve finite sequences",
            "30 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-predicate-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded checked-exit witnesses:",
            "Previous state-mixed checkpoint:",
            "Previous state-mixed task checkpoint:",
            "Previous state-mixed blocker checkpoint:",
            "Previous state-mixed action checkpoint:",
            "Previous state-mixed validation checkpoint:",
            "Previous state-mixed artifact checkpoint:",
        ],
    },
];
