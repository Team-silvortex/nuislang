use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "fn discarded_record_declaration",
            "has_discarded_record(&plan, layouts).then_some(plan)",
            "discarded_records::bounded(then_body, else_body, layouts)",
            "(!layouts.scalar(&ty.name) && kind != JoinKind::DiscardedRecords)",
            "Some((Join { condition, yes, no }, false))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-constructor",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs",
        required_patterns: &[
            "fn discarded_record_declaration",
            "scalar_joins::nested_prefix::discarded_record_declaration",
            "self.extract(condition, yes, no, &inner, !*export)",
            ".with_prefix(locals)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "self.nested_declaration(stmt, scope, suffix)",
            "self.record_declaration(stmt, scope, suffix)",
            "self.discarded_record_declaration(stmt, scope, suffix)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-preflight",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_discarded_records.rs",
        required_patterns: &[
            "let mut remaining = 4096usize",
            "neutral(ty, layouts, &mut remaining)",
            "depth >= 64",
            "remaining.checked_sub(cost)",
            "pending.len() + fields.len() > *remaining",
            "depth + 1 >= 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-budget-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_discarded_records_tests.rs",
        required_patterns: &[
            "discarded_record_neutral_expansion_has_exact_node_and_cast_bounds",
            "discarded_record_neutral_depth_cycles_and_invalid_fields_are_rejected_before_synthesis",
            "discarded_record_neutrals_share_one_declaration_budget_without_narrowing_flat_layouts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_discarded_record_tests.rs",
        required_patterns: &[
            "discarded_record_prefix_has_independent_authority_and_no_exported_identity",
            "discarded_record_prefix_revokes_missing_or_effectful_dependencies_atomically",
            "discarded_record_prefix_rejects_erased_reads_effects_resources_and_bad_types",
            "crate::optimize::simplify_nir_module",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/discarded_record_prefix.ns",
        required_patterns: &[
            "let unused: Record = if discarded_decide(inner, condition)",
            "const p: i64 = discarded_first(first)",
            "const p: i64 = discarded_last(last)",
            "leaf: Leaf { unused: p, value: discarded_first(first) }",
            "unused: 1 / tail",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-oracle",
        path: "tools/nuisc/tests/native_application_bridge/discarded_record_prefix_cases.rs",
        required_patterns: &[
            "ENTRY_MARKERS",
            "if self.outer && retained",
            "else if !self.outer && !retained",
            "checked_div(self.first)",
            "checked_div(self.last)",
            "wrapping_sub(value)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/discarded_record_prefix.rs",
        required_patterns: &[
            "discarded_record_prefix_native_keeps_one_predicate_and_selected_fault_prefixes",
            "discarded_record_prefix_reference_preserves_predicate_counts_and_last_state",
            "typed_effect_returns::reordered",
            "cases::ENTRY_MARKERS[index]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-delivery",
        path: "tools/nuis/tests/native_session_workflow/discarded_record_prefix.rs",
        required_patterns: &[
            "discarded_record_prefix_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified discarded-record prefix witnesses:",
            "Previous record-prefix checkpoint:",
            "Previous record-prefix task checkpoint:",
            "Previous record-prefix blocker checkpoint:",
            "Previous record-prefix action checkpoint:",
            "Previous record-prefix validation checkpoint:",
            "Previous record-prefix artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-discarded-record-prefix-receipt",
        path: "docs/reference/nuis-native-discarded-record-prefix-v1.md",
        required_patterns: &[
            "Discarded Record Contract",
            "Neutral Layout Preflight",
            "Twelve failure inputs",
            "active/99",
        ],
    },
];
