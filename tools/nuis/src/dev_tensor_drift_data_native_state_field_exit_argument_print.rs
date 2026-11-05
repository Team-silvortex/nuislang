use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print.ns",
        required_patterns: &[
            "confirm_continue(current.total, permit",
            "confirm_break(current.total, seal",
            "fn confirm_continue(value: f64, mode: i64, guard: i64)",
            "return (mode == 0 && value < 0.0) || (mode == 1 && value == 0.0)",
            "(mode != 0 && mode != 1 && value != value)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_field_exit_argument_print_accepts_updated_record_predicates",
            "state_field_exit_argument_print_keeps_type_and_epoch_refusals",
            "source.replace(exit, \"\")",
            "(\"confirm_break\", \"current.code\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-model",
        path: "tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn plan",
            "pub fn matches",
            "result.state as u64 ^ case.kind().mask()",
            "result.calls[5] += 1",
            "magnitude > infinity",
            "matches(kind, bits, exits.flat.permit)",
            "matches(kind, bits, exits.seal)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-dispatch",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.field_exits()",
            "super::state_field_exits::evaluate(",
            "current,",
            "Err(()) => return Err(trace)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-raw-observers",
        path: "tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print_cases.rs",
        required_patterns: &[
            "OBSERVERS_F32",
            "OBSERVERS_F64",
            "bitcast float %arg0 to i32",
            "zext i32 %field_bits to i64",
            "bitcast double %arg0 to i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print.rs",
        required_patterns: &[
            "state_field_exit_argument_print_native_preserves_ieee_operand_faults",
            "state_field_exit_argument_print_native_keeps_state_driven_budgets",
            "state_field_exit_argument_print_native_charges_outlined_classifier_entries",
            "state_field_exit_argument_print_oracle_pins_ieee_schedule_and_fault_order",
            "[7, 2, 6, 4, 1, 2]",
            "row.words().len(), 25",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_field_exit_argument_print_fresh_cache_and_source_free_delivery_use_updated_fields",
            "cases::STATE_FIELD_EXIT",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_field_exit_argument_print;",
            "pub(super) const STATE_FIELD_EXIT_ARGUMENT_PRINT",
            "state_field_exit_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-receipt",
        path: "docs/reference/nuis-native-state-field-exit-argument-print-v1.md",
        required_patterns: &[
            "Updated Field Exit Conditions",
            "42 raw sequences",
            "twelve finite sequences",
            "22 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-field-exit-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded field-exit witnesses:",
            "Previous state-grouped checkpoint:",
            "Previous state-grouped task checkpoint:",
            "Previous state-grouped blocker checkpoint:",
            "Previous state-grouped action checkpoint:",
            "Previous state-grouped validation checkpoint:",
            "Previous state-grouped artifact checkpoint:",
        ],
    },
];
