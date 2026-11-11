use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print.ns",
        required_patterns: &[
            "let joined: i64 = if next != 0 { negative_finish(next, negative_finish_fault) } else { next ^ 4294967296 }",
            "let joined: i64 = if next != 0 { nonnegative_finish(next, nonnegative_finish_fault) } else { next ^ 4294967296 }",
            "let encoded = joined ^ 8589934592",
            "let after_check = 0 / joined_fault",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print_cases.rs",
        required_patterns: &[
            "execute_then(prior, native, 0,",
            "let input = returned ^ tag",
            "if input == 0",
            "self.finish(input ^ tag, offset, after_tag, trace)",
            "trace.extend([95, encoded])",
            "pub fn join_failures",
            "pub fn delivery_failure_sequences",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_join_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_join_exit_admits_fresh_effectful_results_before_later_work",
            "state_return_join_exit_keeps_types_scope_readiness_and_budgets",
            "!pure.contains_key(name)",
            "effects.contains_key(name)",
            "else { joined }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print.rs",
        required_patterns: &[
            "state_return_join_exit_consumes_full_width_join_results_in_later_work",
            "state_return_join_exit_observes_joined_values_before_post_join_faults",
            "state_return_join_exit_reference_keeps_complete_accepted_state",
            "bad[43] = Value::Bool(false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-effects",
        path: "tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print.rs",
        required_patterns: &[
            "state_return_join_exit_requires_each_leaf_and_join_effect_site_grant",
            "assert_eq!(joined.len(), 2)",
            "assert_eq!(own.len(), 3",
            "[44, 46, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print.rs",
        required_patterns: &[
            "state_return_join_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
            "&[90, 70]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_join_exit_argument_print.rs",
        required_patterns: &[
            "state_return_join_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "cases::delivery_failure_sequences(kind)",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-receipt",
        path: "docs/reference/nuis-native-state-return-join-exit-argument-print-v1.md",
        required_patterns: &[
            "Joined Results And Later Work",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_join_exit_retains_complete_gate_lineage",
            "Previous state-return-gate-exit checkpoint",
            "NUIS_STATE_RETURN_JOIN_EXIT_20261111_GATE_HISTORY",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-join-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration.rs",
        required_patterns: &[
            "let join = shape(stmt, scope, suffix)?",
            "computed_call_expression_roots",
            "scalar_arms(&join, scope, catalog, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-join-lowering",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "scalar_joins::iteration::declaration",
            "fn joined_value",
            "__nuis_scalar_value_join",
            "self.result = owner_result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-join-hygiene",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs",
        required_patterns: &[
            "fn rename_join",
            "visible.insert(name.clone(), replacement.clone())",
            "*other = replacement",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-join-exit-join-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_tests.rs",
        required_patterns: &[
            "iteration_value_join_retains_exact_owned_types_and_private_return_boundaries",
            "iteration_value_join_rejects_incomplete_mismatched_or_preexisting_destinations",
            "iteration_value_join_keeps_shared_suffix_growth_linear",
            "iteration_value_join_preserves_one_identity_when_moving_a_colliding_suffix",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "bounded-buffer-scalar-helper-admission-control-block",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_block.rs",
        required_patterns: &[
            "then_returns && else_returns",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-counted-value-transitive-boundaries-control-block",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_block.rs",
        required_patterns: &[
            "control_loops::validate(",
            "loop_layouts?,",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-admission-control-block",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_block.rs",
        required_patterns: &[
            "if returned",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-isolated-literal-profile-control-block",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_block.rs",
        required_patterns: &[
            "NirStmt::Print(value) if matches!(profile, Profile::IterationPrints)",
        ],
    },
];
