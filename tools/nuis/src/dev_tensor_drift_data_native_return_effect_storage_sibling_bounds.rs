use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-catalog-and-immutable-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_sibling_bound_tests.rs",
        required_patterns: &[
            "effect_return_storage_sibling_bounds_admit_completed_snapshot_without_new_authority",
            "effect_return_storage_sibling_bounds_preserve_candidate_fallback_and_original_read_rejection",
            "denied.remove(callee)", "assert_eq!(module, before)",
            "shared.signal.name(), independent.signal.name()", "Some(\"candidate_0\".into())",
            "Some(\"frozen\".into())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-typed-structural-rejection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_sibling_bound_tests.rs",
        required_patterns: &[
            "effect_return_storage_sibling_bounds_reject_typed_recomputation_effect_reordering_and_argument_bypasses",
            "body.swap(second, second + 1)", "args[0] = NirExpr::Var(\"carry\".into())",
            "callee: \"bound_seed\".into()", "args[1] = NirExpr::Var(\"divisor\".into())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-independent-yir-and-oracle",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_sibling_bound_tests.rs",
        required_patterns: &[
            "effect_return_storage_sibling_bounds_match_independent_values_effects_and_four_word_reduction",
            "effect_return_storage_sibling_bounds_oracle_distinguishes_child_and_parent_exits",
            "maximum_carry_width(&shared) + 4", "failure.previous.take().unwrap()",
            "assert_eq!(state.total, if variant.leading { 27 } else { 23 })",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-real-shared-snapshot-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.ns",
        required_patterns: &[
            "let frozen = bound_seed(carry, trips, root)",
            "let first_width = bound_extent(frozen, divisor, 1)",
            "let second_width = bound_extent(frozen, divisor + 0 / (second + 1), second)",
            "if mode == 4 { break; }", "if mode == 5 { continue; }",
            "if mode == 2 { break; }", "if mode == 3 { continue; }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-semantic-state-and-distinct-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "let frozen = trips / (1 + parity * parity)", "for child in 0..2",
            "reserve(width, &mut budget)?", "pub fn healthy", "pub fn failures",
            "call.previous = Some(prefix.last().unwrap().next())", "pub const LOOP_LIMIT: u64 = 18",
            "pub const ENTRY_LIMIT: u64 = 192",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-native-state-and-faults",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "sibling_bounds_reference_preserves_snapshot_state_words_and_fault_retention",
            "sibling_bounds_native_keeps_frozen_record_and_exact_reached_failure_prefixes",
            "sibling_bounds_native_entry_limit_160_still_rejects_long_callback",
            "check_helper_entry_limit_rejection",
            "ApplicationSessionPhase::Faulted", "trace.contains(&943)", "!trace.contains(&946)",
            "session.completion_status().is_err()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-native-reservation-not-iteration-charging",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "sibling_bounds_native_charges_each_full_reached_bound_even_after_child_break",
            "[(2, 5), (9, 5), (9, 6)]", "trace.ends_with(&[2, 1, 16, 945])",
            "limit,", "cases::ENTRY_LIMIT",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-source-free-exits-and-both-poisons",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "sibling_bounds_fresh_cache_and_source_free_delivery_keep_snapshot_and_exit_targets",
            "for mode in 0..=9", "call.second = -1", "for restore in 0..2",
            "fresh.host_objects == cached.host_objects", "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-sibling-bounds-v1.md",
        required_patterns: &[
            "One Snapshot, Separate Reached Loops", "Exit Targets And Poisoned Siblings",
            "Reservation And Failure Order", "Certificates And Delivery", "active/99",
            "not measured physical memory or runtime speedup", "Remaining Frontier",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-sibling-bounds-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified shared sibling-bound record witnesses:",
            "Previous record-dependent bound checkpoint:",
            "Previous record-dependent bound task checkpoint:",
            "Previous record-dependent bound blocker checkpoint:",
            "Previous record-dependent bound action checkpoint:",
            "Previous record-dependent bound validation checkpoint:",
            "Previous record-dependent bound artifact checkpoint:",
        ],
    },
];
