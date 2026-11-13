use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print.ns",
        required_patterns: &[
            "let staged: i64 = if inner",
            "local ^ 34359738368",
            "local ^ 68719476736",
            "control = control + 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn execute_tagged",
            "state[1].wrapping_add(1)",
            "if guards[2] == 0",
            "pub fn poisons",
            "pub fn failure_sequences",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_nested_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_nested_exit_admits_selected_nested_results_without_purity_leaks",
            "state_return_nested_exit_rejects_scope_readiness_types_and_unstepped_work",
            "!pure.contains_key(name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print.rs",
        required_patterns: &[
            "state_return_nested_exit_native_executes_only_selected_checks",
            "state_return_nested_exit_reference_keeps_complete_accepted_state",
            "bad[9] = Value::Bool(false)",
            "state_return_nested_exit_consumes_full_width_arm_local_results",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-authority",
        path: "tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print.rs",
        required_patterns: &[
            "state_return_nested_exit_requires_each_leaf_and_join_effect_site_grant",
            "for missing in &sites",
            "[10, 12, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print.rs",
        required_patterns: &[
            "state_return_nested_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_nested_exit_argument_print.rs",
        required_patterns: &[
            "state_return_nested_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-shared-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "fn prefix_roots",
            "let mut remaining = 128usize",
            "body.len() > 32",
            "if depth < 8",
            "NirStmt::Print(value) if prints",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-recursive-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs",
        required_patterns: &[
            "super::nested_prefix::prefix_roots",
            "blocked.contains(name)",
            "if prints && matches!(stmt, NirStmt::If { .. })",
            "inner.insert(join.name.to_owned(), join.ty.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_nested_tests.rs",
        required_patterns: &[
            "iteration_nested_join_exports_only_exact_result_and_keeps_colliding_locals_private",
            "iteration_nested_join_shares_statement_expression_and_depth_budgets",
            "128 shared statements",
            "129 shared statements",
            "iteration_nested_join_keeps_repeated_value_boundaries_linear",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-receipt",
        path: "docs/reference/nuis-native-state-return-nested-exit-argument-print-v1.md",
        required_patterns: &[
            "Nested Selected Prefixes",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-nested-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_nested_exit_retains_complete_print_lineage",
            "Previous state-return-print-exit checkpoint",
            "NUIS_STATE_RETURN_NESTED_EXIT_20261111_PRINT_HISTORY",
        ],
    },
];
