use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-origins",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs",
        required_patterns: &[
            "let certified_origin = origins::value(value, &certified_locals)",
            "origins::join(&join, result, &certified_locals)",
            "if &ty == result && certified_origin",
            "certified_locals.insert(name.clone())",
            "inner.contains_key(name)",
            "let mut certified_locals = certified_locals.clone()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-renderer",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs",
        required_patterns: &[
            "plan::Step::Binding { stmt, ty }",
            "*declared = Some(ty.clone())",
            "self.record_field_value(value, &inner)",
            "*body = rewritten",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_alias_return_tests.rs",
        required_patterns: &[
            "alias_record_return_preserves_original_copies_and_adds_no_reconstruction_helpers",
            "alias_record_return_propagates_only_fresh_exact_let_const_values_through_lexical_scopes",
            "alias_record_return_rejects_unrelated_origins_repacking_join_only_results_and_stale_names",
            "alias_record_return_revokes_original_initializer_and_intervening_dependencies_atomically",
            "alias_record_return_chains_consume_the_existing_whole_function_budgets",
            "alias_record_return_executes_unsimplified_copy_chains_without_replaying_fields",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/alias_record_return.ns",
        required_patterns: &[
            "let alias: Record = completed",
            "const alias: Record = completed",
            "const final_record: Record = alias",
            "let final_record = alias",
            "const after: i64 = local_after(post_div)",
            "return final_record",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-oracle",
        path: "tools/nuisc/tests/native_application_bridge/alias_record_return_cases.rs",
        required_patterns: &[
            "local_record_return_cases.rs",
            "Pure copies add no work",
            "pub use prior::",
            "ENTRY_MARKERS",
            "include_str!(\"alias_record_return.ns\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/alias_record_return.rs",
        required_patterns: &[
            "alias_record_return_native_keeps_one_predicate_and_selected_fault_prefixes",
            "alias_record_return_reference_preserves_predicate_counts_and_last_state",
            "typed_effect_returns::reordered",
            "compiled.yir.functions.len() <= 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-delivery",
        path: "tools/nuis/tests/native_session_workflow/alias_record_return.rs",
        required_patterns: &[
            "alias_record_return_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified value-alias record-return witnesses:",
            "Previous local-origin checkpoint:",
            "Previous local-origin task checkpoint:",
            "Previous local-origin blocker checkpoint:",
            "Previous local-origin action checkpoint:",
            "Previous local-origin validation checkpoint:",
            "Previous local-origin artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-alias-record-return-receipt",
        path: "docs/reference/nuis-native-alias-record-return-v1.md",
        required_patterns: &[
            "Certified Value Copies",
            "Thirty-two core failures",
            "36 native/reference failures",
            "bypasses source simplification",
            "active/99",
        ],
    },
];
