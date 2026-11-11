use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print.ns",
        required_patterns: &[
            "let next = negative_update(value, negative_helper_fault); if next != 0 { return negative_finish(next, negative_finish_fault)",
            "let next = nonnegative_update(value, nonnegative_helper_fault); if next != 0 { return nonnegative_finish(next, nonnegative_finish_fault)",
            "return (value - 3) ^ 4294967296",
            "return value ^ 4294967296",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print_cases.rs",
        required_patterns: &[
            "execute_then(prior, native, 0,",
            "let input = returned ^ tag",
            "if input == 0",
            "pub fn zero_gate_failures",
            "trace.extend([if negative { 620 } else { 621 }, input, guard])",
            "trace.push(if negative { 88 } else { 94 })",
            "Ok(input ^ tag)",
            "pub fn delivery_failures",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_gate_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_return_gate_exit_accepts_return_selected_effects_without_pure_promotion",
            "state_return_gate_exit_keeps_scope_types_and_original_readiness",
            "!pure.contains_key(name)",
            "effects.contains_key(name)",
            "negative_finish(false, negative_finish_fault)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print.rs",
        required_patterns: &[
            "state_return_gate_exit_consumes_tagged_returns_and_latches_the_branch",
            "state_return_gate_exit_pins_return_flow_and_first_failure",
            "state_return_gate_exit_skips_zero_returns_but_preserves_earlier_checks",
            "state_return_gate_exit_reference_keeps_complete_accepted_state",
            "(i64::MIN, \"(-9223372036854775807 - 1)\")",
            "bad[42] = Value::Bool(false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-effects",
        path: "tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print.rs",
        required_patterns: &[
            "state_return_gate_exit_requires_each_leaf_effect_site_grant",
            "assert_eq!(own.len(), 3",
            "\"negative_finish\"",
            "\"nonnegative_finish\"",
            "[43, 45, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print.rs",
        required_patterns: &[
            "state_return_gate_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
            "&[90, 70]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_gate_exit_argument_print.rs",
        required_patterns: &[
            "state_return_gate_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "cases::delivery_failure_sequences(kind)",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-receipt",
        path: "docs/reference/nuis-native-state-return-gate-exit-argument-print-v1.md",
        required_patterns: &[
            "Return-Controlled Calls",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-return-gate-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_return_gate_exit_retains_complete_chain_lineage",
            "Previous state-branch-chain-exit checkpoint",
            "NUIS_STATE_RETURN_GATE_EXIT_20261111_CHAIN_HISTORY",
        ],
    },
];
