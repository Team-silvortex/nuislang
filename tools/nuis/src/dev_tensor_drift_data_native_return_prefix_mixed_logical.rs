use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-parent-owner-root-and-prefix-continuation-route",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["Root candidates have no delegation and still require actual write evidence", "!owner_type(staged.local.get(name)?)", "if prefix.len() > 16", "for (index, statement) in prefix.iter().enumerate()", "mixed::classify(then_body, else_body, scope, &staged, &authorized, depth)?", "mixed::continue_flat(", "&body[index + 1..]", "return Some(finish(staged, proof))", "fn finish(", "proof.inputs.difference(&staged.defined)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-private-child-token-and-nested-terminal-context-route",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["effectful_selections_return_mixed_continuations.rs", "pub(super) use continuations::prove as continue_flat", "pub(super) struct Child(String)", "entering.contains_key(&child)", "!staged.defined.contains(&child)", "staged.constants.contains(&child)", "writes.len() == 2 && writes.contains(owner.name()) && writes.contains(&child.0)", "return Some(super::finish(staged, proof))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-flat-only-lexical-tail-original-charge-and-single-ancestor",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuations.rs",
        required_patterns: &["effectful_selections_return_mixed_continuation_tests.rs", "fn flat(", "fn fresh_names(", "fn copy_flat(", "regions::Statement::Selection(_) | regions::Statement::PrivatePair(_)", "suffix.is_empty()", "!flat(suffix, 16)", "tree::shape(yes, scope, &fresh, &mut remaining, depth + 1)?", "tree::shape(no, scope, &fresh, &mut remaining, depth + 1)?", "Prove the shared tail once in its lexical scope", "nested::consume_node(budget)?", "predicates::prove(condition, scope, signatures, budget)?", "let yes = tree::arm(", "let no = tree::arm(", "nested::join(condition, inputs, calls, yes, no, scope)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-six-original-budget-scope-capture-and-expansion-test-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuation_tests.rs",
        required_patterns: &["keep_both_orders_and_private_current_versions", "reject_scope_collisions_hidden_work_and_recursive_copy", "capture_old_reads_not_permission_or_discarded_child", "charge_shared_tail_once_without_new_transport_work", "keep_original_prefix_and_flat_copy_limits", "share_nodes_edges_depth_and_capture_union", "(10, 21, 4, true)", "for count in [15, 16]", "for count in [16, 17]", "for count in [17, 18]", "for count in [10, 11]", "for levels in [5, 6]", "for count in [30, 31]", "assert_eq!(module, before)", "assert_eq!(names, before_names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-six-selected-skipped-prefix-sites-and-post-prefix-child-fixture",
        path: "tools/nuisc/tests/native_application_bridge/return_prefix_mixed_logical_effectful_scalar_selections.ns",
        required_patterns: &["if seed {", "if seed {} else {", "let local = child && parent_work(second, 1 / parent_rhs)", "let child = child || child_work(first, 1 / child_rhs)", "const seal: bool = child || child_tail_work(seed, 1 / child_tail)", "let child = after_child_work(child, 1 / child_after)", "let child = after_child_work(first, 1 / child_after)", "let ignored = child_unused_work(1 / child_unused)", "return local && answer_work(reply, 1 / answer)", "return local || answer_work(seed, 1 / answer)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-artifact-exact-state-guards-transitive-effect-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_prefix_mixed_logical_tests.rs",
        required_patterns: &["native_return_prefix_mixed_logical_effectful_scalar_selections", "child_rhs:i64;parent_rhs:i64;child_after:i64;child_unused:i64", "checked_functions >= 13", "guarded_calls >= 32", "yir_core::EdgeKind::Effect", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-independent-gated-oracle-source-free-state-and-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_prefix_mixed_logical.rs",
        required_patterns: &["Independent imperative replay: each flat continuation reads branch-current versions", "if seed {", "if !seed {", "let mut child = local", "trace.push(95)", "trace.push(96)", "const STAGES: [i32; 13]", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_prefix_mixed_logical_tests.rs", "mod return_prefix_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_prefix_mixed_logical.rs", "mod return_prefix_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-complete-terminal-mixed-history-and-nested-prefix-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Flat-prefix-mixed return bool-update follow-up:", "Previous terminal-mixed-update checkpoint:", "Previous terminal-mixed-update task checkpoint:", "Previous terminal-mixed-update blocker checkpoint:", "Previous terminal-mixed-update action checkpoint:", "Previous terminal-mixed-update validation checkpoint:", "Previous terminal-mixed-update artifact checkpoint:", "nested or sequential mixed prefix selections with observed child continuations", "nuis-native-private-flat-mixed-continuations-v1.md", "native_return_prefix_mixed_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-flat-continuation-contract-limits-and-native-proof",
        path: "docs/reference/nuis-native-private-flat-mixed-continuations-v1.md",
        required_patterns: &["## Branch-Current Continuations", "Only the final ancestor is returned", "## Bounded Copying", "Selection trees and another continuation", "ten nodes / 21 expressions / four logical edges", "64 nodes, 256 expressions, 32 logical edges", "31 captures plus a gate", "Thirteen selected", "Nested or sequential mixed prefix selections", "formal memory safety", "strict Clippy"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-prefix-mixed-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_prefix_mixed_logical.rs", "mod return_prefix_mixed_logical;", "return_prefix_mixed_logical::CHECKS"],
    },
];
