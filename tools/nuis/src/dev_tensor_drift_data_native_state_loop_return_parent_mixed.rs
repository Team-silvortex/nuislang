use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-real-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_mixed.ns",
        required_patterns: &[
            "if outer == skip",
            "if outer == resume",
            "break;",
            "continue;",
            "let stopped = checkpoint(outer)",
            "let fallback = 1 / finish",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-case-matrix",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_mixed_cases.rs",
        required_patterns: &[
            "breaking::source_from(variant, SOURCE)",
            "self.inner.inner.inner.execute_with_parent_exits",
            "Some(self.resume)",
            "Some(self.skip)",
            "pub fn continued_then_break",
            "suffix.resume = -1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-independent-source-model",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs",
        required_patterns: &[
            "parent_break == Some(outer)",
            "parent_continue == Some(outer)",
            "if !leading_parent",
            "trace.push(33)",
            "1i64.checked_div(divisor)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-unit-signals-steps-and-scope",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_mixed_tests.rs",
        required_patterns: &[
            "iteration_effects_return_parent_mixed_keeps_local_break_and_return_signals_independent",
            "iteration_effects_return_parent_mixed_oracle_keeps_break_continue_and_return_distinct",
            "iteration_effects_return_parent_mixed_rejects_step_scope_and_effect_authority_leaks",
            "outlined.break_controls[parent_helper]",
            "assert_eq!(call.skip, call.resume)",
            "assert_eq!(cases::sequences(variant).len(), 32)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-native-reference-and-compatibility",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_mixed.rs",
        required_patterns: &[
            "state_loop_return_parent_mixed_reference_preserves_actual_state_raw_words_and_fault_retention",
            "state_loop_return_parent_mixed_native_preserves_false_zeros_nan_payloads_and_typed_records",
            "state_loop_return_parent_mixed_native_reserves_full_bounds_and_replays_after_prefix_successes",
            "state_loop_return_parent_mixed_keeps_preceding_parent_break_native_profiles_unchanged",
            "for limit in [2, 4, 5]",
            "accepted.push(call)",
            "cases::FALLBACK_OBSERVER",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-aot-grants-and-layout",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_mixed_tests.rs",
        required_patterns: &[
            "native_state_loop_return_parent_mixed_keep_exact_layout_grants_and_effect_order",
            "State{result:bool;total:i64;gain:f32;marker:f64}",
            "sites.iter().filter(|s| *s != site).cloned()",
            "lacks dependency order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_mixed.rs",
        required_patterns: &[
            "state_loop_return_parent_mixed_build_cache_and_source_free_restores_preserve_exact_payloads",
            "cases::continued_then_break(variant)",
            "failures.push(fallback)",
            "fresh.host_objects == cached.host_objects",
            "for restore in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-six-field-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful mixed-parent-exit typed loop returns follow-up:",
            "Previous parent-break typed loop-return checkpoint:",
            "Previous parent-break typed loop-return task checkpoint:",
            "Previous parent-break typed loop-return blocker checkpoint:",
            "Previous parent-break typed loop-return action checkpoint:",
            "Previous parent-break typed loop-return validation checkpoint:",
            "Previous parent-break typed loop-return artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-mixed-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-return-parent-mixed-v1.md",
        required_patterns: &[
            "Mixed Parent Exits And Real Returns",
            "Observable Steps And Fallback",
            "sixteen delivered images",
            "not post-abort memory inspection",
            "active/99",
        ],
    },
];
