use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-bounded-declaration-order",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_storage.rs",
        required_patterns: &[
            "MAX_EFFECT_STORAGE_CANDIDATES: usize = 8",
            "candidates.truncate(MAX_EFFECT_STORAGE_CANDIDATES)",
            "candidates.retain(|name| writes.contains(name))",
            "(!candidates.is_empty()).then_some(candidates)",
            "validation::LiteralPrints::Reject",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-independent-attempts",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs",
        required_patterns: &[
            "for storage in storage::candidate_effects(function, layouts, catalog)?",
            "effect_prefix::verify_shared(function, independent, &plan, &storage, layouts, catalog)",
            "return Some(plan)",
            "coalesced_effects(function, layouts, catalog, &plan).unwrap_or(plan)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-authority-and-budget-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_candidates_tests.rs",
        required_patterns: &[
            "effect_return_storage_candidates_reject_first_and_certify_later_without_partial_publication",
            "effect_return_storage_candidates_use_declaration_order_not_sorted_names_or_function_order",
            "effect_return_storage_candidates_bound_attempts_and_keep_exact_fallback_after_exhaustion",
            "effect_return_storage_candidates_retain_barriers_and_the_pure_first_candidate_policy",
            "effect_return_storage_candidates_never_bypass_original_read_authority",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-real-yir-reduction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_candidates_keep_real_reduction_order_and_faults",
            "compare_source_cases(cases::SourceKind::Candidates)",
            "maximum_carry_width(&shared) + 4",
            "let source = kind.source(call.variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-native-state-and-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_candidates_reference_keeps_state_and_fault_retention",
            "shared_return_storage_candidates_native_keeps_raw_words_and_selected_failure_prefixes",
            "shared_return_storage_candidates_native_preserves_full_reached_reservations",
            "check_native(cases::SourceKind::Candidates)",
            "check_bounds(cases::SourceKind::Candidates)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_candidates_fresh_cache_and_source_free_delivery_keeps_state_and_faults",
            "delivery(cases::SourceKind::Candidates)", "for restore in 0..2",
            "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-candidates-v1.md",
        required_patterns: &[
            "Bounded Candidate Attempts", "Executable Controls",
            "at most eight", "declaration order", "active/99",
            "not physical-memory or speedup evidence",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-candidates-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified bounded alternative-candidate storage sharing follow-up:",
            "Previous ordered initialization-prefix checkpoint:",
            "Previous ordered initialization-prefix task checkpoint:",
            "Previous ordered initialization-prefix blocker checkpoint:",
            "Previous ordered initialization-prefix action checkpoint:",
            "Previous ordered initialization-prefix validation checkpoint:",
            "Previous ordered initialization-prefix artifact checkpoint:",
        ],
    },
];
