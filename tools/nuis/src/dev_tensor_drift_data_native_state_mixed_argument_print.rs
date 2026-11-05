use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_mixed_argument_print.ns",
        required_patterns: &[
            "if child == cut { print(78); child = child + 1; continue; }",
            "if child == stop_at { print(81); break; }",
            "print(checked64(0, suffix_guard - (outer * 16 + child)))",
            "suffix_guard: i64, stop_at: i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_mixed_argument_print_accepts_ordered_and_mirrored_exit_guards",
            "state_mixed_argument_print_rejects_missing_duplicate_and_moved_steps",
            "child = child + 2; continue;",
            "child = child + 1; print(82); continue;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-work-model",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "The earlier continue consumes a colliding break predicate.",
            "self.completed_child_counts_from(previous)",
            "count - i64::from(cut >= 0 && cut < count) - i64::from(stopped)",
            "state_breaks::schedule(self, previous)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-profile",
        path: "tools/nuisc/tests/native_application_bridge/state_mixed_argument_print_cases.rs",
        required_patterns: &[
            "super::state_continues::failures(kind)",
            "before_break.child.base.guards[2] = 18",
            "guard: 3",
            "stop: 0",
            "(1, 1)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_mixed_argument_print.rs",
        required_patterns: &[
            "state_mixed_argument_print_native_preserves_exit_order_and_faults",
            "state_mixed_argument_print_native_preserves_reserved_work",
            "state_mixed_argument_print_oracle_pins_collision_indices_and_accepted_state",
            "row.input[12] = 0",
            "(matrix.failures)(kind).len(), 22",
            "row.words().len(), 15",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-helper-counts",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "row.suffix_calls_from(previous)",
            "cases::GuardClock::StateMixed { .. }",
            "row.completed_child_counts_from(previous)",
            "check_calls(*row, Some(previous), &event.events)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_mixed_argument_print_fresh_cache_and_source_free_delivery_preserve_exit_order",
            "cases::STATE_MIXED",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_mixed_argument_print;",
            "pub(super) const STATE_MIXED_ARGUMENT_PRINT",
            "state_mixed_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-receipt",
        path: "docs/reference/nuis-native-state-mixed-argument-print-v1.md",
        required_patterns: &[
            "State-Bounded Mixed Exits",
            "31 raw sequences",
            "twelve finite sequences",
            "22 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-mixed-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded mixed-exit witnesses:",
            "Previous state-continue checkpoint:",
            "Previous state-continue task checkpoint:",
            "Previous state-continue blocker checkpoint:",
            "Previous state-continue action checkpoint:",
            "Previous state-continue validation checkpoint:",
            "Previous state-continue artifact checkpoint:",
        ],
    },
];
