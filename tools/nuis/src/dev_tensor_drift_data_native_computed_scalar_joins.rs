use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs",
        required_patterns: &[
            "pub(super) fn computed_declaration",
            "(join.condition, 0, false)",
            "(join.yes, 0, false)",
            "(join.no, 0, false)",
            "control_values::value_type(join.condition, scope, catalog, layouts)",
            "scalar_arms(&join, scope, catalog, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-shared-factory",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "scalar_joins::computed_declaration(",
            "let mut args = vec![condition]",
            "Branch calls, projections and arithmetic stay inside the helper.",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_computed_tests.rs",
        required_patterns: &[
            "computed_scalar_join_requires_both_completed_dependencies_and_is_idempotent",
            "computed_scalar_join_evaluates_only_one_original_predicate_as_argument_zero",
            "computed_scalar_join_keeps_effects_logic_types_scope_and_shared_budget_fail_closed",
            "computed_scalar_join_shares_comparison_kinds_without_widening_result_types",
            "effectful_pure.contains_key(\"decide\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/computed_scalar_join.ns",
        required_patterns: &[
            "@noinline fn decide",
            "let proof = 1 / condition",
            "return choice == (proof != 0)",
            "if decide(choice, condition) { quotient(value, yes) } else { value % no }",
            "unused: 1 / tail",
            "while index < trips",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/computed_scalar_join_cases.rs",
        required_patterns: &[
            "DECISION_OBSERVER",
            "input.choice == (proof != 0)",
            "checked_div(self.condition)",
            "trace.push(83)",
            "condition: i64::MIN",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/computed_scalar_join.rs",
        required_patterns: &[
            "computed_scalar_join_native_keeps_one_predicate_and_selected_fault_prefixes",
            "computed_scalar_join_reference_preserves_predicate_counts_and_last_state",
            "decisions(&event.events)",
            "cases::DECISION_OBSERVER",
            "execute_native(Some(-2))",
            "typed_effect_returns::reordered",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-delivery",
        path: "tools/nuis/tests/native_session_workflow/computed_scalar_join.rs",
        required_patterns: &[
            "computed_scalar_join_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified computed-predicate declaration witnesses:",
            "Previous guarded-scalar checkpoint:",
            "Previous guarded-scalar task checkpoint:",
            "Previous guarded-scalar blocker checkpoint:",
            "Previous guarded-scalar action checkpoint:",
            "Previous guarded-scalar validation checkpoint:",
            "Previous guarded-scalar artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-scalar-join-receipt",
        path: "docs/reference/nuis-native-computed-scalar-predicates-v1.md",
        required_patterns: &[
            "Computed Predicate Contract",
            "4,096-work/64-depth",
            "Eight failure cases",
            "marker 83",
            "strict scalar-write",
            "active/99",
        ],
    },
];
