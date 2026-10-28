use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-frontend-owned",
        path: "tools/nuisc/src/frontend/unary_lowering.rs",
        required_patterns: &[
            "if operand_ty == i32_type()",
            "Match wrapping i32 Add/Sub/Mul without evaluating the operand twice",
            "rhs: Box::new(NirExpr::CastI32ToI64(Box::new(lowered_operand)))",
            "operand_ty == &i32_type()",
            "unary_i32_negation_exact_owned_builtin_profile",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-generic-owned",
        path: "tools/nuisc/src/frontend/generic_rewrite/exprs_operators.rs",
        required_patterns: &[
            "AstUnaryOp::Neg =>",
            "is_plain_scalar(operand_ty, \"i32\")",
            "ty.generic_args.is_empty()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-frontend-controls",
        path: "tools/nuisc/src/frontend/tests_frontend_core/unary_i32.rs",
        required_patterns: &[
            "i32_negation_source_uses_once_only_wide_subtract_and_wrap",
            "i32_negation_source_wraps_minimum_without_changing_other_numeric_profiles",
            "i32_negation_source_generic_lambda_and_exact_kind_rules_agree",
            "if_print_frontier_preserves_source_order_before_and_after_both_arms",
            "value.wrapping_neg()",
            "let mut reached",
            "missing source-ordered effect path",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-print-frontier",
        path: "tools/nuisc/src/lowering/if_lowering.rs",
        required_patterns: &[
            "from: selected",
            "to: print_name.clone()",
            "chain_statement_effect(state, &print_name)",
            "Ok(LoweredIfOutcome::Printed)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-canonical-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "fn i32_neg_operand(word: &NirExpr) -> Option<&NirExpr>",
            "(NirExpr::Int(0), NirExpr::CastI32ToI64(operand))",
            "i32_neg_operand(word).is_some()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-budget-and-provenance",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "pending.push(Step::NegI32)",
            "pending.push(Step::Value(operand, depth.checked_add(3)?, lazy_rhs))",
            "work = work.checked_sub(3)?",
            "Step::NegI32 =>",
            "if proof.ty != scalar_type(\"i32\")",
            "proof.comparison_only = true",
            "right.captures.len() > 31",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_negation_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i32_negation_argument_print_epoch_accepts_exact_lazy_comparison_operands",
            "i32_negation_argument_print_epoch_rejects_forged_zero_casts_and_unproved_inputs",
            "i32_negation_argument_print_epoch_retains_comparison_only_scope_and_composition",
            "i32_negation_argument_print_epoch_bounds_all_four_nodes_depth_and_captures",
            "i32_negation_argument_print_epoch_source_wraps_once_and_suppresses_skipped_faults",
            "i32_negation_argument_print_epoch_source_composes_checked_division_and_minimum_wrapping",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-exact-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_negation_argument_print_epoch_tests.rs",
        required_patterns: &[
            "neg(balanced(2042))",
            "4096/4097 work",
            "[(58, true), (59, false)]",
            "for count in [31, 32]",
            "for count in [32, 33]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-source-epochs",
        path: "tools/nuisc/tests/native_application_bridge/i32_negation_argument_print_epoch.ns",
        required_patterns: &[
            "relay32(-(-(-(checked32(current.inner.code, first)",
            "-checked32(local.inner.code, first)",
            "relay32(-i32_from_i64(107), second)",
            "relay32(-(checked32(local.inner.code, second) % right",
            "code: left",
            "code: right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i32_negation_argument_print_epoch_cases.rs",
        required_patterns: &[
            "fn neg_word(value: i32) -> i32",
            "wrapped_word(-i64::from(value))",
            "neg_word(neg_word(neg_word(wrapped_word(",
            "relay32(neg_word(107), self.second",
            "left == i32::MIN && right == -1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-actual-arguments",
        path: "tools/nuisc/tests/native_application_bridge/i32_negation_argument_print_epoch_cases.rs",
        required_patterns: &[
            "sext i32 %arg0 to i64",
            "sext i32 %arg10 to i64",
            "for word in [96, 99, 106, 107]",
            "counts[0] += 1",
            "counts[1] += 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-native-state",
        path: "tools/nuisc/tests/native_application_bridge/i32_negation_argument_print_epoch.rs",
        required_patterns: &[
            "case.helper_counts(previous)",
            "ApplicationSessionPhase::Faulted",
            "cases::sequences(kind).len(), 33",
            "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 20",
            "check_i64_helper_entry_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/i32_negation_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-receipt",
        path: "docs/reference/nuis-native-i32-negation-argument-print-v1.md",
        required_patterns: &[
            "Wrapping Negation, Not General Widening",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-negation-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Wrapping i32 negation argument print witnesses:",
            "Previous i32-literal checkpoint:",
            "Previous i32-literal task checkpoint:",
            "Previous i32-literal blocker checkpoint:",
            "Previous i32-literal action checkpoint:",
            "Previous i32-literal validation checkpoint:",
            "Previous i32-literal artifact checkpoint:",
        ],
    },
];
