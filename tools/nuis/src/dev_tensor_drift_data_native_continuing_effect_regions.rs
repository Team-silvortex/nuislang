use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-static-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod continuing_effect_regions;", "continuing_effect_regions::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-shared-original-and-expanded-ledger",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_bounds.rs",
        required_patterns: &["statements.checked_sub(1)?", "statements.checked_sub(body.len())", "computed_expression_roots(expressions)", "NirStmt::Print(value) if prefix => expressions.push((value, 0, false))", "NirStmt::Return(Some(value)) => expressions.push((value, 0, true))", "expressions.push((condition, 0, true))", "_ => return false"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-original-condition-and-scalar-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["let yes_end = region_end(then_body)?", "if !preflight(body, end)", "!suffix::reserve_staged_prefix(&body[end..], &body[..end])", "if !entry::admitted(condition, scope, catalog, layouts)", "*has_initializer_work |= entry::has_work(condition)", "!ready.get(name).is_some_and(scalar)", "__nuis_effect_condition"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-no-child-scope-join-or-private-source-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["let mut yes_scope = scope.clone()", "let mut yes_values = values.clone()", "let mut no_scope = scope.clone()", "let mut no_values = values.clone()", "scope.contains_key(name)", "control_values::value_type(value, scope, catalog, layouts)", "suffix::validate(&body[end..], scope, result, catalog, layouts)", "aliases::rewrite(&mut tail, &aliases)", "_ => return None"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-saved-parent-and-child-paths",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["let live = install_steps(", "arm.steps,", "for step in steps", "condition: slot", "name: slot.clone()", "ty: scalar_type(\"bool\")", "for selected in [true, false]", "__nuis_effect_path", "lhs: Box::new(condition.clone())", "yes, &gates[0]", "no, &gates[1]", "plan.pure.condition = NirExpr::Var(plan.gate)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-independent-runtime-and-atomic-veto",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_control_tests.rs",
        required_patterns: &["assert_eq!(cases, 432)", "expected(kind, nested, mode, shape, index % 3, input)", "empty_condition", "let ignored = observe(tail)", "condition-private", "internal-return", "print-only", "assert_eq!(module, before, \"{mutation}\")", "assert_eq!(module, once)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-child-and-tail-budget-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_control_budget_tests.rs",
        required_patterns: &["[29, 30]", "[26, 27]", "for edges in [16, 17]", "for edges in [10, 11]", "for args in [4093, 4094]", "for depth in [63, 64]", "effects::regions::bounds::preflight", "assert_eq!(module, before)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-default-source-aot-order-and-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_control_native_tests.rs",
        required_patterns: &["conditional_return_control_regions_execute_default_aot_order_and_selected_traps", "expected(kind, nested, mode, shape, position, INPUTS[index])", "condition_only", "let ignored = observe(tail)", "let after = observe(tail)", "run(&text, result.map(|_| output.as_str()))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-proof-and-honest-boundaries",
        path: "docs/reference/nuis-native-continuing-effect-regions-v1.md",
        required_patterns: &["Continuing control-region verification:", "432 core cases", "884 ordinary/reversed semantic executions", "Sixteen default-source AOT variants", "Four selected failures", "partial stdout", "fully continuing", "`active/99`", "historical"],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuing-effect-regions-tensor-retains-prior-checkpoints",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Continuing control-region follow-up:", "Previous logical staged-initializer checkpoint:", "Logical staged-initializer follow-up:", "2101 distinct tests", "conditional_return_control_regions"],
    },
];
