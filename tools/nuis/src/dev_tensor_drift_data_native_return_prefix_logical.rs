use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-original-owned-bool-authority-and-shared-return-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_returns.rs",
        required_patterns: &["effectful_selections_return_prefixes.rs", "return_type != Some(&scalar_type(\"bool\"))",
            "prefixes::prepare(condition, yes, no, scope, signatures, pure)", "struct ReturnedValue", "fn finish(",
            "if captures.len() > 31", "destination: None"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-shared-guarded-stage-and-predicate-installer",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["StagedPredicate(Vec<regions::Statement>, predicates::Predicate)",
            "SelectedValue::StagedPredicate(statements, predicate)", "body.extend(regions::install(statements, names, bindings, helpers))",
            "predicates::install(predicate, names, bindings, helpers)", "let returns = plan.destination.is_none()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-independent-fresh-scopes-current-captures-and-shared-original-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["prefix.len() <= 16", "statements.split_last()", "if scope.contains_key(name)",
            "std::slice::from_ref(statement)", "&staged.constants", "predicates::prove(value, &local, signatures, budget)",
            "reads.difference(&defined)", "calls.extend(effects)", "finish(condition, calls, yes, no, scope, pure)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-structural-placement-seals-original-work-limits-and-atomic-late-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefix_tests.rs",
        required_patterns: &["return_prefix_logical_effectful_scalar_selections_keep_selected_private_versions_seals_unused_calls_and_original_exits",
            "return_prefix_logical_effectful_scalar_selections_reject_outer_writes_mixed_exits_hidden_roots_cross_arm_leaks_and_late_errors_atomically",
            "return_prefix_logical_effectful_scalar_selections_charge_original_prefixes_returns_and_condition_once",
            "return_prefix_logical_effectful_scalar_selections_bound_prefix_lengths_all_original_captures_and_shared_expressions_edges_depth",
            "assert_eq!(names, before_names)", "for count in [16, 17]", "for count in [31, 32]", "for width in [63, 64]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-transitive-effect-order-source-free-policy-and-polarity-evidence",
        path: "tools/nuisc/src/aot_application_effect_call_return_prefix_logical_tests.rs",
        required_patterns: &["native_return_prefix_logical_effectful_scalar_selections_keep_transitive_guard_check_call_order_and_return_polarities",
            "native_return_prefix_logical_effectful_scalar_selections_preserve_policy_checkpoint_and_removed_order_vetoes",
            "assert_eq!(predicates, 9)", "assert_eq!(guarded_checks, 6)", "yir_core::EdgeKind::Effect",
            "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-source-private-current-versions-constant-seals-unused-effects-and-caller-exit",
        path: "tools/nuisc/tests/native_application_bridge/return_prefix_logical_effectful_scalar_selections.ns",
        required_patterns: &["let local = prefix(saved, wanted, 1 / ya)", "let local = prefix(saved, wanted, 1 / na)",
            "let local = local && saved", "const sealed: bool = local || false", "let unused = suffix(1 / yz)",
            "let unused = suffix(1 / nz)", "return sealed && check2", "return check3(local, wanted, 1 / dc)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-native-state-traces-source-free-byte-identity-and-required-unused-check-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_prefix_logical.rs",
        required_patterns: &["native_return_prefix_logical_effectful_scalar_selections_build_cache_restore_and_keep_selected_returns_and_caller_continuation",
            "native_return_prefix_logical_effectful_scalar_selections_trap_before_later_return_effects_or_publication",
            "trace.extend([96, 97])", "compile_cache: hit", "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "(5, 12, false, true, false, false)", "(5, 13, true, false, false, true)", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_prefix_logical_tests.rs", "mod return_prefix_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_prefix_logical.rs", "mod return_prefix_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-complete-paired-return-checkpoint-and-next-nested-return-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Private logical return prefixes follow-up:", "Previous paired-return checkpoint:",
            "Previous paired-return task checkpoint:", "Previous paired-return blocker checkpoint:", "Previous paired-return action checkpoint:",
            "Previous paired-return artifact checkpoint:", "Prove nested paired logical bool return selections", "native_return_prefix_logical_effectful_scalar_selections"],
    },
];
