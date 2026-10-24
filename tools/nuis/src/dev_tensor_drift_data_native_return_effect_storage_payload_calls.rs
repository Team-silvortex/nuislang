use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-payload-calls-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_payload_call_tests.rs",
        required_patterns: &[
            "effect_return_storage_payload_calls_require_catalog_and_preserve_snapshot_authority",
            "effect_return_storage_payload_calls_oracle_distinguishes_ordered_failures_and_raw_fields",
            "effect_return_storage_payload_calls_reject_reordering_duplication_bypass_and_early_pending",
            "effect_return_storage_payload_calls_match_independent_execution_and_carry_width",
            "body.insert(index, body[index].clone())", "args.swap(1, 2)",
            "body.swap(index, index + 1)", "maximum_carry_width(&shared) + 4",
            "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-payload-calls-nuis-helpers",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_payload_helpers.ns",
        required_patterns: &[
            "fn payload_first", "fn payload_second", "fn payload_finish",
            "print(970)", "print(971)", "print(972)", "let checked = 0 / root;",
            "print(973)", "seed.total + first + second + checked + 7",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-payload-calls-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub fn payload_call_variants", "pub payload_calls: bool",
            "fn payload(", "trace.extend([971, 972])", "total: state.total + 10",
            "for mode in payload_fault_modes(variant)", "call.previous = Some(prefix.last().unwrap().next())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-payload-calls-native",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "payload_calls_reference_preserves_last_state_after_ordered_return_failures",
            "payload_calls_native_keeps_raw_words_and_exact_argument_and_callee_failure_prefixes",
            "payload_calls_native_reserves_entered_bounds_before_any_return_expression_effects",
            "for limit in [required - 1, required]", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-payload-calls-source-free",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "payload_calls_fresh_cache_and_source_free_delivery_keep_ordered_return_failures",
            "delivery(cases::payload_call_variants()[0])", "for mode in cases::payload_fault_modes(variant)",
            "for restore in 0..2", "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-payload-calls-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-payload-calls-v1.md",
        required_patterns: &[
            "Selected Return Evaluation", "Publication And Reservations",
            "Source-Free Delivery", "Remaining Frontier", "active/99",
            "four logical YIR words", "before constructing the result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-payload-calls-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified effectful return-payload call witnesses:",
            "Previous returned-snapshot payload checkpoint:",
            "Previous returned-snapshot payload task checkpoint:",
            "Previous returned-snapshot payload blocker checkpoint:",
            "Previous returned-snapshot payload action checkpoint:",
            "Previous returned-snapshot payload validation checkpoint:",
            "Previous returned-snapshot payload artifact checkpoint:",
        ],
    },
];
