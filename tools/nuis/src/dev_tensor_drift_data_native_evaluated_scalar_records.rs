use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-bounded-lexical-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_record_views.rs",
        required_patterns: &[
            "pub(super) fn normalize_evaluated(",
            "normalized_mode(function, layouts, Some(evaluated), 65_536)",
            "scope.ready.insert(name.to_owned(), ty)",
            "fn evaluated_scalar_type(",
            "scalar_helpers::typed_call_type(callee, &types, catalog)",
            "scalar_aliases::validate_expansion(&result, &mut context.remaining)?",
            "e.transport_types.contains(&ty.name)",
            "self.erasable.insert(name.to_owned())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-transactional-integration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs",
        required_patterns: &[
            "let scalar_catalog = scalar_helpers::collect(module)",
            "record_views::normalize_evaluated(",
            "&scalar_catalog,",
            "&control_names,",
            "&transport_types,",
            "valid_caller(&module.functions[*i].body, &name, &plan)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-unit-and-oracle-evidence",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_evaluated_record_views_tests.rs",
        required_patterns: &[
            "evaluated_record_views_reuse_local_calls_and_codecs_without_erasing_definitions",
            "evaluated_record_views_preserve_exact_mixed_scalar_kinds_and_signed_zero",
            "evaluated_record_views_keep_whole_transport_and_opaque_or_changed_versions",
            "evaluated_record_views_require_types_control_identity_transport_and_budget",
            "evaluated_record_views_keep_loop_local_evaluation_lexical",
            "evaluated_record_views_reject_deep_rhs_without_installing_partial_views",
            "evaluated_record_views_project_64_fields_only_after_every_caller_validates",
            "evaluated_record_views_preserve_selected_failures_with_an_independent_oracle",
            "assert_eq!(module, before)",
            "yir.nodes.reverse()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-full-width-native-evidence",
        path: "tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs",
        required_patterns: &[
            "typed_sparse_evaluated_scalar_records_remove_copies_without_repeating_rhs",
            "typed_sparse_evaluated_scalar_records_keep_whole_record_transport",
            "body.matches(\"call i64 @nuis_fn_relay(\").count(), 3",
            "!body.contains(\"[64 x i64]\")",
            "check_sparse_captures(&source, Some(&[3, 3]), 0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-distinct-fixtures",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn evaluated_scalar_source(checked: bool)",
            "pub fn evaluated_scalar_transport_source()",
            "const unused: i64 = checked(payload.f62)",
            "return consume_snapshot(second)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-source-free-selected-checks",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_evaluated_scalar_records_preserve_source_free_calls_and_checks",
            "check_selected_unused_call(&aliases::evaluated_scalar_source(true))",
            "&aliases::evaluated_scalar_transport_source(),",
            "Some(&[2, 2])",
            "if flag == 0 { 3 } else { 1 }",
            "!stderr.contains(\"SIGKILL\")",
            "for _ in 0..2",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-tensor-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Evaluated-scalar record-view follow-up:",
            "retains all three original calls with no full-record reconstruction",
            "not constant authority, a benchmark, fresh device execution, aggregate-call-result closure or a new changing-backedge proof",
            "native_scalar_call_field_checks native_evaluated_scalar_records",
            "active/86 is unchanged",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-evaluated-scalar-record-documentation-boundary",
        path: "docs/reference/nuis-native-scalar-loop-snapshots-v1.md",
        required_patterns: &[
            "## Evaluated Scalar Record Views",
            "Every scalar definition remains at its original site",
            "all three original scalar calls",
            "general aggregate-call-result closure or new changing-backedge proof",
            "Eight unit tests",
        ],
    },
];
