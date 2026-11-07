use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_parameter_exit_argument_print.ns",
        required_patterns: &[
            "continue_mode_value(permit, continue_mode_fault",
            "continue_guard_value(confirm_guard - (outer * 16 + child), continue_guard_fault",
            "break_mode_value(seal, break_mode_fault",
            "break_guard_value(seal_guard - (outer * 16 + child), break_guard_fault",
            "let update_check = 0 / (ready_guard - (outer * 16 + child));",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_parameter_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_parameter_exit_argument_print_accepts_ordered_later_arguments",
            "state_parameter_exit_argument_print_keeps_exact_parameter_types_and_epochs",
            ".replace(exit, \"\")",
            "continue_mode_value(current.flag,",
            "break_mode_value(current.total,",
            "return value == 0;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-model",
        path: "tools/nuisc/tests/native_application_bridge/state_parameter_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn call_counts",
            "pub fn evaluate",
            "quotient(case.kind(), bits, divisors[slot]).ok_or(())?",
            "[target, remaining].into_iter().enumerate()",
            "parameter_faults[slot * 2 + position] - offset",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-dispatch",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.parameter_faults().is_some()",
            "super::state_parameter_exits::evaluate(",
            "GuardClock::StateParameterExit { guard, .. }",
            "Err(()) => return Err(trace)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-input-transport",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "pub const STATE_PARAMETER_EXIT",
            "pub fn parameter_faults",
            "parameter_faults: [i64; 4]",
            "state_parameter_exits::helper(self).words()",
            "words.extend(faults)",
            "state_parameter_exits::helper(self).args()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-observers",
        path: "tools/nuisc/tests/native_application_bridge/state_parameter_exit_argument_print_cases.rs",
        required_patterns: &[
            "parameter!(\"continue_mode_value\", 613)",
            "parameter!(\"continue_guard_value\", 614)",
            "parameter!(\"break_mode_value\", 615)",
            "parameter!(\"break_guard_value\", 616)",
            "observers!(OBSERVERS_F32)",
            "observers!(OBSERVERS_F64)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_parameter_exit_argument_print.rs",
        required_patterns: &[
            "state_parameter_exit_argument_print_native_reorders_parameters_and_fault_priority",
            "state_parameter_exit_argument_print_native_consumes_full_width_parameter_returns",
            "state_parameter_exit_argument_print_native_preserves_reservations_and_entry_limits",
            "state_parameter_exit_argument_print_native_requires_each_parameter_effect_grant",
            "parameter print inherited a caller grant",
            "row.words().len(), 35",
            "source.replace(inline_update, wrapped_update)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_parameter_exit_argument_print_fresh_cache_and_source_free_delivery_preserve_arguments",
            "cases::STATE_PARAMETER_EXIT",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_parameter_exit_argument_print;",
            "pub(super) const STATE_PARAMETER_EXIT_ARGUMENT_PRINT",
            "state_parameter_exit_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-receipt",
        path: "docs/reference/nuis-native-state-parameter-exit-argument-print-v1.md",
        required_patterns: &[
            "Ordered Later Confirmation Arguments",
            "31 raw sequences",
            "eight finite sequences",
            "34 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded parameter-exit witnesses:",
            "Previous state-helper-exit checkpoint:",
            "Previous state-helper-exit task checkpoint:",
            "Previous state-helper-exit blocker checkpoint:",
            "Previous state-helper-exit action checkpoint:",
            "Previous state-helper-exit validation checkpoint:",
            "Previous state-helper-exit artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-parameter-exit-argument-print-lineage-regression",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_parameter_exit_retains_complete_helper_exit_lineage",
            "2d873db1233bf9d11e1c4dd382324f202bcc7d311fc0885c6721de7f846df1c3",
            "7d387bb7117daf185d5b4627b9b57e4b3decc71ef777a8db27f3c58571552de6",
            "38797a0a7b107bb5a38a5eac8a388d26faad84c7c0b5893ee73c7fcca0ad56d0",
            "18680433a47202e69b559d368fa950dc1fab4835f1e30a478acbbf3b4394da1f",
            "6cd903b22989f87f96421f9481c8388bdd85d181b4cec1ab29b58df93630dfe4",
            "b0b5fb87120e78604158c7aaf4b849195db4aafcf659e31af55ada211c2d07c3",
        ],
    },
];
