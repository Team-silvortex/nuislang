use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-scoped-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "roots_with_fields(pending, computed_gate, false, |_, _| false, |_, _, _| false)",
            "roots_with_fields(\n        pending,\n        true,\n        true,",
            "roots_with_fields(\n        pending,\n        true,\n        false,",
            "computed_bool_operands: bool",
            "computed_bool_operands\n                    && logical_root",
            "matches!(op, NirBinaryOp::Eq | NirBinaryOp::Ne | NirBinaryOp::Xor)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-eager-context",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "Eager bool operators evaluate both operands without authorizing a lazy RHS.",
            "(lhs.as_ref(), depth + 1, eager_bool, false, lazy_rhs)",
            "(rhs.as_ref(), depth + 1, eager_bool, false, lazy_rhs)",
            "let mut logical_edges = 32usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-proof-and-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "pending.push(Step::Binary(*op, lazy_rhs))",
            "Step::Binary(op, lazy_rhs)",
            "control_values::binary_type(op, left.ty, right.ty)?",
            "right.captures.len() > 31",
            "if proof.comparison_only && &proof.ty != result",
            "let right = lazy_rhs || matches!(op, NirBinaryOp::And | NirBinaryOp::Or)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_equality_argument_print_epoch_tests.rs",
        required_patterns: &[
            "bool_equality_argument_print_epoch_keeps_exact_types_epochs_and_profiles",
            "bool_equality_argument_print_epoch_rejects_mixed_effect_missing_and_provenance_escapes",
            "bool_equality_argument_print_epoch_visits_both_operands_in_source_order",
            "bool_equality_argument_print_epoch_checks_rhs_even_after_constant_lhs",
            "bool_equality_argument_print_epoch_bounds_shared_edges_work_depth_and_capture_union",
            "bool_equality_argument_print_epoch_source_preserves_eager_operands_and_nested_guards",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-shared-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_equality_argument_print_epoch_tests.rs",
        required_patterns: &[
            "for count in [31, 32]",
            "for count in [4096, 4097]",
            "vec![(&value, 64, false)]",
            "computed_record_expression_roots",
            "assert_eq!(actual, arguments)",
            "assert_eq!(picks, expected)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-source-epochs",
        path: "tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch.ns",
        required_patterns: &[
            "== ((current.inner.code < right) || (checked32(right, second) > current.inner.code))",
            "!= ((local.inner.code == left) && (checked32(left, first) <= right))",
            "code: left",
            "code: right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-independent-operands",
        path: "tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch_cases.rs",
        required_patterns: &[
            "let pick = left == right",
            "let pick = left != right",
            concat!(
                "checked32(\n                                self.right,\n",
                "                                self.second,\n                                trace,\n",
                "                                native,\n                                counts,\n",
                "                            )"
            ),
            "checked32(self.left, self.first, trace, native, counts)",
            "counts[0] += 1",
            "counts[1] += 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-actual-word-probes",
        path: "tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch_cases.rs",
        required_patterns: &[
            "zext i1 %arg0 to i64",
            "sext i32 %arg1 to i64",
            "sext i32 %arg2 to i64",
            "for word in [96, 99, 106, 107, 108]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-matrix-and-state",
        path: "tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch.rs",
        required_patterns: &[
            "case.helper_counts(previous)",
            "cases::sequences(kind).len(), 35",
            "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 16",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-suppressed-and-reached-poison",
        path: "tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch_cases.rs",
        required_patterns: &[
            "skipped_argument.first = 0",
            "skipped_or.second = 0",
            "bad.base.row.trips = 2",
            "bad.left = if second == 0 { bad.right } else { -1 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-entry-budget",
        path: "tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch.rs",
        required_patterns: &[
            "compiled.yir.functions.len() <= 32",
            "bool_equality_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/bool_equality_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-receipt",
        path: "docs/reference/nuis-native-bool-equality-argument-print-v1.md",
        required_patterns: &[
            "Eager Comparisons, Guarded Operands",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-equality-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Computed bool equality argument print witnesses:",
            "Previous logical-i32 checkpoint:",
            "Previous logical-i32 task checkpoint:",
            "Previous logical-i32 blocker checkpoint:",
            "Previous logical-i32 action checkpoint:",
            "Previous logical-i32 validation checkpoint:",
            "Previous logical-i32 artifact checkpoint:",
        ],
    },
];
