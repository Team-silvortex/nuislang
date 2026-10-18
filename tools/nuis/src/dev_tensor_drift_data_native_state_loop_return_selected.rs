use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-real-demand-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected.ns",
        required_patterns: &[
            "@noinline fn extent(trips: i64, divisor: i64) -> i64",
            "let selected = gate != odd",
            "let width = extent(trips, divisor)",
            "while index < width",
            "return returned(choice, current, narrow, wide, index, outer, 1 / root)",
            "work(state.result, state.total + 1, state.gain, state.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-shared-exact-generator",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_values_cases.rs",
        required_patterns: &[
            "self.source_from(SOURCE, \"trips\")",
            "pub fn source_from(self, template: &str, child_bound: &str)",
            "while index < {child_bound}",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-independent-source-model",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs",
        required_patterns: &[
            "variant.source_from(SOURCE, \"width\")",
            "A skipped invocation does not even divide its bound",
            "self.trips.checked_div(self.divisor)?",
            "reserve(width, &mut budget)",
            "pub fn late(variant: Variant)",
            "inactive_sequence(variant, -1, i64::MIN)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-unit-authority-and-prefixes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_selected_tests.rs",
        required_patterns: &[
            "iteration_effects_return_selected_admits_typed_returns_without_pure_authority",
            "iteration_effects_return_selected_oracle_checks_demand_bounds_and_exact_return_prefixes",
            "iteration_effects_return_selected_rejects_scope_effect_and_step_authority_leaks",
            "let escaped = width",
            "continue; print(99)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-aot-grants-and-order",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_return_selected_tests.rs",
        required_patterns: &[
            "native_state_loop_return_selected_keep_exact_layout_grants_and_effect_order",
            "State{result:bool;total:i64;gain:f32;marker:f64}",
            "sites.iter().filter(|s| *s != site).cloned()",
            "lacks dependency order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-native-and-reference-proof",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected.rs",
        required_patterns: &[
            "state_loop_return_selected_reference_preserves_actual_state_raw_words_and_fault_retention",
            "state_loop_return_selected_native_preserves_false_zeros_nan_payloads_and_typed_records",
            "state_loop_return_selected_native_reserves_only_reached_bounds_after_completed_parent_work",
            "ApplicationSessionPhase::Faulted",
            "assert_eq!(rejected_callbacks, 104)",
            "(\"extent\", cases::BOUND_OBSERVER)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-source-free-proof",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_selected.rs",
        required_patterns: &[
            "state_loop_return_selected_build_cache_and_source_free_restores_preserve_exact_payloads",
            "cases::inactive_sequence(variant, -1, i64::MIN)",
            "compile_cache: hit",
            "fresh.host_objects == cached.host_objects",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "for _ in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful selected typed effectful loop returns follow-up:",
            "Previous typed loop-return checkpoint:",
            "Previous typed loop-return task checkpoint:",
            "Previous typed loop-return blocker checkpoint:",
            "Previous typed loop-return action checkpoint:",
            "Previous typed loop-return validation checkpoint:",
            "Previous typed loop-return artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-selected-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-return-selected-v1.md",
        required_patterns: &[
            "Demand-Only Child Entry",
            "Selected Returns And Prior Progress",
            "thirty-two delivered images",
            "not post-abort memory inspection",
            "active/99",
        ],
    },
];
