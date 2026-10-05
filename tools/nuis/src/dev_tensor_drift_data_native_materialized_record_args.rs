use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-readiness-not-erasability",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_record_views.rs",
        required_patterns: &[
            "Whole uses retain the materialized binding through the read",
            "self.evaluated.as_ref().is_some_and(|e| e.call_results)",
            ".filter(|e| e.call_results && matches!(value, NirExpr::StructLiteral { .. }))",
            "Non-total constructors are never added to erasable",
            "scope.ready.insert(name.to_owned(), ty)",
            "declared.is_none_or(|d| d == ty)",
            "!evaluated.transport_types.contains(&ty.name)",
            "scalar_aliases::validate_expansion(&result, &mut context.remaining)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-transactional-catalog",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs",
        required_patterns: &[
            "result_catalog.retain(|name, _| !generated.contains(name))",
            "if !scoped.contains(&name)",
            "valid_caller(&module.functions[*i].body, &name, &plan)",
            "matches!(input, Input::Keep(_)) || access(arg).is_some()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-unit-and-oracle",
        path:
            "tools/nuisc/src/lowering/buffer_loop_outline/capture_materialized_record_args_tests.rs",
        required_patterns: &[
            "materialized_record_args_keep_constructors_aliases_and_calls_at_original_sites",
            "materialized_record_args_admit_total_views_without_erasing_whole_call_operands",
            "materialized_record_args_reject_nominal_kind_effect_and_version_mismatches",
            "materialized_record_args_keep_scope_control_transport_and_scalar_mode_boundaries",
            "materialized_record_args_keep_whole_consumers_and_checked_unused_constructors",
            "materialized_record_args_budget_and_depth_roll_back_the_whole_body",
            "materialized_record_args_preserve_checked_order_with_an_independent_oracle",
            "materialized_record_args_commit_only_after_every_caller_agrees",
            "for budget in 1..2048",
            "yir.nodes.reverse()",
            "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-full-native-returns",
        path: "tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs",
        required_patterns: &[
            "typed_sparse_materialized_record_arguments_keep_stored_operands_and_full_results",
            "typed_sparse_materialized_record_arguments_keep_whole_consumers_conservative",
            "body.matches(\"call [64 x i64] @nuis_fn_produce(\").count()",
            "body.matches(\"call i64 @nuis_fn_relay(\").count()",
            "Check actual evaluation order and SSA operand identity",
            "value(calls[0])",
            "value(calls[1])",
            "value(calls[4])",
            "value(calls[3])",
            "!body.contains(\"insertvalue [64 x i64]\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-stored-checked-fixture",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn materialized_record_argument_source(checked: bool)",
            "pub fn materialized_record_argument_transport_source()",
            "let first_alias = first_input; let first = produce(first_alias)",
            "let later_alias = later_input; const later: Payload = produce(later_alias)",
            "checked(payload.f62 + 1)",
            "checked(payload.f62)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-source-free-publication",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_materialized_record_arguments_preserve_source_free_order_and_checks",
            "let checked = aliases::materialized_record_argument_source(true)",
            "&aliases::materialized_record_argument_transport_source(),",
            "check_selected_unused_call(&checked)",
            "if flag == 0 { 3 } else { 1 }",
            "for _ in 0..2",
            "!stderr.contains(\"SIGKILL\")",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-tensor-scope",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Materialized record call-argument follow-up:",
            "materialized readiness is not constructor elision or computed-call spilling",
            "native_materialized_record_arguments",
            "active/86 is unchanged",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-materialized-record-arguments-documented-boundary",
        path: "docs/reference/nuis-native-scalar-loop-snapshots-v1.md",
        required_patterns: &[
            "## Materialized Record Call Arguments",
            "Read readiness and erasability remain separate",
            "budgets 1 through 2047",
            "Every caller must",
            "There is no spilling of",
        ],
    },
];
