use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-classifier-bounded-private-child-actual-two-name-evidence",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["pub(super) struct Child(String)", "if writes.len() < 2", "!writes.remove(owner.name()) || writes.len() != 1", "entering.contains_key(&child)", "!staged.defined.contains(&child)", "staged.constants.contains(&child)", "staged.local.get(&child)? == &scalar_type(\"bool\")", "transport::numeric(staged.local.get(&child)?)", "let mut remaining = 64", "if writes.len() > 2", "if depth < 8"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-original-single-target-staging-terminal-only-export-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["if writes.len() > 1", "silently discard the child", "Some(&Target(name))", "super::super::append_authorized(", "nested::consume_node(budget)?", "predicates::prove(condition, scope, signatures, budget)?", "nested::join(condition, inputs, calls, yes, no, scope)", "if prefix.len() > 16", "if writes.contains(owner.name())", "writes.iter().any(|n| n != owner.name() && n != &child.0)", "reads.difference(&staged.defined)", "name: owner.name().into()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-six-nir-source-version-atomic-and-original-budget-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_update_tests.rs",
        required_patterns: &["keep_one_ancestor_export_both_orders_polarities_and_terminal_nesting", "capture_source_versions_not_permission_or_private_exports", "reject_extra_outward_targets_seals_hidden_work_prefix_continuations_and_late_leaks", "charge_original_writes_once_without_retention_transport_work", "keep_shared_prefix_nodes_edges_depth_and_discovery_limits", "share_complete_external_capture_union_and_atomic_overflow", "(9, 18, 3, true)", "for writes in [15, 16]", "for writes in [11, 12]", "for levels in [5, 6]", "for count in [30, 31]", "vec![write(\"local\"); 64]", "assert_eq!(module, before)", "assert_eq!(names, before_names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-original-mixed-orders-private-child-and-post-update-work-fixture",
        path: "tools/nuisc/tests/native_application_bridge/return_mixed_update_logical_effectful_scalar_selections.ns",
        required_patterns: &["parent_rhs: i64, child_after: i64", "fn parent_work", "print(95)", "fn after_child_work", "print(96)", "let local = child && parent_work(second, 1 / parent_rhs)", "let child = local && child_work(seed, 1 / child_rhs)", "let local = child || parent_work(seed, 1 / parent_rhs)", "let child = after_child_work(first, 1 / child_after)", "let child = after_child_work(local, 1 / child_after)", "const seal: bool = child || child_tail_work(seed, 1 / child_tail)", "let ignored = child_unused_work(1 / child_unused)", "return local && answer_work(reply, 1 / answer)", "return local || answer_work(seed, 1 / answer)", "parent_rhs: parent_rhs, child_after: child_after"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-artifact-exact-state-checked-guard-effect-path-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_mixed_update_logical_tests.rs",
        required_patterns: &["keep_guarded_transitive_update_stage_return_order_and_exact_state", "preserve_source_free_policy_and_removed_update_order_vetoes", "child_rhs:i64;parent_rhs:i64;child_after:i64;child_unused:i64", "assert_eq!(node(sensitive[0]).op.instruction, \"guard_return\")", "checked_functions >= 13", "guarded_calls >= 32", "yir_core::EdgeKind::Effect", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-independent-native-oracle-state-cache-source-free-and-thirteen-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_mixed_update_logical.rs",
        required_patterns: &["Independent imperative replay of source versions; only the ancestor is public", "let mut child = local", "result = if local", "trace.push(95)", "trace.push(96)", "const STAGES: [i32; 13]", "native_return_mixed_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation", "native_return_mixed_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "fs::read(restored.join(&report.artifact_binary_name)).unwrap()", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-artifact-test-and-parent-proof-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_mixed_update_logical_tests.rs", "mod return_mixed_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_mixed_update_logical.rs", "mod return_mixed_update_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-complete-terminal-child-history-and-prefix-continuation-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Terminal-mixed-private-update return bool-update follow-up:", "Previous terminal-private-child-update checkpoint:", "Previous terminal-private-child-update task checkpoint:", "Previous terminal-private-child-update blocker checkpoint:", "Previous terminal-private-child-update action checkpoint:", "Previous terminal-private-child-update validation checkpoint:", "Previous terminal-private-child-update artifact checkpoint:", "mixed ancestor/private-child prefix selections with observed child continuations", "nuis-native-private-mixed-return-updates-v1.md", "native_return_mixed_update_logical_effectful_scalar_selections"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-bounded-contract-original-work-real-execution-and-limits",
        path: "docs/reference/nuis-native-private-mixed-return-updates-v1.md",
        required_patterns: &["## One Outward Ancestor", "exactly those two eligible actual source", "original execution budget", "only the actual current ancestor is exported", "nine nodes / 18 expressions / three logical edges", "64 nodes, 256 expressions, 32 logical edges", "31 captures plus a gate", "Thirteen independently", "Mixed ancestor/private-child prefix selections with observed child continuations", "formal memory safety", "strict Clippy"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-mixed-update-drift-and-proof-module-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_mixed_update_logical.rs", "mod return_mixed_update_logical;", "return_mixed_update_logical::CHECKS"],
    },
];
