use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-existing-runtime-script-bounds",
        path: "crates/yir-runtime-host/src/native_application_session/input.rs",
        required_patterns: &["const MAX_EVENTS: usize = 64", "const MAX_ARGC: usize = 2 * MAX_EVENTS + 7", "arguments.len() >= MAX_ARGC", "events.len() < MAX_EVENTS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-source-real-state-consumption",
        path: "tools/nuisc/tests/native_application_bridge/return_stateful_bool.ns",
        required_patterns: &["previous != ((value == 9) && (proof == 1))", "const input: bool = flag", "answer_work(local, input, 1 / answer)", "const on: bool = flag != true", "const off: bool = flag", "observe(seed, gate, first, state.result", "return State { result: result }"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-independent-parity-oracle-and-repeated-inputs",
        path: "tools/nuisc/tests/native_application_bridge/return_stateful_bool_cases.rs",
        required_patterns: &["previous ^ toggle", "[toggle(), self, self, toggle(), self]", "[identity(), toggle(), identity()]", "pub fn source_variants() -> [String; 4]", "%seen_previous = zext i1 %arg1 to i64", "pub fn before_failure", "(4..=11)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-reference-native-counterfactuals-and-publication",
        path: "tools/nuisc/tests/native_application_bridge/return_stateful_bool.rs",
        required_patterns: &["return_stateful_bool_oracle_distinguishes_open_stale_ignored_and_repeated_input_results", "return_stateful_bool_reference_consumes_latest_state_and_retains_failed_event_publication", "return_stateful_bool_native_reuses_real_outputs_across_five_events_and_eight_traps", "assert_ne!(result, event.result(open))", "ignored && ignores_false && ignores_true", "assert_ne!(last, opened)", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "probe::check_sequences"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-sequence-probe-preserves-real-output-and-pair-adapter",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &["pub(super) struct Sequence", "pub(super) struct SequenceFailure", "pub(super) fn check_sequences", "rows: vec![a.clone(), b.clone()]", "load volatile i64", "store volatile i64 %last", "usize::from(index != 0)", "llvm.insert_str(at, observer)", "expected.extend(&sequence.rows.last().unwrap().state)", "expected.extend(&failure.before_failure)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-aot-guards-effects-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_stateful_bool_tests.rs",
        required_patterns: &["native_return_stateful_bool_keeps_guard_first_effect_order_and_explicit_print_policy", "State{result:bool}", "assert!(helpers > 0)", "yir_core::EdgeKind::Effect", "seen.contains(pair[1].name.as_str())", "lacks dependency order", "native-session-aot-bundle:counter", "owned_aggregate_alloc_v1", "owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-bounded-cli-and-source-free-identities",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_stateful_bool.rs",
        required_patterns: &["return_stateful_bool_build_cache_and_two_source_free_restores_consume_latest_state", "assert_eq!(expected.len(), 41)", "rows.chunks(8)", "for _ in 0..65", "rejected_before_open(run)", "assert!(repeated_input_changes_result)", "assert!(!opened && last)", "[7, 9, 10, 11]", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "fs::remove_dir_all(project.0.join(\".nuis\"))", "restore changed input artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_stateful_bool.rs", "mod return_stateful_bool;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_stateful_bool_tests.rs", "mod return_stateful_bool;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_stateful_bool.rs", "mod return_stateful_bool;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_stateful_bool.rs", "stateful_bool::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["State-dependent bool session follow-up:", "Previous sealed-binding checkpoint:", "Previous sealed-binding task checkpoint:", "Previous sealed-binding blocker checkpoint:", "Previous sealed-binding action checkpoint:", "Previous sealed-binding validation checkpoint:", "Previous sealed-binding artifact checkpoint:", "nuis-native-stateful-bool-sessions-v1.md", "typed multi-field State"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-bool-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-bool-sessions-v1.md",
        required_patterns: &["state.result", "arithmetic witnesses, not mutated-compiler runs", "896 healthy accepted callbacks", "192 accepted callbacks", "2880 accepted CLI callbacks", "80 events", "65 events reject before open", "six complete preceding", "whole-workspace formatting", "## Next Boundary"],
    },
];
