use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-dependency-validation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "while let Some(name) = ready.pop_first()",
            "validate_body(functions[name.as_str()], &admitted, layouts, allow_loops)",
            "candidates.remove(&name)",
            "if *count == 0",
            "collect_profile(module, &control_values::FlatLayouts::new(), false)",
            "dependency_collection_handles_deep_unvalidated_structure_iteratively",
            "let mut pending = vec![body]",
            "let mut pending = vec![expr]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-admission-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/loop_calls_tests.rs",
        required_patterns: &[
            "loop_call_catalog_validates_callees_before_iteration_callers",
            "loop_call_catalog_blocks_invalid_dependencies_and_recursive_components",
            "loop_call_dependency_discovery_is_iterative_and_not_a_two_level_profile",
            "2048",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-admission-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops.rs",
        required_patterns: &["control_loops/loop_calls_tests.rs", "mod loop_calls_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-execution-tests",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_loop_calls.rs",
        required_patterns: &[
            "loop_helpers_execute_scalar_and_flat_results_at_actual_iteration_positions",
            "loop_helpers_cross_multiple_validated_loop_call_boundaries",
            "loop_helper_lazy_and_unselected_calls_skip_child_preflight",
            "loop_helper_preflight_remains_per_selected_invocation",
            "loop_helper_discarded_results_and_arguments_keep_arithmetic_traps",
            "loop_helper_arguments_run_before_the_callees_preflight",
            "loop_helper_dependency_order_does_not_bypass_native_closure_budgets",
            "CallProbe",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &[
            "native_application_bridge/aggregate_loop_calls.rs",
            "mod aggregate_loop_calls;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_loop_calls_loops.ns",
            "native_iteration_loop_calls_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_LOOP_CALLS_SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-lifecycle-source",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_loop_calls_loops.ns",
        required_patterns: &[
            "fn split(value: i64, width: i64) -> Parts",
            "let quotient: i64 = value / index;",
            "let remainder: i64 = value % index;",
            "let saved: Parts = parts;",
            "negative(split(width, width - index).quotient)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-native-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "loop_bearing_iteration_helpers_run_through_default_native_entry",
            "discarded_loop_helper_keeps_its_checked_body_failure",
            "native binary exceeded its test deadline",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-iteration-loops-boundaries",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Loop-Bearing Iteration Calls",
            "Per-invocation induction limits alone are not a whole-callback work budget.",
            "aggregate_loop_calls_loops.ns",
            "### Shared Callback Loop-Work Reservations",
        ],
    },
];
