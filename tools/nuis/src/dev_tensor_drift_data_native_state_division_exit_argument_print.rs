use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_division_exit_argument_print.ns",
        required_patterns: &[
            "confirm_continue(current.total / continue_divisor, permit",
            "confirm_break(current.total / break_divisor, seal",
            "continue_divisor: f64, break_divisor: f64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_division_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_division_exit_argument_print_accepts_checked_ready_arguments",
            "state_division_exit_argument_print_keeps_exact_types_and_original_epochs",
            "SOURCE.replace(exit, \"\")",
            "current.total / current.flag",
            "continue_divisor: {other}",
            "break_divisor: {other}",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-model",
        path: "tools/nuisc/tests/native_application_bridge/state_division_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn quotient",
            "divisor as u64 & (kind.mask() - 1) == 0",
            "f32::from_bits(bits as u32) / f32::from_bits(divisor as u32)",
            "f64::from_bits(bits as u64) / f64::from_bits(divisor as u64)",
            "pub fn plan",
            "quotient(kind, bits, divisors[slot]).ok_or(())?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-dispatch",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.division_exits()",
            "super::state_division_exits::evaluate(",
            "GuardClock::StateDivisionExit { guard, .. }",
            "Err(()) => return Err(trace)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-observers",
        path: "tools/nuisc/tests/native_application_bridge/state_division_exit_argument_print_cases.rs",
        required_patterns: &[
            "OBSERVERS_F32",
            "OBSERVERS_F64",
            "fcmp uno float %arg0, %arg0",
            "fcmp uno double %arg0, %arg0",
            "select i1 %division_nan, i64 2143289344",
            "select i1 %division_nan, i64 9221120237041090560",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_division_exit_argument_print.rs",
        required_patterns: &[
            "state_division_exit_argument_print_native_distinguishes_operand_order",
            "state_division_exit_argument_print_native_keeps_reservations_and_entry_limits",
            "state_division_exit_argument_print_oracle_pins_ieee_schedule_and_argument_fault_order",
            "row.words().len(), 27",
            "argument failed after callee entry",
            "rejected.rows[0].input = row.words()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_division_exit_argument_print_fresh_cache_and_source_free_delivery_preserve_quotients",
            "cases::STATE_DIVISION_EXIT",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_division_exit_argument_print;",
            "pub(super) const STATE_DIVISION_EXIT_ARGUMENT_PRINT",
            "state_division_exit_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-receipt",
        path: "docs/reference/nuis-native-state-division-exit-argument-print-v1.md",
        required_patterns: &[
            "Checked Division Exit Arguments",
            "28 raw sequences",
            "ten finite sequences",
            "19 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded division-exit witnesses:",
            "Previous state-field-exit checkpoint:",
            "Previous state-field-exit task checkpoint:",
            "Previous state-field-exit blocker checkpoint:",
            "Previous state-field-exit action checkpoint:",
            "Previous state-field-exit validation checkpoint:",
            "Previous state-field-exit artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-division-exit-argument-print-lineage-regression",
        path: "tools/nuis/src/dev_tensor_tests.rs",
        required_patterns: &[
            "dev_tensor_state_division_exit_retains_complete_field_exit_lineage",
            "15e61462dd2f9ddcb722522b85e366b8ac913e22fb42ca046cbca666747ca7da",
            "71744b494a0ed788ca57b8c430894fc2489ece1702b801e085f83b2c4277a3d9",
            "4a21d28b8e0782d1e94b18490804e25a7316b24c24449b119c8c5e8833f5483e",
            "07b21bc487919f23bbdbcc45d6a1c591c629010e450b58fd849c551059b01f93",
            "198ca27e54366a001545f7d5119a0c72d1839e62da5d6fc8014b0c0a012e58ba",
            "c94482d3e50f756b426235344e2a38f45e9b4d5265b008f13182363e0e3198d9",
        ],
    },
];
