use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print.ns",
        required_patterns: &[
            "print(97); print(prefixed ^ (0 / print_fault))",
            "print(99); print(prefixed ^ (0 / skip_print_fault))",
            "print(98); prefixed",
            "print(100); prefixed",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print_cases.rs",
        required_patterns: &[
            "execute_then(prior, native, 0,",
            "print_guard.wrapping_sub(offset) == 0",
            "trace.push(prefixed)",
            "pub fn print_failures",
            "pub fn skipped_print_sequences",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_print_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_print_exit_admits_selected_arm_local_prefixes_before_later_work",
            "!pure.contains_key(name)",
            "print(prefix_check)",
            "print(negative_finish(prefixed, print_fault))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print.rs",
        required_patterns: &[
            "state_return_print_exit_native_executes_only_selected_checks",
            "state_return_print_exit_reference_keeps_complete_accepted_state",
            "bad[47] = Value::Bool(false)",
            "state_return_print_exit_consumes_full_width_arm_local_results",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-authority",
        path: "tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print.rs",
        required_patterns: &[
            "state_return_print_exit_requires_each_leaf_and_join_effect_site_grant",
            "assert_eq!(prefixes.len(), 4)",
            "assert_eq!(own.len(), 3",
            "[48, 50, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print.rs",
        required_patterns: &[
            "state_return_print_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_print_exit_argument_print.rs",
        required_patterns: &[
            "state_return_print_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "cases::delivery_failure_sequences(kind)",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs",
        required_patterns: &[
            "fn declaration_with_prints",
            "plan(stmt, scope, suffix, catalog, layouts, false)",
            "if let NirStmt::Print(value) = stmt",
            "iteration_prints::admit(value, &inner, catalog, layouts)",
            "computed_expression_roots(roots)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-hygiene",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs",
        required_patterns: &[
            "NirStmt::Print(value) => rename_expr(value, &inner)",
            "let mut inner = visible.clone()",
            "rename_expr(result, &inner)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_print_tests.rs",
        required_patterns: &[
            "iteration_print_join_exports_only_exact_result_and_keeps_colliding_locals_private",
            "iteration_print_join_rejects_leaks_rebinding_forward_reads_and_unregistered_effects",
            "iteration_print_join_shares_arm_root_budgets_before_inference",
            "iteration_print_join_keeps_repeated_value_boundaries_linear",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-receipt",
        path: "docs/reference/nuis-native-state-return-print-exit-argument-print-v1.md",
        required_patterns: &[
            "Selected Prefix Prints",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-print-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_print_exit_retains_complete_prefix_lineage",
            "Previous state-return-prefix-exit checkpoint",
            "NUIS_STATE_RETURN_PRINT_EXIT_20261111_PREFIX_HISTORY",
        ],
    },
];
