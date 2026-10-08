use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-update-logical-owned-bool-preceding-version-stage-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs",
        required_patterns: &["constant && local.contains_key(name)", "local.get(name).is_some_and(|ty| ty != &scalar_type(\"bool\"))",
            "predicates::prove(value, &local, signatures, budget)?", "inputs.extend(reads.difference(&defined).cloned())",
            "local.insert(name.clone(), inferred)", "constants.contains(name)", "Statement::Logical(statement.clone(), predicate)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-bounded-known-constant-write-preflight",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs",
        required_patterns: &["pub(super) fn writes_logical_constants", "if constants.is_empty()", "yes.len() + no.len() > 64",
            "while let Some(statement) = pending.pop()", "visited + pending.len() + then_body.len() + else_body.len() > 64"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-parent-and-installed-constant-seals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["let mut constants = BTreeSet::new()", "regions::writes_logical_constants(then_body, else_body, &constants)",
            "constants.insert(name.clone())", ".is_some_and(|(_, constant)| *constant)", "scope.insert(name, ty)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-five-types-version-captures-atomic-vetoes-and-shared-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_update_logical_tests.rs",
        required_patterns: &["update_logical_effectful_scalar_selections_keep_private_prefix_middle_suffix_types_and_hygiene",
            "update_logical_effectful_scalar_selections_capture_preceding_bool_versions_before_publication",
            "update_logical_effectful_scalar_selections_reject_constants_hidden_roots_other_targets_and_late_errors_atomically",
            "update_logical_effectful_scalar_selections_keep_exact_owned_types_and_shared_bounded_proofs", "assert_eq!(names, before_names)",
            "budget.logical_edges", "writes_logical_constants"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-transitive-yir-order-five-types-policy-and-checkpoint-vetoes",
        path: "tools/nuisc/src/aot_application_effect_call_update_logical_tests.rs",
        required_patterns: &["native_update_logical_effectful_scalar_selections_keep_transitive_guard_check_call_order_and_exact_types",
            "native_update_logical_effectful_scalar_selections_preserve_policy_checkpoint_and_removed_order_vetoes",
            "assert_eq!(predicates, 4)", "yir_core::EdgeKind::Effect", "lacks dependency order", "verify_nuis_compiled_artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-real-bool-prefix-middle-suffix-original-version-checks",
        path: "tools/nuisc/tests/native_application_bridge/update_logical_effectful_scalar_selections.ns",
        required_patterns: &["let before = saved", "let saved = saved && check1", "let saved = saved || check2",
            "let saved = check3(saved, wanted, 1 / dc) && (saved || check4", "expected_second(seed, first, second, reply)",
            "seed: bool", "result: bool", "(saved == wanted) == (proof == 1)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-source-free-byte-identity-poisoned-skips-and-original-stage-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_update_logical.rs",
        required_patterns: &["native_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_preceding_bool_versions",
            "native_update_logical_effectful_scalar_selections_trap_before_later_effects_or_bool_publication",
            "compile_cache: hit", "for _ in 0..3", "fs::remove_dir_all(project.0.join(\".nuis\"))", "SIGILL", "SIGTRAP",
            "rejected_before_open(run)", "(6, 6, false, true, true, false)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-scoped-contract-and-selected-acceptance",
        path: "docs/reference/nuis-native-effectful-scalar-selection-v1.md",
        required_patterns: &["## Guarded Logical Bool Updates", "read-before-write", "known outer constants", "480 successful", "7 selected",
            "Bool update acceptance:"],
    },
    DevTensorDriftCheckSpec {
        id: "native-update-logical-six-complete-staging-history-fields-and-next-child-leaf-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Guarded logical bool updates follow-up:", "Previous staging-logical checkpoint:", "Previous staging-logical task checkpoint:",
            "Previous staging-logical blocker checkpoint:", "Previous staging-logical action checkpoint:", "Previous staging-logical artifact checkpoint:",
            "Prove guarded logical bool updates in single-statement selected child leaves", "native_update_logical_effectful_scalar_selections"],
    },
];
