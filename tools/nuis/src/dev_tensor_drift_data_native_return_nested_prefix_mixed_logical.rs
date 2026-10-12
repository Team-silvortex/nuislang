use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-original-shape-budget-depth-and-lexical-isolation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuation_tree.rs",
        required_patterns: &["remaining.checked_sub(body.len())?", "body.len() > 17", "!fresh_tail.is_disjoint(&fresh_names(body, tail_scope))", "body.split_last()", "depth >= 8 || !flat(prefix, 16)", "shape(then_body, tail_scope, fresh_tail, remaining, depth + 1)?", "shape(else_body, tail_scope, fresh_tail, remaining, depth + 1)", "flat(body, 17).then_some(())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-current-condition-original-children-leaf-only-copy",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuation_tree.rs",
        required_patterns: &["effectful_selections_return_mixed_continuation_tree_tests.rs", "nested::consume_node(budget)?", "predicates::prove(condition, &staged.local, signatures, budget)?", "Each original child is proved once", "let yes = arm(", "let no = arm(", "super::super::super::finish(staged, proof)", "reads.difference(&staged.defined)", "staged.statements.extend(copy_flat(&tail.statements)?)", "NirExpr::Var(owner.name().into())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-shared-tail-no-selection-cloning",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuations.rs",
        required_patterns: &["effectful_selections_return_mixed_continuation_tree.rs", "regions::Statement::Selection(_) | regions::Statement::PrivatePair(_)", "!flat(suffix, 16)", "let mut remaining = 64", "Prove the shared tail once in its lexical scope", "tree::shape(yes", "tree::shape(no", "tree::arm("],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-six-budget-capture-scope-and-expansion-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuation_tree_tests.rs",
        required_patterns: &["keep_current_conditions_both_orders_and_retention", "reject_nested_scope_leaks_seals_extra_writes_and_sequences", "charge_original_tree_and_shared_tail_once", "keep_exact_nodes_expressions_edges_depth_and_prefix_limits", "capture_old_reads_and_bound_union_without_permission_versions", "copy_flat_recipes_only_at_original_leaves", "(8, 16, 3, true)", "for count in [15, 16]", "for count in [7, 8]", "for count in [16, 17]", "for count in [30, 31]", "assert_eq!(counts(&p.value), (17, 17 * 16))", "assert_eq!(module, before)", "assert_eq!(names, before_names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-native-current-child-gates-common-suffix",
        path: "tools/nuisc/tests/native_application_bridge/return_nested_prefix_mixed_logical_effectful_scalar_selections.ns",
        required_patterns: &["if condition(reply, 1 / cond)", "let child = child && child_work(reply, 1 / child_rhs)", "if child {", "if seed {} else {", "if child {} else {", "let local = child && parent_work(second, 1 / parent_rhs)", "const seal: bool = child || after_work(local, 1 / after)", "let local = local && seal", "let ignored = unused_work(1 / unused)", "return local && answer_work(reply, 1 / answer)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-artifact-state-guards-transitive-effect-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_nested_prefix_mixed_logical_tests.rs",
        required_patterns: &["native_return_nested_prefix_mixed_logical_effectful_scalar_selections", "child_rhs:i64;parent_rhs:i64;after:i64;unused:i64", "checked_functions >= 9", "guarded_calls >= 16", "yir_core::EdgeKind::Effect", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-independent-oracle-restoration-and-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_nested_prefix_mixed_logical.rs",
        required_patterns: &["Independent imperative replay: nested gates read preceding source writes", "let mut child = local", "child = child && reply", "if !seed", "trace.push(95)", "trace.push(96)", "const STAGES: [i32; 9]", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-artifact-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_nested_prefix_mixed_logical_tests.rs", "mod return_nested_prefix_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_nested_prefix_mixed_logical.rs", "mod return_nested_prefix_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-complete-flat-history-and-sequential-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Nested-prefix-mixed return bool-update follow-up:", "Previous flat-prefix-mixed checkpoint:", "Previous flat-prefix-mixed task checkpoint:", "Previous flat-prefix-mixed blocker checkpoint:", "Previous flat-prefix-mixed action checkpoint:", "Previous flat-prefix-mixed validation checkpoint:", "Previous flat-prefix-mixed artifact checkpoint:", "Prove sequential mixed prefix selections with observed child continuations", "nuis-native-private-nested-mixed-continuations-v1.md", "native_return_nested_prefix_mixed_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-bounded-contract",
        path: "docs/reference/nuis-native-private-nested-mixed-continuations-v1.md",
        required_patterns: &["## Original Branch Tree", "Only the final ancestor is returned", "## Expansion Bound", "64 nodes, 256 expressions, 32 logical edges", "31 captures plus a gate", "eight nodes / 16 expressions / three logical edges", "17 leaves and 272 flat recipes", "Nine selected", "Sequential mixed prefix selections", "formal memory safety", "strict Clippy"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-nested-prefix-mixed-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_nested_prefix_mixed_logical.rs", "mod return_nested_prefix_mixed_logical;", "return_nested_prefix_mixed_logical::CHECKS"],
    },
];
