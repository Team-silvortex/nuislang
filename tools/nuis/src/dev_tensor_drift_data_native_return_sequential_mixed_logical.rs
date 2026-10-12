use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-closed-carrier-existing-scalar-no-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["pub(super) struct Selection", "NirExpr::CastBoolToI64", "NirBinaryOp::Mul", "NirExpr::Int(2)", "captures.len() > 31", "__nuis_effect_call_pair", "plan.destination = Some((carrier.clone(), false))", "NirBinaryOp::Xor", "equal(1)", "equal(3)", "NirBinaryOp::Ge", "no generated division or trapping work"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-flat-scope-retention-budget-linear-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["tree::isolated(body, scope, future)", "tree::preflight(yes, no, suffix, depth)?", "depth >= 8", "staged.constants.contains(owner.name())", "nested::consume_node(budget)?", "predicates::prove(condition, scope, signatures, budget)?", "transport::prepare", "Statement::PrivatePair", "suffix.len() > 16", "No arm receives a copy of the remaining sequence", "NirExpr::Var(owner.name().into())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-six-closed-budgets-captures-and-linear-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequence_tests.rs",
        required_patterns: &["keep_both_current_versions_orders_and_retention", "reject_scope_seals_extra_writes_nested_sequences_and_hidden_work_atomically", "charge_original_source_once_and_keep_exact_adjacent_budgets", "transport_all_four_pairs_with_fresh_names_and_no_traps", "bound_retained_capture_union_without_permission_versions", "install_linear_plans_without_remaining_tree_copies", "(7, 7, true)", "for count in [64, 65]", "for count in [256, 257]", "for count in [32, 33]", "for count in [31, 32]", "for count in [1, 8, 16]", "assert!(copy_flat(&statements).is_none())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-no-copy-route-and-private-install",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuations.rs",
        required_patterns: &["effectful_selections_return_mixed_sequences.rs", "return sequences::prove(", "regions::Statement::PrivatePair(_)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-nir-private-carrier-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs",
        required_patterns: &["PrivatePair(Box<transport::Selection>)", "Statement::PrivatePair(plan)", "transport::install(*plan, names, bindings, helpers, structs)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-native-current-pairs-source-exits-and-unused-work",
        path: "tools/nuisc/tests/native_application_bridge/return_sequential_mixed_logical_effectful_scalar_selections.ns",
        required_patterns: &["if condition(reply, 1 / cond)", "let local = child && parent_work(seed, 1 / parent_rhs)", "if child {", "let local = local && after_work(child, 1 / after)", "let child = local && seed", "if local { let child = child || second; }", "const seal: bool = child || false", "let ignored = unused_work(1 / unused)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-artifact-state-guards-local-effects-and-vetoes",
        path: "tools/nuisc/src/aot_application_effect_call_return_sequential_mixed_logical_tests.rs",
        required_patterns: &["native_return_sequential_mixed_logical_effectful_scalar_selections", "checked_functions >= 9", "guarded_calls >= 16", "yir_core::EdgeKind::Effect", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-independent-oracle-poison-cache-source-free-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_sequential_mixed_logical.rs",
        required_patterns: &["Independent imperative replay: each sequential gate reads both current", "let mut child = local", "child = local && seed", "const STAGES: [i32; 9]", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-artifact-unit-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_sequential_mixed_logical_tests.rs", "mod return_sequential_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-cli-unit-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_sequential_mixed_logical.rs", "mod return_sequential_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-full-predecessor-fields-and-next-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Sequential-mixed return bool-update follow-up:", "Previous nested-prefix-mixed checkpoint:", "Previous nested-prefix-mixed task checkpoint:", "Previous nested-prefix-mixed blocker checkpoint:", "Previous nested-prefix-mixed action checkpoint:", "Previous nested-prefix-mixed validation checkpoint:", "Previous nested-prefix-mixed artifact checkpoint:", "Prove nested mixed branches with branch-local sequential continuations", "nuis-native-private-sequential-mixed-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-sequential-mixed-contract-and-drift-registration",
        path: "docs/reference/nuis-native-private-sequential-mixed-continuations-v1.md",
        required_patterns: &["## Private Transport", "0..=3", "No selection or continuation tree is cloned", "64 nodes, 256 expressions, 32 logical edges", "31 captures plus a gate", "seven nodes / seven expressions / zero logical edges", "Nine selected", "formal memory safety", "strict Clippy", "dev_tensor_drift_data_native_return_sequential_mixed_logical.rs"],
    },
];
