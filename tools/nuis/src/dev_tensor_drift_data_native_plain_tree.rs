use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-existing-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_trees.rs",
        required_patterns: &["effectful_selections_return_plain_tree_tests.rs", "tree.calls.iter().any(|name| !pure.contains(name))", "Option<ReturnedValue>", "inputs.extend(child.inputs.difference(&staged.defined).cloned())", "prefixes::body(statements, scope, signatures, budget, depth)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-bounds-and-atomic-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_plain_tree_tests.rs",
        required_patterns: &["return_plain_tree_calls_literals_names_and_effect_only_ancestors_use_one_proof", "return_plain_tree_partial_cross_scope_write_seal_type_and_late_vetoes_are_atomic", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "return_plain_tree_exact_source_budget_needs_zero_logical_edges", "logical_edges: 0", "return_plain_tree_capture_depth_and_original_node_caps_stay_bounded", "for count in [31, 32]", "for depth in [8, 9]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-all-leaf-kinds-source",
        path: "tools/nuisc/tests/native_application_bridge/return_plain_tree.ns",
        required_patterns: &["if choose(gate, 1 / root)", "const sealed: bool = flag || false", "return answer_work(local, 1 / answer);", "return sealed;", "return true;", "return false;", "let ignored = unused(local, 1 / extra)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_plain_tree_cases.rs",
        required_patterns: &["pub fn source_variants() -> [String; 4]", "(4..=10)", "pub fn before_failure", "pub fn trace", "mirror(&wrapped)", "self.seed + 1 == 9", "self.flag.into()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-reference-native-failures",
        path: "tools/nuisc/tests/native_application_bridge/return_plain_tree.rs",
        required_patterns: &["return_plain_tree_reference_all_leaf_kinds_and_atomic_failed_event_publication", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "return_plain_tree_native_selected_only_callee_inputs_canaries_and_seven_traps", "before_failure: c.before_failure(slot)", "cases::OBSERVER"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-aot-order",
        path: "tools/nuisc/src/aot_application_effect_call_return_plain_tree_tests.rs",
        required_patterns: &["native_return_plain_tree_keeps_guard_first_effect_order_and_explicit_print_policy", "assert!(helpers > 0)", "\"guard_return\"", "lacks dependency order", "native-session-aot-bundle:counter"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_plain_tree.rs",
        required_patterns: &["return_plain_tree_build_cache_and_two_source_free_restores", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[7, 9, 10]", "restore changed input artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_plain_tree.rs", "plain_tree::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Ordinary complete bool-return-tree follow-up:", "Previous ordinary bool-prefix checkpoint:", "Previous ordinary bool-prefix task checkpoint:", "Previous ordinary bool-prefix blocker checkpoint:", "Previous ordinary bool-prefix action checkpoint:", "Previous ordinary bool-prefix validation checkpoint:", "Previous ordinary bool-prefix artifact checkpoint:", "nuis-native-plain-bool-return-trees-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-tree-receipt",
        path: "docs/reference/nuis-native-plain-bool-return-trees-v1.md",
        required_patterns: &["complete bool-return trees", "zero original logical edges", "seven argument boundaries", "sixteen delivery images", "No generalized field-arm exit discovery", "whole-workspace formatting"],
    },
];
