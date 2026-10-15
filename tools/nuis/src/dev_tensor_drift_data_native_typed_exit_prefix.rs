use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-source",
        path: "tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix.ns",
        required_patterns: &["let b = exit_narrow(b, a, 1 / narrow_proof);", "let c = exit_wide(c, b, 1 / wide_proof);", "let d = exit_flag(d, c, 1 / flag_proof);", "let e = exit_peer(e, d, 1 / peer_proof);", "sibling_owner(local, 1 / no_owner)", "let finished = tail_work(local, 1 / tail);"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_typed_exit_prefix_tests.rs",
        required_patterns: &["return_typed_exit_prefix_preserves_six_data_fields_one_tail_and_input_bound", "return_typed_exit_prefix_has_unmasked_current_return_operand_witnesses", "return_typed_exit_prefix_keeps_atomic_authority_and_shape_vetoes", "assert_eq!(module, original)", "assert_eq!(names, original_names)", "return_typed_exit_prefix_charges_original_work_and_inherits_depth", "assert_eq!(nodes, 39)", "f.params.len() <= 32"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-cases",
        path: "tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix_cases.rs",
        required_patterns: &["STAGES: [i64; 12]", "pub fn witnesses()", "pub fn delivery_cases()", "stale[field] = c.before()[field]", "pub fn mirror_source()", "if !self.pick", "if self.pick", "0xff801234", "0xfff0000000001234"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-runtime",
        path: "tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix.rs",
        required_patterns: &["return_typed_exit_prefix_reference_five_current_fields_order_and_fault_publication", "return_typed_exit_prefix_native_raw_ieee_current_fields_canaries_and_twelve_traps", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "typed_effect_returns::reordered", "probe::check(", "cases::OBSERVER"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-probe",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &["pub(super) struct Row", "pub(super) struct Failure", "input_width", "load volatile i64", "llvm.insert_str(at, observer)", "for mode in 0..=failures.len()", "expected.extend(&failure.before_failure)", "owned_aggregate_alloc_v1", "owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_typed_exit_prefix_tests.rs",
        required_patterns: &["native_return_typed_exit_prefix_retains_five_fields_one_tail_and_effect_policy", "child_4:bool", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_typed_exit_prefix.rs",
        required_patterns: &["return_typed_exit_prefix_cli_audits_lazy_proofs_and_current_value_witnesses", "cases::delivery_cases().len(), 22", "return_typed_exit_prefix_build_cache_and_two_source_free_restores", "cases::delivery_cases()", "[20, 30, 28]", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_typed_exit_prefix.rs", "mod return_typed_exit_prefix;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_typed_exit_prefix_tests.rs", "mod return_typed_exit_prefix;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_typed_exit_prefix.rs", "mod return_typed_exit_prefix;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_typed_exit_prefix.rs", "typed_exit_prefix::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Typed returning-prefix execution follow-up:", "Previous returning exit-prefix checkpoint:", "Previous returning exit-prefix task checkpoint:", "Previous returning exit-prefix blocker checkpoint:", "Previous returning exit-prefix action checkpoint:", "Previous returning exit-prefix validation checkpoint:", "Previous returning exit-prefix artifact checkpoint:", "nuis-native-typed-returning-prefixes-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-exit-prefix-receipt",
        path: "docs/reference/nuis-native-typed-returning-prefixes-v1.md",
        required_patterns: &["one inherited certificate", "six independent stale-operand witnesses", "32 paired IEEE rows", "eight delivery images", "whole-workspace formatting", "ordinary bool-call suffix"],
    },
];
