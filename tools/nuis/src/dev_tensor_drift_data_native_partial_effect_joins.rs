use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod partial_effect_joins;", "effect_result_joins::CHECKS,", "partial_effect_joins::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-source-continuation-and-paired-final-presence",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["pub continues: bool", "if !yes.continues && !no.continues", "destination(continuing.body)?", "destination(arm.body)? != (target, constant)", "scope.contains_key(target)", "arm.scope.get(target)? != ty", "let yes = value(&yes)?", "let no = value(&no)?", "atom(value, ty, ready).then(|| Some(value.clone()))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-independently-typed-source-exits-and-merged-live-paths",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["if !continues", "!= Some(result)", "continues: yes_continues", "continues: no_continues", "Step::Join(joined) => joins::install(joined, &condition", "exits.push((condition.clone(), name))", "let early = has_exit(&yes) || has_exit(&no)", "lhs: Box::new(yes)", "rhs: Box::new(no)", "condition = NirExpr::Var(live)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-independent-imperative-oracle",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_partial_join_fixtures.rs",
        required_patterns: &["pub(super) fn expected", "if late || !stop", "if divisor == 0", "if computed_exit && input.tail == 0", "let value = 100 / divisor", "else if value > 0", "prints.extend([89, 77, 19])"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-result-presence-order-hygiene-and-atomic-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_partial_join_tests.rs",
        required_patterns: &["assert_eq!(sources.len(), 480)", "const selected:", "left: 1000", "Some(0), &[99, 50, 44, 0]", "__nuis_effect_live_0", "assert_eq!(module, once)", "\"missing\"", "\"unreachable\"", "\"wrong-exit\"", "\"forward\"", "assert_eq!(module, before, \"{mutation}\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-shared-original-statement-and-exit-node-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_partial_join_tests.rs",
        required_patterns: &["for copies in [15, 16]", "effects::regions::bounds::preflight(then_body, 2)", "4096 - roots(then_body) + 1", "for args in [limit, limit + 1]", "Bounds grant no type authority", "assert_eq!(module, before)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-default-source-aot-and-selected-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_partial_join_native_tests.rs",
        required_patterns: &["conditional_return_partial_effect_joins_execute_default_aot_presence_exits_and_selected_traps", "result.map(|_| output.as_str())", "const selected:", "left: 1000", "let unused = 10 / tail;", "run(&successful_main(&text), None)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-honest-proof-and-execution-boundaries",
        path: "docs/reference/nuis-native-partial-effect-result-joins-v1.md",
        required_patterns: &["Partial effect-result join verification:", "480 core sources", "489 distinct semantic sources", "978 ordinary/reversed executions", "Thirteen default-source AOT variants", "Three selected traps", "partial stdout", "`active/99`", "historical", "No fresh Linux/GPU"],
    },
    DevTensorDriftCheckSpec {
        id: "native-partial-effect-joins-tensor-preserves-prior-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Partial effect-result join follow-up:", "Previous paired-result join checkpoint:", "Paired effect-result join follow-up:", "149 distinct selected tests", "conditional_return_partial_effect_joins"],
    },
];
