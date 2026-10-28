use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-exact-type",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values.rs",
        required_patterns: &[
            "NirBinaryOp::Div | NirBinaryOp::Rem if lhs == scalar_type(\"i32\")",
            "if lhs != rhs",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-lazy-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "Step::Binary(op, lazy_rhs)",
            "let checked_i32 = lazy_rhs",
            "&& !checked_i32",
            "comparison_only: checked_i32",
            "right.captures.len() > 31",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-reference-width",
        path: "crates/yir-domain-cpu/src/execute_scalar.rs",
        required_patterns: &[
            "has_i32_operand(node, state)",
            "checked_i32_div_rem(node, state, false)",
            "checked_i32_div_rem(node, state, true)",
            "state.expect_i32(&node.op.args[0])?",
            "lhs.checked_rem(rhs)",
            "lhs.checked_div(rhs)",
            "map(Value::I32)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-reference-controls",
        path: "crates/yir-domain-cpu/src/tests_scalar_logic.rs",
        required_patterns: &[
            "checked_i32_division_and_remainder_preserve_width_and_signed_results",
            "checked_i32_division_and_remainder_reject_zero_and_overflow_without_mutation",
            "checked_i32_division_and_remainder_do_not_coerce_mixed_kinds",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-llvm-contract",
        path: "crates/yir-lower-llvm/src/scalar_lowering.rs",
        required_patterns: &[
            "i64::from(i32::MIN)",
            "icmp eq i{bits} {rhs}, 0",
            "icmp eq i{bits} {rhs}, -1",
            "integer_divisor_invalid",
            "call void @llvm.trap()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-llvm-controls",
        path: "crates/yir-lower-llvm/src/tests/i32_division_tests.rs",
        required_patterns: &[
            "checked_i32_division_and_remainder_keep_i32_and_guard_before_operation",
            "checked_i32_division_and_remainder_use_distinct_guard_labels",
            "srem i32",
            "sdiv i32",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_division_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i32_division_argument_print_epoch_requires_exact_lazy_operands_and_read_permission",
            "i32_division_argument_print_epoch_rejects_mixed_effectful_and_forged_wide_values",
            "i32_division_argument_print_epoch_keeps_results_comparison_only",
            "i32_division_argument_print_epoch_retains_capture_work_depth_and_logical_bounds",
            "i32_division_argument_print_epoch_source_preserves_checked_signed_and_skipped_results",
            "i32_division_argument_print_epoch_unused_pure_helper_operations_still_trap_when_reached",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_division_argument_print_epoch_tests.rs",
        required_patterns: &[
            "balanced(2044)",
            "4096/4097",
            "for count in [31, 32]",
            "for count in [32, 33]",
            "for _ in 0..65",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-source-epochs",
        path: "tools/nuisc/tests/native_application_bridge/i32_division_argument_print_epoch.ns",
        required_patterns: &[
            "checked32(current.inner.code, first) / right",
            "checked32(local.inner.code, second) % right",
            "code: left",
            "code: right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i32_division_argument_print_epoch_cases.rs",
        required_patterns: &[
            "left == i32::MIN && right == -1",
            "i32::try_from(if remainder {",
            "left % right",
            "left / right",
            ".ok()",
            "checked_word(value, self.right, false)?",
            "checked_word(value, self.right, true)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-native-state",
        path: "tools/nuisc/tests/native_application_bridge/i32_division_argument_print_epoch.rs",
        required_patterns: &[
            "case.helper_counts(previous)",
            "ApplicationSessionPhase::Faulted",
            "cases::sequences(kind).len(), 33",
            "cases::failures(kind).len(), 20",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/i32_division_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-receipt",
        path: "docs/reference/nuis-native-i32-division-argument-print-v1.md",
        required_patterns: &[
            "Checked Width, Not Wrapping Quotients",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Checked i32 division argument print witnesses:",
            "Previous i32-arithmetic checkpoint:",
            "Previous i32-arithmetic task checkpoint:",
            "Previous i32-arithmetic blocker checkpoint:",
            "Previous i32-arithmetic action checkpoint:",
            "Previous i32-arithmetic validation checkpoint:",
            "Previous i32-arithmetic artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-division-argument-strict-native-width",
        path: "crates/yir-lower-llvm/src/function_lowering/strict_scalar.rs",
        required_patterns: &[
            "(\"div\" | \"rem\", Some(LlvmValueRef::I32(_)), Some(LlvmValueRef::I32(_)))",
            "CpuCallScalarKind::I32",
            "checked_i32_division_requires_matching_native_widths",
        ],
    },
];
