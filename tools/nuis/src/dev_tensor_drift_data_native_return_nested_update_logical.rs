use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-bounded-terminal-authority-private-stage-and-shared-budget-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_nested_update_tests.rs", "let mut remaining = 64", "remaining.checked_sub(body.len())?", "if prefix.len() > 16", "if depth >= 8", "!staged.defined.contains(name)", "staged.constants.contains(name)", "staged.local.get(name)? != &scalar_type(\"bool\")", "let authorized = Target(name)", "super::stage_authorized(prefix, scope, signatures, budget, depth, Some(&authorized))?", "if target == name", "Retention is a transport outcome", "proof.inputs.difference(&staged.defined)", "proof.calls.extend(staged.calls)", "SelectedValue::StagedNested(staged.statements, plan)", "prefix.extend(statements)", "nested::join(condition, reads, effects, yes, no, scope)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-private-seals-retention-exact-original-work-and-atomic-veto-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_nested_update_tests.rs",
        required_patterns: &["return_nested_update_logical_effectful_scalar_selections_keep_private_stages_current_target_and_nested_retention", "return_nested_update_logical_effectful_scalar_selections_reject_inherited_sealed_mixed_hidden_and_child_exports_atomically", "return_nested_update_logical_effectful_scalar_selections_charge_each_original_condition_stage_and_leaf_once", "return_nested_update_logical_effectful_scalar_selections_share_child_prefix_depth_capture_and_discovery_limits", "late malformed NIR sibling read", "(8, 15, 2, true)", "for levels in [7, 8]", "for count in [16, 17]", "for count in [30, 31]", "target(then_body, &eligible, &mut 1, 2).is_none()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-original-source-private-unused-stage-nested-retention-and-real-returns",
        path: "tools/nuisc/tests/native_application_bridge/return_nested_update_logical_effectful_scalar_selections.ns",
        required_patterns: &["let child_value = child_work(local, 1 / child)", "let child_value = child_value || false", "let child_value = child_value && true", "const child_seal: bool = child_value || false", "let child_ignored = child_unused_work(1 / child_unused)", "if nested_condition(reply, 1 / nested_cond)", "if child_condition(child_seal, 1 / child_cond) {} else", "let local = local || yes_work(reply, 1 / yes)", "let local = local && no_work(reply, 1 / no)", "if selected", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-yir-transitive-effect-order-exact-state-and-source-free-policy-veto-tests",
        path: "tools/nuisc/src/aot_application_effect_call_return_nested_update_logical_tests.rs",
        required_patterns: &["native_return_nested_update_logical_effectful_scalar_selections_keep_guarded_transitive_update_stage_return_order_and_exact_state", "native_return_nested_update_logical_effectful_scalar_selections_preserve_source_free_policy_and_removed_update_order_vetoes", "assert_eq!(node(sensitive[0]).op.instruction, \"guard_return\")", "yir_core::EdgeKind::Effect", "checked_functions >= 16", "guarded_calls >= 24", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-independent-native-oracle-all-states-poisoned-inactive-args-and-selected-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_nested_update_logical.rs",
        required_patterns: &["fn oracle(", "const STAGES: [i32; 16]", "trace.extend([76, 75, 74])", "trace.push(73)", "native_return_nested_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation", "native_return_nested_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "STAGES.into_iter().enumerate()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_nested_update_logical_tests.rs", "mod return_nested_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_nested_update_logical.rs", "mod return_nested_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-full-flat-private-update-checkpoint-and-ordered-update-follow-up",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Nested private return bool-update follow-up:", "Previous flat-private-update checkpoint:", "Previous flat-private-update task checkpoint:", "Previous flat-private-update blocker checkpoint:", "Previous flat-private-update action checkpoint:", "Previous flat-private-update validation checkpoint:", "Previous flat-private-update artifact checkpoint:", "Prove ordered current-target bool updates inside private return update arms", "native_return_nested_update_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-update-current-contract-and-explicit-ordered-write-boundary",
        path: "docs/reference/nuis-native-effectful-scalar-selection-v1.md",
        required_patterns: &["## Nested Private Return Bool Updates", "terminal exports", "bounded private staging", "original-work budget: 64 nodes", "Ordered writes to the exported target", "return_nested_update_logical_effectful_scalar_selections.ns"],
    },
];
