use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "NirExpr::CastI64ToI32(word) if lazy_rhs && i32_literal(word)",
            "pending.push(Step::LiteralI32)",
            "pending.push(Step::Value(word, depth.checked_add(1)?, lazy_rhs))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-signed-canonical",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "fn i32_literal(word: &NirExpr) -> bool",
            "(NirExpr::Int(0), NirExpr::Int(magnitude)) if *magnitude >= 0",
            "magnitude.checked_neg()",
            "value.is_some_and(|value| i32::try_from(value).is_ok())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-provenance",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "Step::LiteralI32 =>",
            "if proof.ty != scalar_type(\"i64\")",
            "proof.ty = scalar_type(\"i32\")",
            "proof.ready_atom = false",
            "proof.ready_predicate = false",
            "proof.comparison_only = true",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-budget-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_literal_argument_print_epoch_tests.rs",
        required_patterns: &[
            "(1362, false, 1), (1361, true, 2)",
            "Each cast/literal is charged",
            "for count in [31, 32]",
            "for count in [32, 33]",
            "[(60, true), (61, false)]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_literal_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i32_literal_argument_print_epoch_accepts_exact_in_range_lazy_comparisons",
            "i32_literal_argument_print_epoch_rejects_out_of_range_and_nonliteral_casts",
            "i32_literal_argument_print_epoch_retains_scope_and_comparison_only_provenance",
            "i32_literal_argument_print_epoch_counts_both_nodes_and_preserves_capture_bounds",
            "i32_literal_argument_print_epoch_source_preserves_signed_results_and_skipped_faults",
            "i32_literal_argument_print_epoch_source_literal_helpers_keep_exact_arguments",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-source",
        path: "tools/nuisc/tests/native_application_bridge/i32_literal_argument_print_epoch.ns",
        required_patterns: &[
            "/ right + i32_from_i64(1)",
            "< i32_from_i64(0)",
            "<= i32_from_i64(2147483647)",
            "relay32(i32_from_i64(107), second)",
            "% right - i32_from_i64(-1)",
            "!= i32_from_i64(-2147483648)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i32_literal_argument_print_epoch_cases.rs",
        required_patterns: &[
            "fn wrapped_word(value: i64) -> i32",
            "let word = value & 0xffff_ffff",
            "word - 0x1_0000_0000",
            "<= i64::from(i32::MAX)",
            "relay32(107, self.second",
            "!= i32::MIN",
            "left == i32::MIN && right == -1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-actual-arguments",
        path: "tools/nuisc/tests/native_application_bridge/i32_literal_argument_print_epoch_cases.rs",
        required_patterns: &[
            "sext i32 %arg0 to i64",
            "sext i32 %arg10 to i64",
            "for word in [96, 99, 106, 107]",
            "checked_word(value, self.right, false)",
            "checked_word(value, self.right, true)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-matrix",
        path: "tools/nuisc/tests/native_application_bridge/i32_literal_argument_print_epoch.rs",
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
        id: "native-i32-literal-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/i32_literal_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-receipt",
        path: "docs/reference/nuis-native-i32-literal-argument-print-v1.md",
        required_patterns: &[
            "Exact Literals, Not General Casts",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-literal-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Exact i32 literal argument print witnesses:",
            "Previous i32-division checkpoint:",
            "Previous i32-division task checkpoint:",
            "Previous i32-division blocker checkpoint:",
            "Previous i32-division action checkpoint:",
            "Previous i32-division validation checkpoint:",
            "Previous i32-division artifact checkpoint:",
        ],
    },
];
