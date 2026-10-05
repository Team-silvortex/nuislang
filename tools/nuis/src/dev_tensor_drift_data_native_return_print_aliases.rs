use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-flat-static-registration-with-stable-order",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &[
            "Keep registration order without growing the iterator type",
            "return_print_aliases::CHECKS,", "computed_return_prints::CHECKS,",
            "caller_records::CHECKS,", "caller_spills::CHECKS,",
            ".into_iter()", ".flatten()",
            "dev_tensor_drift_native_iteration_registration_tests.rs",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-original-preflight-and-full-prefix-reservation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_print_aliases.rs",
        required_patterns: &[
            ".rposition(|stmt| matches!(stmt, NirStmt::Print(_)))",
            "if !preflight(body, end)", "!suffix::reserve_prefix(&body[end..], &roots)",
            "Every removed alias still consumes its original statement and node",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-total-exact-original-scope-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_print_aliases.rs",
        required_patterns: &[
            "scope.contains_key(name)", "NirExpr::Var(_) | NirExpr::Int(_) | NirExpr::Bool(_)",
            "control_values::value_type(value, &scope, catalog, layouts)",
            "!scalar(&ty) || declared.is_some_and(|declared| declared != &ty)",
            "print_values::prepare(value, &scope, catalog, layouts)",
            "suffix::validate(&body[end..], scope, result, catalog, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-nonexpanding-two-arm-atomic-rewrite",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_print_aliases.rs",
        required_patterns: &[
            "let yes = prepare_arm(then_body", "let no = prepare_arm(else_body",
            "Some((materialize(yes), materialize(no)))",
            "aliases.get(name).copied().unwrap_or(value)",
            ".filter(|stmt| matches!(stmt, NirStmt::Print(_)))",
            "while let Some(expr) = expressions.pop()", "*expr = (*atom).clone()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-reuse-independent-effects-and-tail-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns.rs",
        required_patterns: &[
            "effects::aliases::prepare(",
            "condition, &yes, &no, result, &scope, catalog, layouts, &checked",
            "output.extend(effects::install(",
            "if !has_return", "!entry::has_work(condition)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-runtime-scopes-captures-and-atomic-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_print_aliases_tests.rs",
        required_patterns: &[
            "assert_eq!(cases, 616)", "assert_eq!(module, once)",
            "assert_eq!(module, before, \"{mutation}\")", "prefix_stamp", "vec![\"stamp\"]",
            "tail-shadow", "branch-shadow", "print-before-alias", "print-only-work",
            "let stamp: i64 = stamp + 2;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-elision-does-not-buy-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_print_aliases_budget_tests.rs",
        required_patterns: &[
            "for count in [30, 31]", "for count in [27, 28]",
            "for args in [4092, 4093]", "preflight(&body, 3), args == 4092",
            "suffix::reserve_prefix(&tail, &[&atom, &value, &atom])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-default-source-aot-selected-work-and-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_print_aliases_native_tests.rs",
        required_patterns: &[
            "conditional_return_print_aliases_execute_native_selected_work_traps_and_zero_exits",
            "alias_source(kind, mode, entry, shape, input, stamp)",
            "alias_expected(kind, mode, entry, shape, input, stamp)",
            "run(&text, result.map(|_| output.as_str()))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-independent-receipt-and-boundaries",
        path: "docs/reference/nuis-native-return-print-aliases-v1.md",
        required_patterns: &[
            "616 cases", "1232 ordinary/reversed YIR executions", "Sixteen AOT variants",
            "Seven selected traps", "Elision cannot buy admission budget",
            "print-only work grants no tail eligibility", "Interleaved alias verification",
            "after", "`98ca713e`", "active/96",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-print-aliases-tensor-preserves-prior-computed-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Interleaved return-print alias follow-up:",
            "Previous computed print-argument checkpoint:",
            "Computed return-print argument acceptance:", "Final selected acceptance passes 2079 distinct tests",
            "Previous leading print-prefix checkpoint:", "conditional_return_print_aliases",
        ],
    },
];
