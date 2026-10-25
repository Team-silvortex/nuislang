use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-source-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "fn record_declaration",
            "JoinKind::Records",
            "has_record_join(&plan, layouts).then_some(plan)",
            "_ => layouts.scalar(&ty.name)",
            "!result_supported || !super::observed(name, suffix)?",
            "ty != other_ty",
            "(kind == JoinKind::Scalars && !layouts.scalar(&ty.name))",
            "let mut remaining = 128usize",
            "computed_expression_roots(roots)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-shared-constructor",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs",
        required_patterns: &[
            "fn record_declaration",
            "fn typed_nested_declaration",
            "scalar_joins::nested_prefix::record_declaration",
            "self.extract(condition, yes, no, &inner, !*export)",
            ".with_prefix(locals)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_prefix_tests.rs",
        required_patterns: &[
            "record_prefix_join_requires_completed_field_dependencies_without_widening_scalar_proof",
            "record_prefix_join_retains_field_order_layout_and_private_child_scopes",
            "record_prefix_join_rejects_mismatched_partial_resource_and_hidden_field_shapes",
            "record_prefix_join_shares_field_expression_statement_and_logical_budgets",
            "record_prefix_join_supports_nested_value_layouts_and_preserves_unused_field_failures",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/record_prefix_join.ns",
        required_patterns: &[
            "let chosen: Choice = if nested_decide(inner, inner_div)",
            "Choice { value: record_word(temp, record_guard), checksum: record_guard, tag: true, unused: record_unused(record_guard) }",
            "Choice { unused: record_unused(record_guard), tag: false, checksum: record_guard, value: 0 - record_word(temp, record_guard) }",
            "seed == (1 / yes_prefix + record_guard - 1)",
            "seed == (1 / no_prefix + record_guard - 1)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/record_prefix_join_cases.rs",
        required_patterns: &[
            "ENTRY_MARKERS",
            "trace.push(96)",
            "trace.push(97)",
            "trace.push(98)",
            "guard.wrapping_add(1)",
            "[2, i64::MIN, i64::MAX]",
            "[0, -1]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/record_prefix_join.rs",
        required_patterns: &[
            "record_prefix_join_native_keeps_one_predicate_and_selected_fault_prefixes",
            "record_prefix_join_reference_preserves_predicate_counts_and_last_state",
            "cases::ENTRY_MARKERS[index]",
            "typed_effect_returns::reordered",
            "compiled.yir.functions.len() <= 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-delivery",
        path: "tools/nuis/tests/native_session_workflow/record_prefix_join.rs",
        required_patterns: &[
            "record_prefix_join_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified record-prefix declaration witnesses:",
            "Previous nested-scalar checkpoint:",
            "Previous nested-scalar task checkpoint:",
            "Previous nested-scalar blocker checkpoint:",
            "Previous nested-scalar action checkpoint:",
            "Previous nested-scalar validation checkpoint:",
            "Previous nested-scalar artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-prefix-receipt",
        path: "docs/reference/nuis-native-record-prefix-joins-v1.md",
        required_patterns: &[
            "Record Prefix Contract",
            "Thirty-two core failures",
            "thirty-six native/reference failures",
            "markers 83-89 and 94-98",
            "active/99",
        ],
    },
];
