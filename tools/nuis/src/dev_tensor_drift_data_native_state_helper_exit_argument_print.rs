use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_helper_exit_argument_print.ns",
        required_patterns: &[
            "continue_lhs(current.total, continue_lhs_guard",
            "continue_rhs(continue_divisor, continue_rhs_guard",
            "break_lhs(current.total, break_lhs_guard",
            "break_rhs(break_divisor, break_rhs_guard",
            "let other = -value; if pick { return value; } return other;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_helper_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_helper_exit_argument_print_accepts_ordered_checked_producers",
            "state_helper_exit_argument_print_keeps_types_and_original_epochs",
            ".replace(exit, \"\")",
            "continue_lhs(current.flag,",
            "break_rhs(current.code,",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-model",
        path: "tools/nuisc/tests/native_application_bridge/state_helper_exit_argument_print_cases.rs",
        required_patterns: &[
            "pub fn call_counts",
            "pub fn evaluate",
            "[bits, divisors[slot]].into_iter().enumerate()",
            "operand_guards[slot * 2 + side] - offset",
            "quotient(case.kind(), bits, divisors[slot]).ok_or(())?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-dispatch",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.helper_operands().is_some()",
            "super::state_helper_exits::evaluate(",
            "GuardClock::StateHelperExit { guard, .. }",
            "Err(()) => return Err(trace)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-input-transport",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "pub const STATE_HELPER_EXIT",
            "pub fn helper_operands",
            "operand_guards: [i64; 4]",
            "state_helper_exits::division(self).words()",
            "words.extend(guards)",
            "state_helper_exits::division(self).args()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-observers",
        path: "tools/nuisc/tests/native_application_bridge/state_helper_exit_argument_print_cases.rs",
        required_patterns: &[
            "producer32!(\"continue_lhs\", 609)",
            "producer32!(\"break_rhs\", 612)",
            "producer64!(\"continue_rhs\", 610)",
            "producer64!(\"break_lhs\", 611)",
            "bitcast float %arg0 to i32",
            "bitcast double %arg0 to i64",
            "state_division_exits::OBSERVERS_F32[4]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_helper_exit_argument_print.rs",
        required_patterns: &[
            "state_helper_exit_argument_print_native_reverses_producer_order_and_fault_priority",
            "state_helper_exit_argument_print_native_consumes_producer_return_values",
            "state_helper_exit_argument_print_native_keeps_work_and_entry_limits",
            "state_helper_exit_argument_print_native_requires_explicit_producer_print_grants",
            "producer print inherited a caller grant",
            "row.words().len(), 31",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_helper_exit_argument_print_fresh_cache_and_source_free_delivery_preserve_producers",
            "cases::STATE_HELPER_EXIT",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_helper_exit_argument_print;",
            "pub(super) const STATE_HELPER_EXIT_ARGUMENT_PRINT",
            "state_helper_exit_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-receipt",
        path: "docs/reference/nuis-native-state-helper-exit-argument-print-v1.md",
        required_patterns: &[
            "Ordered Checked Producer Operands",
            "28 raw sequences",
            "eight finite sequences",
            "24 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded helper-exit witnesses:",
            "Previous state-division-exit checkpoint:",
            "Previous state-division-exit task checkpoint:",
            "Previous state-division-exit blocker checkpoint:",
            "Previous state-division-exit action checkpoint:",
            "Previous state-division-exit validation checkpoint:",
            "Previous state-division-exit artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-helper-exit-argument-print-lineage-regression",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_helper_exit_retains_complete_division_exit_lineage",
            "ed49db10cdfcb14135c9bdc51e5f56a3d4eb3647a0a0fba020782f4a1cd0218f",
            "1c889bcf26f65d6093d6db15d261a319bb0a05715e6bebfed0a10c6232972573",
            "14c19f1dc8c0e2f5c2b4eadc35084726c70f012fe9d7c51a33b06e2dd1198e6c",
            "176cfcfd297b3acf2c55bb39d88112ce7bdc39e10b3cdf7ec88c52c42b884ca8",
            "b343c2c3c5a1de1495a8142200e85e6ee83654402372dcc510369660c2e56e56",
            "0db81cf97516cb09a1e9855c1cc8db9eeec929dd088aab2540605ba6261cf447",
        ],
    },
];
