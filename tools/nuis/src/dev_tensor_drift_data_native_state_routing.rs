use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-routing-source-prior-bool-current-count",
        path: "tools/nuisc/tests/native_application_bridge/return_state_routing.ns",
        required_patterns: &["gate != state.result", "first != (total % 2 == 0)", "advance(state.total, 1 / advancing)", "marker: state.marker"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-independent-inputs-and-poison-masks",
        path: "tools/nuisc/tests/native_application_bridge/return_state_routing_cases.rs",
        required_patterns: &["pub previous: Option<State>", "self.input.gate ^ self.prior()", "self.input.first ^ (self.total() % 2 == 0)", "if !self.poisoned", "failure must target selected work", "assert_eq!(rows[0], SOURCE)", "pub const EVENTS: usize = 6"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-reference-native-and-counterfactuals",
        path: "tools/nuisc/tests/native_application_bridge/return_state_routing.rs",
        required_patterns: &["return_state_routing_oracle_distinguishes_identical_inputs_from_poisoned_masks", "return_state_routing_reference_checks_selected_work_and_faulted_last_publication", "return_state_routing_native_branches_on_actual_previous_output_and_new_total", "pair[0].words(None), pair[1].words(None)", "assert_eq!(routes.len(), 4)", "wrong_bool.route().gate", "wrong_total.route().first", "assert_state(&session, last)", "ApplicationSessionPhase::Faulted"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-actual-output-word-probe",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &["load volatile i64", "store volatile i64 %last", "llvm.insert_str(at, observer)", "state_width + 2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-guard-effect-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_state_routing_tests.rs",
        required_patterns: &["State{result:bool;total:i64;marker:f64}", "guard_return", "yir_core::EdgeKind::Effect", "lacks dependency order", "native-session-aot-bundle:counter", "assert_eq!(entries, 2)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-identical-cli-inputs-and-source-free-identities",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_state_routing.rs",
        required_patterns: &["return_state_routing_build_cache_and_two_source_free_restores_preserve_state_derived_branches", "pair[0].args(None), pair[1].args(None)", "for poisoned in [false, true]", "assert_eq!(expected.len(), 8)", "[7, 9, 10, 11, 13]", "marker.parse::<f64>().unwrap().to_bits()", "fresh.host_objects == cached.host_objects", "restore changed input artifact", "for _ in 0..2", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-script-bound-not-widened",
        path: "crates/yir-runtime-host/src/native_application_session/input.rs",
        required_patterns: &["const MAX_EVENTS: usize = 64", "arguments.len() >= MAX_ARGC", "events.len() < MAX_EVENTS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_state_routing.rs", "mod return_state_routing;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_state_routing_tests.rs", "mod return_state_routing;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_state_routing.rs", "mod return_state_routing;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_state_routing.rs", "state_routing::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-six-field-lineage-and-next-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["State-derived branch selection follow-up:", "Previous stateful-fields checkpoint:", "Previous stateful-fields task checkpoint:", "Previous stateful-fields blocker checkpoint:", "Previous stateful-fields action checkpoint:", "Previous stateful-fields validation checkpoint:", "Previous stateful-fields artifact checkpoint:", "nuis-native-state-derived-branch-selection-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-routing-scoped-receipt",
        path: "docs/reference/nuis-native-state-derived-branch-selection-v1.md",
        required_patterns: &["8192 healthy accepted callbacks", "216 accepted callbacks", "2288 accepted CLI callbacks", "identical complete event arguments", "state-aware poisoned arguments", "quoted no-op heredoc", "## Next Boundary"],
    },
];
