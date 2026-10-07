use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod effect_result_joins;", "exit_only_regions::CHECKS,", "effect_result_joins::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-fresh-paired-owned-scalars-only",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["if !yes.continues && !no.continues", "destination(continuing.body)?", "destination(arm.body)? != (target, constant)", "scope.contains_key(target)", "arm.scope.get(target)? != ty", "NirStmt::Let { name, .. } => Some((name, false))", "NirStmt::Const { name, .. } => Some((name, true))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-typed-staged-atoms-without-source-replay",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["let yes = value(&yes)?", "let no = value(&no)?", "atom(value, ty, ready).then(|| Some(value.clone()))", "ready.get(condition) != Some(&scalar_type(\"bool\"))", "NirExpr::Var(name) => ready.get(name) == Some(ty)", "NirExpr::Int(_) => ty == &scalar_type(\"i64\")", "NirExpr::Bool(_) => ty == &scalar_type(\"bool\")", "captured_params(inputs, ready)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-live-and-saved-selection-with-inert-seeds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["__nuis_join_live", "__nuis_effect_join_value", "let mut args = vec![live.clone()]", "condition: NirExpr::Var(gate)", "condition: NirExpr::Var(joined.condition)", "(Some(yes), Some(no))", "NirStmt::Return(Some(yes))", "NirStmt::Return(Some(no))", "NirStmt::Return(Some(seed))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-child-scope-isolation-and-independent-exit-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["mod joins;", "let mut yes_scope = scope.clone()", "let mut no_scope = scope.clone()", "continues: yes_continues", "continues: no_continues", "if let Some((name, joined)) = joined", "values.insert(name, NirExpr::Var(joined.name.clone()))", "Step::Join(joined) => joins::install(joined, &condition", "Step::Exit { .. } => true", "_ => false"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-independent-source-semantics",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_join_fixtures.rs",
        required_patterns: &["pub(super) fn expected", "let selected = match kind", "if selected { input.left } else { input.right }", "if divisor == 0", "if input.tail == 0", "prints.extend([89, 77, 19])"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-atomic-veto-order-hygiene-and-shared-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_join_tests.rs",
        required_patterns: &["assert_eq!(sources.len(), 192)", "expected(kind, word, early, input)", "const selected:", "Some(0), &[99, 0, 0, 44, 0]", "__nuis_effect_join_0", "assert_eq!(module, once)", "assert_eq!(module, before, \"{mutation}\")", "for copies in [21, 22]", "effects::regions::bounds::preflight(then_body, 2)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-default-source-aot-and-selected-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_join_native_tests.rs",
        required_patterns: &["conditional_return_effect_joins_execute_default_aot_nested_selection_and_traps", "run(&successful_main(&text), result.map(|_| output.as_str()))", "right: 1000", "const selected:", "print(result); return 0;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-frontend-const-result-without-child-leaks",
        path: "tools/nuisc/src/frontend/tests_control_flow/if_expressions.rs",
        required_patterns: &["const_control_expression_results_are_visible_only_after_validated_branches", "const chosen: i64 = {expression}", "return local;", "let local = 7; local\", \"chosen", "let local = 9; local\", \"true", "const chosen: i64\", \"const chosen"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-honest-proof-and-execution-boundaries",
        path: "docs/reference/nuis-native-effect-result-joins-v1.md",
        required_patterns: &["Paired effect-result join verification:", "203 distinct semantic sources", "406 ordinary/reversed executions", "Thirteen default-source AOT variants", "Three selected traps", "partial stdout", "`active/99`", "historical", "No fresh Linux/GPU"],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-result-joins-tensor-retains-all-prior-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Paired effect-result join follow-up:", "Previous exit-only region checkpoint:", "Exit-only region follow-up:", "180 distinct selected tests", "conditional_return_effect_joins"],
    },
];
