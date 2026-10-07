use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod float_literal_negation;", "float_literal_negation::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-typed-atoms-and-unchanged-computation",
        path: "tools/nuisc/src/frontend/unary_lowering.rs",
        required_patterns: &["infer_nir_expr_type(&lowered_operand", "lower_overloaded_unary_operator(*op", "NirExpr::F32(value) => NirExpr::F32(negated_float_literal(value))", "NirExpr::F64(value) => NirExpr::F64(negated_float_literal(value))", "value.strip_prefix('-')", "fn sign_flipped_word", "!operand_ty.is_ref && !operand_ty.is_optional"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-frontdoor-types-and-once-only-structure",
        path: "tools/nuisc/src/frontend/tests_frontend_core/unary_float_literals.rs",
        required_patterns: &["unary_float_literals_preserve_sign_spelling_and_nested_negation", "unary_float_literals_respect_inferred_and_expected_contexts", "unary_float_literals_leave_nonliteral_operands_once_only_and_unfolded", "unary_float_literals_do_not_bypass_expected_type_validation", "for binding in [\"let\", \"const\"]", "let operand = sign_flip_operand(value, ty)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-cpu-llvm-materializes-exact-parsed-bits",
        path: "crates/yir-lower-llvm/src/preclassified_lowering.rs",
        required_patterns: &["node.op.args[0].parse::<f32>()", "node.op.args[0].parse::<f64>()", "bitcast i32 {} to float", "bitcast i64 {} to double", "value.to_bits()", "LlvmValueRef::F32(reg.clone())", "LlvmValueRef::F64(reg.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-llvm-kind-rejection-and-bit-tests",
        path: "crates/yir-lower-llvm/src/tests/float_literal_tests.rs",
        required_patterns: &["float_literals_materialize_f32_bits_without_zero_addition", "float_literals_materialize_f64_bits_without_zero_addition", "float_literals_reject_invalid_and_wrong_arity_before_emission", "for reversed in [false, true]", "assert!(!llvm.contains(\"fadd float 0.0\")", "assert!(emit_module(&module).is_err())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-native-word-oracle-and-storage-order",
        path: "tools/nuisc/tests/native_application_bridge/float_unary_literals.rs",
        required_patterns: &["unary_float_literals_f32_preserve_native_and_yir_bits", "unary_float_literals_f64_preserve_native_and_yir_bits", "for reversed in [false, true]", "value.to_bits()", ".map(f64::to_bits)", "assert_eq!(state_words(session.state()), words)", "write_and_link_with_source", "run_bounded", "assert_eq!(actual, expected"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-source-effect-join-selector",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_typed_snapshot_tests.rs",
        required_patterns: &[".replace(\"print(70); 0.0\", \"print(70); -0.0\")", "assert_eq!(*value, negative_zero)", "assert_eq!(joined.params.len(), 2)", "assert!(!scalar(&scalar_type(\"f64\")))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-bounded-proof-and-open-dynamic-gap",
        path: "docs/reference/nuis-native-float-literal-negation-v1.md",
        required_patterns: &["Floating literal negation acceptance:", "four linked native artifacts", "General nonliteral float negation still uses positive-zero subtraction", "NaN sign/payload negation is not certified", "`active/99`", "No fresh Linux/GPU run"],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-literal-negation-tensor-history-and-next-contract",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Floating literal negation follow-up:", "Previous typed-snapshot checkpoint:", "Previous typed-snapshot task checkpoint:", "Previous typed-snapshot blocker checkpoint:", "Previous typed-snapshot action checkpoint:", "Previous typed-snapshot artifact checkpoint:", "unary_float_literals", "general nonliteral floating negation"],
    },
];
