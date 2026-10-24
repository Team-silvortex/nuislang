use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs",
        required_patterns: &[
            "pub(super) fn logical_declaration",
            "prefix::computed_expression_roots",
            "(join.condition, 0, true)",
            "(join.yes, 0, false)",
            "(join.no, 0, false)",
            "scalar_arms(&join, scope, catalog, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-shared-factory",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "scalar_joins::logical_declaration(",
            ".gated_condition(join.condition, scope)",
            ".unwrap_or_else(|| join.condition.clone())",
            "self.extract(condition, yes, no, scope, false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_logical_tests.rs",
        required_patterns: &[
            "logical_scalar_join_requires_completed_dependencies_before_any_mutation",
            "logical_scalar_join_uses_existing_lazy_edges_once_and_is_order_independent",
            "logical_scalar_join_total_predicates_need_no_lazy_helper",
            "logical_scalar_join_preserves_edge_depth_work_and_scope_boundaries_atomically",
            "assert_eq!(names, old_names)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/logical_scalar_join.ns",
        required_patterns: &[
            "(decide(choice, condition) && right(rhs)) || fallback(other)",
            "@noinline fn right",
            "@noinline fn fallback",
            "unused: 1 / tail",
            "while index < trips",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/logical_scalar_join_cases.rs",
        required_patterns: &[
            "let conjunction = if lhs",
            "let decision = if conjunction",
            "checked_div(self.condition)",
            "checked_div(self.rhs)",
            "checked_div(self.other)",
            "rhs: i64::MIN",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/logical_scalar_join.rs",
        required_patterns: &[
            "logical_scalar_join_native_keeps_one_predicate_and_selected_fault_prefixes",
            "logical_scalar_join_reference_preserves_predicate_counts_and_last_state",
            "cases::OBSERVERS",
            "calls(&event.events, callee)",
            "execute_native(Some(-2))",
            "typed_effect_returns::reordered",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-delivery",
        path: "tools/nuis/tests/native_session_workflow/logical_scalar_join.rs",
        required_patterns: &[
            "logical_scalar_join_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified logical-predicate declaration witnesses:",
            "Previous computed-predicate checkpoint:",
            "Previous computed-predicate task checkpoint:",
            "Previous computed-predicate blocker checkpoint:",
            "Previous computed-predicate action checkpoint:",
            "Previous computed-predicate validation checkpoint:",
            "Previous computed-predicate artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-scalar-join-receipt",
        path: "docs/reference/nuis-native-logical-scalar-predicates-v1.md",
        required_patterns: &[
            "Logical Predicate Contract",
            "4,096-work/64-depth",
            "Eleven failure cases",
            "markers 83/84/85",
            "strict scalar-write",
            "active/99",
        ],
    },
];
