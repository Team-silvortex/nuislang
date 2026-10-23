use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-preheader-bounds-independent-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_preheader_bound_tests.rs",
        required_patterns: &[
            "effect_return_storage_preheader_bounds_admit_dominating_record_without_storage_authority",
            "effect_return_storage_preheader_bounds_reject_typed_recomputation_and_effect_reordering",
            "effect_return_storage_preheader_bounds_match_independent_values_effects_and_carry_reduction",
            "Some(\"saved_extent\".into())", "denied.remove(callee)", "assert_eq!(module, before)",
            "maximum_carry_width(&shared) + 4", "failure.previous.take().unwrap()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-preheader-bounds-source-lifetime-and-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub preheader: bool", "pub fn preheader_variants", "if variant.preheader",
            "source.matches(binding).count(), 1", "source.matches(header).count(), 1",
            "self.variant.preheader && self.selected", "Some(frozen) => frozen",
            "stage == 4 && (!variant.preheader || variant.two_epochs)", "reserve(width, &mut budget)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-preheader-bounds-native-backedge-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "preheader_bounds_reference_keeps_snapshot_across_parent_backedges_and_fault_retention",
            "preheader_bounds_native_keeps_one_snapshot_across_parent_backedges",
            "preheader_bounds_native_reserves_full_bounds_again_after_parent_backedge",
            "[(0, 9), (2, 9), (5, 5), (4, 4), (9, 6), (0, 10)]",
            "ApplicationSessionPhase::Faulted", "session.completion_status().is_err()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-preheader-bounds-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "preheader_bounds_fresh_cache_and_source_free_delivery_keep_snapshot_across_backedges",
            "delivery(cases::preheader_variants()[0])", "fresh.host_objects == cached.host_objects",
            "for restore in 0..2", "fs::remove_dir_all(&output)", "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-preheader-bounds-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-preheader-bounds-v1.md",
        required_patterns: &[
            "One Snapshot Across Backedges", "Independent Reservations And Failures",
            "four-word logical carry reduction", "not measured physical memory or speedup",
            "Certificates And Delivery", "Remaining Frontier", "active/99",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-preheader-bounds-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified parent-preheader bound record witnesses:",
            "Previous shared sibling-bound record checkpoint:",
            "Previous shared sibling-bound record task checkpoint:",
            "Previous shared sibling-bound record blocker checkpoint:",
            "Previous shared sibling-bound record action checkpoint:",
            "Previous shared sibling-bound record validation checkpoint:",
            "Previous shared sibling-bound record artifact checkpoint:",
        ],
    },
];
