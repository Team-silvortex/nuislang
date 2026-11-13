use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_local_exit_argument_print.ns",
        required_patterns: &[
            "let audit: bool = value > 0 && checkflag",
            "const audit: bool = value > 0 || checkflag",
            "let selected: bool = if audit",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-shared-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn lazy_poisons",
            "which != 1 && which != 5",
            "trace.push(84)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_local_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_local_exit_admits_selected_nested_results_without_purity_leaks",
            "state_return_local_exit_rejects_scope_readiness_types_and_unstepped_work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_local_exit_argument_print.rs",
        required_patterns: &[
            "checks::check_native(SOURCE)",
            "checks::check_reference(SOURCE)",
            "checks::check_unused(SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-authority-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_local_exit_argument_print.rs",
        required_patterns: &[
            "checks::check_grants(SOURCE)",
            "checks::check_limits(SOURCE)",
            "checks::check_width(SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_local_exit_argument_print.rs",
        required_patterns: &[
            "check_delivery(",
            "state_return_local_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-roots",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration.rs",
        required_patterns: &[
            "nested_prefix::prefix_roots(join.condition, join.yes, join.no, true, true)",
            "computed_record_call_expression_roots(roots, catalog, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-print-isolation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "NirStmt::Print(value) if prints => roots.push(root(value, false, false))",
            "let mut remaining = 128usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-lowering",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "fn logical_value",
            "self.logical_value(value, &scope)",
            "scope.insert(name.clone(), ty)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_logical_local_tests.rs",
        required_patterns: &[
            "iteration_logical_locals_keep_let_const_guards_and_dependent_branches",
            "iteration_logical_locals_reject_hidden_roots_scope_type_and_print_leaks",
            "shared 32/33 logical edges across local/field/predicate roots",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-receipt",
        path: "docs/reference/nuis-native-state-return-local-exit-argument-print-v1.md",
        required_patterns: &[
            "Local Boolean Initializers",
            "Predecessor Lineage",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-local-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_record_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_local_exit_retains_complete_logical_lineage",
            "NUIS_STATE_RETURN_LOCAL_EXIT_20261113_LOGICAL_HISTORY",
        ],
    },
];
