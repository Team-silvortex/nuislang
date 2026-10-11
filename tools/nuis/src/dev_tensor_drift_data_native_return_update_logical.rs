use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-update-private-bool-flat-arm-seal-original-budget-and-shared-join-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["if depth >= 8", "NirStmt::Let { name, .. } if eligible.contains(name) => Some(Some(name.clone()))", "!staged.defined.contains(name)", "staged.constants.contains(name)", "staged.local.get(name)? != &scalar_type(\"bool\")", "nested::consume_node(budget)?", "predicates::prove(condition, scope, signatures, budget)", "nested::leaf(statement, &staged.local, signatures, budget)", "Retention is a transport outcome", "inputs: BTreeSet::from([name.into()])", "nested::join(condition, reads, effects, yes, no, scope)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-source-ordered-private-stage-current-scope-and-read-subtraction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["effectful_selections_return_updates.rs", "for statement in prefix", "updates::prove(", "condition, then_body, else_body, staged, signatures, budget, depth, authorized,", "depth,", "proof.inputs.difference(&staged.defined)", "regions::Statement::Selection(plan)", "if scope.contains_key(name)", "&staged.constants", "staged.local = next.local", "staged.constants = next.constants"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-shared-guarded-installer-and-unchanged-existing-target-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs",
        required_patterns: &["Selection(Box<Plan>)", "Statement::Selection(plan)", "output.extend(super::install(*plan, names, bindings, helpers).0)", "Vec::with_capacity(statements.len())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-reusable-original-leaf-and-guarded-join-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_nested.rs",
        required_patterns: &["pub(super) fn leaf(", "pub(super) fn join(", "join(condition, condition_inputs, condition_calls, yes, no, scope)", "if yes.name != no.name || yes.ty != no.ty", "if captures.len() > 31", "destination: Some((name.clone(), false))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-placement-retention-original-work-shared-bounds-and-atomic-negatives",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_update_tests.rs",
        required_patterns: &["return_update_logical_effectful_scalar_selections_keep_private_current_retained_versions_seals_and_guarded_order", "return_update_logical_effectful_scalar_selections_reject_incoming_sealed_nonbool_hidden_nested_and_exit_writes_atomically", "return_update_logical_effectful_scalar_selections_charge_only_original_work_not_retained_transport", "return_update_logical_effectful_scalar_selections_share_prefix_node_capture_edge_and_ancestor_depth_limits", "assert_eq!(names, before_names)", "(6, 12, 2, true)", "for updates in [15, 16]", "for levels in [7, 8]", "for paired in [2, 3]", "for count in [31, 32]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-transitive-order-exact-state-source-free-policy-and-order-vetoes",
        path: "tools/nuisc/src/aot_application_effect_call_return_update_logical_tests.rs",
        required_patterns: &["native_return_update_logical_effectful_scalar_selections_keep_guarded_transitive_update_stage_return_order_and_exact_state", "native_return_update_logical_effectful_scalar_selections_preserve_source_free_policy_and_removed_update_order_vetoes", "assert_eq!(node(sensitive[0]).op.instruction, \"guard_return\")", "yir_core::EdgeKind::Effect", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-source-paired-retained-leaf-local-seals-unused-work-and-caller-continuation",
        path: "tools/nuisc/tests/native_application_bridge/return_update_logical_effectful_scalar_selections.ns",
        required_patterns: &["let local = initialize(seed, 1 / init)", "let local = local || yes_work(reply, 1 / yes)", "let local = local && no_work(reply, 1 / no)", "if retained_condition(local, 1 / retained_cond) {} else", "const sealed: bool = local || false", "let ignored = unused_work(1 / unused)", "let selected = selected && leaf_update(reply, 1 / leaf_rhs)", "return selected && answer_work(reply, 1 / answer)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-native-oracle-full-state-traces-cache-source-free-bytes-and-selected-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_update_logical.rs",
        required_patterns: &["native_return_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation", "native_return_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication", "fn current(", "fn result(", "fn trace(", "compile_cache: hit", "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))", "(9, 84, false, true, false, true)", "(13, 78, true, true, true, false)", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_update_logical_tests.rs", "mod return_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_update_logical.rs", "mod return_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-update-full-ancestor-checkpoint-and-nested-private-update-follow-up",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Private return bool-update follow-up:", "Previous private-ancestor-staging checkpoint:", "Previous private-ancestor-staging task checkpoint:", "Previous private-ancestor-staging blocker checkpoint:", "Previous private-ancestor-staging action checkpoint:", "Previous private-ancestor-staging artifact checkpoint:", "Prove nested private bool-update arms before complete logical return trees", "native_return_update_logical_effectful_scalar_selections"],
    },
];
