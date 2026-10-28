use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-scoped-result",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "lazy_rhs && matches!(result.name.as_str(), \"bool\" | \"i32\" | \"i64\")",
            "scalar_helpers::pure_scalar_signature(callee, catalog)?",
            "pending.push(Step::Exact(ty))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-comparison-only-forwarding",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "comparison_only: bool",
            "comparison_only |= proof.comparison_only",
            "if proof.comparison_only && &proof.ty != result",
            "comparison_only: comparison_only || result == &scalar_type(\"i32\")",
            "left.comparison_only || right.comparison_only",
            "!proofs[0].comparison_only",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-budget-and-read-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "(!right.total || !right.ready_predicate)",
            "right.captures.len() > 31",
            "depth > 64 || work == 0",
            "let mut work = 4096usize",
            "control_values::binary_type(op, left.ty, right.ty)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_call_argument_print_epoch_tests.rs",
        required_patterns: &[
            "i32_call_argument_print_epoch_requires_exact_comparison_result_and_lazy_scope",
            "i32_call_argument_print_epoch_keeps_new_results_comparison_only",
            "i32_call_argument_print_epoch_certifies_every_parameter_and_dependency",
            "i32_call_argument_print_epoch_retains_scoped_logical_parameter_boundary",
            "i32_call_argument_print_epoch_shares_capture_depth_and_read_order_bounds",
            "i32_call_argument_print_epoch_source_keeps_signed_results_and_skipped_faults",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-source-signatures",
        path: "tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch.ns",
        required_patterns: &[
            "fn checked32(value: i32, guard: i64) -> i32",
            "fn relay32(value: i32, guard: i64) -> i32",
            "return checked32(value, guard)",
            "left: i32, right: i32",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-source-read-epochs",
        path: "tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch.ns",
        required_patterns: &[
            "code: left",
            "code: right",
            "relay32(checked32(current.inner.code, first), second) < right",
            "checked32(local.inner.code, first) <= right",
            "relay32(right, second) >= left",
            "relay32(checked32(local.inner.code, second), first) != right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-independent-signed-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "trace.extend([106, i64::from(value), guard])",
            "trace.extend([107, i64::from(value), guard])",
            "counts[0] += 1",
            "counts[1] += 1",
            "i32::MIN",
            "i32::MAX",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-signed-input-probes",
        path: "tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "sext i32 %arg0 to i64",
            "sext i32 %arg10 to i64",
            "sext i32 %arg11 to i64",
            "for word in [96, 99, 106, 107]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-structural-counts-and-state",
        path: "tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch.rs",
        required_patterns: &[
            "[\"checked32\", \"relay32\"]",
            "case.helper_counts(previous)",
            "assert_calls(&event.events, *row, Some(previous))",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-explicit-signed-matrix",
        path: "tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch.rs",
        required_patterns: &[
            "i32_call_argument_print_epoch_signed_matrix_keeps_exact_input_words",
            "cases::sequences(kind).len(), 33",
            "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 16",
            "Value::I32(left)",
            "i64::from(left)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-entry-rejection",
        path: "tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch.rs",
        required_patterns: &[
            "compiled.yir.functions.len() <= 32",
            "i32_call_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/i32_call_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-receipt",
        path: "docs/reference/nuis-native-i32-call-argument-print-v1.md",
        required_patterns: &[
            "Comparison-Only Results",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-call-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Comparison-only i32 call argument print witnesses:",
            "Previous float-compare checkpoint:",
            "Previous float-compare task checkpoint:",
            "Previous float-compare blocker checkpoint:",
            "Previous float-compare action checkpoint:",
            "Previous float-compare validation checkpoint:",
            "Previous float-compare artifact checkpoint:",
        ],
    },
];
