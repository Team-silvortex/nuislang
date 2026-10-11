use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-logical-original-paired-owned-bool-authority-and-shared-predicate-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_returns.rs",
        required_patterns: &["return_type != Some(&scalar_type(\"bool\"))", "[NirStmt::Return(Some(yes))]", "[NirStmt::Return(Some(no))]",
            "for _ in 0..3", "predicates::prove(condition, scope, signatures, budget)", "predicates::prove(yes, scope, signatures, budget)",
            "predicates::prove(no, scope, signatures, budget)", "if captures.len() > 31", "destination: None"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-shared-installer-retains-source-return-and-guarded-value-plans",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["effectful_selections_returns.rs", "returns::prepare(", "function.return_type.as_ref()",
            "Predicate(predicates::Predicate)", "SelectedValue::Predicate(predicate)", "predicates::install(predicate, names, bindings, helpers)",
            "let returns = plan.destination.is_none()", "output.push(NirStmt::Return(Some(NirExpr::Var(destination.clone()))))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-atomic-shapes-parent-tail-and-original-shared-limits",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_logical_tests.rs",
        required_patterns: &["return_logical_effectful_scalar_selections_keep_original_returns_polarities_conditions_and_parent_effects",
            "return_logical_effectful_scalar_selections_reject_mixed_single_nested_hidden_pure_and_invalid_types_atomically",
            "return_logical_effectful_scalar_selections_charge_original_returns_once_with_shared_condition_edges",
            "return_logical_effectful_scalar_selections_share_complete_arm_capture_expression_edge_and_depth_bounds",
            "assert_eq!(names, before_names)", "for edges in [32, 33]", "for depth in [31, 32]", "for width in [63, 64]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-transitive-yir-effect-order-policy-checkpoint-and-both-return-polarities",
        path: "tools/nuisc/src/aot_application_effect_call_return_logical_tests.rs",
        required_patterns: &["native_return_logical_effectful_scalar_selections_keep_transitive_guard_check_call_order_and_return_polarities",
            "native_return_logical_effectful_scalar_selections_preserve_policy_checkpoint_and_removed_order_vetoes",
            "assert_eq!(predicates, 5)", "assert_eq!(guarded_checks, 5)", "yir_core::EdgeKind::Effect", "lacks dependency order", "verify_nuis_compiled_artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-real-return-site-suffix-veto-and-caller-continuation-source",
        path: "tools/nuisc/tests/native_application_bridge/return_logical_effectful_scalar_selections.ns",
        required_patterns: &["if choose_first(first, 1 / first_gate) || choose_second(second, 1 / second_gate)",
            "return saved && check2", "return check3(saved, wanted, 1 / dc) && (saved || check4", "print(80)", "print(79)",
            "let result = observe", "result: result"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-native-state-output-source-free-byte-identity-and-selected-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_logical.rs",
        required_patterns: &["native_return_logical_effectful_scalar_selections_build_cache_restore_and_keep_selected_returns_and_caller_continuation",
            "native_return_logical_effectful_scalar_selections_trap_before_later_return_effects_or_publication",
            "compile_cache: hit", "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))", "rejected_before_open(run)", "(5, 6, true, false, false, true)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_logical_tests.rs", "mod return_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_logical.rs", "mod return_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-complete-top-level-let-history-and-next-private-prefix-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Paired logical bool returns follow-up:", "Previous top-level checkpoint:", "Previous top-level task checkpoint:",
            "Previous top-level blocker checkpoint:", "Previous top-level action checkpoint:", "Previous top-level artifact checkpoint:",
            "Prove guarded logical bool return prefixes in top-level two-sided scalar selections", "native_return_logical_effectful_scalar_selections"],
    },
];
