use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-plain-const-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs",
        required_patterns: &["effectful_selections_plain_bool_const_tests.rs", "if constant\n            || matches!(", "constant && local.contains_key(name)", "predicates::prove(value, &local, signatures, budget)?", "Statement::Logical(statement.clone(), predicate)", "ty.is_some_and(|ty| ty != &inferred)", "constants.insert(name.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_plain_bool_const_tests.rs",
        required_patterns: &["return_plain_const_literals_names_calls_and_selected_seals_share_existing_proof", "return_plain_const_type_seal_scope_hidden_effect_and_late_vetoes_are_atomic", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "return_plain_const_exact_original_budget_adds_no_logical_edges_or_helpers", "logical_edges: 0", "(4, 7, true)", "return_plain_const_capture_depth_and_prefix_limits_stay_bounded", "for count in [31, 32]", "for depth in [31, 32]", "for depth in [8, 9]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-source",
        path: "tools/nuisc/tests/native_application_bridge/return_plain_const.ns",
        required_patterns: &["const enabled: bool = true", "const input: bool = flag", "const sealed: bool = seal(input, enabled, 1 / sealing)", "const called: bool = answer_work(local, 1 / answer)", "const copied: bool = first", "const on: bool = true", "const off: bool = false"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_plain_const_cases.rs",
        required_patterns: &["pub fn source_variants() -> [String; 4]", "(4..=11)", "(!self.flag).into()", "self.seed + 1 == 9", "out.push(50)", "pub fn before_failure", "mirror(&wrapped)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-native",
        path: "tools/nuisc/tests/native_application_bridge/return_plain_const.rs",
        required_patterns: &["return_plain_const_reference_all_leaf_kinds_and_atomic_failed_event_publication", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "return_plain_const_native_selected_only_callee_inputs_canaries_and_eight_traps", "before_failure: c.before_failure(slot)", "cases::OBSERVER"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_plain_const_tests.rs",
        required_patterns: &["native_return_plain_const_keeps_guard_first_effect_order_and_explicit_print_policy", "assert!(helpers > 0)", "\"guard_return\"", "lacks dependency order", "native-session-aot-bundle:counter"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_plain_const.rs",
        required_patterns: &["return_plain_const_build_cache_and_two_source_free_restores", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[7, 9, 10, 11]", "restore changed input artifact", "c.result() != i64::from(flag)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_plain_const.rs", "mod return_plain_const;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_plain_const_tests.rs", "mod return_plain_const;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-delivery-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_plain_const.rs", "mod return_plain_const;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_plain_const.rs", "plain_const::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Ordinary bool const staging follow-up:", "Previous ordinary bool-return-tree checkpoint:", "Previous ordinary bool-return-tree task checkpoint:", "Previous ordinary bool-return-tree blocker checkpoint:", "Previous ordinary bool-return-tree action checkpoint:", "Previous ordinary bool-return-tree validation checkpoint:", "Previous ordinary bool-return-tree artifact checkpoint:", "nuis-native-plain-bool-const-staging-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-const-receipt",
        path: "docs/reference/nuis-native-plain-bool-const-staging-v1.md",
        required_patterns: &["fresh bool const", "zero original logical edges", "eight independently poisoned argument boundaries", "sixteen delivery images", "state-dependent bool computation", "whole-workspace formatting"],
    },
];
