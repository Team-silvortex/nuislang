use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-snapshot-returns-independent-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_snapshot_return_tests.rs",
        required_patterns: &[
            "effect_return_storage_snapshot_returns_admit_both_records_without_mutable_snapshot_authority",
            "effect_return_storage_snapshot_returns_reject_wrong_record_live_carry_and_early_pending",
            "effect_return_storage_snapshot_returns_match_independent_effects_values_and_carry_reduction",
            "effect_return_storage_snapshot_returns_oracle_keeps_complete_fields_and_skips_later_effects",
            "body.swap(index, index + 1)", "maximum_carry_width(&shared) + 4",
            "failure.previous.take().unwrap()", "denied.remove(callee)", "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-snapshot-returns-complete-record-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub snapshot_returns: bool", "pub fn snapshot_return_variants", "assert!(variant.two_epochs)",
            "[(1, 10, \"recent\"), (6, 11, \"frozen\")]", "Option<State>",
            "total: frozen", "trace.extend([argument.total, self.divisor])",
            "argument.total.checked_div(self.divisor)", "call.mode = if matches!(stage, 2 | 3) { 6 } else { 1 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-snapshot-returns-native-raw-state-and-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "snapshot_returns_reference_preserves_returned_words_and_faulted_last_state",
            "snapshot_returns_native_preserves_older_newer_words_and_reached_fault_prefixes",
            "snapshot_returns_native_reserves_full_entered_bounds_but_skips_later_work",
            "for mode in [1, 6, 10, 11]", "for limit in [required - 1, required]",
            "ApplicationSessionPhase::Faulted", "session.completion_status().is_err()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-snapshot-returns-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "snapshot_returns_fresh_cache_and_source_free_delivery_keep_complete_returned_records",
            "delivery(cases::snapshot_return_variants()[0])", "for mode in [10, 11]",
            "for restore in 0..2", "fresh.host_objects == cached.host_objects",
            "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-snapshot-returns-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-snapshot-returns-v1.md",
        required_patterns: &[
            "Return The Selected Snapshot", "Publication And Exit Order", "Full Entered Reservations",
            "Native And Source-Free Delivery", "Remaining Frontier", "active/99",
            "four-word logical reduction", "not measured", "Direct immutable record reads here do not themselves fault",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-snapshot-returns-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified returned-snapshot payload witnesses:",
            "Previous distinct-epoch bound record checkpoint:",
            "Previous distinct-epoch bound record task checkpoint:",
            "Previous distinct-epoch bound record blocker checkpoint:",
            "Previous distinct-epoch bound record action checkpoint:",
            "Previous distinct-epoch bound record validation checkpoint:",
            "Previous distinct-epoch bound record artifact checkpoint:",
        ],
    },
];
