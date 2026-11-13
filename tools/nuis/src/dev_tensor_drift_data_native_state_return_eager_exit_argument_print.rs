use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_eager_exit_argument_print.ns",
        required_patterns: &[
            "checkflag(value > 1, field_fault)) == outer",
            "checkflag(value < -1, field_fault)) != inner) ^ inner",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-pair-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_eager_pair.ns",
        required_patterns: &[
            "left_gate && left(left_value, left_guard)",
            "right_gate || right(right_value, right_guard)",
            "print(30); let checked = 0 / after_guard",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-pair-execution",
        path: "tools/nuisc/tests/native_application_bridge/state_return_eager_pair.rs",
        required_patterns: &[
            "state_return_eager_pair_keeps_eager_order_lazy_children_and_complete_state",
            "rows.len() + failures.len()",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_eager_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_eager_exit_admits_selected_nested_results_without_purity_leaks",
            "state_return_eager_exit_rejects_scope_readiness_types_and_unstepped_work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_eager_local_tests.rs",
        required_patterns: &[
            "iteration_eager_locals_lower_both_operands_without_purity_leaks",
            "iteration_eager_locals_keep_argument_print_scope_and_type_boundaries",
            "iteration_eager_locals_share_all_root_budgets_without_field_permission",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-roots",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "struct Root",
            "fn computed_initializer_expression_roots",
            "computed_bool_operands || eager_root",
            "let mut logical_edges = 32usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-isolation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "fn initializer_roots",
            "NirStmt::Print(value) if prints => roots.push(root(value, false, false))",
            "roots.push(root(condition, true, false))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-lowering",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "fn logical_value",
            "NirBinaryOp::Xor",
            "logical_conditions::predicate",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_eager_exit_argument_print.rs",
        required_patterns: &[
            "checks::check_native(SOURCE)",
            "checks::check_reference(SOURCE)",
            "checks::check_grants(SOURCE)",
            "checks::check_limits(SOURCE)",
            "checks::check_unused(SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_eager_exit_argument_print.rs",
        required_patterns: &[
            "check_delivery(",
            "state_return_eager_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-receipt",
        path: "docs/reference/nuis-native-state-return-eager-exit-argument-print-v1.md",
        required_patterns: &[
            "Eager Boolean Operands",
            "Predecessor Lineage",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-eager-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_record_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_eager_exit_retains_complete_local_lineage",
            "NUIS_STATE_RETURN_EAGER_EXIT_20261113_LOCAL_HISTORY",
        ],
    },
];
