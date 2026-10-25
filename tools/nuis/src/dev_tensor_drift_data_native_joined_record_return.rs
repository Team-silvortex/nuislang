use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs",
        required_patterns: &[
            "continuation_neutrals_bounded(body, layouts)",
            "continuation_declaration(",
            "&body[index + 1..]",
            "Step::Join(join)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-pure-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs",
        required_patterns: &[
            "Continuation,",
            "fn continuation_declaration",
            "fn continuation_neutrals_bounded",
            "JoinKind::Continuation => true",
            "kind.discarded_records()",
            "name != other",
            "ty != other_ty",
            "super::observed(name, suffix)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-neutral-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_discarded_records.rs",
        required_patterns: &[
            "fn function_bounded",
            "blocks_bounded(vec![body], layouts, false)",
            "let mut remaining = 4096usize",
            "else if require_result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-renderer",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs",
        required_patterns: &[
            "fn joined_declaration",
            "*stmt = self.joined_declaration(&join, scope, fields)",
            "scope.insert(name, ty)",
            "*declared = Some(ty.clone())",
            "self.nested_arm(&join.yes, scope, fields)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-controls",
        path:
            "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_joined_return_tests.rs",
        required_patterns: &[
            "joined_record_return_exports_only_the_typed_result_and_is_order_independent",
            "joined_record_return_reuses_scalar_record_and_bool_join_authority",
            "joined_record_return_revokes_all_join_dependencies_before_mutating",
            "joined_record_return_rejects_partial_nominal_shadowing_effect_and_scope_leaks",
            "joined_record_return_shares_logical_work_across_join_and_return_roots",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/joined_record_return.ns",
        required_patterns: &[
            "let joined: Leaf = if join_decide(outer, join_div)",
            "let prepared: i64 = if spare",
            "join_unused(join_tail + 1)",
            "print(89)",
            "const prepared: i64 = joined.value",
            "return Record { spare: spare && field_spare",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-oracle",
        path: "tools/nuisc/tests/native_application_bridge/joined_record_return_cases.rs",
        required_patterns: &[
            "trace.push(105)",
            "trace.push(106)",
            "trace.push(89)",
            "checked_div(self.join_div)",
            "join_tail.wrapping_add(1)",
            "rows.extend(join_failures())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/joined_record_return.rs",
        required_patterns: &[
            "joined_record_return_native_keeps_one_predicate_and_selected_fault_prefixes",
            "joined_record_return_reference_preserves_predicate_counts_and_last_state",
            "typed_effect_returns::reordered",
            "compiled.yir.functions.len() <= 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-delivery",
        path: "tools/nuis/tests/native_session_workflow/joined_record_return.rs",
        required_patterns: &[
            "joined_record_return_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified continuation-join record-return witnesses:",
            "Previous direct-return checkpoint:",
            "Previous direct-return task checkpoint:",
            "Previous direct-return blocker checkpoint:",
            "Previous direct-return action checkpoint:",
            "Previous direct-return validation checkpoint:",
            "Previous direct-return artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-joined-record-return-receipt",
        path: "docs/reference/nuis-native-joined-record-return-v1.md",
        required_patterns: &[
            "Continuation Join Contract",
            "Thirty core failures",
            "native/reference failures",
            "4,096-node neutral-expansion",
            "active/99",
        ],
    },
];
