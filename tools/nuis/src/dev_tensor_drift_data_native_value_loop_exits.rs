use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-source-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/sequences.rs",
        required_patterns: &[
            "control_flow::contains_exit(iteration.effects, false)",
            "updates.contains(&prepared.binding_name)",
            "iteration.normalize(scope)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-normalization",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_flow.rs",
        required_patterns: &[
            "normalize_with_step(effects, scope, None, true)",
            "bounded && depth >= 32",
            "depth + usize::from(bounded)",
            "A child loop owns its control scope",
            "same_step(rewritten.last()?, step)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-advanced-index",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/exits.rs",
        required_patterns: &[
            "boundary.before.push(copy(&advanced, induction))",
            "effects.insert(0, copy(&advanced, induction))",
            "boundary.after.push(copy(induction, &advanced))",
            "carries.push(advanced)",
            "carries.push(flag.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-shared-outlining",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &[
            "branches::collect_bindings(&function.body, names)",
            "self.break_controls.insert(name.clone(), flag.clone())",
            "plan.breaking.as_ref() == Some(&param.name)",
            "body.push(exits::guard(flag))",
            "output.extend(boundary.after)",
            "preserve_entry_flow && function.name == \"main\"",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-entry-fast-path",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/entry_flow.rs",
        required_patterns: &[
            "collect_pure_helper_functions(module)",
            "collect_inlineable_pure_helper_exprs(module)",
            "collect_pure_helper_blocks(module)",
            "prepare_flow_while(condition, body, &helpers, &inline, &blocks)",
            "prepare_post_flow_while(condition, body, &helpers, &inline, &blocks)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-child-capture",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/validation.rs",
        required_patterns: &[
            "if let EffectTypes::Values(..) = types",
            "iteration.normalize(locals)?",
            "child_mutations.writable.insert(flow.running.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-source-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/exits_tests.rs",
        required_patterns: &[
            "leading_step_exits_reuse_scoped_guards_and_keep_child_control_local",
            "leading_step_exit_recovery_avoids_future_source_bindings",
            "leading_step_exits_do_not_admit_extra_steps_or_hidden_invalid_effects",
            ".repeat(33)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-execution",
        path: "tools/nuisc/tests/native_application_bridge/value_loop_exits.rs",
        required_patterns: &[
            "value_loop_exits_keep_nested_scope_advanced_indices_and_typed_snapshots",
            "value_loop_exits_skip_only_their_suffix_and_preserve_selected_failures",
            "early_parent_exits_skip_child_preflight",
            "value_loop_exits_do_not_bypass_full_or_late_induction_preflight",
            "CallProbe",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-budgets",
        path: "tools/nuisc/tests/native_application_bridge/value_loop_exit_budgets.rs",
        required_patterns: &[
            "value_loop_exits_reserve_full_work_but_charge_only_entered_helpers",
            "value_loop_exits_parent_break_does_not_refund_unentered_iterations",
            "value_loop_exits_continue_keeps_exact_iteration_and_entry_debits",
            "The outer loop still keeps",
            "exhausted.remaining_loop = 2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/value_loop_exits.rs",
        required_patterns: &[
            "leading_step_break_preserves_advanced_index_and_zero_trip_seed",
            "leading_step_nested_exits_keep_parent_suffix_and_continue_scope",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "native_value_loop_exits_build_cache_and_standalone_relocation",
            "check_workflow(VALUE_LOOP_EXITS_SOURCE)",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops.rs",
        required_patterns: &["mod exits;", "mod exits_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-leading-exit-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Leading-Step Value Loop Exits",
            "reserves its full bound without refund",
            "### Trailing-Step Value Loops",
        ],
    },
];
