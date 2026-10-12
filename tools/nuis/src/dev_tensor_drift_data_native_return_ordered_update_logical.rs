use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-owner-only-capability-and-terminal-authority-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_ordered_update_tests.rs", "pub(super) struct Target<'a>(&'a str)", "authorized: Option<&Target<'_>>", "!staged.defined.contains(name)", "!authorized.is_some_and(|target| target.name() == name)", "staged.constants.contains(name)", "!owner_type(staged.local.get(name)?)", "let authorized = Target(name)", "super::stage_authorized(&[], scope, signatures, budget, depth, Some(&authorized))?", "super::append_authorized(", "for (index, statement) in prefix.iter().enumerate()" ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-shared-staging-no-permission-as-definition-and-readonly-default-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["stage_authorized(prefix, scope, signatures, budget, depth, None)", "authorized: Option<&updates::Target<'_>>", "updates::owner_type(scope.get(target.name())?)", "Permission does not define a new version", "defined: BTreeSet::new()", "depth, authorized,", "if scope.contains_key(name)", "!matches!(statement, NirStmt::Let { .. })", "!authorized.is_some_and(|target| target.name() == name)", "next.inputs.difference(&staged.defined)", "staged.defined.extend(next.defined)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-current-captures-shared-exact-boundaries-and-atomic-veto-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_ordered_update_tests.rs",
        required_patterns: &["separate_delegated_authority_from_source_defined_versions", "proof.inputs.contains(\"local\"),old_required", "reject_extra_entering_writes_seals_mixed_targets_and_late_leaks_atomically", "(10, 21, 4, true)", "for count in [16, 17]", "for writes in [14, 15]", "for extra in [false, true]", "for levels in [5, 6]", "levels == 5", "reject_module(module)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-original-source-observed-current-version-unused-work-and-terminal-retention",
        path: "tools/nuisc/tests/native_application_bridge/return_ordered_update_logical_effectful_scalar_selections.ns",
        required_patterns: &["if reply { let local = local || rhs_work(first, 1 / rhs); }", "let observed = before_work(local, 1 / before)", "let local = observed && first", "if mid_condition(local, 1 / mid)", "let ignored = unused_work(1 / unused)", "let local = local || replacement(reply, 1 / replace)", "if mid_condition(reply, 1 / mid) {} else", "return local && answer_work(reply, 1 / answer)", "return local || answer_work(seed, 1 / answer)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-yir-transitive-order-state-policy-and-removed-order-veto-tests",
        path: "tools/nuisc/src/aot_application_effect_call_return_ordered_update_logical_tests.rs",
        required_patterns: &["keep_guarded_transitive_update_stage_return_order_and_exact_state", "preserve_source_free_policy_and_removed_update_order_vetoes", "assert_eq!(node(sensitive[0]).op.instruction, \"guard_return\")", "yir_core::EdgeKind::Effect", "checked_functions >= 10", "guarded_calls >= 18", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-independent-source-order-oracle-states-traps-source-free-byte-policy-checks",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_ordered_update_logical.rs",
        required_patterns: &["Independent imperative replay of the original source", "const STAGES: [i32; 10]", "trace.push(77)", "local = observed && first", "trace.push(83)", "native_return_ordered_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation", "native_return_ordered_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "STAGES.into_iter().enumerate()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_ordered_update_logical_tests.rs", "mod return_ordered_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_ordered_update_logical.rs", "mod return_ordered_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-contract-complete-nested-checkpoint-and-retention-only-discovery-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Ordered private return bool-update follow-up:", "Previous nested-private-update checkpoint:", "Previous nested-private-update task checkpoint:", "Previous nested-private-update blocker checkpoint:", "Previous nested-private-update action checkpoint:", "Previous nested-private-update validation checkpoint:", "Previous nested-private-update artifact checkpoint:", "Prove nonterminal-only current-target bool updates with terminal retention-only selections", "nuis-native-private-ordered-return-updates-v1.md", "native_return_ordered_update_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ordered-update-current-contract-proof-authority-original-budget-and-fresh-acceptance",
        path: "docs/reference/nuis-native-private-ordered-return-updates-v1.md",
        required_patterns: &["## Single-Target Authority", "## Current Versions", "Permission does not define a new version", "Ordinary return staging passes no", "eight selection levels", "31 captures plus a gate", "16 statements", "10 nodes / 21 expressions / four edges", "**601 distinct tests**", "480 successful lifecycle transitions and 26", "Nonterminal-only current-target updates", "formal memory safety", "strict Clippy"],
    },
];
