use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod typed_effect_returns;", "typed_effect_returns::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-exact-separate-policy",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_values.rs",
        required_patterns: &["ty == &scalar_type(&ty.name)", "\"bool\" | \"i64\" | \"i32\" | \"f32\" | \"f64\"", "pub(super) fn capture", "if scalar(result)", "scalar(ty)", "admitted(ty)", "pub(super) fn local", "control_values::zero_value(result", "conditional_return_typed_handoff_keeps_exact_owned_profile_and_legacy_capture_limits", "conditional_return_typed_handoff_keeps_real_zero_exits_distinct_from_continuation_seeds"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-selected-only-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns.rs",
        required_patterns: &["mod return_values;", ".filter(|ty| return_values::admitted(ty))", "if scalar(result)", "effects::regions::prepare(", "return_values::capture(result, ty)", "let seed = return_values::seed(result)", "signal::read(call, &returned, result, bindings)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-prefix-original-lexical-types",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_prefix.rs",
        required_patterns: &["conditional_values::prefix::return_body(body)", "inner.contains_key(name)", "return_values::local(result, &ty)", "declared.is_some_and(|declared| declared != &ty)", "return_values::capture(result, ty)", "inputs.retain(|name| !local_names.contains(name))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-terminal-shared-bounds-and-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_terminal.rs",
        required_patterns: &["bounded_tree(body, true)", "let mut statements = 32usize", "depth >= 64", "return_values::capture(result, ty)", "return_values::local(result, &ty)", "conditional_values::prefix::computed_expression_roots(expressions)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-independent-fallthrough-seeds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_fallthrough.rs",
        required_patterns: &["terminal::partial_inputs", "The seed is not an exit signal", "Some(NirStmt::Return(Some(_))) => {}", "body.push(NirStmt::Return(Some(return_values::seed(result))))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-stored-bool-flag-and-typed-data",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_signal.rs",
        required_patterns: &["(\"exited\", scalar_type(\"bool\")), (\"value\", result.clone())", "let seed = return_values::seed(result)", "body.push(returned(signal, false, seed))", "body.push(returned(signal, true, NirExpr::Var(name)))", "ty: Some(result.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-atomic-veto-and-original-ledger-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_typed_return_tests.rs",
        required_patterns: &["conditional_return_typed_handoff_outlines_exact_selected_regions_and_bool_exit_signals", "conditional_return_typed_handoff_does_not_admit_ordinary_tails_or_typed_print_captures", "conditional_return_typed_handoff_rejects_original_scope_type_effect_and_budget_violations", "conditional_return_typed_handoff_shares_original_and_staged_statement_ledgers", "for count in 0..=32", "assert_eq!(module, before", "assert_eq!(module, once)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-independent-source-and-native-oracles",
        path: "tools/nuisc/tests/native_application_bridge/typed_effect_returns.rs",
        required_patterns: &["result = selected", "1 << 31", "1 << 63", "0xff80_0001", "0xfff0_0000_0000_0001", "module.edges.reverse()", "ApplicationSession::open_registered", "yir_lower_llvm::emit_module(module)", "load volatile i64", "@nuis_fn_event", "write_and_link_with_source", "run_bounded", "emit_registered(&module, \"counter\").is_err()", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-scoped-proof-and-native-effect-gap",
        path: "docs/reference/nuis-native-typed-effect-returns-v1.md",
        required_patterns: &["Typed selected-return acceptance:", "1296", "3888", "ordinary AOT", "pure native scalar session bridge still rejects", "`active/99`", "No fresh Linux/GPU run"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-returns-complete-six-field-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Typed selected-return follow-up:", "Previous floating-sign checkpoint:", "Previous floating-sign task checkpoint:", "Previous floating-sign blocker checkpoint:", "Previous floating-sign action checkpoint:", "Previous floating-sign artifact checkpoint:", "conditional_return_typed_handoff", "native callback effect transport"],
    },
    DevTensorDriftCheckSpec {
        id: "native-selected-effects-do-not-widen-pure-tail-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_boundary_tests.rs",
        required_patterns: &["assert_selected_effect_not_pure", "nested prints must not become pure-tail authority", "nested prints must not become ordinary print-prefix authority", "effects::regions::prepare(", "conditional_return_effect_boundaries_keep_pure_vetoes_and_selected_print_semantics", "assert_eq!(cases, 64)", "conditional_return_effect_boundaries_execute_original_native_selected_prints_and_continuations", "native::run(&text, Some(&expected))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-selected-effect-boundary-regression-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_tests.rs",
        required_patterns: &["mod effect_boundaries;", "pub(super) use effect_boundaries::assert_selected_effect_not_pure;"],
    },
];
