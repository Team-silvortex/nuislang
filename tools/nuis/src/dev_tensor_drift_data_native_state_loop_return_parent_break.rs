use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-real-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_break.ns",
        required_patterns: &[
            "if outer == skip",
            "print(25)",
            "break;",
            "let stopped = checkpoint(outer)",
            "let fallback = 1 / finish",
            "print(32)",
            "return returned(choice, current, narrow, wide, index, outer, 1 / root)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-case-matrix",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_break_cases.rs",
        required_patterns: &[
            "continuing::source_from(variant, SOURCE)",
            "self.inner.inner.execute_with_parent_exits",
            "Some(self.finish)",
            "pub fn child_continued",
            "call.finish = 2",
            "fallback.finish = 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-independent-source-model",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs",
        required_patterns: &[
            "parent_break: Option<i64>",
            "fallback_divisor: Option<i64>",
            "parent_break == Some(outer)",
            "trace.push(33)",
            "1i64.checked_div(divisor)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-unit-signal-and-scope-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_break_tests.rs",
        required_patterns: &[
            "iteration_effects_return_parent_break_keeps_local_break_and_return_signals_independent",
            "iteration_effects_return_parent_break_oracle_distinguishes_fallback_from_false_zero_returns",
            "iteration_effects_return_parent_break_rejects_step_scope_and_effect_authority_leaks",
            "outlined.break_controls[parent_helper]",
            "Some(2)",
            "Some(1)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-native-reference-and-compatibility",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_break.rs",
        required_patterns: &[
            "state_loop_return_parent_break_reference_preserves_actual_state_raw_words_and_fault_retention",
            "state_loop_return_parent_break_native_preserves_false_zeros_nan_payloads_and_typed_records",
            "state_loop_return_parent_break_native_reserves_full_bounds_and_replays_after_prefix_successes",
            "state_loop_return_parent_break_keeps_preceding_parent_continue_native_profiles_unchanged",
            "for limit in [2, 4, 5]",
            "accepted.push(call)",
            "cases::FALLBACK_OBSERVER",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-aot-grants-and-layout",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_break_tests.rs",
        required_patterns: &[
            "native_state_loop_return_parent_break_keep_exact_layout_grants_and_effect_order",
            "State{result:bool;total:i64;gain:f32;marker:f64}",
            "sites.iter().filter(|s| *s != site).cloned()",
            "lacks dependency order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_break.rs",
        required_patterns: &[
            "state_loop_return_parent_break_build_cache_and_source_free_restores_preserve_exact_payloads",
            "fallback.finish = 0",
            "failures.push(fallback)",
            "fresh.host_objects == cached.host_objects",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "for restore in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-six-field-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful parent-break typed loop returns follow-up:",
            "Previous parent-continue typed loop-return checkpoint:",
            "Previous parent-continue typed loop-return task checkpoint:",
            "Previous parent-continue typed loop-return blocker checkpoint:",
            "Previous parent-continue typed loop-return action checkpoint:",
            "Previous parent-continue typed loop-return validation checkpoint:",
            "Previous parent-continue typed loop-return artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-break-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-return-parent-break-v1.md",
        required_patterns: &[
            "Parent Break And Real Returns",
            "Observable Fallback And Step Timing",
            "sixteen delivered images",
            "not post-abort memory inspection",
            "active/99",
        ],
    },
];
