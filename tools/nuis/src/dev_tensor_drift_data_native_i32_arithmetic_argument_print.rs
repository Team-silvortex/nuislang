use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-canonical-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "NirExpr::CastI64ToI32(word) if lazy_rhs",
            "let (left, right) = i32_operands(word)?",
            "op: NirBinaryOp::Add | NirBinaryOp::Sub | NirBinaryOp::Mul",
            "NirExpr::CastI32ToI64(left), NirExpr::CastI32ToI64(right)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-budgeted-source-order",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "work = work.checked_sub(3)?",
            "let child = depth.checked_add(3)?",
            "pending.push(Step::Value(right, child, lazy_rhs))",
            "pending.push(Step::Value(left, child, lazy_rhs))",
            "depth > 64 || work == 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-comparison-provenance",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "Step::WrapI32 =>",
            "left.ty != scalar_type(\"i32\") || right.ty != scalar_type(\"i32\")",
            "left.total &= right.total",
            "left.comparison_only = true",
            "left.captures.append(&mut right.captures)",
            "if comparison_only && result != &scalar_type(\"i32\")",
            "right.captures.len() > 31",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_arithmetic_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i32_arithmetic_argument_print_epoch_accepts_only_typed_lazy_comparisons",
            "i32_arithmetic_argument_print_epoch_rejects_forged_conversions_and_inputs",
            "i32_arithmetic_argument_print_epoch_retains_comparison_only_provenance",
            "i32_arithmetic_argument_print_epoch_checks_read_order_and_complete_capture_union",
            "i32_arithmetic_argument_print_epoch_charges_every_conversion_node_and_depth",
            "i32_arithmetic_argument_print_epoch_source_wraps_and_orders_helper_arguments",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-exact-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_arithmetic_argument_print_epoch_tests.rs",
        required_patterns: &[
            "balanced(817)",
            "for calls in [3, 4]",
            "4096/4097 work",
            "for count in [31, 32]",
            "for count in [32, 33]",
            "0..22",
            "code.wrapping_add(right)",
            "code.wrapping_sub(right)",
            "code.wrapping_mul(right)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-source-epochs",
        path: "tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch.ns",
        required_patterns: &[
            "checked32(current.inner.code, first) + right",
            "checked32(local.inner.code, first) - right",
            "relay32(right * local.inner.code, second)",
            "checked32(local.inner.code, second) * right",
            "code: left",
            "code: right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-independent-modulo-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch_cases.rs",
        required_patterns: &[
            "let word = value & 0xffff_ffff",
            "word >= 0x8000_0000",
            "word - 0x1_0000_0000",
            "i64::from(value) + i64::from(self.right)",
            "i64::from(self.right) * i64::from(code)",
            "i64::from(value) * i64::from(self.right)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-signed-probes",
        path: "tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch_cases.rs",
        required_patterns: &[
            "sext i32 %arg0 to i64",
            "sext i32 %arg10 to i64",
            "sext i32 %arg11 to i64",
            "for word in [96, 99, 106, 107]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-structural-counts-and-state",
        path: "tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch.rs",
        required_patterns: &[
            "[\"checked32\", \"relay32\"]",
            "case.helper_counts(previous)",
            "ApplicationSessionPhase::Faulted",
            "assert_calls(&event.events, *row, Some(previous))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-explicit-matrix",
        path: "tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch.rs",
        required_patterns: &[
            "cases::sequences(kind).len(), 39",
            "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 16",
            "Value::I32(left)",
            "i64::from(left)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-entry-rejection",
        path: "tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch.rs",
        required_patterns: &[
            "compiled.yir.functions.len() <= 32",
            "i32_arithmetic_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/i32_arithmetic_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-receipt",
        path: "docs/reference/nuis-native-i32-arithmetic-argument-print-v1.md",
        required_patterns: &[
            "Canonical Wrapping, Not General Casts",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-arithmetic-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Canonical i32 arithmetic argument print witnesses:",
            "Previous bool-XOR checkpoint:",
            "Previous bool-XOR task checkpoint:",
            "Previous bool-XOR blocker checkpoint:",
            "Previous bool-XOR action checkpoint:",
            "Previous bool-XOR validation checkpoint:",
            "Previous bool-XOR artifact checkpoint:",
        ],
    },
];
