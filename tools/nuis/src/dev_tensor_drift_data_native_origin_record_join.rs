use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs",
        required_patterns: &[
            "mod origins;",
            "let certified_origin = origins::value(value, &certified_locals)",
            "origins::join(&join, result, &certified_locals)",
            "certified_locals.insert(join.yes.name.to_owned())",
            "continuation_neutrals_bounded(body, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-classifier",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/record_return_origins.rs",
        required_patterns: &[
            "fn value(",
            "control_values::has_logical_record_fields(value)",
            "certified.contains(origin)",
            "join.yes.ty == result",
            "join.no.ty == result",
            "arm(&join.yes, result, certified)",
            "arm(&join.no, result, certified)",
            "if *export && join(child, result, &certified)",
            "let Cow::Borrowed(source)",
            "origin_join_rejects_synthetic_or_unrelated_arms_without_a_union_shortcut",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-renderer",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs",
        required_patterns: &[
            "plan::Step::Join(join) => self.joined_declaration(join, &mut inner, true)",
            "plan::prove(body, scope, result, self.catalog, self.layouts)",
            "*body = rewritten",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_origin_join_tests.rs",
        required_patterns: &[
            "origin_record_join_exports_only_the_common_result_and_keeps_original_effect_order",
            "origin_record_join_accepts_original_literals_local_copies_nested_joins_and_inherited_roots",
            "origin_record_join_requires_both_results_and_rejects_unused_roots_and_private_state",
            "origin_record_join_revokes_selected_field_prefix_and_post_dependencies_before_allocation",
            "origin_record_join_shares_expression_edges_and_nested_depth_before_provenance_walks",
            "origin_record_join_executes_raw_two_arm_provenance_without_eager_unselected_fields",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/origin_record_join.ns",
        required_patterns: &[
            "if outer { print(87); } else { print(88); }",
            "let merged: Record = if outer",
            "let completed: Record = Record {",
            "const completed: Record = Record {",
            "const after: i64 = local_after(post_div)",
            "let final_record = merged",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-oracle",
        path: "tools/nuisc/tests/native_application_bridge/origin_record_join_cases.rs",
        required_patterns: &[
            "alias_record_return_cases.rs",
            "independent predecessor execution contract",
            "pub use prior::",
            "ENTRY_MARKERS",
            "include_str!(\"origin_record_join.ns\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/origin_record_join.rs",
        required_patterns: &[
            "origin_record_join_native_keeps_one_predicate_and_selected_fault_prefixes",
            "origin_record_join_reference_preserves_predicate_counts_and_last_state",
            "typed_effect_returns::reordered",
            "compiled.yir.functions.len() <= 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-delivery",
        path: "tools/nuis/tests/native_session_workflow/origin_record_join.rs",
        required_patterns: &[
            "origin_record_join_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified two-arm origin-join witnesses:",
            "Previous value-alias checkpoint:",
            "Previous value-alias task checkpoint:",
            "Previous value-alias blocker checkpoint:",
            "Previous value-alias action checkpoint:",
            "Previous value-alias validation checkpoint:",
            "Previous value-alias artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-origin-record-join-receipt",
        path: "docs/reference/nuis-native-origin-record-join-v1.md",
        required_patterns: &[
            "Two Original Arms",
            "Thirty-two core failures",
            "36 native/reference failures",
            "Raw-NIR execution bypasses simplification",
            "active/99",
        ],
    },
];
