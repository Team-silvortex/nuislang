use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-induction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/induction.rs",
        required_patterns: &[
            "prepare_counted_while(",
            "body.split_first()?",
            "body.split_last()?",
            "control_flow::normalize_leading(self.effects, scope)",
            "control_flow::normalize_trailing(self.effects, scope, self.step)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/sequences.rs",
        required_patterns: &[
            "iteration.normalize(scope)?",
            "updates.contains(&prepared.binding_name)",
            "if !iteration.leading",
            "temporaries::validate(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-bounded-normalization",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_flow.rs",
        required_patterns: &[
            "normalize_with_step(effects, scope, Some(step), true)",
            "same_step(rewritten.last()?, step)",
            "bounded && depth >= 32",
            "depth + usize::from(bounded)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-exclusive-outlining",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &[
            "only one outliner may rewrite its source body",
            "control_catalog",
            "helper.may_loop",
            "continue;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-step-timing",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &[
            "induction::parse(condition, &original)",
            "if iteration.leading",
            "|| plan.breaking.is_some()",
            "body.push(step)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-recovery-scope",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/exits.rs",
        required_patterns: &[
            "if iteration.leading",
            "boundary.after.push(copy(induction, &advanced))",
            "carries.push(flag.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-source-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/trailing_tests.rs",
        required_patterns: &[
            "trailing_value_loops_preserve_pre_step_effects_and_canonical_exits",
            "trailing_value_loops_reject_unstepped_continue_and_hidden_mutations",
            "trailing_value_exit_normalization_bounds_source_and_generated_guards",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-native-oracles",
        path: "tools/nuisc/tests/native_application_bridge/trailing_value_loops.rs",
        required_patterns: &[
            "trailing_value_loops_keep_pre_step_indices_mixed_child_scope_and_typed_snapshots",
            "trailing_value_exits_skip_only_unselected_fallible_effects",
            "early_parent_exits_skip_child_preflight",
            "trailing_value_exits_preserve_full_and_late_induction_preflight",
            "CallProbe",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-budgets",
        path: "tools/nuisc/tests/native_application_bridge/trailing_value_loop_budgets.rs",
        required_patterns: &[
            "trailing_value_loops_reserve_full_work_without_advancing_break_indices",
            "trailing_value_continue_debits_one_step_and_only_entered_helpers",
            "exhausted.remaining_loop = 2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/trailing_value_loops.rs",
        required_patterns: &[
            "trailing_value_break_leaves_index_before_step_and_preserves_zero_trips",
            "trailing_value_loops_mix_with_leading_child_exits_and_explicit_continue_steps",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "native_trailing_value_loops_build_cache_and_standalone_relocation",
            "check_workflow(TRAILING_VALUE_LOOPS_SOURCE)",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-trailing-value-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Trailing-Step Value Loops",
            "Value-admitted functions bypass",
            "Returns From Counted Value Loops",
        ],
    },
];
