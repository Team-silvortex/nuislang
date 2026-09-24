use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

#[path = "dev_tensor_drift_data_native_aggregate_carries.rs"]
mod aggregate_carries;
#[path = "dev_tensor_drift_data_native_aggregate_rebinding.rs"]
mod aggregate_rebinding;
#[path = "dev_tensor_drift_data_native_aggregate_values.rs"]
mod aggregate_values;
#[path = "dev_tensor_drift_data_native_bool_carries.rs"]
mod bool_carries;
#[path = "dev_tensor_drift_data_native_bool_rebinding.rs"]
mod bool_rebinding;
#[path = "dev_tensor_drift_data_native_control_elision.rs"]
mod control_elision;
#[path = "dev_tensor_drift_data_native_counted_returns.rs"]
mod counted_returns;
#[path = "dev_tensor_drift_data_native_iteration_values.rs"]
mod flat_values;
#[path = "dev_tensor_drift_data_native_helper_entries.rs"]
mod helper_entries;
#[path = "dev_tensor_drift_data_native_literal_loops.rs"]
mod literal_loops;
#[path = "dev_tensor_drift_data_native_iteration_loops.rs"]
mod loop_calls;
#[path = "dev_tensor_drift_data_native_loop_work.rs"]
mod loop_work;
#[path = "dev_tensor_drift_data_native_scoped_captures.rs"]
mod scoped_captures;
#[path = "dev_tensor_drift_data_native_terminal_continuations.rs"]
mod terminal_continuations;
#[path = "dev_tensor_drift_data_native_terminal_snapshots.rs"]
mod terminal_snapshots;
#[path = "dev_tensor_drift_data_native_trailing_value_loops.rs"]
mod trailing_value_loops;
#[path = "dev_tensor_drift_data_native_value_loop_exits.rs"]
mod value_loop_exits;

pub(super) fn checks() -> impl Iterator<Item = &'static DevTensorDriftCheckSpec> {
    CHECKS
        .iter()
        .chain(flat_values::CHECKS.iter())
        .chain(loop_calls::CHECKS.iter())
        .chain(loop_work::CHECKS.iter())
        .chain(helper_entries::CHECKS.iter())
        .chain(bool_rebinding::CHECKS.iter())
        .chain(aggregate_rebinding::CHECKS.iter())
        .chain(aggregate_values::CHECKS.iter())
        .chain(scoped_captures::CHECKS.iter())
        .chain(aggregate_carries::CHECKS.iter())
        .chain(bool_carries::CHECKS.iter())
        .chain(literal_loops::CHECKS.iter())
        .chain(value_loop_exits::CHECKS.iter())
        .chain(trailing_value_loops::CHECKS.iter())
        .chain(counted_returns::CHECKS.iter())
        .chain(control_elision::CHECKS.iter())
        .chain(terminal_continuations::CHECKS.iter())
        .chain(terminal_snapshots::CHECKS.iter())
}

const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-completed-closure",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "collect_profile(module, layouts, Some(layouts), &ScalarHelpers::new())",
            "typed_call_type",
            "validate_body(functions[name.as_str()], &admitted, layouts, loop_layouts)",
            "contains_calls",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-local-arguments",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "scalar_helpers::typed_call_type",
            "depth > 64",
            "collect::<Option<Vec<_>>>()",
            "locals.available",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-logical-arguments",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested_conditions.rs",
        required_patterns: &[
            "control_values::value_type(value, &scope, catalog, layouts)",
            "Logical edges can also occur",
            "NirExpr::Call { callee, args }",
            "control_values::collect_inputs(&rhs, &mut inputs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-admission-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/calls_tests.rs",
        required_patterns: &[
            "iteration_calls_use_completed_scalar_closure_and_scoped_arguments",
            "iteration_calls_reject_transitive_effects_cycles_and_wrong_kinds",
            "iteration_call_arguments_retain_expression_depth_and_header_atom_limits",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-runtime-evidence",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_calls.rs",
        required_patterns: &[
            "iteration_scalar_calls_preserve_argument_values_order_and_snapshots",
            "nested_call_arguments_execute_left_to_right_before_entering_the_callee",
            "logical_call_arguments_and_inferred_bool_results_remain_lazy",
            "unselected_iteration_calls_skip_arguments_but_reached_ignored_arguments_trap",
            "iteration_calls_cannot_bypass_induction_preflight_or_native_closure_limits",
            "CallProbe",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-admission-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops.rs",
        required_patterns: &["mod calls_tests;", "control_loops/calls_tests.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &[
            "mod aggregate_calls;",
            "native_application_bridge/aggregate_calls.rs",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_calls_loops.ns",
            "native_iteration_calls_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_CALLS_SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-call-native-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "iteration_helper_calls_keep_boolean_arguments_lazy_in_native_entry",
            "ignored_iteration_call_arguments_are_evaluated_before_the_call",
            "deep_source_calls_report_a_parser_error_instead_of_aborting_the_compiler",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "source-expression-parser-reentry-limit",
        path: "tools/nuisc/src/frontend/parser_exprs.rs",
        required_patterns: &[
            "self.expression_depth >= 32",
            "self.expression_depth -= 1",
            "source expression nesting exceeds parser limit",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "source-expression-parser-reentry-tests",
        path: "tools/nuisc/src/frontend/parser_expression_limits.rs",
        required_patterns: &[
            "expression_reentry_limit_rejects_deep_calls_groups_and_mixed_arguments",
            "expression_reentry_budget_is_released_between_siblings_and_after_errors",
            "4096",
        ],
    },
];
