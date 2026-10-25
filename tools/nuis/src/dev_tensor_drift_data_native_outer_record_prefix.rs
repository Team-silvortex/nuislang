use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "fn outer_record_declaration",
            "JoinKind::OuterRecords => !layouts.scalar(&ty.name)",
            "_ => layouts.scalar(&ty.name)",
            "Self::DiscardedRecords | Self::OuterRecords",
            "!result_supported || !super::observed(name, suffix)?",
            "(!require_nested || nested)",
            "ty != other_ty",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-constructor",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs",
        required_patterns: &[
            "fn outer_record_declaration",
            "scalar_joins::nested_prefix::outer_record_declaration",
            "self.extract(condition, yes, no, &inner, !*export)",
            ".with_prefix(locals)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "mod outer_record_tests",
            "self.outer_record_declaration(stmt, scope, suffix)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_outer_record_tests.rs",
        required_patterns: &[
            "outer_record_prefix_has_independent_authority_and_preserves_layout_order_and_scopes",
            "outer_record_prefix_revokes_field_and_nested_dependencies_before_allocation",
            "outer_record_prefix_rejects_nominal_partial_unobserved_resource_and_effectful_results",
            "outer_record_prefix_keeps_complete_source_budgets_and_neutral_preflight",
            "outer_record_prefix_supports_flat_prefixes_and_all_plain_scalar_field_kinds",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/outer_record_prefix.ns",
        required_patterns: &[
            "let picked: Record = if outer",
            "leaf: Leaf { value: outer_word(value, record_guard), unused: outer_unused(record_guard) }",
            "leaf: Leaf { unused: outer_unused(record_guard), value: 0 - outer_word(value, record_guard) }",
            "if picked.check != guard",
            "if picked.tag != outer",
            "outer_result(picked, outer, record_guard)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-oracle",
        path: "tools/nuisc/tests/native_application_bridge/outer_record_prefix_cases.rs",
        required_patterns: &[
            "ENTRY_MARKERS",
            "trace.push(98)",
            "trace.push(99)",
            "trace.push(100)",
            "guard.wrapping_add(1)",
            "[2, i64::MIN, i64::MAX]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/outer_record_prefix.rs",
        required_patterns: &[
            "outer_record_prefix_native_keeps_one_predicate_and_selected_fault_prefixes",
            "outer_record_prefix_reference_preserves_predicate_counts_and_last_state",
            "typed_effect_returns::reordered",
            "cases::ENTRY_MARKERS[index]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-delivery",
        path: "tools/nuis/tests/native_session_workflow/outer_record_prefix.rs",
        required_patterns: &[
            "outer_record_prefix_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified outer-record prefix witnesses:",
            "Previous discarded-record checkpoint:",
            "Previous discarded-record task checkpoint:",
            "Previous discarded-record blocker checkpoint:",
            "Previous discarded-record action checkpoint:",
            "Previous discarded-record validation checkpoint:",
            "Previous discarded-record artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-record-prefix-receipt",
        path: "docs/reference/nuis-native-outer-record-prefix-v1.md",
        required_patterns: &[
            "Outer Record Contract",
            "Sixteen failure inputs",
            "markers 94 and 96-100",
            "active/99",
        ],
    },
];
