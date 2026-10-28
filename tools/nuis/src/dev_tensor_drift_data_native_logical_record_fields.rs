use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "fn logical_field_declaration",
            "logical_fields::present(&plan).then_some(plan)",
            "JoinKind::LogicalFields => !layouts.scalar(&ty.name)",
            "computed_record_expression_roots(roots, layouts)",
            "kind.discarded_records()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-shared-root-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "fn computed_record_expression_roots",
            "roots_with_fields(pending, computed_gate, false, |_, _| false, |_, _, _| false)",
            "ty == scalar_type(\"bool\")",
            "record_root && bool_field(type_name, name)",
            "(lhs.as_ref(), depth + 1, eager_bool, false, lazy_rhs)",
            "logical_argument(callee, index, lazy_rhs)",
            "let mut work = 4096usize",
            "let mut logical_edges = 32usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-shared-logical-constructor",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical.rs",
        required_patterns: &[
            "fn field_logical_root",
            "self.logical_tree_mode(expression, scope, false)",
            "self.logical_tree_mode(expression, scope, true)",
            "let guarded = force",
            "self.extract(left, yes, no, scope, false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-field-order-constructor",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_fields.rs",
        required_patterns: &[
            "fn record_field_value",
            "self.field_logical_root(value, scope)",
            "self.record_field_value(value, scope)",
            "lowered.push((name.clone(), value))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-shared-plan-constructor",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs",
        required_patterns: &[
            "fn logical_field_declaration",
            "scalar_joins::nested_prefix::logical_field_declaration",
            "let fields = kind == JoinKind::LogicalFields",
            "self.record_field_value(value, &inner)",
            ".with_prefix(locals)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-controls",
        path:
            "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical_field_tests.rs",
        required_patterns: &[
            "logical_record_fields_have_independent_authority_and_no_remaining_field_gates",
            "logical_record_fields_revoke_completed_dependencies_and_effects_atomically",
            "logical_record_fields_reject_hidden_arguments_comparisons_projections_and_bad_layouts",
            "logical_record_fields_share_original_statement_edge_and_expression_budgets",
            "logical_record_fields_keep_nested_local_scopes_and_unprojected_field_failures",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/logical_record_fields.ns",
        required_patterns: &[
            "(field_head(head) && field_tail(rhs)) || false",
            "(field_head(head) || field_tail(rhs)) && true",
            "spare: spare && field_spare(spare_div)",
            "outer_result(picked, outer, record_guard)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/logical_record_fields_cases.rs",
        required_patterns: &[
            "ENTRY_MARKERS",
            "trace.push(101)",
            "trace.push(102)",
            "trace.push(103)",
            "checked_div(self.spare_div)",
            "record_guard: 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/logical_record_fields.rs",
        required_patterns: &[
            "logical_record_fields_native_keeps_one_predicate_and_selected_fault_prefixes",
            "logical_record_fields_reference_preserves_predicate_counts_and_last_state",
            "1 | 3 | 4 | 13",
            "cases::ENTRY_MARKERS[index]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-delivery",
        path: "tools/nuis/tests/native_session_workflow/logical_record_fields.rs",
        required_patterns: &[
            "logical_record_fields_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified logical record-field witnesses:",
            "Previous outer-record checkpoint:",
            "Previous outer-record task checkpoint:",
            "Previous outer-record blocker checkpoint:",
            "Previous outer-record action checkpoint:",
            "Previous outer-record validation checkpoint:",
            "Previous outer-record artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-record-fields-receipt",
        path: "docs/reference/nuis-native-logical-record-fields-v1.md",
        required_patterns: &[
            "Typed Field Root Contract",
            "Twenty-two core failures",
            "twenty-six native/reference failures",
            "markers 94 and 96-103",
            "active/99",
        ],
    },
];
