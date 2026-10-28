use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-scoped-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "roots_with_fields(pending, computed_gate, false, |_, _| false, |_, _, _| false)",
            "roots_with_fields(\n        pending,\n        true,\n        true,",
            "roots_with_fields(\n        pending,\n        true,\n        false,",
            "computed_bool_operands: bool",
            "computed_bool_operands\n                    && logical_root",
            "matches!(op, NirBinaryOp::Eq | NirBinaryOp::Ne | NirBinaryOp::Xor)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-eager-context",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "Eager bool operators evaluate both operands without authorizing a lazy RHS.",
            "(lhs.as_ref(), depth + 1, eager_bool, false, lazy_rhs)",
            "(rhs.as_ref(), depth + 1, eager_bool, false, lazy_rhs)",
            "let mut logical_edges = 32usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-proof-and-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "pending.push(Step::Binary(*op, lazy_rhs))",
            "Step::Binary(op, lazy_rhs)",
            "control_values::binary_type(op, left.ty, right.ty)?",
            "right.captures.len() > 31",
            "if proof.comparison_only && &proof.ty != result",
            "let right = lazy_rhs || matches!(op, NirBinaryOp::And | NirBinaryOp::Or)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_xor_argument_print_epoch_tests.rs",
        required_patterns: &[
            "bool_xor_argument_print_epoch_keeps_exact_types_epochs_and_profiles",
            "bool_xor_argument_print_epoch_rejects_mixed_effect_missing_and_provenance_escapes",
            "bool_xor_argument_print_epoch_visits_both_operands_in_source_order",
            "bool_xor_argument_print_epoch_checks_rhs_even_after_constant_lhs",
            "bool_xor_argument_print_epoch_bounds_shared_edges_work_depth_and_capture_union",
            "bool_xor_argument_print_epoch_source_preserves_eager_operands_and_nested_guards",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-shared-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_xor_argument_print_epoch_tests.rs",
        required_patterns: &[
            "for count in [31, 32]",
            "for count in [4096, 4097]",
            "vec![(&value, 64, false)]",
            "computed_record_expression_roots",
            "assert_eq!(actual, arguments)",
            "assert_eq!(picks, expected)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-source-epochs",
        path: "tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch.ns",
        required_patterns: &[
            "^ ((current.inner.code < right) || (checked32(right, second) > current.inner.code))",
            "^ ((local.inner.code == left) && (checked32(left, first) <= right))",
            "code: left",
            "code: right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-independent-operands",
        path: "tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch_cases.rs",
        required_patterns: &[
            "let pick = left ^ right",
            "let right = code == self.left",
            concat!(
                "checked32(\n                                self.right,\n",
                "                                self.second,\n                                trace,\n",
                "                                native,\n                                counts,\n",
                "                            )"
            ),
            "checked32(self.left, self.first, trace, native, counts)",
            "counts[0] += 1",
            "counts[1] += 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-actual-word-probes",
        path: "tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch_cases.rs",
        required_patterns: &[
            "zext i1 %arg0 to i64",
            "sext i32 %arg1 to i64",
            "sext i32 %arg2 to i64",
            "for word in [96, 99, 106, 107, 108]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-matrix-and-state",
        path: "tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch.rs",
        required_patterns: &[
            "case.helper_counts(previous)",
            "cases::sequences(kind).len(), 35",
            "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 16",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-suppressed-and-reached-poison",
        path: "tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch_cases.rs",
        required_patterns: &[
            "skipped_argument.first = 0",
            "skipped_or.second = 0",
            "bad.base.row.trips = 2",
            "bad.left = if second == 0 { bad.right } else { -1 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-entry-budget",
        path: "tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch.rs",
        required_patterns: &[
            "compiled.yir.functions.len() <= 32",
            "bool_xor_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/bool_xor_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-receipt",
        path: "docs/reference/nuis-native-bool-xor-argument-print-v1.md",
        required_patterns: &[
            "Eager XOR, Guarded Operands",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Native bool XOR argument print witnesses:",
            "Previous bool-equality checkpoint:",
            "Previous bool-equality task checkpoint:",
            "Previous bool-equality blocker checkpoint:",
            "Previous bool-equality action checkpoint:",
            "Previous bool-equality validation checkpoint:",
            "Previous bool-equality artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-source-caret-token",
        path: "tools/nuisc/src/frontend/lexer.rs",
        required_patterns: &[
            "| '^' => tokens.push(Token::Symbol(ch))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-source-precedence",
        path: "tools/nuisc/src/frontend/parser_exprs.rs",
        required_patterns: &[
            "fn parse_xor(&mut self)",
            "let mut expr = self.parse_xor()?",
            "while self.peek_symbol('^')",
            "op: AstBinaryOp::Xor",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-source-exact-kinds",
        path: "tools/nuisc/src/frontend/binary_lowering.rs",
        required_patterns: &[
            "AstBinaryOp::Xor => NirBinaryOp::Xor",
            "lhs == rhs && (lhs == &bool_type() || lhs == &super::i64_type())",
            "binary `^` expects matching plain bool or i64 operands",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-nir-exact-inference",
        path: "tools/nuisc/src/frontend/types/nir_expr.rs",
        required_patterns: &[
            "lhs_ty == rhs_ty && (lhs_ty == i64_type() || lhs_ty == bool_type())",
            "Some(lhs_ty)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-source-regressions",
        path: "tools/nuisc/src/frontend/tests_frontend_core/bool_xor.rs",
        required_patterns: &[
            "bool_xor_source_precedence_association_and_rendering_are_explicit",
            "bool_xor_source_truth_table_emits_native_yir_xor",
            "bool_xor_source_keeps_i64_builtin_strict_and_rejects_cross_kinds",
            "bool_xor_nir_inference_rejects_modified_and_nonprimitive_operands",
            "bool_xor_source_generic_and_lambda_inference_agree",
            "bool_xor_source_caret_survives_stage_neutral_tokens",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-xor-legacy-i64-entry-stays-integer",
        path: "tools/nuisc/src/frontend/call_routing.rs",
        required_patterns: &[
            "infer_nir_expr_type(&left, bindings, signatures, struct_table) != Some(i64_type())",
            "infer_nir_expr_type(&right, bindings, signatures, struct_table)\n                    != Some(i64_type())",
            "i64_xor(...) expects exactly two plain i64 operands",
        ],
    },
];
