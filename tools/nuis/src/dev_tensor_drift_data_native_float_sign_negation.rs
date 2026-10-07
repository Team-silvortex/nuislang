use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod float_sign_negation;", "float_sign_negation::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-exact-sign-recipe",
        path: "tools/nuisc/src/frontend/unary_lowering.rs",
        required_patterns: &["infer_nir_expr_type(&lowered_operand", "lower_overloaded_unary_operator(*op", "operand_ty == f32_type()", "operand_ty == f64_type()", "NirExpr::PackF32Word(Box::new(operand))", "NirExpr::PackF64Word(Box::new(operand))", "1_i64 << 31", "i64::MIN", "op: NirBinaryOp::Xor", "unary_float_negation_exact_owned_builtin_profile"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-once-only-frontdoor",
        path: "tools/nuisc/src/frontend/tests_frontend_core/unary_float_literals.rs",
        required_patterns: &["fn sign_flip_operand", "assert_eq!(*op, NirBinaryOp::Xor)", "let operand = sign_flip_operand(value, ty)", "sign_flip_operand(operand, ty)", "crate::nir_verify::verify_nir_module(&module)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-bounded-pure-value-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values.rs",
        required_patterns: &["if lhs != rhs", "NirBinaryOp::Xor if lhs == scalar_type(\"i64\") => Some(lhs)", "| NirExpr::UnpackF64Word(value) => expression(value)", "fn collect_inputs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-exact-word-and-aggregate-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values/typed_tests.rs",
        required_patterns: &["unary_float_negation_sign_words_admit_exact_i64_xor_without_profile_widening", "unary_float_negation_word_wrappers_keep_aggregate_iteration_classification", "[\"reference\", \"optional\", \"generic\"]", "scalar_helpers::collect_typed_values", "assert!(has_aggregate_expressions(&function.body))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-checked-native-xor",
        path: "crates/yir-lower-llvm/src/native_session/admission.rs",
        required_patterns: &["pub(super) fn admitted", "| \"xor\"", "MAX_TOTAL_NODES", "validate"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-native-bit-oracle",
        path: "tools/nuisc/tests/native_application_bridge/float_unary_negation.rs",
        required_patterns: &["unary_float_negation_f32_preserves_zero_nan_payloads_and_once_only_calls", "unary_float_negation_f64_preserves_zero_nan_payloads_and_once_only_calls", "unary_float_negation_native_bridge_rejects_wrong_word_and_mask_kinds", "word ^ sign", "0x7f80_0001", "0x7ff0_0000_0000_0001", "module.edges.reverse()", "load volatile i64", "write_and_link_with_source", "run_bounded", "assert_eq!("],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-call-dependency-traversal",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &["fn collect_expr_calls", "let mut pending = vec![expr]", "| NirExpr::PackF32Word(base)", "| NirExpr::UnpackF32Word(base)", "| NirExpr::PackF64Word(base)", "| NirExpr::UnpackF64Word(base) => pending.push(base)", "helper.dependencies.iter().cloned()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-selected-execution",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_typed_snapshot_tests.rs",
        required_patterns: &["conditional_return_typed_snapshots_keep_nonliteral_float_negation_in_selected_execution", "return -(value + 0.5);", "print(70); -local", "let called = input.outer && !(partial && input.nested)", "assert_eq!(cases, 128)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-scoped-proof",
        path: "docs/reference/nuis-native-float-sign-negation-v1.md",
        required_patterns: &["Floating sign negation acceptance:", "eight linked native artifacts", "128 source cases", "integer", "`active/99`", "No fresh Linux/GPU run"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-sign-negation-complete-history-and-open-tail",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Floating sign negation follow-up:", "Previous literal-negation checkpoint:", "Previous literal-negation task checkpoint:", "Previous literal-negation blocker checkpoint:", "Previous literal-negation action checkpoint:", "Previous literal-negation artifact checkpoint:", "unary_float_negation", "typed enclosing return-tail"],
    },
];
