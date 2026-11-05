use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_continue_argument_print.ns",
        required_patterns: &[
            "if child == cut { print(78); child = child + 1; continue; }",
            "print(checked64(0, suffix_guard - (outer * 16 + child)))",
            "print(child);",
            "cut: i64, suffix_guard: i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-source-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_continue_argument_print_accepts_one_step_and_selected_suffix",
            "state_continue_argument_print_rejects_missing_duplicate_and_moved_steps",
            "child = child + 2; continue;",
            "child = child + 1; print(81); continue;",
            "continue; child = child + 1;",
            "__nuis_buffer_continue",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-work-model",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.continue_control()",
            "observed_index = child_index + 1;",
            "trace.extend([601, 0, guard])",
            "if guard == 0",
            "trace.push(78)",
            "trace.push(79)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-profile",
        path: "tools/nuisc/tests/native_application_bridge/state_continue_argument_print_cases.rs",
        required_patterns: &[
            "48 + cut",
            "(0, 49), (0, 50), (1, 16), (3, 50)",
            "before_continue.child.base.guards[2] = 48",
            "after_continue.child.base.guards[0] = 17",
            "super::state_bounds::failures(kind)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_continue_argument_print.rs",
        required_patterns: &[
            "state_continue_argument_print_native_preserves_steps_suffixes_and_faults",
            "state_continue_argument_print_reference_counts_reached_suffix_calls",
            "state_continue_argument_print_native_preserves_reservations_and_fault_priority",
            "state_continue_argument_print_oracle_pins_full_work_indices_and_skipped_poison",
            "(matrix.failures)(kind).len(), 20",
            "row.words().len(), 14",
            "prefix.ends_with(&[76, 602, 3, 3, 2, 1])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-helper-counts",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "row.suffix_calls_from(previous)",
            "cases::GuardClock::StateContinue(_, _, _)",
            "children + row.parents + 1",
            "check_calls(*row, Some(previous), &event.events)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_continue_argument_print_fresh_cache_and_source_free_delivery_preserve_steps",
            "cases::STATE_CONTINUE",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_continue_argument_print;",
            "pub(super) const STATE_CONTINUE_ARGUMENT_PRINT",
            "state_continue_argument_print::CHECKS",
            "pub(super) const STATE_BREAK_ARGUMENT_PRINT",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-receipt",
        path: "docs/reference/nuis-native-state-continue-argument-print-v1.md",
        required_patterns: &[
            "State-Bounded Continue",
            "27 raw sequences",
            "eight finite sequences",
            "20 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-continue-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded continue witnesses:",
            "Previous state-break checkpoint:",
            "Previous state-break task checkpoint:",
            "Previous state-break blocker checkpoint:",
            "Previous state-break action checkpoint:",
            "Previous state-break validation checkpoint:",
            "Previous state-break artifact checkpoint:",
        ],
    },
];
