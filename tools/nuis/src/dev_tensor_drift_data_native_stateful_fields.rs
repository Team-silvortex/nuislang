use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-source-dependent-reconstruction",
        path: "tools/nuisc/tests/native_application_bridge/return_stateful_fields.ns",
        required_patterns: &["result: bool, total: i64, marker: f64", "advance(state.total, 1 / advancing)", "observe(total, gate, first, state.result", "marker: state.marker", "value % 2 == 0", "print(5)", "const input: bool = flag"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-raw-marker-and-parity-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_stateful_fields_cases.rs",
        required_patterns: &["0xfff8000012345678", "0x8000000000000000", "let total = state.total + 1", "marker: state.marker", "^ if self.gate", "123.5f64.to_bits()", "(4..=11)", "pub fn source_variants() -> [String; 4]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-runtime-witnesses-bits-and-fault-retention",
        path: "tools/nuisc/tests/native_application_bridge/return_stateful_fields.rs",
        required_patterns: &["return_stateful_fields_oracle_exposes_current_total_bool_and_marker_dependencies", "return_stateful_fields_reference_keeps_dependent_results_raw_float_bits_and_last_publication", "return_stateful_fields_native_reuses_three_real_words_and_preserves_float_payloads", "event.result(previous.result, previous.total)", "assert_ne!(event.marker, marker)", "state_words(session.state())", "ApplicationSessionPhase::Faulted", "assert_state(&session, expected)", "probe::check_sequences"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-generic-actual-output-word-probe",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &["for slot in 0..state_width", "load volatile i64", "store volatile i64 %last", "state_width + 2", "llvm.insert_str(at, observer)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-semantic-aot-order-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_stateful_fields_tests.rs",
        required_patterns: &["native_return_stateful_fields_keeps_guard_first_effect_order_and_explicit_print_policy", "State{result:bool;total:i64;marker:f64}", "assert!(calls.len() >= 2)", "assert_eq!(calls[0], \"call_i64\")", "calls[1..].iter().all(|call| *call == \"call_bool\")", "assert_eq!(entries, 2)", "yir_core::EdgeKind::Effect", "lacks dependency order", "native-session-aot-bundle:counter"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-bounded-cli-bits-identities-and-restores",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_stateful_fields.rs",
        required_patterns: &["return_stateful_fields_build_cache_and_two_source_free_restores_consume_latest_state", "rows.chunks(4).zip(cases::CLI_MARKERS)", "assert_eq!(expected.len(), 21)", "marker.parse::<f64>().unwrap().to_bits()", "assert_marker_bits(&run, last.marker)", "[7, 9, 10, 11, 13]", "for _ in 0..65", "rejected_before_open(run)", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "restore changed input artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-unchanged-runtime-script-bound",
        path: "crates/yir-runtime-host/src/native_application_session/input.rs",
        required_patterns: &["const MAX_EVENTS: usize = 64", "arguments.len() >= MAX_ARGC", "events.len() < MAX_EVENTS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_stateful_fields.rs", "mod return_stateful_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_stateful_fields_tests.rs", "mod return_stateful_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_stateful_fields.rs", "mod return_stateful_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_stateful_fields.rs", "stateful_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-six-field-lineage-and-next-gates",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Typed state reconstruction follow-up:", "Previous stateful-bool checkpoint:", "Previous stateful-bool task checkpoint:", "Previous stateful-bool blocker checkpoint:", "Previous stateful-bool action checkpoint:", "Previous stateful-bool validation checkpoint:", "Previous stateful-bool artifact checkpoint:", "nuis-native-stateful-field-reconstruction-v1.md", "state-derived branch selection"],
    },
    DevTensorDriftCheckSpec {
        id: "native-stateful-fields-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-field-reconstruction-v1.md",
        required_patterns: &["3584 healthy accepted callbacks", "216 accepted callbacks", "3056 accepted CLI callbacks", "decimal-input coverage, not raw NaN payload coverage", "64-event bound", "six complete preceding", "whole-workspace formatting", "## Next Boundary"],
    },
];
