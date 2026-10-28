use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-frontend-entry",
        path: "tools/nuisc/src/frontend/call_routing.rs",
        required_patterns: &[
            "\"i64_from_i32\" =>",
            "i64_from_i32(...) does not accept type arguments",
            "i64_from_i32(...) expects exactly one argument",
            "Some(&i32_type())",
            "NirExpr::CastI32ToI64(Box::new(lowered))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-ast-exact-kind",
        path: "tools/nuisc/src/frontend/types/ast_calls.rs",
        required_patterns: &[
            "\"i64_from_i32\" if generic_args.is_empty()",
            "let [value] = args else",
            "inner == ast_named_type(\"i32\")",
            "ast_named_type(\"i64\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-lambda-local-kind",
        path: "tools/nuisc/src/frontend/lambda_expansion_types.rs",
        required_patterns: &[
            "callee == \"i64_from_i32\" && generic_args.is_empty()",
            "? == named_type(\"i32\")",
            "named_type(\"i64\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-frontend-controls",
        path: "tools/nuisc/src/frontend/tests_frontend_core/i32_widening.rs",
        required_patterns: &[
            "i32_widening_source_emits_one_exact_signed_extension",
            "i32_widening_source_preserves_signed_range_and_negation_order",
            "i32_widening_source_rejects_nonexact_inputs_arity_and_type_arguments",
            "i32_widening_source_generic_arguments_and_lambda_local_inference_agree",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-budget-and-permission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "NirExpr::CastI32ToI64(operand) if lazy_rhs",
            "pending.push(Step::WidenI32)",
            "pending.push(Step::Value(operand, depth.checked_add(1)?, lazy_rhs))",
            "proof.ty = scalar_type(\"i64\")",
            "proof.comparison_only = true",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-kind-preserving-calls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "if proof.comparison_only && &proof.ty != result",
            "comparison_only |= proof.comparison_only",
            "comparison_only: comparison_only || result == &scalar_type(\"i32\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_widening_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i32_widening_argument_print_epoch_requires_exact_lazy_comparison_reads",
            "i32_widening_argument_print_epoch_rejects_wrong_kinds_effects_and_narrowing",
            "i32_widening_argument_print_epoch_retains_kind_preserving_call_provenance",
            "i32_widening_argument_print_epoch_charges_conversion_depth_work_and_capture_union",
            "i32_widening_argument_print_epoch_source_preserves_signed_values_and_skipped_faults",
            "i32_widening_argument_print_epoch_source_keeps_checked_i32_faults_before_conversion",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-exact-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_widening_argument_print_epoch_tests.rs",
        required_patterns: &[
            "balanced(2043)",
            "4096/4097 work",
            "[(60, true), (61, false)]",
            "for count in [31, 32]",
            "for count in [32, 33]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-source-epochs",
        path: "tools/nuisc/tests/native_application_bridge/i32_widening_argument_print_epoch.ns",
        required_patterns: &[
            "relay64(i64_from_i32(-(-(-(checked32(current.inner.code, first)",
            "relay64(i64_from_i32(-i32_from_i64(107))",
            "relay64(i64_from_i32(-(checked32(local.inner.code, second) % right",
            "return checked64(value + 4294967296, guard)",
            "code: left",
            "code: right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i32_widening_argument_print_epoch_cases.rs",
        required_patterns: &[
            "fn checked64(",
            "fn relay64(",
            "value.wrapping_add(4294967296)",
            "i64::from(neg_word(107))",
            "left == i32::MIN && right == -1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-actual-wide-arguments",
        path: "tools/nuisc/tests/native_application_bridge/i32_widening_argument_print_epoch_cases.rs",
        required_patterns: &[
            "(\"checked64\", 108), (\"relay64\", 109)",
            "counts[2] += 1",
            "counts[3] += 1",
            "for word in [96, 99, 108, 109]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-native-state",
        path: "tools/nuisc/tests/native_application_bridge/i32_widening_argument_print_epoch.rs",
        required_patterns: &[
            "case.helper_counts(previous)",
            "ApplicationSessionPhase::Faulted",
            "cases::sequences(kind).len(), 33",
            "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 20",
            "words[1] > i64::from(i32::MAX)",
            "check_i64_helper_entry_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/i32_widening_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-receipt",
        path: "docs/reference/nuis-native-i32-widening-argument-print-v1.md",
        required_patterns: &[
            "Explicit Signed Widening",
            "Scoped Comparison Permission",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-widening-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Sign-preserving i32 widening argument print witnesses:",
            "Previous i32-negation checkpoint:",
            "Previous i32-negation task checkpoint:",
            "Previous i32-negation blocker checkpoint:",
            "Previous i32-negation action checkpoint:",
            "Previous i32-negation validation checkpoint:",
            "Previous i32-negation artifact checkpoint:",
        ],
    },
];
