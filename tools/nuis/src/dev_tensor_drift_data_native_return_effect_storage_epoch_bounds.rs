use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-epoch-bounds-independent-identity-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_epoch_bound_tests.rs",
        required_patterns: &[
            "effect_return_storage_epoch_bounds_admit_distinct_readonly_records_without_storage_authority",
            "effect_return_storage_epoch_bounds_reject_typed_wrong_epoch_recomputation_and_reordering",
            "effect_return_storage_epoch_bounds_match_independent_values_effects_and_four_word_reduction",
            "effect_return_storage_epoch_bounds_oracle_distinguishes_epochs_reconstruction_and_fault_order",
            "assert_eq!(widths, [2, 4, 4, 4])", "denied.remove(callee)", "assert_eq!(module, before)",
            "maximum_carry_width(&shared) + 4", "failure.previous.take().unwrap()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-epoch-bounds-source-lifetime-and-seven-faults",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub two_epochs: bool", "pub fn epoch_variants", "assert!(variant.preheader)",
            "bound_seed(carry, trips, root * (second + 3) + 0 / (second + 2))",
            "bound_extent(recent, divisor, 1)", "if child == 0 { recent } else { frozen }",
            "stage == 4 && (!variant.preheader || variant.two_epochs)",
            "5 => call.second = -2", "6 => call.second = -3", "reserve(width, &mut budget)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-epoch-bounds-native-state-and-changing-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "epoch_bounds_reference_keeps_distinct_snapshots_and_last_state_after_seven_faults",
            "epoch_bounds_native_keeps_two_snapshot_identities_across_parent_backedges",
            "epoch_bounds_native_reserves_distinct_changing_bounds_after_backedges",
            "for mode in [0, 2, 5]", "for limit in [required - 1, required]",
            "ApplicationSessionPhase::Faulted", "session.completion_status().is_err()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-epoch-bounds-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "epoch_bounds_fresh_cache_and_source_free_delivery_keep_distinct_snapshot_identities",
            "delivery(cases::epoch_variants()[0])", "for restore in 0..2",
            "fresh.host_objects == cached.host_objects", "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-epoch-bounds-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-epoch-bounds-v1.md",
        required_patterns: &[
            "Two Live Snapshot Identities", "Reached Construction And Seven Faults",
            "Distinct Full Reservations", "Certificates And Delivery", "Remaining Frontier",
            "four-word logical reduction", "not measured physical memory", "active/99",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-epoch-bounds-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified distinct-epoch bound record witnesses:",
            "Previous parent-preheader bound record checkpoint:",
            "Previous parent-preheader bound record task checkpoint:",
            "Previous parent-preheader bound record blocker checkpoint:",
            "Previous parent-preheader bound record action checkpoint:",
            "Previous parent-preheader bound record validation checkpoint:",
            "Previous parent-preheader bound record artifact checkpoint:",
        ],
    },
];
