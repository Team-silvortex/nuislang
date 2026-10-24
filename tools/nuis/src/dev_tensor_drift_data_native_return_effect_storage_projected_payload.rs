use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs",
        required_patterns: &[
            "return_effect_storage_projected_payload_tests.rs", "mod projected_payload_tests;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_field_payload_tests.rs",
        required_patterns: &[
            "if variant.projected_payload", "project_record", "project_marker_arg",
            "project_result_arg", "project_gain_arg", "project_total_arg",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_projected_payload_tests.rs",
        required_patterns: &[
            "effect_return_storage_projected_payload_requires_record_and_argument_call_authority",
            "effect_return_storage_projected_payload_oracle_distinguishes_each_call_and_fault_stage",
            "effect_return_storage_projected_payload_rejects_duplication_bypass_and_removed_checks",
            "effect_return_storage_projected_payload_matches_independent_execution_and_carry_width",
            "for mutation in 0..6", "(true, 11) => 1573",
            "assert_eq!(cases::failures(variant).len(), 23)",
            "scalar_helpers::validate_iteration_body", "verify(function, &independent, &shared",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_projected_cases.rs",
        required_patterns: &[
            "project_record({other}, project_marker_arg(1), 1).marker",
            "project_record(carry, project_result_arg(2), 1).result != true",
            "project_record(carry, project_total_arg(4), 1).total",
            "pub(super) fn returning", "trace.extend([989, 986])",
            "gain: gain ^ (1 << 31)", "marker: marker ^ (1 << 63)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-nuis",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_projected_helpers.ns",
        required_patterns: &[
            "fn project_record", "print(986)", "let checked = 0 / root", "print(987)",
            "marker: -seed.marker", "gain: -seed.gain", "total: seed.total + delta + checked",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-family",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub fn projected_payload_variants", "pub projected_payload: bool",
            "else if variant.projected_payload",
            "return projected::payload", "for mode in payload_fault_modes(variant)",
            "if matches!(mode, 18 | 19 | 24..=27 | 32..=35)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-native",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "projected_payload_reference_preserves_complete_state_after_argument_and_callee_faults",
            "projected_payload_native_preserves_once_only_calls_raw_fields_and_fault_prefixes",
            "projected_payload_native_reserves_entered_work_before_projected_calls",
            "for limit in [required - 1, required]", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "projected_payload_fresh_cache_and_source_free_delivery_preserve_each_call_occurrence",
            "delivery(cases::projected_payload_variants()[0])", "for restore in 0..2",
            "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-projected-payload-v1.md",
        required_patterns: &[
            "Call Then Project", "Authority And Failure Gates", "23 failure categories",
            "four logical YIR carry words", "Remaining Frontier", "active/99",
            "unprojected field", "existing general lowering path",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-projected-payload-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified projected-call return-payload witnesses:",
            "Previous mixed-origin payload checkpoint:", "Previous mixed-origin payload task checkpoint:",
            "Previous mixed-origin payload blocker checkpoint:", "Previous mixed-origin payload action checkpoint:",
            "Previous mixed-origin payload validation checkpoint:", "Previous mixed-origin payload artifact checkpoint:",
        ],
    },
];
