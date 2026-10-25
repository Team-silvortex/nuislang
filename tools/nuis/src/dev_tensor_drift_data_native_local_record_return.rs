use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-origins",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs",
        required_patterns: &[
            "certified_locals: &BTreeSet<String>",
            "let mut certified_locals = certified_locals.clone()",
            "origins::value(value, &certified_locals)",
            "certified_locals.insert(name.clone())",
            "NirExpr::Var(name) if certified_locals.contains(name)",
            "inner.contains_key(name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-renderer",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs",
        required_patterns: &[
            "plan::Step::Binding { stmt, ty }",
            "self.record_field_value(value, &inner)",
            "plan::Step::Return(value)",
            "*body = rewritten",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-controls",
        path:
            "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_local_return_tests.rs",
        required_patterns: &[
            "local_record_return_certifies_original_initializers_without_a_literal_return",
            "local_record_return_supports_fresh_let_const_inference_and_inherited_scopes",
            "local_record_return_does_not_borrow_unreturned_origins_or_general_alias_authority",
            "local_record_return_revokes_initializer_and_post_construction_dependencies_atomically",
            "local_record_return_keeps_whole_function_expression_and_statement_budgets",
            "local_record_return_executes_unused_lazy_fields_before_post_construction_work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/local_record_return.ns",
        required_patterns: &[
            "let completed: Record = Record {",
            "const completed: Record = Record {",
            "print(83)",
            "const after: i64 = local_after(post_div)",
            "print(84)",
            "return completed",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-oracle",
        path: "tools/nuisc/tests/native_application_bridge/local_record_return_cases.rs",
        required_patterns: &[
            "trace.push(83)",
            "trace.push(107)",
            "checked_div(self.post_div)",
            "trace.push(84)",
            "rows.extend(post_failures())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/local_record_return.rs",
        required_patterns: &[
            "local_record_return_native_keeps_one_predicate_and_selected_fault_prefixes",
            "local_record_return_reference_preserves_predicate_counts_and_last_state",
            "typed_effect_returns::reordered",
            "compiled.yir.functions.len() <= 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-delivery",
        path: "tools/nuis/tests/native_session_workflow/local_record_return.rs",
        required_patterns: &[
            "local_record_return_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified local-origin record-return witnesses:",
            "Previous continuation-join checkpoint:",
            "Previous continuation-join task checkpoint:",
            "Previous continuation-join blocker checkpoint:",
            "Previous continuation-join action checkpoint:",
            "Previous continuation-join validation checkpoint:",
            "Previous continuation-join artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-record-return-receipt",
        path: "docs/reference/nuis-native-local-record-return-v1.md",
        required_patterns: &[
            "Certified Local Origins",
            "Thirty-two core failures",
            "36 native/reference failures",
            "markers 94 and 96-107",
            "active/99",
        ],
    },
];
