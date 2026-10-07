use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-static-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod staged_return_effects;", "staged_return_effects::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-original-entry-and-budget-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["!entry::admitted(condition, scope, catalog, layouts)", "if !preflight(body, end)", "!suffix::reserve_staged_prefix(&body[end..], &body[..end])", "let mut bindings = bindings.clone();"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-original-scope-and-scalar-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["scope.contains_key(name)", "control_values::value_type(value, scope, catalog, layouts)", "!scalar(&ty) || declared.is_some_and(|declared| declared != &ty)", "!ready.get(name).is_some_and(scalar)", "suffix::validate(&body[end..], scope, result, catalog, layouts)", "aliases::rewrite(&mut tail, &aliases)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-ordered-selected-once-only-snapshots",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["*bindings = plan.bindings;", "let live = install_steps(", "arm.steps,", "for step in steps", "__nuis_conditional_prefix_value", "initializer.return_type = Some(ty.clone())", "then_body: vec![NirStmt::Return(Some(value))]", "else_body: vec![NirStmt::Return(Some(seed))]", "print_values::install(", "plan.pure.condition = NirExpr::Var(plan.gate);"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-independent-source-exit-and-work-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns.rs",
        required_patterns: &["effects::regions::prepare(", "effects::regions::install(", "if !has_return", "&& !selected_initializer_work", "Ordinary print arguments still grant no tail eligibility."],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-runtime-order-unused-work-and-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions_tests.rs",
        required_patterns: &["assert_eq!(cases, 616)", "assert_eq!(cases, 36)", "let invocations = events", "position < prints[1]", "assert_eq!(module, once)", "assert_eq!(module, before, \"{mutation}\")", "entry-private", "effectful-callee", "unused-checked"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-original-expanded-and-root-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions_budget_tests.rs",
        required_patterns: &["[29, 30]", "[26, 27]", "for args in [4092, 4093]", "for depth in [63, 64]", "before cloning or recursive inference", "assert_eq!(module, before)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-default-source-aot-order-and-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions_native_tests.rs",
        required_patterns: &["conditional_return_staged_initializers_execute_native_order_unused_work_and_traps", "staged_expected(kind, mode, entry, shape, INPUTS[index], stamp)", "simple_expected(kind, outer, stamp)", "run(&text, result.map(|_| output.as_str()))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-independent-receipt-and-boundaries",
        path: "docs/reference/nuis-native-staged-return-effects-v1.md",
        required_patterns: &["616 cases", "1232 ordinary/reversed YIR executions", "Eighteen AOT variants", "Five selected failures", "Ordinary computed print arguments still grant", "no tail eligibility", "Staged initializer verification:", "`98ca713e`", "`active/97`"],
    },
    DevTensorDriftCheckSpec {
        id: "native-staged-return-effects-tensor-preserves-prior-alias-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Staged return-effect initializer follow-up:", "Previous interleaved atom-alias checkpoint:", "Interleaved return-print alias acceptance:", "Final selected acceptance passes 2087 distinct tests", "conditional_return_staged_initializers"],
    },
];
