use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs",
        required_patterns: &[
            "(exits && lazy_return).then_some(steps)",
            "let mut remaining = 128usize",
            "body.len() > 32",
            "if depth < 8",
            "NirStmt::Print(value) => roots.push((value, 0, false))",
            "computed_record_expression_roots(roots, layouts)",
            "inner.contains_key(name)",
            "Some(result)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-renderer",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs",
        required_patterns: &[
            "fn direct_record_returns",
            "plan::prove(body, scope, result, self.catalog, self.layouts)",
            "*body = rewritten",
            "plan::Step::Print(stmt) => (*stmt).clone()",
            "self.record_field_value(value, &inner)",
            "*declared = Some(ty.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-early-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "builder.direct_record_returns(",
            "function.return_type.as_ref()",
            "builder.declarations(&mut function.body",
            "if function.is_async",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-shared-field-presence",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values.rs",
        required_patterns: &[
            "fn has_logical_record_fields",
            "while let Some(NirExpr::StructLiteral",
            "op: NirBinaryOp::And | NirBinaryOp::Or",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_return_tests.rs",
        required_patterns: &[
            "direct_record_return_preserves_effect_boundaries_original_scopes_and_idempotence",
            "direct_record_return_revokes_prefix_and_field_dependencies_atomically",
            "direct_record_return_rejects_bad_types_hidden_leaves_scope_leaks_and_unsupported_control",
            "direct_record_return_shares_function_statement_expression_edge_and_depth_budgets",
            "direct_record_return_executes_unprojected_logical_fields_after_prefix_effects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/direct_record_return.ns",
        required_patterns: &[
            "fn returned_record(",
            "print(86)",
            "print(87)",
            "print(88)",
            "const prepared: i64 = return_prefix(value, prefix_div)",
            "return Record { spare: spare && field_spare",
            "let picked: Record = returned_record(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-oracle",
        path: "tools/nuisc/tests/native_application_bridge/direct_record_return_cases.rs",
        required_patterns: &[
            "ENTRY_MARKERS",
            "trace.push(86)",
            "trace.push(if input.outer { 87 } else { 88 })",
            "trace.push(104)",
            "checked_div(self.prefix_div)",
            "prefix_div: 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/direct_record_return.rs",
        required_patterns: &[
            "direct_record_return_native_keeps_one_predicate_and_selected_fault_prefixes",
            "direct_record_return_reference_preserves_predicate_counts_and_last_state",
            "typed_effect_returns::reordered",
            "cases::ENTRY_MARKERS[index]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-delivery",
        path: "tools/nuis/tests/native_session_workflow/direct_record_return.rs",
        required_patterns: &[
            "direct_record_return_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified direct record-return witnesses:",
            "Previous logical-field checkpoint:",
            "Previous logical-field task checkpoint:",
            "Previous logical-field blocker checkpoint:",
            "Previous logical-field action checkpoint:",
            "Previous logical-field validation checkpoint:",
            "Previous logical-field artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-record-return-receipt",
        path: "docs/reference/nuis-native-direct-record-return-v1.md",
        required_patterns: &[
            "Direct Return Contract",
            "Twenty-four core failures",
            "twenty-eight native/reference failures",
            "markers 94 and 96-104",
            "active/99",
        ],
    },
];
