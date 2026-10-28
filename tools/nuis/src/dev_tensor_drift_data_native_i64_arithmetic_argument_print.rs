use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-exact-five-operations",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "let restricted_i64 = lazy_rhs",
            "left.ty == scalar_type(\"i64\")",
            "right.ty == scalar_type(\"i64\")",
            "NirBinaryOp::Add",
            "| NirBinaryOp::Sub",
            "| NirBinaryOp::Mul",
            "| NirBinaryOp::Div",
            "| NirBinaryOp::Rem",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-comparison-provenance",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "&& !restricted_i64",
            "comparison_only: checked_i32 || restricted_i64",
            "Arithmetic composes the permission; only a comparison may consume it",
            "if proof.comparison_only && &proof.ty != result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-profile-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_arithmetic_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i64_arithmetic_argument_print_epoch_requires_exact_lazy_comparison_reads",
            "i64_arithmetic_argument_print_epoch_rejects_kind_scope_and_effect_escapes",
            "i64_arithmetic_argument_print_epoch_preserves_restriction_through_operations_and_calls",
            "i64_arithmetic_argument_print_epoch_charges_full_tree_depth_and_capture_union",
            "i64_arithmetic_argument_print_epoch_source_wraps_and_keeps_exact_helper_words",
            "i64_arithmetic_argument_print_epoch_source_checks_div_rem_before_forwarding",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-exact-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_arithmetic_argument_print_epoch_tests.rs",
        required_patterns: &[
            "balanced64(2043, true)",
            "4096/4097 work",
            "[(59, true), (60, false)]",
            "for count in [31, 32]",
            "for count in [32, 33]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-source-result-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_arithmetic_argument_print_epoch_tests.rs",
        required_patterns: &[
            "fn wrap64(value: i128) -> i64",
            "fn arithmetic64(left: i64, right: i64, op: &str)",
            "assert_eq!(actual_picks, picks)",
            "4611686018427387904",
            "assert_eq!(result.is_err(), gate)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-native-source",
        path: "tools/nuisc/tests/native_application_bridge/i64_arithmetic_argument_print_epoch.ns",
        required_patterns: &[
            " + 9223372036854775807) - 1) * 4294967296) / second",
            " + 9223372036854775807) - 1) * 4294967296) % first",
            "relay64(i64_from_i32(-i32_from_i64(107))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i64_arithmetic_argument_print_epoch_cases.rs",
        required_patterns: &[
            "fn wide_word(value: i64) -> i64",
            "fn checked_wide(left: i64, right: i64, remainder: bool)",
            "value.rem_euclid(1i128 << 64)",
            "left == i64::MIN && right == -1",
            "i128::from(left) / i128::from(right)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-wide-overflow-fault-rows",
        path: "tools/nuisc/tests/native_application_bridge/i64_arithmetic_argument_print_epoch_cases.rs",
        required_patterns: &[
            "these rows fail only in reached i64 Div/Rem",
            "overflow.left = 2147483645",
            "overflow.right = if remainder { i32::MIN } else { 1 }",
            "overflow.first = if remainder { -1 } else { 1 }",
            "overflow.second = if remainder { 1 } else { -1 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-native-publication",
        path: "tools/nuisc/tests/native_application_bridge/i64_arithmetic_argument_print_epoch.rs",
        required_patterns: &[
            "cases::sequences(kind).len(), 33",
            "cases::failures(kind).len(), 22",
            "cases::checked_wide(i64::MIN, -1, remainder)",
            "ApplicationSessionPhase::Faulted",
            "check_i64_helper_entry_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/i64_arithmetic_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-cpu-arithmetic-contract",
        path: "crates/yir-domain-cpu/src/execute_scalar.rs",
        required_patterns: &[
            "lhs.wrapping_add(rhs)",
            "lhs.wrapping_sub(rhs)",
            "lhs.wrapping_mul(rhs)",
            "lhs.checked_div(rhs)",
            "lhs.checked_rem(rhs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-llvm-arithmetic-contract",
        path: "crates/yir-lower-llvm/src/scalar_lowering.rs",
        required_patterns: &[
            "= add i64 {lhs}, {rhs}",
            "= sub i64 {lhs}, {rhs}",
            "= mul i64 {lhs}, {rhs}",
            "check_integer_divisor(&lhs, &rhs, 64",
            "i64::MIN",
            "call void @llvm.trap()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-receipt",
        path: "docs/reference/nuis-native-i64-arithmetic-argument-print-v1.md",
        required_patterns: &[
            "Exact Arithmetic Composition",
            "31/32",
            "512 helper entries",
            "22 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-arithmetic-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Restricted i64 arithmetic argument print witnesses:",
            "Previous i32-widening checkpoint:",
            "Previous i32-widening task checkpoint:",
            "Previous i32-widening blocker checkpoint:",
            "Previous i32-widening action checkpoint:",
            "Previous i32-widening validation checkpoint:",
            "Previous i32-widening artifact checkpoint:",
        ],
    },
];
