use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-preferred-terminal-target-fallback-owned-exact-unsealed-eligibility-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_retention_update_tests.rs", "effectful_selections_return_retention_targets.rs", "let fallback = if yes_target.is_none() && no_target.is_none()", "**ty == scalar_type(\"bool\")", "!staged.constants.contains(*name)", "staged.defined.contains(*name)", "authorized.is_some_and(|target| target.name() == *name)", "let mut remaining = 64", "retention_targets::discover(yes, &eligible, &mut remaining, depth + 1)?", "retention_targets::discover(no, &eligible, &mut remaining, depth + 1)?", "retention_targets::merge(yes, no)?", ".or(fallback.as_ref())", "!owner_type(staged.local.get(name)?)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-readonly-bounded-actual-write-discovery-not-fresh-authority-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_retention_targets.rs",
        required_patterns: &["This is write evidence only", "remaining.checked_sub(body.len())?", "eligible.contains(name)", "NirStmt::Let { .. } | NirStmt::Const { .. } => None", "if depth < 8", "discover(then_body, eligible, remaining, depth + 1)?", "discover(else_body, eligible, remaining, depth + 1)?", "target = merge(target, next)?", "_ => return None", "yes != no", "Some(yes.or(no))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-old-read-replacement-private-scope-exact-work-and-adjacent-boundary-veto-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_retention_update_tests.rs",
        required_patterns: &["keep_nonterminal_writes_current_conditions_private_work_and_both_polarities", "preserve_first_reads_replacements_and_conditional_current_captures", "proof.inputs.contains(\"local\")", "reject_fresh_or_no_write_authority_extra_targets_seals_hidden_work_and_late_leaks", "(8, 16, 2, true)", "for count in [16, 17]", "for writes in [14, 15]", "for levels in [5, 6]", "levels == 5", "write.clone(); 64", "write; 65", "Some(None)", "reject_module(module)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-original-ordered-writes-unused-checks-current-tail-conditions-and-complete-return-fixture",
        path: "tools/nuisc/tests/native_application_bridge/return_retention_update_logical_effectful_scalar_selections.ns",
        required_patterns: &["let observed = before_work(local, 1 / before)", "let local = observed && first", "let ignored = unused_work(1 / unused)", "let local = local || replacement(reply, 1 / replace)", "if after_condition(local, 1 / after)", "if terminal_condition(second, 1 / terminal) {} else {}", "let local = local && no_work(reply, 1 / no)", "return local && answer_work(reply, 1 / answer)", "return local || answer_work(seed, 1 / answer)", "print(92)", "print(93)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-artifact-guarded-transitive-effect-order-exact-state-and-source-free-policy-veto-tests",
        path: "tools/nuisc/src/aot_application_effect_call_return_retention_update_logical_tests.rs",
        required_patterns: &["keep_guarded_transitive_update_stage_return_order_and_exact_state", "preserve_source_free_policy_and_removed_update_order_vetoes", "after:i64;terminal:i64;answer:i64", "assert_eq!(node(sensitive[0]).op.instruction, \"guard_return\")", "yir_core::EdgeKind::Effect", "checked_functions >= 12", "guarded_calls >= 22", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-independent-original-order-oracle-inactive-poison-all-states-selected-traps-cache-and-restored-bytes-tests",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_retention_update_logical.rs",
        required_patterns: &["Independent imperative replay of the original source", "const STAGES: [i32; 12]", "trace.push(92)", "trace.push(93)", "native_return_retention_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation", "native_return_retention_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "STAGES.into_iter().enumerate()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_retention_update_logical_tests.rs", "mod return_retention_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_retention_update_logical.rs", "mod return_retention_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-complete-ordered-checkpoint-and-trailing-private-work-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Retention-tail private return bool-update follow-up:", "Previous ordered-private-update checkpoint:", "Previous ordered-private-update task checkpoint:", "Previous ordered-private-update blocker checkpoint:", "Previous ordered-private-update action checkpoint:", "Previous ordered-private-update validation checkpoint:", "Previous ordered-private-update artifact checkpoint:", "Prove private trailing scalar work after current-target bool updates", "nuis-native-private-retention-return-updates-v1.md", "native_return_retention_update_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-current-contract-discovery-authority-shared-work-and-original-native-evidence",
        path: "docs/reference/nuis-native-private-retention-return-updates-v1.md",
        required_patterns: &["## Bounded Write Evidence", "## Original Retention Work", "Terminal exports remain the preferred", "Fresh child declarations cannot enter", "64-statement counter", "no-write candidates do not manufacture authority", "64 nodes, 256 expressions, 32 logical edges", "31 captures plus a gate", "eight nodes / 16 expressions", "Twelve independently selected", "Private scalar work after the last target update", "formal memory safety", "strict Clippy"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-retention-update-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_retention_update_logical.rs", "mod return_retention_update_logical;", "return_retention_update_logical::CHECKS"],
    },
];
