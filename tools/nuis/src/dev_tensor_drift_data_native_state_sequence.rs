use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-source-fresh-second-condition-and-work",
        path: "tools/nuisc/tests/native_application_bridge/state_sequence.ns",
        required_patterns: &[
            "let mut current = value",
            "if choose(gate, 1 / root)",
            "if select(second != (current % 2 == 0), 1 / next_root)",
            "refine(current, 1 / next_work)",
            "current = local - 2",
            "return finish(current, 1 / tail)",
            "gate != state.result",
            "marker: state.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-independent-stage-and-trap-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_sequence_cases.rs",
        required_patterns: &[
            "self.input.second ^ (self.first() % 2 == 0)",
            "2 * self.first() - 1",
            "[-8, -1, 8]",
            "(!self.poisoned || self.second_route()).into()",
            "[4, 5, 6, 7, 8, 9, 11]",
            "pub const SECOND_OBSERVER",
            "pub const EVENTS: usize = 6",
            "failure must target selected work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-runtime-freshness-and-fault-publication",
        path: "tools/nuisc/tests/native_application_bridge/state_sequence.rs",
        required_patterns: &[
            "state_sequence_oracle_exposes_fresh_predicate_and_second_work_inputs",
            "state_sequence_reference_checks_current_handoff_and_last_publication",
            "state_sequence_native_observes_both_real_callees_and_current_handoff",
            "assert_eq!(routes.len(), 4)",
            "assert_eq!(retained, [true; 3])",
            "assert_eq!(cases::failures().len(), 17)",
            "assert_state(&session, last)",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-generic-multiple-callee-observer-and-compatibility",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "for (callee, observer) in observers",
            "aggregate_values::definition(&llvm, callee)",
            "llvm.insert_str(at, observer)",
            "pub(super) fn check_sequences_observed",
            "load volatile i64",
            "&[(\"answer_work\", observer)]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-shared-scope-and-seal-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &[
            "scope.insert(name, ty)",
            "scope.insert(name.clone(), ty)",
            "regions::writes_constants",
            "function.body = output",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-guard-effect-policy-and-layout",
        path: "tools/nuisc/src/aot_application_effect_call_state_sequence_tests.rs",
        required_patterns: &[
            "State{result:bool;total:i64;marker:f64}",
            "guard_return",
            "yir_core::EdgeKind::Effect",
            "lacks dependency order",
            "native-session-aot-bundle:counter",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-cli-four-routes-and-source-free-identities",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_sequence.rs",
        required_patterns: &[
            "state_sequence_build_cache_and_two_source_free_restores_preserve_sequential_handoff",
            "pair[0].args(None), pair[1].args(None)",
            "assert_eq!(routes.len(), 4)",
            "assert_eq!(expected.len(), 8)",
            "matches!(slot, 6 | 7 | 8)",
            "fresh.host_objects == cached.host_objects",
            "restore changed input artifact",
            "for _ in 0..2",
            "marker.parse::<f64>().unwrap().to_bits()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-script-bound-still-64",
        path: "crates/yir-runtime-host/src/native_application_session/input.rs",
        required_patterns: &[
            "const MAX_EVENTS: usize = 64",
            "arguments.len() >= MAX_ARGC",
            "events.len() < MAX_EVENTS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &[
            "native_application_bridge/state_sequence.rs",
            "mod state_sequence;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &[
            "aot_application_effect_call_state_sequence_tests.rs",
            "mod state_sequence;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_state_sequence.rs", "mod state_sequence;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &[
            "dev_tensor_drift_data_native_state_sequence.rs",
            "state_sequence::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-six-field-lineage-and-next-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful sequential optional updates follow-up:",
            "Previous state-skip checkpoint:",
            "Previous state-skip task checkpoint:",
            "Previous state-skip blocker checkpoint:",
            "Previous state-skip action checkpoint:",
            "Previous state-skip validation checkpoint:",
            "Previous state-skip artifact checkpoint:",
            "nuis-native-stateful-sequential-updates-v1.md",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-sequence-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-sequential-updates-v1.md",
        required_patterns: &[
            "12288 healthy accepted callbacks",
            "408 accepted callbacks",
            "1408 accepted CLI callbacks",
            "identical complete event arguments",
            "state-aware poisoned arguments",
            "quoted no-op heredoc",
            "## Next Boundary",
        ],
    },
];
