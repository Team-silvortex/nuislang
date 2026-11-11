use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print.ns",
        required_patterns: &[
            "return negative_update(value, negative_helper_fault)",
            "return nonnegative_update(value, nonnegative_helper_fault)",
            "print(83); print(value); let checked = 0 / fault; print(84)",
            "print(85); print(value); let checked = 0 / fault; print(86)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print_cases.rs",
        required_patterns: &[
            "execute_selected(prior, native,",
            "after(returned, negative, offset, trace)",
            "trace.extend([if negative { 618 } else { 619 }, input, guard])",
            "trace.extend([if negative { 83 } else { 85 }, input])",
            "trace.push(if negative { 84 } else { 86 })",
            "for at in [16, 17, 48, 49]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_branch_helper_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_branch_helper_exit_accepts_checked_effectful_leaf_calls",
            "state_branch_helper_exit_preserves_readiness_and_leaf_types",
            "scalar_helpers::collect_with_layouts",
            "scalar_helpers::collect_iteration_calls",
            "!pure.contains_key(name)",
            "accepted wrong leaf type",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_helper_exit_native_executes_only_selected_checks",
            "state_branch_helper_exit_reference_keeps_complete_accepted_state",
            "state_branch_helper_exit_pins_selected_effects_and_fault_priority",
            "state_branch_helper_exit_consumes_returned_values_not_leaf_inputs",
            "bad[40] = Value::Bool(false)",
            "row.words()[..39], row.base.words()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-effect-grants",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_helper_exit_requires_each_leaf_effect_site_grant",
            "assert_eq!(own.len(), 3",
            "selected leaf inherited a caller or sibling effect grant",
            "[41, 43, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_helper_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
            "&[90, 70]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_branch_helper_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_helper_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "compile_cache: hit",
            "materialize-artifact",
            "assert_eq!(states(&failed), state)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-receipt",
        path: "docs/reference/nuis-native-state-branch-helper-exit-argument-print-v1.md",
        required_patterns: &[
            "Selected Effectful Helpers",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-helper-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_branch_helper_exit_retains_complete_branch_lineage",
            "Previous state-branch-control-exit checkpoint",
            "NUIS_STATE_BRANCH_HELPER_EXIT_20261111_BRANCH_HISTORY",
        ],
    },
];
