use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-child-logical-original-leaf-stage-plan-before-clone-and-single-node-charge",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_nested.rs",
        required_patterns: &["statement: &NirStmt", "std::slice::from_ref(statement)", "regions::stage(",
            "SelectedValue::Region(staged.statements, NirExpr::Var(name.clone()))", "inputs: staged.inputs", "calls: staged.calls",
            "leaf(statement, scope, signatures, budget)", "declared.as_ref().is_some_and(|ty| ty != old_ty)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-original-single-target-constant-seals-and-ordered-current-versions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_ordered.rs",
        required_patterns: &["staging.constants.contains(&proof.name)", "scope.get(&proof.name)?", "name != &proof.name",
            "staging.local.insert(proof.name.clone(), proof.ty)", "proof.inputs.difference(&staging.defined)", "children.push((stage, child))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-bool-polarities-preceding-captures-atomic-vetoes-and-exact-shared-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_child_logical_tests.rs",
        required_patterns: &["child_logical_effectful_scalar_selections_keep_original_declarations_polarities_and_preceding_captures",
            "child_logical_effectful_scalar_selections_reject_constants_hidden_leaves_exports_other_targets_and_late_errors_atomically",
            "child_logical_effectful_scalar_selections_charge_original_leaf_once_with_shared_condition_work",
            "child_logical_effectful_scalar_selections_keep_capture_union_selection_depth_expression_depth_and_sibling_bounds",
            "assert_eq!(names, before_names)", "for count in [30, 31, 32]", "for depth in [31, 32]", "for depth in [8, 9]", "for count in [20, 21]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-transitive-yir-guard-check-call-order-and-policy-checkpoint-vetoes",
        path: "tools/nuisc/src/aot_application_effect_call_child_logical_tests.rs",
        required_patterns: &["native_child_logical_effectful_scalar_selections_keep_transitive_guard_check_call_order_and_bool_leaf_polarities",
            "native_child_logical_effectful_scalar_selections_preserve_policy_checkpoint_and_removed_order_vetoes",
            "assert_eq!(predicates, 4)", "yir_core::EdgeKind::Effect", "lacks dependency order", "verify_nuis_compiled_artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-real-two-sided-leaves-inverted-retention-and-preceding-merge-source",
        path: "tools/nuisc/tests/native_application_bridge/child_logical_effectful_scalar_selections.ns",
        required_patterns: &["if choose_first(first, 1 / first_gate)", "let saved = saved && check1", "let saved = saved || check2",
            "if choose_second(second, 1 / second_gate) {} else", "let wanted = expected_first(seed, first, reply)",
            "let saved = check3(saved, wanted, 1 / dc) && (saved || check4", "result: bool"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-native-state-stdout-source-free-policy-byte-identity-and-selected-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_child_logical.rs",
        required_patterns: &["native_child_logical_effectful_scalar_selections_build_cache_restore_and_keep_previous_child_merge",
            "native_child_logical_effectful_scalar_selections_trap_before_later_child_effects_or_publication",
            "compile_cache: hit", "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))", "SIGILL", "SIGTRAP",
            "rejected_before_open(run)", "(4, 6, true, false, false, false)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-leaf-test-registration-and-shared-entry-frontier",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_nested.rs",
        required_patterns: &["effectful_selections_child_logical_tests.rs", "mod logical_tests;", "body.len() > 1", "matches!(*body, [NirStmt::If { .. }])",
            "if !proof.calls.iter().any(|name| !pure.contains(name))", "if captures.len() > 31", "if depth >= 8"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-scoped-contract-and-selected-acceptance",
        path: "docs/reference/nuis-native-effectful-scalar-selection-v1.md",
        required_patterns: &["## Guarded Logical Child Leaves", "original leaf once", "read-before-write", "480 successful", "7 selected",
            "Child logical leaf acceptance:"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-logical-six-complete-bool-update-history-fields-and-next-top-level-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Guarded logical child leaves follow-up:", "Previous bool-update checkpoint:", "Previous bool-update task checkpoint:",
            "Previous bool-update blocker checkpoint:", "Previous bool-update action checkpoint:", "Previous bool-update artifact checkpoint:",
            "Prove guarded logical bool updates in top-level single-statement selection arms", "native_child_logical_effectful_scalar_selections"],
    },
];
