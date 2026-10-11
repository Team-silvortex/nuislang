use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-top-logical-explicit-let-root-entry-and-shared-leaf-stage",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_nested.rs",
        required_patterns: &["[NirStmt::Let {", "op: NirBinaryOp::And | NirBinaryOp::Or", "std::slice::from_ref(statement)",
            "regions::stage(", "scope.get(name)?", "if !proof.calls.iter().any(|name| !pure.contains(name))", "if captures.len() > 31"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-atomic-declarations-versions-constant-seals-and-shared-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_top_logical_tests.rs",
        required_patterns: &["top_logical_effectful_scalar_selections_keep_declarations_polarities_retention_and_condition_only_effects",
            "top_logical_effectful_scalar_selections_reject_constants_exports_hidden_roots_other_targets_and_late_errors_atomically",
            "top_logical_effectful_scalar_selections_capture_current_merges_and_keep_previously_published_constants_sealed",
            "top_logical_effectful_scalar_selections_share_edge_expression_and_complete_arm_capture_limits",
            "assert_eq!(names, before_names)", "for edges in [32, 33]", "for captures in [31, 32]", "for expressions in [255, 256]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-yir-transitive-effect-paths-and-policy-order-vetoes",
        path: "tools/nuisc/src/aot_application_effect_call_top_logical_tests.rs",
        required_patterns: &["native_top_logical_effectful_scalar_selections_keep_transitive_guard_check_call_order_and_top_arm_polarities",
            "native_top_logical_effectful_scalar_selections_preserve_policy_checkpoint_and_removed_order_vetoes",
            "yir_core::EdgeKind::Effect", "lacks dependency order", "verify_nuis_compiled_artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-real-three-independent-selections-and-condition-only-effects",
        path: "tools/nuisc/tests/native_application_bridge/top_logical_effectful_scalar_selections.ns",
        required_patterns: &["if active(enabled)", "let saved = saved && check1", "let wanted = expected_first(seed, enabled, reply)",
            "let saved = saved || check2", "let saved = check3(saved, wanted, 1 / dc) && (saved || check4",
            "if choose_second(second, 1 / second_gate) {} else", "let saved = saved && reply"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-native-matrix-source-free-byte-identity-and-selected-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_top_logical.rs",
        required_patterns: &["native_top_logical_effectful_scalar_selections_build_cache_restore_and_keep_previous_top_merge",
            "native_top_logical_effectful_scalar_selections_trap_before_later_top_effects_or_publication", "compile_cache: hit",
            "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))", "rejected_before_open(run)", "(4, 6, true, false, true, true)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-outliner-test-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_nested.rs",
        required_patterns: &["effectful_selections_top_logical_tests.rs", "mod top_logical_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_top_logical_tests.rs", "mod top_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_top_logical.rs", "mod top_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-top-logical-scoped-contract-complete-predecessor-and-next-return-root-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Top-level logical arms follow-up:", "Previous child-leaf checkpoint:", "Previous child-leaf task checkpoint:",
            "Previous child-leaf blocker checkpoint:", "Previous child-leaf action checkpoint:", "Previous child-leaf artifact checkpoint:",
            "Prove guarded logical bool return roots in top-level two-sided scalar selections", "native_top_logical_effectful_scalar_selections"],
    },
];
