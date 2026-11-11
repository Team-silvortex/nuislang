use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print.ns",
        required_patterns: &[
            "fn advance_control(value: i64, fault: i64) -> i64",
            "let checked = 0 / fault; return value + 1;",
            "advance_control(current.control, control_update_fault - (outer * 16 + child))",
            "total = -(-total);",
            "let parity = round % 2; let active = parity * parity;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-source-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_checked_control_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_checked_control_exit_argument_print_accepts_scoped_update_producer",
            "state_checked_control_exit_argument_print_keeps_original_epochs_and_exact_types",
            "SOURCE.replace(\"control = control + 0;\", \"\")",
            "function `advance_control` argument 1 expects `i64`",
            "current.flag",
            "current.code",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-model",
        path: "tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn execute",
            "self.base.execute(prior, native)",
            "control_update_fault.wrapping_sub(parent * 16 + child)",
            "trace.extend([617, control, fault])",
            "control = control.wrapping_add(1)",
            "words.push(self.control_update_fault)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-observation-boundaries",
        path: "tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn records",
            "601 | 609..=612 | 617 if native => 3",
            "603..=608 | 613..=616 if native => 4",
            "[71, 75, 76, 80, 617]",
            "for fault in [i64::MIN, i64::MAX]",
            "for trips in [i64::MIN, i64::MAX]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print.rs",
        required_patterns: &[
            "state_checked_control_exit_native_distinguishes_reordered_update_checks",
            "state_checked_control_exit_native_consumes_full_width_update_returns",
            "state_checked_control_exit_oracle_decodes_records_and_pins_fault_priority",
            "row.words().len(), 38",
            "trace.ends_with(&[617, updates as i64, 0])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-quota-and-effects",
        path: "tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print.rs",
        required_patterns: &[
            "state_checked_control_exit_native_preserves_work_and_entry_boundaries",
            "state_checked_control_exit_native_requires_an_explicit_update_effect_grant",
            "poison.execute(None, true).unwrap_err()",
            "control producer inherited an effect grant",
            "[38, 40, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-reference-retention",
        path: "tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print.rs",
        required_patterns: &[
            "state_checked_control_exit_reference_preserves_state_on_rejected_and_failed_events",
            "rejected[37] = Value::Bool(false)",
            "ApplicationSessionPhase::Open",
            "ApplicationSessionPhase::Faulted",
            "state_words(session.state()), prior.map(|word| word as u64)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_checked_control_exit_argument_print.rs",
        required_patterns: &[
            "state_checked_control_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
            "assert_eq!(states(&failed), state)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_checked_control_exit_argument_print;",
            "pub(super) const STATE_CHECKED_CONTROL_EXIT_ARGUMENT_PRINT",
            "state_checked_control_exit_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-receipt",
        path: "docs/reference/nuis-native-state-checked-control-exit-argument-print-v1.md",
        required_patterns: &[
            "Independently Checked Control Updates",
            "33 raw sequences",
            "eleven finite sequences",
            "19 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded checked-control witnesses:",
            "Previous state-control-exit checkpoint:",
            "Previous state-control-exit task checkpoint:",
            "Previous state-control-exit blocker checkpoint:",
            "Previous state-control-exit action checkpoint:",
            "Previous state-control-exit validation checkpoint:",
            "Previous state-control-exit artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-checked-control-exit-argument-print-lineage-regression",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_checked_control_exit_retains_complete_control_lineage",
            "aa05ce5c46222c478d014a3b79f9664f71cd21c2239c500769735c4c84eb844d",
            "6e43c43c2fa77931d5c3ffbb0d9e045912f4d0bc0d9d70814d4e42ab4e85fb30",
            "4b8043312f6d2a366aa82281638cc6c37d54e7f3461b0b430450e28b35264810",
            "183df07ea7ae339c8ba653770a890821fd8498f31b2e522078f7b6a37fb4f8c1",
            "c4d94b5d842624556b3dee91152a02f743d04f899f8f0b180218cfd1647d8bf2",
            "b58763494f2f72e4587961a17c5489fd866eba6af8ce2c5575a3eefac606d780",
        ],
    },
];
