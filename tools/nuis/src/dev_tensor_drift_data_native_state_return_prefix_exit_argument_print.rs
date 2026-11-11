use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print.ns",
        required_patterns: &[
            "let staged = negative_finish(next, negative_finish_fault)",
            "const local_tag: i64 = 17179869184",
            "let prefix_check = 0 / prefix_fault",
            "return (encoded ^ 8589934592) ^ 17179869184",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print_cases.rs",
        required_patterns: &[
            "execute_then(prior, native, 0,",
            "let prefixed = staged ^ prefix_tag",
            "selected_fault.wrapping_sub(offset) == 0",
            "trace.extend([95, encoded])",
            "pub fn prefix_failures",
            "pub fn skipped_prefix_sequences",
            "pub fn delivery_failure_sequences",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_prefix_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_prefix_exit_admits_selected_arm_local_prefixes_before_later_work",
            "!pure.contains_key(name)",
            "effects.contains_key(name)",
            "let staged = joined;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print.rs",
        required_patterns: &[
            "state_return_prefix_exit_keeps_completed_leaf_effects_before_prefix_faults",
            "state_return_prefix_exit_consumes_full_width_arm_local_results",
            "state_return_prefix_exit_reference_keeps_complete_accepted_state",
            "bad[45] = Value::Bool(false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-effects",
        path: "tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print.rs",
        required_patterns: &[
            "state_return_prefix_exit_requires_each_leaf_and_join_effect_site_grant",
            "assert_eq!(joined.len(), 2)",
            "assert_eq!(own.len(), 3",
            "[46, 48, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print.rs",
        required_patterns: &[
            "state_return_prefix_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
            "&[90, 70]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_prefix_exit_argument_print.rs",
        required_patterns: &[
            "state_return_prefix_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "cases::delivery_failure_sequences(kind)",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-receipt",
        path: "docs/reference/nuis-native-state-return-prefix-exit-argument-print-v1.md",
        required_patterns: &[
            "Selected Arm Prefixes",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_prefix_exit_retains_complete_join_lineage",
            "Previous state-return-join-exit checkpoint",
            "NUIS_STATE_RETURN_PREFIX_EXIT_20261111_JOIN_HISTORY",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-prefix-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration.rs",
        required_patterns: &[
            "fn prefixed_declaration",
            "prefixed::declaration(stmt, scope, suffix, catalog, layouts)?",
            "roots.push((value, 0, false))",
            "computed_call_expression_roots(roots, catalog).then_some(join)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-prefix-lowering",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "scalar_joins::iteration::prefixed_declaration",
            "fn prefixed_joined_value",
            "body.push(NirStmt::Return(Some(value)))",
            "fn joined_body",
            "self.result = owner_result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-prefix-hygiene",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs",
        required_patterns: &[
            "yes_prefix @ ..",
            "no_prefix @ ..",
            "let mut inner = visible.clone()",
            "rename_expr(result, &inner)",
            "fn rename_binding",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-prefix-exit-prefix-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_prefix_tests.rs",
        required_patterns: &[
            "iteration_prefix_join_exports_only_exact_result_and_keeps_colliding_locals_private",
            "iteration_prefix_join_rejects_leaks_rebinding_forward_reads_and_unregistered_effects",
            "iteration_prefix_join_shares_arm_root_budgets_before_inference",
            "iteration_prefix_join_keeps_repeated_value_boundaries_linear",
        ],
    },
];
