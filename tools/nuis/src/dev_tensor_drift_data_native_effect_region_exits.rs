use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod effect_region_exits;", "effect_region_exits::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-original-result-and-reachability-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["NirStmt::Return(Some(value)) =>", "!expression(value, scope)", "!= Some(result)", "if !continues", "continues = false", "continues = yes_continues || no_continues", "if !continues && end < body.len()", "!ready.get(name).is_some_and(scalar)", "__nuis_effect_exit_value"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-distinct-exit-and-value-publication",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["Step::Exit {", "exits.push((condition.clone(), name))", "condition = NirExpr::Bool(false)", "let early = has_exit(&yes) || has_exit(&no)", "__nuis_effect_live", "op: NirBinaryOp::Or", "lhs: Box::new(yes)", "rhs: Box::new(no)", "for (condition, name) in exits", "then_body: vec![NirStmt::Return(Some(NirExpr::Var(name)))]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-guard-original-tail-not-synthetic-seeds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["super::super::prepare_staged(", "pure.yes = Some(continuing_tail(&yes", "pure.no = Some(continuing_tail(&no", "pure.stored_exit = true", "pure.readiness = None", "suffix::prepare(&arm.tail", "then_body: tail", "__nuis_effect_continuation", "value: live"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-independent-runtime-and-atomic-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_exit_tests.rs",
        required_patterns: &["assert_eq!(cases, 900)", "expected(kind, mode, deep, shape, input)", "__nuis_effect_exit_value_0", "print(43); return false;", "let ignored_exit = observe(tail)", "no-source-exit", "both-exit", "after-exit", "logical-leaf", "assert_eq!(module, before, \"{mutation}\")", "assert_eq!(module, once)", "conditional_return_effect_exits_compose_with_real_partial_tail_exits_and_continuation_checks", "fixtures::partial_expected(mode, index)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-original-and-expanded-return-root-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_exit_tests.rs",
        required_patterns: &["for edges in [16, 17]", "for args in [4092, 4093]", "for depth in [63, 64]", "for copies in [28, 29]", "effects::regions::bounds::preflight", "suffix::reserve_staged_prefix", "assert_eq!(module, before)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-default-source-aot-first-exit-and-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_exit_native_tests.rs",
        required_patterns: &["conditional_return_effect_exits_execute_default_aot_nested_returns_and_selected_traps", "expected(kind, mode, deep, shape, INPUTS[index])", "run(&text, result.map(|_| output.as_str()))", "print(43); return false;", "if nested { return false; } print(99);", "let ignored_exit = observe(tail)", "conditional_return_effect_exits_execute_default_aot_partial_tail_exits_and_continuation_checks", "fixtures::partial_source(mode, index)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-honest-documentation-boundaries",
        path: "docs/reference/nuis-native-effect-region-exits-v1.md",
        required_patterns: &["900 core sources", "912 distinct sources", "1824 ordinary/reversed", "Twenty-five default-source AOT variants", "Seven selected traps", "partial stdout", "tail source return", "`active/99`", "historical"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-region-exits-tensor-preserves-checkpoint-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Effect-region exit follow-up:", "Previous continuing control-region checkpoint:", "Continuing control-region follow-up:", "2108 distinct selected tests", "conditional_return_effect_exits"],
    },
];
