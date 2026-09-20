use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-scoped-layouts",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "control_values::supported_type(ty, layouts)",
            "NirExpr::StructLiteral",
            "NirExpr::FieldAccess",
            "seen.insert(name)",
            "depth > 64",
            "ty != &scalar_type(\"bool\") && control_values::supported_type(ty, layouts)",
            "!control_values::supported_type(existing, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-explicit-effect-types",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/validation.rs",
        required_patterns: &[
            "enum EffectTypes",
            "EffectTypes::Buffer(catalog)",
            "Self::Values(catalog, layouts)",
            "control_values::value_type(value, scope, catalog, layouts)",
            "control_values::collect_inputs(value, inputs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-shared-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values.rs",
        required_patterns: &[
            "pub(super) fn collect_inputs",
            "pub(super) fn has_aggregate_expressions",
            "NirExpr::FieldAccess { base, .. } => collect_inputs(base, inputs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-lazy-field-operands",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested_conditions.rs",
        required_patterns: &[
            "NirExpr::StructLiteral",
            "NirExpr::FieldAccess",
            "base: Box::new(predicate(*base, scope, names, helpers, guarded))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-admission-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/aggregate_calls_tests.rs",
        required_patterns: &[
            "iteration_flat_values_keep_nominal_layouts_and_local_capture_scope",
            "iteration_flat_values_reject_effects_cycles_and_nominal_rebinding_drift",
            "iteration_flat_value_layout_width_is_not_a_native_slot_table",
            "Buffer catalog widened",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-execution-tests",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_local_values.rs",
        required_patterns: &[
            "flat_iteration_results_project_copy_and_pass_exact_values",
            "flat_iteration_arguments_keep_short_circuit_and_ignored_result_checks",
            "flat_iteration_snapshots_survive_mutation_and_branch_captures",
            "flat_iteration_calls_stay_behind_complete_induction_preflight",
            "flat_iteration_literal_fields_evaluate_in_source_order_before_the_call",
            "flat_iteration_ignored_aggregate_arguments_still_execute_and_trap",
            "CallProbe",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-frontdoor-tests",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_local_values_loops.ns",
            "native_iteration_flat_values_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_LOCAL_VALUES_SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-values-native-entry-tests",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "flat_iteration_values_compose_with_default_native_entry",
            "discarded_flat_iteration_result_keeps_native_arithmetic_failure",
        ],
    },
];
