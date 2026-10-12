use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-classifier-retains-only-current-ancestor-without-new-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_terminal_child_update_tests.rs", "fn terminal_private_child(", "let eligible = eligible(staged, Some(authorized));", "let mut remaining = 64;", "retention_targets::discover(yes, &eligible, &mut remaining, depth + 1)?", "retention_targets::merge(yes, no)?", "child != authorized.name() && staged.defined.contains(&child) && !scope.contains_key(&child)", "Classification is write evidence, not another permission or source version", "if terminal_private_child(then_body, else_body, scope, &staged, &authorized, depth)?", "super::append_authorized(", "SelectedValue::Expression(NirExpr::Var(name.into()))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-shared-eligibility-and-original-owner-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["fn eligible(", "**ty == scalar_type(\"bool\")", "!staged.constants.contains(*name)", "staged.defined.contains(*name)", "authorized.is_some_and(|target| target.name() == *name)", "Root candidates have no delegation and still require actual write evidence", "!owner_type(staged.local.get(name)?)", "=> branch(", "inputs.extend(proof.inputs.difference(&staged.defined).cloned())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-atomic-private-child-tests-original-budgets-and-adjacent-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_terminal_child_update_tests.rs",
        required_patterns: &["keep_child_private_and_current_ancestor_through_both_polarities", "capture_old_reads_and_retained_parent_not_child_exports", "reject_no_ancestor_write_incoming_sealed_dual_targets_hidden_work_and_late_leaks", "charge_original_child_work_once_without_parent_transport_work", "keep_prefix_nodes_edges_depth_and_readonly_classification_bounds", "share_complete_external_capture_union", "proof.inputs.contains(\"local\")", "(9, 16, 2, true)", "for writes in [15, 16]", "for writes in [12, 13]", "for levels in [5, 6]", "vec![write; 65]", "for count in [30, 31]", "assert_eq!(module, before)", "assert_eq!(names, before_names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-original-private-child-distinct-value-seal-unused-work-and-return-fixture",
        path: "tools/nuisc/tests/native_application_bridge/return_terminal_child_update_logical_effectful_scalar_selections.ns",
        required_patterns: &["let observed = before_work(local, 1 / before)", "let local = observed || ancestor_work(reply, 1 / ancestor)", "if reply { let local = local && ancestor_work(first, 1 / ancestor); }", "let child = child_initial(local, 1 / child_init)", "if child_condition(reply, 1 / child_cond)", "let child = child || child_work(first, 1 / child_rhs)", "const seal: bool = child || child_tail_work(seed, 1 / child_tail)", "let ignored = child_unused_work(1 / child_unused)", "return local && answer_work(reply, 1 / answer)", "return local || answer_work(seed, 1 / answer)", "print(94)", "print(83)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-artifact-exact-state-guarded-effect-order-and-policy-vetoes",
        path: "tools/nuisc/src/aot_application_effect_call_return_terminal_child_update_logical_tests.rs",
        required_patterns: &["keep_guarded_transitive_update_stage_return_order_and_exact_state", "preserve_source_free_policy_and_removed_update_order_vetoes", "child_init:i64;child_cond:i64;child_rhs:i64;child_unused:i64;child_tail:i64;answer:i64", "assert_eq!(node(sensitive[0]).op.instruction, \"guard_return\")", "checked_functions >= 11", "guarded_calls >= 20", "yir_core::EdgeKind::Effect", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-independent-oracle-private-child-complete-state-cache-source-free-and-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_terminal_child_update_logical.rs",
        required_patterns: &["Independent imperative replay: child updates are work, never the parent's result", "let mut child = local", "result = if local", "trace.push(83)", "trace.push(94)", "const STAGES: [i32; 11]", "native_return_terminal_child_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation", "native_return_terminal_child_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_terminal_child_update_logical_tests.rs", "mod return_terminal_child_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_terminal_child_update_logical.rs", "mod return_terminal_child_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-full-trailing-checkpoint-and-independent-mixed-target-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Terminal-private-child return bool-update follow-up:", "Previous trailing-private-update checkpoint:", "Previous trailing-private-update task checkpoint:", "Previous trailing-private-update blocker checkpoint:", "Previous trailing-private-update action checkpoint:", "Previous trailing-private-update validation checkpoint:", "Previous trailing-private-update artifact checkpoint:", "terminal mixed ancestor/private-child updates with one outward target", "nuis-native-private-terminal-child-return-updates-v1.md", "native_return_terminal_child_update_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-bounded-contract-private-child-original-work-and-native-evidence",
        path: "docs/reference/nuis-native-private-terminal-child-return-updates-v1.md",
        required_patterns: &["## Private Child And Ancestor", "classification grants no new permission or definition", "transports only the actual current ancestor", "still needs actual source write evidence", "64 nodes, 256 expressions, 32 logical edges", "31 captures plus a gate", "nine nodes / 16 expressions / two logical edges", "The child can intentionally differ from the ancestor", "Eleven independently", "Terminal mixed ancestor/private-child updates", "formal memory safety", "strict Clippy"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-terminal-child-update-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_terminal_child_update_logical.rs", "mod return_terminal_child_update_logical;", "return_terminal_child_update_logical::CHECKS"],
    },
];
