use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-eligible-terminal-actual-write-owner-and-same-scope-tail-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_trailing_update_tests.rs", "**ty == scalar_type(\"bool\")", "!staged.constants.contains(*name)", "staged.defined.contains(*name)", "authorized.is_some_and(|target| target.name() == *name)", "target(yes, &eligible, &mut remaining, depth + 1)?", "NirStmt::Let { name, .. } if eligible.contains(name)", "NirStmt::Let { .. } | NirStmt::Const { .. } => Some(None)", "let fallback = if yes_target.is_none() && no_target.is_none()", "retention_targets::discover(yes, &eligible", "super::append_authorized(", "Some(&authorized)", ".or(delegated.as_ref())?", "Root candidates have no delegation and still require actual write evidence", "inputs.extend(proof.inputs.difference(&staged.defined).cloned())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-common-append-seal-current-captures-and-single-target-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["if prefix.len() > 16", "fn append_authorized(", "Permission does not define a new version", "scope.contains_key(name)", "!authorized.is_some_and(|target| target.name() == name)", "std::slice::from_ref(statement)", "&staged.local,", "&staged.constants,", "next.inputs.difference(&staged.defined)", "staged.defined.extend(next.defined)", "staged.constants = next.constants"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-private-work-original-budget-adjacent-boundaries-and-atomic-veto-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_trailing_update_tests.rs",
        required_patterns: &["keep_final_private_let_const_unused_work_and_complete_returns", "keep_old_reads_definite_replacements_and_private_tail_captures", "proof.inputs.contains(\"local\")", "reject_no_write_extra_targets_final_seal_mutation_hidden_work_and_late_leaks", "charge_final_original_work_once_without_invented_retention_work", "(7, 14, 2, true)", "for count in [16, 17]", "for writes in [14, 15]", "for levels in [6, 7]", "levels == 6", "fresh.clone(); 64", "fresh; 65", "reject_module(module)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-original-tail-logical-seal-unused-call-and-complete-return-fixture",
        path: "tools/nuisc/tests/native_application_bridge/return_trailing_update_logical_effectful_scalar_selections.ns",
        required_patterns: &["const seal: bool = local || tail_work(second, 1 / tail)", "let final_private = final_work(seal, 1 / final)", "if after_condition(local, 1 / after)", "if terminal_condition(second, 1 / terminal) {} else {}", "return local && answer_work(reply, 1 / answer)", "return local || answer_work(seed, 1 / answer)", "tail: tail, final: final", "print(94)", "print(95)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-artifact-exact-state-guarded-effect-order-and-source-free-policy-veto-tests",
        path: "tools/nuisc/src/aot_application_effect_call_return_trailing_update_logical_tests.rs",
        required_patterns: &["keep_guarded_transitive_update_stage_return_order_and_exact_state", "preserve_source_free_policy_and_removed_update_order_vetoes", "terminal:i64;tail:i64;final:i64;answer:i64", "assert_eq!(node(sensitive[0]).op.instruction, \"guard_return\")", "checked_functions >= 14", "guarded_calls >= 26", "yir_core::EdgeKind::Effect", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-independent-original-oracle-all-states-inactive-poison-cache-source-free-bytes-and-selected-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_trailing_update_logical.rs",
        required_patterns: &["Independent imperative replay of the original source", "const STAGES: [i32; 14]", "trace.push(94)", "trace.push(95)", "native_return_trailing_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation", "native_return_trailing_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_trailing_update_logical_tests.rs", "mod return_trailing_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_trailing_update_logical.rs", "mod return_trailing_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-full-retention-checkpoint-and-independent-next-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Trailing-private-work return bool-update follow-up:", "Previous retention-tail-private-update checkpoint:", "Previous retention-tail-private-update task checkpoint:", "Previous retention-tail-private-update blocker checkpoint:", "Previous retention-tail-private-update action checkpoint:", "Previous retention-tail-private-update validation checkpoint:", "Previous retention-tail-private-update artifact checkpoint:", "terminal private-child selections after existing-target updates", "nuis-native-private-trailing-return-updates-v1.md", "native_return_trailing_update_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-current-contract-authority-same-stage-budget-and-real-native-evidence",
        path: "docs/reference/nuis-native-private-trailing-return-updates-v1.md",
        required_patterns: &["## Target And Tail", "only eligible existing owned unsealed bools", "same staging environment", "no-write candidates cannot manufacture authority", "at-most-16-statement prefix plus one", "64 nodes, 256 expressions, 32 logical edges", "31 captures plus a gate", "seven nodes / 14 expressions / two logical edges", "Fourteen independently selected", "Terminal private-child selections", "formal memory safety", "strict Clippy"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-trailing-update-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_trailing_update_logical.rs", "mod return_trailing_update_logical;", "return_trailing_update_logical::CHECKS"],
    },
];
