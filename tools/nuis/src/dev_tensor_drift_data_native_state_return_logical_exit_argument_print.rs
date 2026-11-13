use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print.ns",
        required_patterns: &[
            "audit: value > 0 && checkflag(value > 1, field_fault)",
            "audit: value > 0 || checkflag(value < -1, field_fault)",
            "choose(staged.audit)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn lazy_poisons",
            "let rhs = if self.inner",
            "trace.push(84)",
            "which != 1 && which != 5",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_logical_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_logical_exit_admits_selected_nested_results_without_purity_leaks",
            "state_return_logical_exit_rejects_scope_readiness_types_and_unstepped_work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print.rs",
        required_patterns: &[
            "state_return_logical_exit_keeps_unused_field_effects_and_preceding_field_failures",
            "state_return_logical_exit_reference_keeps_complete_accepted_state",
            "rows.len() + failures.len()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-authority",
        path: "tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print.rs",
        required_patterns: &[
            "for missing in &sites",
            "[12, 14, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print.rs",
        required_patterns: &[
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_logical_exit_argument_print.rs",
        required_patterns: &[
            "state_return_logical_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "fn computed_record_call_expression_roots",
            "fn boolean_field",
            "let mut logical_edges = 32usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-lowering",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "fn logical_value",
            "self.logical_value(value, &scope)",
            "logical_conditions::predicate",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_logical_record_tests.rs",
        required_patterns: &[
            "iteration_logical_record_fields_keep_effects_guarded_and_scope_private",
            "iteration_logical_record_fields_reject_hidden_roots_and_unproved_calls",
            "iteration_logical_record_fields_share_edges_across_nested_fields_and_arms",
            "shared 32/33 logical edges",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-receipt",
        path: "docs/reference/nuis-native-state-return-logical-exit-argument-print-v1.md",
        required_patterns: &[
            "Logical Record Fields",
            "Predecessor Lineage",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-logical-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_record_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_logical_exit_retains_complete_record_lineage",
            "Previous state-return-record-exit checkpoint",
            "NUIS_STATE_RETURN_LOGICAL_EXIT_20261113_RECORD_HISTORY",
        ],
    },
];
