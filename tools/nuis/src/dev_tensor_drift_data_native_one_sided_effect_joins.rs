use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod one_sided_effect_joins;", "partial_effect_joins::CHECKS,", "one_sided_effect_joins::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-parser-separates-expression-tails-and-function-exits",
        path: "tools/nuisc/src/frontend/parser_blocks.rs",
        required_patterns: &["fn parse_control_expr_block", "self.parse_block_body(true, false)?", "self.parse_block_body(true, true)?", "tail_returns: bool", "AstStmt::Return(Some(expr))", "AstStmt::Expr(expr)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-if-and-if-let-expression-tail-provenance",
        path: "tools/nuisc/src/frontend/parser_exprs.rs",
        required_patterns: &["let then_body = self.parse_control_expr_block()?", "vec![AstStmt::Expr(self.parse_if_expr_after_keyword()?)]", "fn parse_if_let_expr_after_keyword"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-match-expression-tail-provenance",
        path: "tools/nuisc/src/frontend/parser_statements.rs",
        required_patterns: &["self.parse_control_expr_block()?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-rewrite-preserves-explicit-source-return",
        path: "tools/nuisc/src/frontend/stmt_lowering_control_rewrite.rs",
        required_patterns: &["pub(super) fn rewrite_control_expr_terminal_branch", "AstStmt::Expr(value) =>", "rewritten.push(wrap(value.clone()))", "AstStmt::Return(_) | AstStmt::Break | AstStmt::Continue", "rewritten.push(last.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-frontend-presence-without-forward-or-child-read",
        path: "tools/nuisc/src/frontend/tests_control_flow/if_expressions.rs",
        required_patterns: &["control_expression_initializers_preserve_explicit_function_exits_and_tail_values", "control_expression_let_results_are_not_published_before_branch_validation", "control_expression_generic_tail_hints_do_not_leak_to_non_tail_calls", "Return(Some(Int(7)))", "Return(Some(Bool(false)))", "return local;", "match chosen", "zero(); zero()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-generic-result-context-without-non-tail-authority",
        path: "tools/nuisc/src/frontend/generic_rewrite/blocks.rs",
        required_patterns: &["fn rewrite_generic_calls_in_result_block", "fn rewrite_generic_calls_in_result_match_arms", "if result_context && index + 1 == body.len()", "rewritten.push(AstStmt::Expr(rewrite_generic_calls_in_expr", "ExpectedBlockType", "result_context: true", "result_context: false"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-preserves-generic-tail-argument-hoists",
        path: "tools/nuisc/src/frontend/generic_rewrite/blocks_hoists.rs",
        required_patterns: &["pub(super) result_context: bool", "matches!(stmt, AstStmt::Return(_)) || result_context", "hoist_direct_result_wrapper_args", "hoisted.push(if result_context", "AstStmt::Expr(rewritten_value)", "AstStmt::Return(Some(rewritten_value))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-preserves-lambda-result-tail-expected-types",
        path: "tools/nuisc/src/frontend/lambda_expansion_block.rs",
        required_patterns: &["fn expand_lambda_result_block", "expand_lambda_block_inner(input, false)", "expand_lambda_block_inner(input, true)", "result_context && index + 1 == body.len()", "rewrite_block_expr!(expr, expected)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-preserves-tail-result-type-inference",
        path: "tools/nuisc/src/frontend/types/ast_infer.rs",
        required_patterns: &["fn infer_ast_block_result_type", "Some(AstStmt::Return(Some(expr))) | Some(AstStmt::Expr(expr))", "infer_ast_expr_type_inner"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-optional-values-require-validated-source-continuation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["yes: Option<NirExpr>", "no: Option<NirExpr>", "if !yes.continues && !no.continues", "if !arm.continues", "return Some(None)", "destination(arm.body)? != (target, constant)", "arm.scope.get(target)? != ty", "atom(value, ty, ready).then(|| Some(value.clone()))", "(Some(value), None) | (None, Some(value))", "condition: NirExpr::Var(gate)", "then_body: vec![selected]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-source-oracle-quotient-exits-and-zero-result",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_one_sided_join_fixtures.rs",
        required_patterns: &["pub(super) fn expected", "if input.gate == swapped", "if deep && !input.nested", "if partial && input.stop", "if input.tail == 0", "if input.value == 0", "let value = 100 / input.value", "else if value > 0"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-matrix-hygiene-atomicity-and-complete-work-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_one_sided_join_tests.rs",
        required_patterns: &["assert_eq!(sources.len(), 384)", "const selected:", "value: 1000", "__nuis_effect_live_0", "assert_eq!(module, once)", "\"continuing-other\"", "\"both-exit\"", "\"wrong-exit\"", "assert_eq!(module, before, \"{mutation}\")", "for copies in [23, 24]", "let checked = 10 / value;", "Some(0), &[99, 50, 45, 0]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-default-source-aot-selected-traps-and-const-zero",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_one_sided_join_native_tests.rs",
        required_patterns: &["conditional_return_one_sided_effect_joins_execute_default_source_aot_and_selected_traps", "result.map(|_| output.as_str())", "const selected:", "Some(\"99\\n0\\n19\\n19\")", "print(result); return 0;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-honest-proof-and-execution-limits",
        path: "docs/reference/nuis-native-one-sided-effect-result-joins-v1.md",
        required_patterns: &["One-sided effect-result join verification:", "384 core sources", "392 distinct semantic sources", "784 ordinary/reversed executions", "Thirteen default-source AOT variants", "Three selected traps", "partial stdout", "`active/99`", "No fresh Linux/GPU"],
    },
    DevTensorDriftCheckSpec {
        id: "native-one-sided-effect-joins-tensor-preserves-partial-paired-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["One-sided effect-result join follow-up:", "Previous partial-result join checkpoint:", "Partial effect-result join follow-up:", "154 distinct selected tests", "conditional_return_one_sided_effect_joins"],
    },
];
