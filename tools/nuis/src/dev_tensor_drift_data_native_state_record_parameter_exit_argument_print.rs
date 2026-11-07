use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_record_parameter_exit_argument_print.ns",
        required_patterns: &[
            "continue_mode_value(current.total, permit,",
            "continue_guard_value(current.total, confirm_guard",
            "break_mode_value(current.total, seal,",
            "break_guard_value(current.total, seal_guard",
            "let alternate = base ^ 1; if value < 0.0",
            "let alternate = base - 1; if value < 0.0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_record_parameter_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_record_parameter_exit_argument_print_accepts_updated_state_parameters",
            "state_record_parameter_exit_argument_print_keeps_field_types_and_original_epochs",
            ".replace(exit, \"\")",
            "current.flag",
            "current.code",
            "argument 1 expects `{kind}`",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-model",
        path: "tools/nuisc/tests/native_application_bridge/state_record_parameter_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn mode",
            "pub fn guard",
            "base.wrapping_sub(",
            "check.wrapping_sub(offset)",
            "pub fn plan",
            "target = mode(case.kind(), bits, target)",
            "remaining = guard(case.kind(), bits, remaining)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-dispatch",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.has_record_parameters()",
            "super::state_record_parameters::evaluate(",
            "GuardClock::StateRecordParameterExit { guard, .. }",
            "Err(()) => return Err(trace)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-input-transport",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "pub const STATE_RECORD_PARAMETER_EXIT",
            "pub fn has_record_parameters",
            "state_record_parameters::plan(self, previous).requested",
            "state_record_parameters::plan(self, previous).completed",
            "state_record_parameters::parameter(self).words()",
            "state_record_parameters::parameter(self).args()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-observers",
        path: "tools/nuisc/tests/native_application_bridge/state_record_parameter_exit_argument_print_cases.rs",
        required_patterns: &[
            "bitcast float %arg0 to i32",
            "bitcast double %arg0 to i64",
            "call void @word_print(i64 %arg1)",
            "call void @word_print(i64 %arg2)",
            "observers!(OBSERVERS_F32, field32)",
            "observers!(OBSERVERS_F64, field64)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_record_parameter_exit_argument_print.rs",
        required_patterns: &[
            "state_record_parameter_exit_argument_print_native_distinguishes_stale_parent_fields",
            "state_record_parameter_exit_argument_print_native_consumes_derived_full_width_returns",
            "state_record_parameter_exit_argument_print_native_preserves_state_driven_budgets",
            "state_record_parameter_exit_argument_print_native_keeps_individual_effect_grants",
            "record parameter print inherited a caller grant",
            "row.words(), parameter(row).words()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_record_parameter_exit_argument_print_fresh_cache_and_source_free_delivery_use_state",
            "cases::STATE_RECORD_PARAMETER_EXIT",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_record_parameter_exit_argument_print;",
            "pub(super) const STATE_RECORD_PARAMETER_EXIT_ARGUMENT_PRINT",
            "state_record_parameter_exit_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-receipt",
        path: "docs/reference/nuis-native-state-record-parameter-exit-argument-print-v1.md",
        required_patterns: &[
            "Updated Record Parameter Values",
            "34 raw sequences",
            "ten finite sequences",
            "34 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded record-parameter witnesses:",
            "Previous state-parameter-exit checkpoint:",
            "Previous state-parameter-exit task checkpoint:",
            "Previous state-parameter-exit blocker checkpoint:",
            "Previous state-parameter-exit action checkpoint:",
            "Previous state-parameter-exit validation checkpoint:",
            "Previous state-parameter-exit artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-record-parameter-exit-argument-print-lineage-regression",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_record_parameter_exit_retains_complete_parameter_lineage",
            "db09b92bb1e678db5b62a924ed9d57fda653257f2e7a912e3d6bb2b47faf4998",
            "5001213779cb55d5f56a89b19cff6a00e44c37df255a20c97edabf05ec1b2d39",
            "e146d17c077a272b15730f280091ae11bdfa5bdbb6babc564c301d74105d0888",
            "ea6bf29f4aca0115b275be2224d474265d3665cf3db646ee7b6d1cd930ae48cc",
            "187b1452289be1ca7dab7ecf5a76c04cf2e688379f810ba01209acf65836350e",
            "2197dc2e11d723d79e10d23cb41f531d2ada9b4cb2e6788a06aed77d5517c711",
        ],
    },
];
