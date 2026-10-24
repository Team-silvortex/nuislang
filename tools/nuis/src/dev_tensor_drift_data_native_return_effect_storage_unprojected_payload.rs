use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs",
        required_patterns: &[
            "return_effect_storage_unprojected_payload_tests.rs", "mod unprojected_payload_tests;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_unprojected_payload_tests.rs",
        required_patterns: &[
            "effect_return_storage_unprojected_payload_requires_complete_callee_closure",
            "catalog[\"project_record\"].dependencies.contains(name)",
            "denied.functions.retain(|function| function.name != name)",
            "scalar_helpers::collect_iteration_calls(&denied, &layouts)",
            "assert!(!denied.contains_key(\"project_record\"))", "assert!(!denied.contains_key(\"work\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_unprojected_payload_tests.rs",
        required_patterns: &[
            "effect_return_storage_unprojected_payload_oracle_distinguishes_before_and_after_gain",
            "effect_return_storage_unprojected_payload_rejects_call_and_field_guard_bypasses",
            "effect_return_storage_unprojected_payload_oracle_detects_pruned_reordered_and_repeated_fields",
            "effect_return_storage_unprojected_payload_matches_independent_execution_and_carry_width",
            "for mutation in [6, 7]", "assert_eq!(cases::failures(variant).len(), 31)",
            "assert_ne!(actual, expected", "These are alternative, valid source programs",
            "verify(function, &independent, &shared",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_unprojected_cases.rs",
        required_patterns: &[
            "pub(super) fn returning", "pub(super) fn payload",
            "calls == 3 && marker_fault", "calls == 3 && total_fault",
            "trace.extend([992, 993, 994, 995])", "trace.push(996)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-nuis",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_unprojected_helpers.ns",
        required_patterns: &[
            "fn record_marker", "fn record_result", "fn record_gain", "fn record_total",
            "marker: record_marker(-seed.marker, before)", "gain: record_gain(-seed.gain)",
            "total: record_total(seed.total + delta + checked, after)", "let checked = 0 / root",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-family",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub fn unprojected_payload_variants", "pub unprojected_payload: bool",
            "12..=if variant.unprojected_payload", "return unprojected::payload",
            "if matches!(mode, 18 | 19 | 24..=27 | 32..=35)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-native",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "unprojected_payload_reference_retains_state_after_discarded_field_faults",
            "unprojected_payload_native_evaluates_every_callee_field_before_projection",
            "unprojected_payload_native_reserves_entered_work_before_callee_fields",
            "for limit in [required - 1, required]", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "unprojected_payload_fresh_cache_and_source_free_delivery_keep_discarded_field_effects",
            "delivery(cases::unprojected_payload_variants()[0])", "for restore in 0..2",
            "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-unprojected-payload-v1.md",
        required_patterns: &[
            "Complete Callee Fields", "Authority And Negative Controls", "Native And Delivery Gates",
            "31 categories", "Fifteen type-valid caller mutations", "active/99",
            "conditional and short-circuit work", "Four fewer logical YIR carry words",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-unprojected-payload-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified unprojected-callee-field witnesses:",
            "Previous projected-call payload checkpoint:", "Previous projected-call payload task checkpoint:",
            "Previous projected-call payload blocker checkpoint:", "Previous projected-call payload action checkpoint:",
            "Previous projected-call payload validation checkpoint:", "Previous projected-call payload artifact checkpoint:",
        ],
    },
];
