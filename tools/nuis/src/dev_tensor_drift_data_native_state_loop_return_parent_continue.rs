use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-real-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_continue.ns",
        required_patterns: &[
            "if outer == skip",
            "print(25)",
            "current = current + outer",
            "outer = outer + 1",
            "let width = extent(trips, divisor)",
            "return returned(choice, current, narrow, wide, index, outer, 1 / root)",
            "hole, divisor, skip, after",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-source-case-matrix",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_continue_cases.rs",
        required_patterns: &[
            "parent_leading: bool",
            "if outer != skip {} else",
            "self.inner.execute_with_parent_flow",
            "Some(self.skip)",
            "child_continued(variant)",
            "call.after = 0",
            "inactive_sequence(variant, -1, i64::MIN)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-independent-source-model",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs",
        required_patterns: &[
            "self.execute_with_parent_flow(observed, budget, leading_parent, None)",
            "parent_continue: Option<i64>",
            "parent_continue == Some(outer)",
            "trace.push(25)",
            "current.total += outer",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-unit-scope-and-step-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_continue_tests.rs",
        required_patterns: &[
            "iteration_effects_return_parent_continue_admits_both_parent_steps_without_pure_authority",
            "iteration_effects_return_parent_continue_oracle_checks_steps_skipped_tails_and_later_reservations",
            "iteration_effects_return_parent_continue_rejects_step_scope_and_effect_authority_leaks",
            "child_exit < parent_exit",
            "Some(4)",
            "Some(0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-real-native-reference-and-budget",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_continue.rs",
        required_patterns: &[
            "state_loop_return_parent_continue_reference_preserves_actual_state_raw_words_and_fault_retention",
            "state_loop_return_parent_continue_native_preserves_false_zeros_nan_payloads_and_typed_records",
            "state_loop_return_parent_continue_native_reserves_reached_work_after_continue_and_replays_successes",
            "for limit in [1, 4, 5]",
            "accepted.push(call)",
            "ApplicationSessionPhase::Faulted",
            "cases::BOUND_OBSERVER",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-aot-grants-and-layout",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_continue_tests.rs",
        required_patterns: &[
            "native_state_loop_return_parent_continue_keep_exact_layout_grants_and_effect_order",
            "State{result:bool;total:i64;gain:f32;marker:f64}",
            "sites.iter().filter(|s| *s != site).cloned()",
            "lacks dependency order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_continue.rs",
        required_patterns: &[
            "state_loop_return_parent_continue_build_cache_and_source_free_restores_preserve_exact_payloads",
            "profiles.truncate(1)",
            "child_continued(variant)",
            "fresh.host_objects == cached.host_objects",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "for restore in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-six-field-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful parent-continue typed loop returns follow-up:",
            "Previous trailing-parent typed loop-return checkpoint:",
            "Previous trailing-parent typed loop-return task checkpoint:",
            "Previous trailing-parent typed loop-return blocker checkpoint:",
            "Previous trailing-parent typed loop-return action checkpoint:",
            "Previous trailing-parent typed loop-return validation checkpoint:",
            "Previous trailing-parent typed loop-return artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-continue-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-return-parent-continue-v1.md",
        required_patterns: &[
            "Parent Continue And Selected Returns",
            "Completed Prefix And Demand Reservations",
            "sixteen delivered images",
            "not post-abort memory inspection",
            "active/99",
        ],
    },
];
