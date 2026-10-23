use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-fresh-definition-and-exact-writes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs",
        required_patterns: &[
            "if scope.contains_key(storage)", "if !mutable || ty != *result",
            "None if !initialized => initialized = true",
            "Some(previous) if initialized && previous == result",
            "scalar_helpers::validate_iteration_body(function, &independent.body",
            "scalar_helpers::validate_iteration_body(function, &shared.body",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-source-and-mutation-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_reassignment_tests.rs",
        required_patterns: &[
            "effect_return_storage_reassignment_preserves_each_typed_write_and_independent_pending",
            "effect_return_storage_reassignment_refuses_constants_parameters_missing_and_changed_types",
            "effect_return_storage_reassignment_certificate_rejects_typed_but_reordered_or_missing_work",
            "effect_return_storage_reassignment_retains_original_read_authority_and_shared_failure_fallback",
            "effect_return_storage_reassignment_keeps_bounded_alternative_order_without_duplicate_attempts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-real-yir-reduction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_reassignment_keeps_real_reduction_and_independent_values_effects",
            "compare_source_cases(cases::SourceKind::Reassigned)",
            "maximum_carry_width(&shared) + 4", "match kind.execute(call, false, None)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-explicit-fixture-modes",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs",
        required_patterns: &[
            "pub enum SourceKind", "pub fn with_reassignments", "Self::Reassigned",
            "carry.total + 3", "carry.total - 3", "trace.insert(2, 703)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-native-prefix-failure-and-state",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_reassignment_reference_keeps_state_bits_and_fault_retention",
            "shared_return_storage_reassignment_native_keeps_raw_words_and_ordered_effects",
            "shared_return_storage_reassignment_native_preserves_full_reached_reservations",
            "shared_return_storage_reassignment_failure_keeps_old_state_and_stops_before_later_prints",
            "&[5, 701, 703]", "before_failure: prefix.to_vec()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_reassignment_fresh_cache_and_source_free_delivery_keeps_writes_and_faults",
            "delivery(cases::SourceKind::Reassigned)", "for restore in 0..2",
            "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-reassignment-v1.md",
        required_patterns: &[
            "Fresh Definition And Ordered Writes", "Executable Controls",
            "NIR does not retain", "not physical-memory or speedup evidence",
            "active/99", "[5, 701, 703]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-reassignment-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified ordered same-record prefix reassignment follow-up:",
            "Previous bounded candidate-attempt checkpoint:",
            "Previous bounded candidate-attempt task checkpoint:",
            "Previous bounded candidate-attempt blocker checkpoint:",
            "Previous bounded candidate-attempt action checkpoint:",
            "Previous bounded candidate-attempt validation checkpoint:",
            "Previous bounded candidate-attempt artifact checkpoint:",
        ],
    },
];
