use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-completed-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "scalar_helpers::pure_scalar_signature(callee, catalog)",
            "lazy_rhs && matches!(result.name.as_str(), \"bool\" | \"i32\" | \"i64\")",
            "params.get(index)",
            "Some(&scalar_type(\"bool\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-private-context",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "(expr, depth, logical, true, false)",
            "logical_argument(callee, index, lazy_rhs)",
            "roots_with_fields(pending, computed_gate, false, |_, _| false, |_, _, _| false)",
            "let mut logical_edges = 32usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-result-provenance",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "comparison_only |= proof.comparison_only",
            "if proof.comparison_only && &proof.ty != result",
            "comparison_only: comparison_only || result == &scalar_type(\"i32\")",
            "(!right.total || !right.ready_predicate)",
            "right.captures.len() > 31",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/logical_i32_call_argument_print_epoch_tests.rs",
        required_patterns: &[
            "logical_i32_call_argument_print_epoch_proves_scoped_bool_parameters_and_original_reads",
            "logical_i32_call_argument_print_epoch_cannot_launder_comparison_only_results",
            "logical_i32_call_argument_print_epoch_requires_exact_positions_and_complete_dependencies",
            "logical_i32_call_argument_print_epoch_keeps_nested_i32_argument_provenance",
            "logical_i32_call_argument_print_epoch_shares_argument_edge_work_depth_and_capture_budgets",
            "logical_i32_call_argument_print_epoch_source_keeps_argument_short_circuit_and_signed_results",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-shared-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/logical_i32_call_argument_print_epoch_tests.rs",
        required_patterns: &[
            "for count in [16, 17]",
            "for count in [4096, 4097]",
            "for count in [31, 32]",
            "is_ref: true",
            "assert_eq!(actual_args, bool_args)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-source-read-epochs",
        path: "tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch.ns",
        required_patterns: &[
            "fn select32(pick: bool, value: i32, other: i32, guard: i64) -> i32",
            "(current.inner.code <= right) && (checked32(current.inner.code, first) != right)",
            "(local.inner.code != right) || (checked32(local.inner.code, second) == right)",
            "code: left",
            "code: right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-independent-selector",
        path: "tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            concat!(
                "trace.extend([\n            108,\n            i64::from(pick),\n",
                "            i64::from(value),\n            i64::from(other),\n",
                "            guard,\n        ])"
            ),
            "Some(if pick { value } else { other })",
            "counts[0] += 1",
            "counts[1] += 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-actual-bool-and-signed-probes",
        path: "tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "zext i1 %arg0 to i64",
            "sext i32 %arg1 to i64",
            "sext i32 %arg2 to i64",
            "for word in [96, 99, 106, 107, 108]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-skipped-inner-poison",
        path: "tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "skipped_argument.left = i32::MAX",
            "skipped_argument.right = i32::MIN",
            "skipped_argument.first = 0",
            "skipped_or.second = 0",
            "(108, -108)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-matrix-and-counts",
        path: "tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch.rs",
        required_patterns: &[
            "[\"checked32\", \"select32\"]",
            "case.helper_counts(previous)",
            "cases::sequences(kind).len(), 35",
            "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 16",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-entry-rejection",
        path: "tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch.rs",
        required_patterns: &[
            "compiled.yir.functions.len() <= 32",
            "logical_i32_call_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/logical_i32_call_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-receipt",
        path: "docs/reference/nuis-native-logical-i32-call-argument-print-v1.md",
        required_patterns: &[
            "Scoped Logical Parameters",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-i32-call-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Scoped logical i32 call argument print witnesses:",
            "Previous i32-call checkpoint:",
            "Previous i32-call task checkpoint:",
            "Previous i32-call blocker checkpoint:",
            "Previous i32-call action checkpoint:",
            "Previous i32-call validation checkpoint:",
            "Previous i32-call artifact checkpoint:",
        ],
    },
];
