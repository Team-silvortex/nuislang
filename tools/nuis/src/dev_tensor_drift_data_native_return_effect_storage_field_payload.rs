use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-field-payload-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_field_payload_tests.rs",
        required_patterns: &[
            "effect_return_storage_field_payload_requires_each_field_call_grant",
            "effect_return_storage_field_payload_oracle_keeps_once_only_prefix_and_raw_fields",
            "effect_return_storage_field_payload_rejects_reordering_duplication_and_early_publication",
            "effect_return_storage_field_payload_matches_independent_execution_and_carry_width",
            "fields.swap(0, 3)", "body.insert(index, body[index].clone())",
            "body.swap(index, index + 1)", "args.swap(1, 2)",
            "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-field-payload-nuis",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_field_helpers.ns",
        required_patterns: &[
            "fn field_marker", "fn field_result", "fn field_gain", "fn field_total",
            "print(980)", "print(981)", "print(982)", "let checked = 0 / root;",
            "seed + first + second + checked + 7",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-field-payload-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub fn field_payload_variants", "pub field_payload: bool",
            "marker: field_marker({record}.marker)", "total: field_total({record}.total,",
            "trace.extend([980, 981, 982])", "return payload(argument, self.mode, child, &mut trace)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-field-payload-native",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "field_payload_reference_retains_previous_state_after_partial_record_evaluation",
            "field_payload_native_keeps_source_field_order_and_exact_failure_prefixes",
            "field_payload_native_reserves_entered_bounds_before_any_record_field_effects",
            "for limit in [required - 1, required]", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-field-payload-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "field_payload_fresh_cache_and_source_free_delivery_preserve_complete_publication",
            "delivery(cases::field_payload_variants()[0])", "for restore in 0..2",
            "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-field-payload-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-field-payload-v1.md",
        required_patterns: &[
            "Source Field Order", "Complete Publication", "Source-Free Delivery",
            "Remaining Frontier", "active/99", "four logical YIR words",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-field-payload-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified field-by-field return-payload witnesses:",
            "Previous effectful payload-call checkpoint:",
            "Previous effectful payload-call task checkpoint:",
            "Previous effectful payload-call blocker checkpoint:",
            "Previous effectful payload-call action checkpoint:",
            "Previous effectful payload-call validation checkpoint:",
            "Previous effectful payload-call artifact checkpoint:",
        ],
    },
];
