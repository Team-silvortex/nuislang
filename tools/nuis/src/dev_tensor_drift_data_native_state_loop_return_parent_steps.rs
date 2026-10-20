use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    // AST checks fix placement without relying on checkout line endings.
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-real-trailing-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_steps.ns",
        required_patterns: &[
            "let width = extent(trips, divisor)",
            "while index < width",
            "print(26)",
            "outer = outer + 1",
            "return returned(choice, current, narrow, wide, index, outer, 1 / root)",
            "work(state.result, state.total + 1, state.gain, state.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-shared-source-and-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs",
        required_patterns: &[
            "source_from(variant, SOURCE)",
            "self.execute_with_parent_step(observed, budget, true)",
            "leading_parent: bool",
            "if leading_parent",
            "if !leading_parent",
            "trace.push(26)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-source-level-case-matrix",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_steps_cases.rs",
        required_patterns: &[
            "self.inner.execute_with_parent_step(observed, budget, false)",
            "((prior.total + 1) % 2 != 0)",
            "call.after = 2",
            "call.rounds = 2",
            "inactive_sequence(variant, -1, i64::MIN)",
            "matches!(v.kind, Kind::F32 | Kind::Record)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-unit-admission-and-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_steps_tests.rs",
        required_patterns: &[
            "iteration_effects_return_parent_steps_admits_typed_returns_without_pure_authority",
            "iteration_effects_return_parent_steps_oracle_checks_demand_bounds_and_exact_return_prefixes",
            "iteration_effects_return_parent_steps_rejects_scope_effect_and_step_authority_leaks",
            "control_loops::induction::parse(condition, body)",
            "outer = outer + 1; outer = outer + 1;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-aot-layout-and-grants",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_steps_tests.rs",
        required_patterns: &[
            "native_state_loop_return_parent_steps_keep_exact_layout_grants_and_effect_order",
            "State{result:bool;total:i64;gain:f32;marker:f64}",
            "sites.iter().filter(|s| *s != site).cloned()",
            "lacks dependency order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-real-native-and-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_parent_steps.rs",
        required_patterns: &[
            "state_loop_return_parent_steps_reference_preserves_actual_state_raw_words_and_fault_retention",
            "state_loop_return_parent_steps_native_preserves_false_zeros_nan_payloads_and_typed_records",
            "state_loop_return_parent_steps_native_reserves_only_reached_bounds_after_completed_parent_work",
            "ApplicationSessionPhase::Faulted",
            "assert_eq!(rejected_callbacks, 52)",
            "(\"extent\", cases::BOUND_OBSERVER)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_steps.rs",
        required_patterns: &[
            "state_loop_return_parent_steps_build_cache_and_source_free_restores_preserve_exact_payloads",
            "profiles.truncate(1)",
            "fresh.host_objects == cached.host_objects",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "for restore in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-six-field-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful trailing-parent typed loop returns follow-up:",
            "Previous selected typed loop-return checkpoint:",
            "Previous selected typed loop-return task checkpoint:",
            "Previous selected typed loop-return blocker checkpoint:",
            "Previous selected typed loop-return action checkpoint:",
            "Previous selected typed loop-return validation checkpoint:",
            "Previous selected typed loop-return artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-parent-steps-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-return-parent-steps-v1.md",
        required_patterns: &[
            "Trailing Parent Steps",
            "Completed And Skipped Step Regions",
            "sixteen delivered images",
            "not post-abort memory inspection",
            "active/99",
        ],
    },
];
