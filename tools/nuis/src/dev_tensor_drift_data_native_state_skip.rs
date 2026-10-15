use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-skip-source-current-retention-and-common-tail",
        path: "tools/nuisc/tests/native_application_bridge/state_skip.ns",
        required_patterns: &[
            "let mut current = value",
            "if choose(gate, 1 / root)",
            "current = local + 2",
            "return finish(current, 1 / tail)",
            "gate != state.result",
            "marker: state.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-independent-retained-value-and-poison-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_skip_cases.rs",
        required_patterns: &[
            "self.entering() + 1 + if self.route() { 3 } else { 0 }",
            "let total = self.selected() + 3",
            "[-8, -1, 0, 7, 8]",
            "(!self.poisoned || self.route()).into()",
            "[3, 4, 5, 6, 8]",
            "if choose(gate != true, 1 / root) {{}} else",
            "pub const EVENTS: usize = 6",
            "failure must target selected work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-negative-zero-nonneutral-and-fault-runtime",
        path: "tools/nuisc/tests/native_application_bridge/state_skip.rs",
        required_patterns: &[
            "state_skip_oracle_exposes_retained_nonzero_negative_and_zero_inputs",
            "state_skip_reference_retains_skipped_values_and_last_publication",
            "state_skip_native_carries_real_retained_values_through_common_tail",
            "assert_eq!(retained, [true; 3])",
            "pair[0].words(None), pair[1].words(None)",
            "assert_eq!(cases::failures().len(), 9)",
            "assert_state(&session, last)",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-shared-retained-leaf-and-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_nested.rs",
        required_patterns: &[
            "if yes.is_empty() || no.is_empty()",
            "value: NirExpr::Var(live.name.clone())",
            "let retained = leaf(",
            "nodes: 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-guard-effect-policy-and-layout",
        path: "tools/nuisc/src/aot_application_effect_call_state_skip_tests.rs",
        required_patterns: &[
            "State{result:bool;total:i64;marker:f64}",
            "guard_return",
            "yir_core::EdgeKind::Effect",
            "lacks dependency order",
            "native-session-aot-bundle:counter",
            "Some(&\"call_bool\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-cli-identical-inputs-and-source-free-retention",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_skip.rs",
        required_patterns: &[
            "state_skip_build_cache_and_two_source_free_restores_preserve_skipped_values",
            "pair[0].args(None), pair[1].args(None)",
            ".zip([false, true, false, true])",
            "assert_eq!(expected.len(), 8)",
            "matches!(slot, 5 | 6 | 8)",
            "fresh.host_objects == cached.host_objects",
            "restore changed input artifact",
            "for _ in 0..2",
            "marker.parse::<f64>().unwrap().to_bits()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-script-bound-still-64",
        path: "crates/yir-runtime-host/src/native_application_session/input.rs",
        required_patterns: &[
            "const MAX_EVENTS: usize = 64",
            "arguments.len() >= MAX_ARGC",
            "events.len() < MAX_EVENTS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/state_skip.rs", "mod state_skip;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &[
            "aot_application_effect_call_state_skip_tests.rs",
            "mod state_skip;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_state_skip.rs", "mod state_skip;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &[
            "dev_tensor_drift_data_native_state_skip.rs",
            "state_skip::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-six-field-lineage-and-next-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful skipped-arm retention follow-up:",
            "Previous state-join checkpoint:",
            "Previous state-join task checkpoint:",
            "Previous state-join blocker checkpoint:",
            "Previous state-join action checkpoint:",
            "Previous state-join validation checkpoint:",
            "Previous state-join artifact checkpoint:",
            "nuis-native-stateful-skipped-arm-retention-v1.md",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-skip-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-skipped-arm-retention-v1.md",
        required_patterns: &[
            "10240 healthy accepted callbacks",
            "216 accepted callbacks",
            "1312 accepted CLI callbacks",
            "identical complete event arguments",
            "state-aware poisoned arguments",
            "quoted no-op heredoc",
            "## Next Boundary",
        ],
    },
];
