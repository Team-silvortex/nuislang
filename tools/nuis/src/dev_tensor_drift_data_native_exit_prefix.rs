use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-dispatch",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["effectful_selections_return_exit_prefix.rs", "prefix: &'a [NirStmt]", "let (value, prefix) = returned(then_body)?", "continuing(prefix)", "exit_prefix::prove(&split"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-paired",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_paired_in_fields.rs",
        required_patterns: &["exit_prefix::prove(&arm", "&clean(&yes)", "&clean(&no)", "std::iter::once(gate.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_exit_prefix.rs",
        required_patterns: &["fields: &updates::PrivateFields", "exit.prefix", "fields.tail(", "depth + 2", "reads.difference(&staged.defined)", "SelectedValue::StagedPredicate", "nested::consume_node(budget)?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_exit_prefix_tests.rs",
        required_patterns: &["return_exit_prefix_uses_original_certificate_and_one_common_tail", "calls(\"tail_work\"), 1", "return_exit_prefix_rejects_new_authority_types_seals_flat_children_and_exits", "assert_eq!(module, original)", "assert_eq!(names, original_names)", "return_exit_prefix_does_not_export_locals_or_discover_deeper_returns", "return_exit_prefix_still_accepts_single_sided_and_direct_exits", "return_exit_prefix_retains_five_field_predecessor_authority", "return_exit_prefix_inherits_node_expression_logical_and_depth_limits", "assert_eq!(nodes, 24)", "assert_eq!(edges, 1)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-source",
        path: "tools/nuisc/tests/native_application_bridge/return_exit_prefix.ns",
        required_patterns: &["let local = exit_owner(local, 1 / work);", "if inner_condition(inner, 1 / choice)", "let a = exit_small(a, 1 / child);", "let d = exit_flag(d, 1 / child);", "return answer_work(local, a, d, 1 / answer);", "let finished = tail_work(local, 1 / tail);"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-cases",
        path: "tools/nuisc/tests/native_application_bridge/return_exit_prefix_cases.rs",
        required_patterns: &["pub fn fields", "self.take && self.inner", "pub fn next", "pub(super) fn failures", "(4, 11, true)", "(8, 22, true)", "(10, 25, false)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-runtime",
        path: "tools/nuisc/tests/native_application_bridge/return_exit_prefix.rs",
        required_patterns: &["return_exit_prefix_reference_current_fields_lazy_work_and_last_publication", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "return_exit_prefix_native_observed_fields_effect_cuts_canaries_and_reentry", "for mode in 0..=7", "case.next()", "sext i32 %arg1 to i64", "zext i1 %arg2 to i64", "run.status.signal()", "trace.len() - 4"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_exit_prefix_tests.rs",
        required_patterns: &["native_return_exit_prefix_keeps_single_tail_and_guard_first_effect_order", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "owned_aggregate_drop_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_exit_prefix.rs", "mod return_exit_prefix;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_exit_prefix_tests.rs", "mod return_exit_prefix;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Returning exit-branch prefix follow-up:", "Previous paired source-arm exit checkpoint:", "Previous paired source-arm exit task checkpoint:", "Previous paired source-arm exit blocker checkpoint:", "Previous paired source-arm exit action checkpoint:", "Previous paired source-arm exit validation checkpoint:", "Previous paired source-arm exit artifact checkpoint:", "nuis-native-returning-exit-prefixes-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-receipt",
        path: "docs/reference/nuis-native-returning-exit-prefixes-v1.md",
        required_patterns: &["one inherited certificate", "seven selected argument boundaries", "not a fresh CLI delivery replay", "five-child native", "whole-workspace formatting"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-exit-prefix-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_exit_prefix.rs", "exit_prefix::CHECKS"],
    },
];
