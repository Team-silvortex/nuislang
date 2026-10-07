use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod exit_only_regions;", "effect_region_exits::CHECKS,", "exit_only_regions::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-proven-source-exits-and-continuation-tails",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_tails.rs",
        required_patterns: &["yes.continuation.is_none() && no.continuation.is_none()", "signal::prepare(&body", "if proof.has_return", "inputs.extend(proof.inputs)", "arm.has_initializer_work", "block_has_checked_arithmetic", "contains_calls(body)", "captured_params(inputs, scope)", "stored_exit: true"],
    },
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-integration-retains-original-tail-masks",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["mod tails;", ".or_else(||", "tails::prepare(", "super::super::prepare_staged(", "suffix::validate(&body[end..], scope", "pure.yes = Some(continuing_tail(&yes", "pure.no = Some(continuing_tail(&no", "has_exit(&steps).then(||"],
    },
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-independent-continuation-oracle",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_exit_fixtures.rs",
        required_patterns: &["pub(super) fn exit_only_source", "pub(super) fn exit_only_expected", "evaluate(kind, mode, deep, shape, input, true)", "if exit_only", "prints.push(77)", "if input.tail == 0", "prints.push(89)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-independent-semantics-atomicity-and-shared-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_exit_only_tests.rs",
        required_patterns: &["assert_eq!(cases, 900)", "expected(kind, mode, deep, shape, input)", "all_exiting", "__nuis_effect_continuation_0", "tail - 2", "no-source-exit", "tail-return", "branch-local", "assert_eq!(module, before", "assert_eq!(module, once)", "for args in [4093, 4094]", "for copies in [29, 30]", "suffix::reserve_staged_prefix(&[], &prefix)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-default-source-aot-and-selected-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_exit_only_native_tests.rs",
        required_patterns: &["conditional_return_exit_only_execute_default_aot_continuations_first_exits_and_traps", "run(&successful_main(&text), result.map(|_| output.as_str()))", "all_exiting(mode)", "tail - 2", "run(&successful_main(&base), None)", "print(result); return 0;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-honest-proof-and-acceptance-boundaries",
        path: "docs/reference/nuis-native-exit-only-regions-v1.md",
        required_patterns: &["900 core sources", "Seventeen default-source AOT variants", "Five selected traps", "partial stdout", "`active/99`", "historical", "source exit", "No fresh Linux/GPU"],
    },
    DevTensorDriftCheckSpec {
        id: "native-exit-only-regions-tensor-preserves-earlier-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Exit-only region follow-up:", "Previous effect-region exit checkpoint:", "Effect-region exit follow-up:", "284 distinct selected tests", "conditional_return_exit_only"],
    },
];
