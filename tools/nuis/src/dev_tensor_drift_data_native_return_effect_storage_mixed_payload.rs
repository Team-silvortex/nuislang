use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "NirExpr::FieldAccess { base, .. } => bool_atom(base)",
            "left == scalar_type(\"bool\") && bool_atom(lhs) && bool_atom(rhs)",
            "updates.contains(name) && !locals.available.contains(name) && own != Some(name)",
            "scalar_helpers::typed_call_type(callee, &types, catalog)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-projection-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_projection_tests.rs",
        required_patterns: &[
            "loop_bool_projections_preserve_ordered_self_and_sibling_read_authority",
            "loop_bool_projections_keep_type_call_and_compound_expression_boundaries",
            "for nested in [false, true]", "[NirBinaryOp::Eq, NirBinaryOp::Ne]",
            "check(&valid, &ScalarHelpers::new()).is_none()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_mixed_payload_tests.rs",
        required_patterns: &[
            "effect_return_storage_mixed_payload_keeps_catalog_and_snapshot_authority",
            "effect_return_storage_mixed_payload_oracle_distinguishes_epochs_and_original_self_reads",
            "effect_return_storage_mixed_payload_rejects_wrong_origins_and_forwarded_return_fields",
            "effect_return_storage_mixed_payload_matches_independent_execution_and_carry_width",
            "for mutation in 0..3", "(true, 11) => 1569",
            "scalar_helpers::validate_iteration_body", "verify(function, &independent, &shared",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub fn mixed_payload_variants", "pub mixed_payload: bool",
            "field_result(carry.result != true)",
            "carry.total + frozen.total * 100 + recent.total * 10 + mixed_bias(carry.result)",
            "trace.push(984)", "result: !state.result",
            "+ if state.result { 1000 } else { 2000 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-nuis",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_mixed_helpers.ns",
        required_patterns: &["fn mixed_bias", "print(984)", "return 1000", "return 2000"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-native",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "mixed_payload_reference_preserves_live_and_snapshot_origins_before_publication",
            "mixed_payload_native_keeps_original_self_reads_and_exact_failure_prefixes",
            "mixed_payload_native_preserves_full_reservations_before_alias_sensitive_returns",
            "for limit in [required - 1, required]", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "mixed_payload_fresh_cache_and_source_free_delivery_preserve_all_return_origins",
            "delivery(cases::mixed_payload_variants()[0])", "for restore in 0..2",
            "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-mixed-payload-v1.md",
        required_patterns: &[
            "Boolean Projection Admission", "Mixed Origins And Publication",
            "Native And Delivery Gates", "Remaining Frontier", "active/99",
            "four logical YIR carry words", "admission evidence only",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-mixed-payload-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified mixed-origin return-payload witnesses:",
            "Previous field-payload checkpoint:", "Previous field-payload task checkpoint:",
            "Previous field-payload blocker checkpoint:", "Previous field-payload action checkpoint:",
            "Previous field-payload validation checkpoint:", "Previous field-payload artifact checkpoint:",
        ],
    },
];
