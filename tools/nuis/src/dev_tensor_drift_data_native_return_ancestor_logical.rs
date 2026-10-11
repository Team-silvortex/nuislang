use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-shared-fresh-stage-readonly-entering-bindings-and-leaf-reuse",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["pub(super) fn stage(", "if prefix.len() > 16", "if scope.contains_key(name)", "std::slice::from_ref(statement)", "&staged.constants", "stage(prefix, scope, signatures, budget, depth)?", "predicates::prove(value, &local, signatures, budget)", "reads.difference(&defined)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-source-ordered-stage-child-current-scope-private-capture-subtraction-and-complete-discovery",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_trees.rs",
        required_patterns: &["body.last()", "statements.split_last()", "prefixes::stage(prefix, scope, signatures, budget, depth)", "&staged.local", "inputs.extend(child.returned.inputs.difference(&staged.defined).cloned())", "calls.extend(child.returned.calls)", "SelectedValue::StagedNested(staged.statements, plan)", "logical_return: child.logical_return", "if depth >= 8"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-common-guarded-stage-before-child-installer-original-returns",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["StagedNested(Vec<regions::Statement>, Box<Plan>)", "SelectedValue::StagedNested(stage, child)", "body.extend(regions::install(stage, names, bindings, helpers))", "let (statements, result, _) = install(*child, names, bindings, helpers)", "let returns = plan.destination.is_none()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-original-work-capture-depth-current-seal-placement-and-atomic-negative-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_ancestor_tests.rs",
        required_patterns: &["return_ancestor_logical_effectful_scalar_selections_keep_guarded_current_stages_seals_unused_effects_and_private_descendant_captures", "return_ancestor_logical_effectful_scalar_selections_reject_entering_writes_cross_child_leaks_hidden_work_partial_exits_and_late_invalid_types_atomically", "return_ancestor_logical_effectful_scalar_selections_charge_original_stages_child_conditions_and_leaf_returns_once", "return_ancestor_logical_effectful_scalar_selections_bound_private_stages_unused_parent_reads_and_shared_node_edge_expression_depth_work", "assert!(unused < child_gate)", "assert_eq!(names, before_names)", "for count in [16, 17]", "for count in [31, 32]", "for depth in [8, 9]", "for extra in [1, 2]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-transitive-stage-condition-leaf-order-policy-polarity-and-source-free-vetoes",
        path: "tools/nuisc/src/aot_application_effect_call_return_ancestor_logical_tests.rs",
        required_patterns: &["native_return_ancestor_logical_effectful_scalar_selections_keep_stage_condition_leaf_transitive_order_current_seals_and_return_polarities", "native_return_ancestor_logical_effectful_scalar_selections_preserve_source_free_policy_and_removed_stage_order_vetoes", "assert_eq!(predicates, 11)", "assert_eq!(guarded_checks, 13)", "assert_eq!(child_carriers, 2)", "yir_core::EdgeKind::Effect", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-source-current-bool-i64-seals-unused-calls-and-four-complete-return-leaves",
        path: "tools/nuisc/tests/native_application_bridge/return_ancestor_logical_effectful_scalar_selections.ns",
        required_patterns: &["let local = prepare_yes(second, 1 / yp)", "let local = local && stage_rhs(true, 1 / ylogic)", "const sealed: bool = local || false", "let unused = unused_yes(1 / yu)", "let local = prepare_no(second, 1 / np)", "let local = local + 0", "const sealed: bool = (local == 1) || false", "let unused = unused_no(1 / nu)", "if sealed && yes_condition", "if sealed || no_condition", "answer(reply, sealed, second", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-native-full-truth-traces-source-free-policy-bytes-and-selected-unused-stage-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_ancestor_logical.rs",
        required_patterns: &["native_return_ancestor_logical_effectful_scalar_selections_build_cache_restore_and_keep_staged_child_returns_and_caller_continuation", "native_return_ancestor_logical_effectful_scalar_selections_trap_at_selected_ancestor_stages_conditions_and_leaf_rhs_before_publication", "trace.push(87)", "trace.push(88)", "trace.push(89)", "trace.extend([84, 86])", "compile_cache: hit", "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))", "(16, 89, false, true, false, false)", "(18, 86, true, false, true, false)", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-artifact-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_ancestor_logical_tests.rs", "mod return_ancestor_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_ancestor_logical.rs", "mod return_ancestor_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-ancestor-complete-return-tree-full-checkpoint-and-next-private-control-stage-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Private return ancestor staging follow-up:", "Previous complete-return-tree checkpoint:", "Previous complete-return-tree task checkpoint:", "Previous complete-return-tree blocker checkpoint:", "Previous complete-return-tree action checkpoint:", "Previous complete-return-tree artifact checkpoint:", "Prove private bool-update selections before complete logical return trees", "native_return_ancestor_logical_effectful_scalar_selections"],
    },
];
