use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-scalar-call-field-exact-bounded-demand",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_record_copies.rs",
        required_patterns: &[
            "fn scalar_observation(",
            "charge(remaining, fields.len())?",
            "layouts.scalar(&ty.name) && control_values::supported_type(&ty, layouts)",
            "if !direct || !scalar_observation(&path, candidates, layouts, remaining)?",
            "pending.push((child, false))",
            "dead_records::ready_type(value, inputs, layouts, remaining, depth)",
            "!transport_types.contains(&ty.name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-call-field-positive-negative-differential-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_record_scalar_calls_tests.rs",
        required_patterns: &[
            "private_record_copies_scalar_calls_prune_unobserved_copies_but_keep_checked_fields",
            "private_record_copies_scalar_calls_require_exact_nested_scalar_paths",
            "private_record_copies_scalar_calls_do_not_authorize_aggregate_transport",
            "private_record_copies_scalar_calls_keep_unknown_paths_and_budget_failure_conservative",
            "private_record_copies_scalar_calls_keep_constructed_and_encoded_transport_provenance",
            "private_record_copies_scalar_calls_preserve_selected_constructor_work_differentially",
            "yir.nodes.reverse()",
            "execute(&module),",
            "expected,",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-call-field-full-width-native-proof",
        path: "tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs",
        required_patterns: &[
            "typed_sparse_scalar_call_fields_reduce_private_captures_without_dropping_calls",
            "typed_sparse_scalar_call_fields_keep_unobserved_checked_constructor_calls",
            "signature.matches(\"i64 %\").count(), 2",
            "body.matches(\"call i64 @nuis_fn_relay(\").count(), 2",
            "check_sparse_captures(&source, Some(&[3, 3]), 0)",
            "signature.contains(\"[64 x i64] %arg1\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-call-field-distinct-aggregate-fixture",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn scalar_transport_source()",
            "pub fn scalar_checked_transport_source()",
            "pub fn aggregate_transport_source()",
            "return consume_snapshot(saved)",
            "f62: checked(payload.f62)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-call-field-source-free-selected-failure",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_scalar_call_field_checks_remain_selected_after_source_free_restore",
            "&aliases::scalar_transport_source(), Some(&[2, 2])",
            "&aliases::aggregate_transport_source(), Some(&[0, 2])",
            "observed.len(),",
            "if flag == 0 { 3 } else { 1 }",
            "for _ in 0..2",
            "rejected_before_open(project.command(",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-call-field-selected-tensor-evidence",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Scalar call-field follow-up:",
            "captures two i64 fields plus its guard",
            "not a benchmark, fresh device/cross-platform execution or general local/call-result capture closure",
            "native_scalar_call_field_checks native_evaluated_scalar_records native_post_loop_snapshots",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-call-field-documentation-boundary",
        path: "docs/reference/nuis-native-scalar-loop-snapshots-v1.md",
        required_patterns: &[
            "## Scalar Call-Field Demand",
            "two i64 fields plus its guard",
            "not aggregate transport",
            "48 independent arithmetic-oracle cases",
            "No public/source/callback/FFI ABI",
            "not general local/call-result capture closure",
        ],
    },
];
