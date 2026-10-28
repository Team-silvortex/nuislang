use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-exact-eager-operation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "let restricted_i64 = lazy_rhs",
            "left.ty == scalar_type(\"i64\")",
            "right.ty == scalar_type(\"i64\")",
            "| NirBinaryOp::Xor",
            "lazy_rhs || matches!(op, NirBinaryOp::And | NirBinaryOp::Or)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-comparison-provenance",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "&& !restricted_i64",
            "comparison_only: checked_i32 || restricted_i64",
            "if proof.comparison_only && &proof.ty != result",
            "ready_atom: false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-profile-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_xor_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i64_xor_argument_print_epoch_requires_exact_lazy_comparison_reads",
            "i64_xor_argument_print_epoch_rejects_kind_scope_and_effect_escapes",
            "i64_xor_argument_print_epoch_preserves_restriction_through_arithmetic_and_helpers",
            "i64_xor_argument_print_epoch_charges_both_operands_work_depth_and_captures",
            "i64_xor_argument_print_epoch_source_keeps_full_width_words_and_comparison_choices",
            "i64_xor_argument_print_epoch_source_is_eager_inside_guarded_contexts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-exact-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_xor_argument_print_epoch_tests.rs",
        required_patterns: &[
            "balanced_xor(2043, true)",
            "4096/4097 work",
            "[(59, true), (60, false)]",
            "for count in [31, 32]",
            "for count in [32, 33]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-source-eager-completion-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_xor_argument_print_epoch_tests.rs",
        required_patterns: &[
            "fn completed_calls(events: &[String])",
            "Reference events record completion",
            "Zero on either side is not a short-circuit rule for XOR",
            "assert_eq!(actual, picks)",
            "entered && (first == 0 || second == 0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-source-independent-word-oracle",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_xor_argument_print_epoch_tests.rs",
        required_patterns: &[
            "fn xor_word(left: i64, right: i64)",
            "for _ in 0..64",
            "left % 2 != right % 2",
            "bits - modulus",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-native-source",
        path: "tools/nuisc/tests/native_application_bridge/i64_xor_argument_print_epoch.ns",
        required_patterns: &[
            "mask_guard: i64",
            "^ checked64(i64_from_i32(current.inner.code) + 4294967296, mask_guard)",
            "relay64(checked64(i64_from_i32(local.inner.code) + 4294967296, mask_guard) ^",
            "i64_from_i32(-i32_from_i64(107)) ^ checked64(4294967296, mask_guard)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-native-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i64_xor_argument_print_epoch_cases.rs",
        required_patterns: &[
            "fn xor_word(left: i64, right: i64)",
            "left % 2 != right % 2",
            "xor_word(left, right)",
            "self.mask_guard",
            "call void @word_print(i64 %arg12)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-mask-fault-rows",
        path: "tools/nuisc/tests/native_application_bridge/i64_xor_argument_print_epoch_cases.rs",
        required_patterns: &[
            "fault in the right XOR operand; child: fault in the left XOR operand",
            "mask_fault.mask_guard = 0",
            "mask_fault.left = if selected { -3 } else { -1 }",
            "eager_zero.left = -3",
            "skipped_mask.mask_guard = 0",
            "zero.mask_guard = 0",
            "these rows fail only in reached i64 Div/Rem",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-native-publication",
        path: "tools/nuisc/tests/native_application_bridge/i64_xor_argument_print_epoch.rs",
        required_patterns: &[
            "cases::sequences(kind).len(), 33",
            "cases::failures(kind).len(), 24",
            "args[12].parse::<i64>()",
            "ApplicationSessionPhase::Faulted",
            "check_i64_helper_entry_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/i64_xor_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-cpu-xor-contract",
        path: "crates/yir-domain-cpu/src/execute_scalar.rs",
        required_patterns: &[
            "\"xor\" => match",
            "Value::Int(",
            "state.expect_int(&node.op.args[0])? ^ state.expect_int(&node.op.args[1])?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-llvm-xor-contract",
        path: "crates/yir-lower-llvm/src/bitwise_lowering.rs",
        required_patterns: &[
            "\"xor\" => \"xor\"",
            "get_i64(registers, &node.op.args[0])",
            "get_i64(registers, &node.op.args[1])",
            "{llvm_op} i64 {lhs}, {rhs}",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-reference-completion-contract",
        path: "crates/yir-exec/src/execution_engine.rs",
        required_patterns: &[
            "let value = self.execute_function(callee, arguments)?",
            "validate_call_result(node, &value)?",
            "self.state.push_resource_event",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-receipt",
        path: "docs/reference/nuis-native-i64-xor-argument-print-v1.md",
        required_patterns: &[
            "Exact Eager Composition",
            "completion records",
            "512 helper entries",
            "24 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i64-xor-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Restricted i64 XOR argument print witnesses:",
            "Previous i64-arithmetic checkpoint:",
            "Previous i64-arithmetic task checkpoint:",
            "Previous i64-arithmetic blocker checkpoint:",
            "Previous i64-arithmetic action checkpoint:",
            "Previous i64-arithmetic validation checkpoint:",
            "Previous i64-arithmetic artifact checkpoint:",
        ],
    },
];
