use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-existing-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_helper_call_tests.rs",
        required_patterns: &[
            "effect_return_storage_helper_calls_keep_catalog_authority_and_exact_prefix",
            "denied.remove(callee)", "assert!(!pure.contains_key(callee))",
            "effect_return_storage_helper_calls_keep_candidate_fallback_and_reject_unadmitted_callees",
            "assert!(!catalog.contains_key(\"prefix_gate\"))",
            "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-structural-proof-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_helper_call_tests.rs",
        required_patterns: &[
            "effect_return_storage_helper_calls_reject_typed_reordered_deleted_or_duplicated_effects",
            "args.swap(1, 2)", "args[0] = NirExpr::Var(\"carry\".into())",
            "shared.body.insert(3, shared.body[3].clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-yir-comparison",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_helper_calls_keep_real_reduction_and_ordered_argument_effects",
            "compare_source_cases(cases::SourceKind::HelperCalls)",
            "maximum_carry_width(&shared) + 4", "assert_eq!(actual, run(independent.clone()))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-real-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs",
        required_patterns: &[
            "pub fn with_helper_calls", "@noinline fn prefix_record(seed: State",
            "prefix_record(prefix_seed(carry), prefix_delta(", "root + 7", "root + 8",
            "print(712); let check = 0 / root; print(713)", "[710, 711, 714, 712, 713]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-native-fault-boundaries",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_helper_calls_reference_keeps_state_bits_and_fault_retention",
            "shared_return_storage_helper_calls_native_keeps_raw_words_and_ordered_effects",
            "shared_return_storage_helper_calls_native_preserves_full_reached_reservations",
            "shared_return_storage_helper_calls_argument_failure_skips_later_argument_and_callee",
            "shared_return_storage_helper_calls_callee_failure_skips_completion_and_publication",
            "&[5, 701, 710, 711, 714, 712, 713, 703, 710, 711]",
            "&[5, 701, 710, 711, 714, 712, 713, 703, 710, 711, 714, 712]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_helper_calls_fresh_cache_and_source_free_delivery_keeps_order_and_faults",
            "delivery(cases::SourceKind::HelperCalls)", "for root in [-7, -8]",
            "for restore in 0..2", "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-helper-calls-v1.md",
        required_patterns: &[
            "Existing Lowering, Not New Authority", "Argument And Callee Failure",
            "active/99", "not a physical-memory or speedup measurement",
            "Source-Free Delivery", "Remaining Frontier",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-helper-calls-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified effectful record-helper prefix-write witnesses:",
            "Previous ordered prefix-reassignment checkpoint:",
            "Previous ordered prefix-reassignment task checkpoint:",
            "Previous ordered prefix-reassignment blocker checkpoint:",
            "Previous ordered prefix-reassignment action checkpoint:",
            "Previous ordered prefix-reassignment validation checkpoint:",
            "Previous ordered prefix-reassignment artifact checkpoint:",
        ],
    },
];
