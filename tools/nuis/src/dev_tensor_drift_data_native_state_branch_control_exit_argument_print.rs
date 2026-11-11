use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_control_exit_argument_print.ns",
        required_patterns: &[
            "if value < 0 { let checked = 0 / fault; return value - 3; }",
            "let checked = 0 / nonnegative_fault; return value + 2;",
            "nonnegative_fault - (outer * 16 + child)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_control_exit_argument_print_cases.rs",
        required_patterns: &[
            "execute_control(prior, native,",
            "if control < 0 { negative } else { nonnegative }",
            "control.wrapping_sub(3)",
            "control.wrapping_add(2)",
            "selected(control, offset, trace)",
            "trace.extend([617, control, negative, nonnegative])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_branch_control_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_branch_control_exit_accepts_selected_checked_returns",
            "state_branch_control_exit_preserves_ready_fields_and_exact_types",
            "return checked;",
            "current.flag",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_control_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_control_exit_native_executes_only_selected_checks",
            "state_branch_control_exit_reference_keeps_complete_accepted_state",
            "state_branch_control_exit_pins_branch_arithmetic_and_fault_priority",
            "prior[1], i64::MAX",
            "bad[38] = Value::Bool(false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_control_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_control_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
            "&[90, 70]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_branch_control_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_control_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "compile_cache: hit",
            "materialize-artifact",
            "assert_eq!(states(&failed), state)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-receipt",
        path: "docs/reference/nuis-native-state-branch-control-exit-argument-print-v1.md",
        required_patterns: &[
            "Selected Return Branches",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-control-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_branch_control_exit_retains_complete_checked_lineage",
            "Previous state-checked-control-exit checkpoint",
            "NUIS_STATE_BRANCH_CONTROL_EXIT_20261111_CHECKED_HISTORY",
        ],
    },
];
