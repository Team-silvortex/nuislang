use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-explicit-discovery-policy",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_storage.rs",
        required_patterns: &[
            "fn candidate_effects", "fn candidate_with_prints",
            "validation::LiteralPrints::Reject", "validation::LiteralPrints::Allow",
            "NirStmt::Print(NirExpr::Int(_))", "_ => break",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-independent-eligibility",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs",
        required_patterns: &[
            "NirStmt::Print(NirExpr::Int(_)) => continue",
            "verify_independent(function, independent, layouts)?",
            "scalar_helpers::validate_iteration_body(function, &independent.body",
            "scalar_helpers::validate_iteration_body(function, &shared.body",
            "if !mutable || ty != *result",
            "None if !initialized => initialized = true",
            "Some(previous) if initialized && previous == result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-source-and-mutation-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_prefix_tests.rs",
        required_patterns: &[
            "effect_return_storage_prefix_discovers_dominating_records_without_pure_authority",
            "effect_return_storage_prefix_cannot_cross_control_flow_or_dynamic_prints",
            "effect_return_storage_prefix_rejects_moved_deleted_or_duplicated_prints",
            "effect_return_storage_prefix_rechecks_declarations_and_never_grants_original_reads",
            "effect_return_storage_prefix_keeps_independent_fallback_for_new_inner_writes",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-reference-carry-reduction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_prefix_keeps_real_carry_reduction_and_ordered_effects",
            "compare_sources(true)", "maximum_carry_width(&shared) + 4",
            "match kind.execute(call, false, None)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-native-initializer-failure-and-exits",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_prefix_reference_keeps_ordered_prints_and_fault_retention",
            "shared_return_storage_prefix_native_keeps_raw_words_and_once_only_effects",
            "shared_return_storage_prefix_native_preserves_full_reached_reservations",
            "shared_return_storage_prefix_native_stops_at_a_failing_initializer_before_later_prints",
            "&[5, 701]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_prefix_fresh_cache_and_source_free_delivery_keeps_ordered_prints",
            "delivery(cases::SourceKind::Prefix)", "kind.execute(*call, false, None).unwrap().1",
            "for restore in 0..2", "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-prefix-v1.md",
        required_patterns: &[
            "Ordered Initialization Discovery", "Executable Controls",
            "four-transport-word", "not physical-memory or speedup evidence",
            "active/99", "first syntactically eligible carry",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-prefix-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified ordered initialization-prefix storage sharing follow-up:",
            "Previous record-storage sharing checkpoint:",
            "Previous record-storage sharing task checkpoint:",
            "Previous record-storage sharing blocker checkpoint:",
            "Previous record-storage sharing action checkpoint:",
            "Previous record-storage sharing validation checkpoint:",
            "Previous record-storage sharing artifact checkpoint:",
        ],
    },
];
