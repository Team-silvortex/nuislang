use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs",
        required_patterns: &[
            "pub(super) fn logical_value_declaration",
            "*join.ty != scalar_type(\"bool\")",
            "(join.condition, 0, true)",
            "(join.yes, 0, true)",
            "(join.no, 0, true)",
            "scalar_arms(&join, scope, catalog, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-shared-factory",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "scalar_joins::logical_value_declaration(",
            "[(&mut yes, join.yes), (&mut no, join.no)]",
            "self.gated_condition(source, scope)",
            "arm.body = vec![NirStmt::Return(Some(value))]",
            "self.extract(condition, yes, no, scope, false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_logical_values_tests.rs",
        required_patterns: &[
            "logical_value_join_requires_completed_pure_dependencies_before_mutation",
            "logical_value_join_helpers_remain_inside_selected_arms_and_are_idempotent",
            "logical_value_join_supports_either_arm_and_ready_computed_or_logical_conditions",
            "logical_value_join_shares_all_root_budgets_and_keeps_hidden_logic_and_scope_closed",
            "logical_value_join_inline_checked_rhs_keeps_selected_and_skipped_faults",
            "assert_eq!(names, old_names)",
            "*condition = chain(10)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/logical_value_join.ns",
        required_patterns: &[
            "let selected: bool = if decide(choice, condition) && true",
            "(yes_head(yes) && yes_tail(rhs)) || false",
            "(no_head(no) || no_tail(other)) && true",
            "value: encode(selected, value)",
            "unused: 1 / tail",
            "while index < trips",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/logical_value_join_cases.rs",
        required_patterns: &[
            "let selected = if condition",
            "checked_div(self.yes)",
            "checked_div(self.rhs)",
            "checked_div(self.no)",
            "checked_div(self.other)",
            "seed.wrapping_add(self.value)",
            "pub const LAST_STATE: i64 = -2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/logical_value_join.rs",
        required_patterns: &[
            "logical_value_join_native_keeps_one_predicate_and_selected_fault_prefixes",
            "logical_value_join_reference_preserves_predicate_counts_and_last_state",
            "cases::OBSERVERS",
            "calls(&event.events, callee)",
            "execute_native(Some(cases::LAST_STATE))",
            "typed_effect_returns::reordered",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-delivery",
        path: "tools/nuis/tests/native_session_workflow/logical_value_join.rs",
        required_patterns: &[
            "logical_value_join_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified logical-value declaration witnesses:",
            "Previous logical-predicate checkpoint:",
            "Previous logical-predicate task checkpoint:",
            "Previous logical-predicate blocker checkpoint:",
            "Previous logical-predicate action checkpoint:",
            "Previous logical-predicate validation checkpoint:",
            "Previous logical-predicate artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-value-receipt",
        path: "docs/reference/nuis-native-logical-scalar-values-v1.md",
        required_patterns: &[
            "Logical Value Contract",
            "4,096-work/64-depth/32-logical-edge",
            "Ten failure cases",
            "markers 83-87",
            "Strict scalar-write",
            "active/99",
        ],
    },
];
