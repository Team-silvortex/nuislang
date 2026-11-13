use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print.ns",
        required_patterns: &[
            "let staged: Selected = if inner",
            "Selected { payload: Payload",
            "control = next.payload.word ^ next.tag",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn execute_tagged",
            "trace.extend([97, local])",
            "pub fn failure_sequences",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_record_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_record_exit_admits_selected_nested_results_without_purity_leaks",
            "state_return_record_exit_rejects_scope_readiness_types_and_unstepped_work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print.rs",
        required_patterns: &[
            "state_return_record_exit_reference_keeps_complete_accepted_state",
            "state_return_record_exit_checks_unprojected_fields_before_parent_effects",
            "state_return_record_exit_consumes_full_width_arm_local_results",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-authority",
        path: "tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print.rs",
        required_patterns: &[
            "for missing in &sites",
            "[10, 12, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print.rs",
        required_patterns: &[
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_record_exit_argument_print.rs",
        required_patterns: &[
            "state_return_record_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-neutral-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "fn guarded_records_bounded",
            "let mut remaining = 4096usize",
            "discarded_records::neutral(ty, layouts, &mut remaining)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs",
        required_patterns: &[
            "guarded_records_bounded(then_body, else_body, layouts)",
            "(!prints && !layouts.scalar(&ty.name))",
            "!control_values::supported_type(ty, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_record_tests.rs",
        required_patterns: &[
            "iteration_record_join_exports_exact_owned_layouts_and_keeps_hygiene",
            "iteration_record_join_rejects_nominal_shape_scope_and_effect_leaks",
            "iteration_record_join_bounds_all_synthetic_record_arms_before_cloning",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-receipt",
        path: "docs/reference/nuis-native-state-return-record-exit-argument-print-v1.md",
        required_patterns: &[
            "Owned Record Results",
            "Predecessor Lineage",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-record-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_record_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_record_exit_retains_complete_nested_lineage",
            "Previous state-return-nested-exit checkpoint",
            "NUIS_STATE_RETURN_RECORD_EXIT_20261111_NESTED_HISTORY",
        ],
    },
];
