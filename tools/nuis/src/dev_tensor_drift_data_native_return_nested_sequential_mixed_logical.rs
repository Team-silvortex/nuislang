use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-recursive-original-body-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["effectful_selections_return_mixed_sequence_tree.rs", "tree::preflight(yes, no, suffix, depth)?", "tree::isolated(body, scope, future)", "body(yes, scope, owner, child, signatures, budget, depth + 1)?", "body(no, scope, owner, child, signatures, budget, depth + 1)?", "append_body(", "&statements[index + 1..]", "staged.constants.contains(owner.name())", "staged.constants.contains(&child.0)", "No arm receives a copy of the remaining sequence", "transport::prepare"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-whole-forest-preflight-and-lexical-isolation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequence_tree.rs",
        required_patterns: &["pub(super) fn continued", "found |= index + 1 < body.len()", "remaining.checked_sub(body.len())?", "body.len() > 16", "if depth < 8", "let mut remaining = 63", "shape(yes, &mut remaining, depth + 1)?", "shape(no, &mut remaining, depth + 1)?", "shape(suffix, &mut remaining, depth)", "fresh_names(body, scope).is_disjoint(future)", "pending.extend([then_body.as_slice(), else_body.as_slice()])"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-six-current-budget-veto-and-linear-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequence_tree_tests.rs",
        required_patterns: &["keep_branch_current_versions_both_orders_polarities_and_retention", "reject_lexical_leaks_collisions_seals_extra_writes_and_exits_atomically", "charge_each_original_node_expression_and_edge_once", "preflight_complete_forest_and_keep_depth_body_and_node_limits", "bound_nested_capture_union_and_keep_retained_old_versions", "install_two_helpers_per_original_selection_without_tail_copying", "(8, 8, true)", "for count in [64, 65]", "for count in [31, 32]", "copy_flat"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-routing-preserves-terminal-tree-route",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuations.rs",
        required_patterns: &["sequences::continued(yes, no)?", "return sequences::prove(", "tree::arm(", "regions::Statement::PrivatePair(_)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-private-transport-unchanged-boundary",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["NirExpr::CastBoolToI64", "NirExpr::Int(2)", "captures.len() > 31", "plan.destination = Some((carrier.clone(), false))", "NirBinaryOp::Xor", "NirBinaryOp::Ge", "no generated division or trapping work"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-native-current-and-sealed-gates",
        path: "tools/nuisc/tests/native_application_bridge/return_nested_sequential_mixed_logical_effectful_scalar_selections.ns",
        required_patterns: &["let current = child", "if inner_condition(current, 1 / inner_cond)", "const local_seal: bool = local || false", "if inner_condition(local_seal, 1 / inner_cond)", "let child = child && inner_child_work(local, 1 / inner_child)", "let local = local && inner_parent_work(child, 1 / inner_parent)", "if local { let child = child || second; }", "let ignored = unused_work(1 / unused)", "print(79)", "print(80)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-artifact-guards-state-and-effect-veto",
        path: "tools/nuisc/src/aot_application_effect_call_return_nested_sequential_mixed_logical_tests.rs",
        required_patterns: &["native_return_nested_sequential_mixed_logical_effectful_scalar_selections", "inner_parent:i64;result:bool", "checked_functions >= 12", "guarded_calls >= 24", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-cli-independent-replay-cache-restoration-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_nested_sequential_mixed_logical.rs",
        required_patterns: &["return_bool_pair_oracle.rs", "use pair_oracle::{oracle, STAGES}", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-artifact-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_nested_sequential_mixed_logical_tests.rs", "mod return_nested_sequential_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_nested_sequential_mixed_logical.rs", "mod return_nested_sequential_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-full-history-and-next-typed-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Nested-sequential-mixed return bool-update follow-up:", "Previous sequential-mixed checkpoint:", "Previous sequential-mixed task checkpoint:", "Previous sequential-mixed blocker checkpoint:", "Previous sequential-mixed action checkpoint:", "Previous sequential-mixed validation checkpoint:", "Previous sequential-mixed artifact checkpoint:", "Prove typed mixed private continuations beyond bool pairs", "nuis-native-private-nested-sequential-mixed-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-sequential-mixed-contract-drift-registration",
        path: "docs/reference/nuis-native-private-nested-sequential-mixed-continuations-v1.md",
        required_patterns: &["## Original Forest", "63 plus the first selection", "2 * S guarded branch helpers", "eight nodes / eight expressions / zero logical edges", "twelve selected", "formal memory safety", "strict Clippy", "dev_tensor_drift_data_native_return_nested_sequential_mixed_logical.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-bool-pair-shared-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_bool_pair_oracle.rs",
        required_patterns: &["const STAGES: [i32; 12]", "Independent imperative replay: nested branch-local sequences observe", "let current = child", "let local_seal = local || false"],
    },
];
