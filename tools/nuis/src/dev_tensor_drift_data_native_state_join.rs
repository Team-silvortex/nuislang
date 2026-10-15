use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-join-source-mutable-local-and-common-tail",
        path: "tools/nuisc/tests/native_application_bridge/state_join.ns",
        required_patterns: &[
            "let mut current = value",
            "current = local + 2",
            "current = local - 2",
            "return finish(current, 1 / tail)",
            "gate != state.result",
            "answer_work(total, state.result",
            "marker: state.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-independent-current-value-and-poison-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_join_cases.rs",
        required_patterns: &[
            "self.entering() + if self.route() { 4 } else { -1 }",
            "let total = self.selected() + 3",
            "self.prior() ^ (total % 2 == 0)",
            "failure must target selected work",
            "[3, 4, 5, 6, 7, 9]",
            "pub const EVENTS: usize = 6",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-oracle-reference-native-and-fault-retention",
        path: "tools/nuisc/tests/native_application_bridge/state_join.rs",
        required_patterns: &[
            "state_join_oracle_exposes_selected_and_common_current_value_dependencies",
            "state_join_reference_checks_rejoin_tail_and_faulted_last_publication",
            "state_join_native_rejoins_selected_current_value_before_publication",
            "call.selected() % 2 == 0",
            "pair[0].words(None), pair[1].words(None)",
            "assert_eq!(cases::failures().len(), 10)",
            "assert_state(&session, last)",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-unchanged-real-word-sequence-probe",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "load volatile i64",
            "store volatile i64 %last",
            "llvm.insert_str(at, observer)",
            "state_width + 2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-guard-effect-policy-and-layout",
        path: "tools/nuisc/src/aot_application_effect_call_state_join_tests.rs",
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
        id: "native-state-join-cli-identical-inputs-joined-traps-and-restores",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_join.rs",
        required_patterns: &[
            "state_join_build_cache_and_two_source_free_restores_preserve_common_tail",
            "pair[0].args(None), pair[1].args(None)",
            ".zip([false, true, false, true])",
            "assert_eq!(expected.len(), 8)",
            "matches!(slot, 6 | 7 | 9)",
            "fresh.host_objects == cached.host_objects",
            "restore changed input artifact",
            "for _ in 0..2",
            "marker.parse::<f64>().unwrap().to_bits()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-script-bound-still-64",
        path: "crates/yir-runtime-host/src/native_application_session/input.rs",
        required_patterns: &[
            "const MAX_EVENTS: usize = 64",
            "arguments.len() >= MAX_ARGC",
            "events.len() < MAX_EVENTS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/state_join.rs", "mod state_join;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &[
            "aot_application_effect_call_state_join_tests.rs",
            "mod state_join;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_state_join.rs", "mod state_join;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &[
            "dev_tensor_drift_data_native_state_join.rs",
            "state_join::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-six-field-lineage-and-next-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful continuing branch join follow-up:",
            "Previous state-routing checkpoint:",
            "Previous state-routing task checkpoint:",
            "Previous state-routing blocker checkpoint:",
            "Previous state-routing action checkpoint:",
            "Previous state-routing validation checkpoint:",
            "Previous state-routing artifact checkpoint:",
            "nuis-native-stateful-branch-joins-v1.md",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-join-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-branch-joins-v1.md",
        required_patterns: &[
            "4096 healthy accepted callbacks",
            "240 accepted callbacks",
            "1312 accepted CLI callbacks",
            "identical complete event arguments",
            "state-aware poisoned arguments",
            "quoted no-op heredoc",
            "## Next Boundary",
        ],
    },
];
