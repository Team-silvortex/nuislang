use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-tree-original-owned-bool-authority-and-shared-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_returns.rs",
        required_patterns: &["effectful_selections_return_trees.rs", "return_type != Some(&scalar_type(\"bool\"))", "trees::prepare(condition, yes, no, scope, signatures, pure)", "fn plan(", "if captures.len() > 31", "destination: None"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-shared-original-budgets-complete-leaves-subtree-reads-and-global-discovery",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_trees.rs",
        required_patterns: &["nodes: 64", "expressions: 256", "logical_edges: 32", "if depth >= 8", "nested::consume_node(budget)", "predicates::prove(condition, scope, signatures, budget)", "prefixes::body(statements, scope, signatures, budget, depth)", "let plan = plan(condition, yes, no, scope)", "tree.calls.iter().any", "inputs.extend(yes.inputs.iter().chain(&no.inputs).cloned())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-guarded-installer-and-real-original-exit",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["SelectedValue::Nested(child)", "body.extend(statements)", "SelectedValue::StagedPredicate(statements, predicate)", "let returns = plan.destination.is_none()", "output.push(NirStmt::Return(Some(NirExpr::Var(destination.clone()))))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-structural-placement-pure-child-discovery-late-atomic-vetoes-and-exact-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_tree_tests.rs",
        required_patterns: &["return_tree_logical_effectful_scalar_selections_keep_ancestor_guards_private_leaf_scopes_and_complete_original_exits", "return_tree_logical_effectful_scalar_selections_reject_partial_mixed_inherited_writes_exports_hidden_and_late_invalid_leaves_atomically", "return_tree_logical_effectful_scalar_selections_charge_original_selections_prefixes_and_returns_once_across_pure_children", "return_tree_logical_effectful_scalar_selections_bound_complete_subtree_condition_captures_shared_work_and_selection_depth", "assert_eq!(names, before_names)", "for depth in [8, 9]", "for count in [31, 32]", "for extra in [0, 1, 2]", "for leaves in [33, 34]", "for depth in [30, 31]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-transitive-function-local-order-policy-and-source-free-artifact",
        path: "tools/nuisc/src/aot_application_effect_call_return_tree_logical_tests.rs",
        required_patterns: &["native_return_tree_logical_effectful_scalar_selections_keep_transitive_ancestor_condition_leaf_check_call_order", "native_return_tree_logical_effectful_scalar_selections_preserve_source_free_policy_and_removed_effect_order_vetoes", "assert_eq!(predicates, 8)", "assert_eq!(guarded_checks, 10)", "assert_eq!(child_carriers, 2)", "yir_core::EdgeKind::Effect", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-four-original-leaves-ancestor-conditions-private-seals-and-caller-continuation",
        path: "tools/nuisc/tests/native_application_bridge/return_tree_logical_effectful_scalar_selections.ns",
        required_patterns: &["if choose(first, 1 / outer)", "if second && yes_condition(1 / ycond)", "if second || no_condition(1 / ncond)", "let local = yes_true(seed, 1 / a)", "let local = yes_false(1 / b)", "let local = no_true(seed, 1 / c)", "let local = no_false(1 / d)", "const sealed: bool = local && true", "return sealed && answer", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-native-truth-traces-source-free-bytes-and-selected-condition-prefix-rhs-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_tree_logical.rs",
        required_patterns: &["native_return_tree_logical_effectful_scalar_selections_build_cache_restore_and_keep_complete_leaf_returns_and_caller_continuation", "native_return_tree_logical_effectful_scalar_selections_trap_at_selected_ancestor_conditions_prefixes_and_leaf_rhs_before_publication", "compile_cache: hit", "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))", "expected[position..]", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-artifact-and-cli-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_tree_logical_tests.rs", "mod return_tree_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_tree_logical.rs", "mod return_tree_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-tree-full-private-prefix-checkpoint-and-next-staged-ancestor-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Nested logical return trees follow-up:", "Previous private-prefix checkpoint:", "Previous private-prefix task checkpoint:", "Previous private-prefix blocker checkpoint:", "Previous private-prefix action checkpoint:", "Previous private-prefix artifact checkpoint:", "Prove private scalar staging before nested logical bool return selections", "native_return_tree_logical_effectful_scalar_selections"],
    },
];
