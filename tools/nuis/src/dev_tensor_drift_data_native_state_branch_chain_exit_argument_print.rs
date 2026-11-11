use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-source",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print.ns",
        required_patterns: &[
            "let next = negative_update(value, negative_helper_fault); return negative_finish(next, negative_finish_fault)",
            "let next = nonnegative_update(value, nonnegative_helper_fault); return nonnegative_finish(next, nonnegative_finish_fault)",
            "return (value - 3) ^ 4294967296",
            "return value ^ 4294967296",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-model",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print_cases.rs",
        required_patterns: &[
            "execute_then(prior, native, 0,",
            "let input = returned ^ tag",
            "trace.extend([if negative { 620 } else { 621 }, input, guard])",
            "trace.push(if negative { 88 } else { 94 })",
            "Ok(input ^ tag)",
            "pub fn delivery_failures",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_branch_chain_exit_argument_print_tests.rs",
        required_patterns: &[
            "state_branch_chain_exit_accepts_ordered_effectful_calls_without_pure_promotion",
            "state_branch_chain_exit_keeps_scope_types_and_original_readiness",
            "!pure.contains_key(name)",
            "effects.contains_key(name)",
            "negative_finish(false, negative_finish_fault)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-native-reference",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_chain_exit_consumes_tagged_returns_and_latches_the_branch",
            "state_branch_chain_exit_pins_return_flow_and_first_failure",
            "state_branch_chain_exit_reference_keeps_complete_accepted_state",
            "(i64::MIN, \"(-9223372036854775807 - 1)\")",
            "bad[42] = Value::Bool(false)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-effects",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_chain_exit_requires_each_leaf_effect_site_grant",
            "assert_eq!(own.len(), 3",
            "\"negative_finish\"",
            "\"nonnegative_finish\"",
            "[43, 45, 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-limits",
        path: "tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_chain_exit_keeps_work_and_helper_entry_limits",
            "check_i64_helper_entry_limit_rejection",
            "check_i64_loop_work_limit_rejection",
            "&[90, 70]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_branch_chain_exit_argument_print.rs",
        required_patterns: &[
            "state_branch_chain_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words",
            "cases::delivery_failures(kind)",
            "compile_cache: hit",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-receipt",
        path: "docs/reference/nuis-native-state-branch-chain-exit-argument-print-v1.md",
        required_patterns: &[
            "Ordered Selected Calls",
            "Predecessor Lineage",
            "192",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-branch-chain-exit-lineage",
        path: "tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_state_branch_chain_exit_retains_complete_helper_lineage",
            "Previous state-branch-helper-exit checkpoint",
            "NUIS_STATE_BRANCH_CHAIN_EXIT_20261111_HELPER_HISTORY",
        ],
    },
];
