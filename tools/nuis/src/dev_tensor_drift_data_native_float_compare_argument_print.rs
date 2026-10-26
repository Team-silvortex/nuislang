use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-exact-typed-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "control_values::binary_type(op, left.ty, right.ty)?",
            "proofs[0].ty == *expected",
            "pending.push(Step::Value(rhs, child, right))",
            "pending.push(Step::Value(lhs, child, lazy_rhs))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-helper-necessity",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "ready_atom: bool",
            "ready_predicate: bool",
            "(!right.total || !right.ready_predicate)",
            "right.captures.len() > 31",
            "left.ready_atom && right.ready_atom",
            "left.ready_predicate && right.ready_predicate",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-original-read-grant",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "binary_float::validate(value, expected, depth, catalog",
            ".project_type(value, scope, layouts)",
            ".or_else(|| ready(value, depth))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/float_compare_argument_print_epoch_tests.rs",
        required_patterns: &[
            "float_compare_argument_print_epoch_proves_six_exact_bool_results_and_scoped_reads",
            "float_compare_argument_print_epoch_rejects_mixed_cast_effect_and_missing_operands",
            "float_compare_argument_print_epoch_bounds_complete_capture_union_and_original_order",
            "float_compare_argument_print_epoch_distinguishes_total_values_from_helper_free_predicates",
            "float_compare_argument_print_epoch_raw_ieee_results_keep_operand_calls_once_only",
            "float_compare_argument_print_epoch_reaches_only_selected_operand_faults",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-boolean-observations",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/float_compare_argument_print_epoch_tests.rs",
        required_patterns: &[
            "expected_picks.push(pick)",
            "assert_eq!(actual_picks, expected_picks",
            "ready_values::total_predicate(&rhs, &frame)",
            "for count in [31, 32]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-source-read-epochs",
        path: "tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch.ns",
        required_patterns: &[
            "sample(current.inner.total, first) == sample(ready, second)",
            "sample(ready, first) < local.inner.total",
            "local.inner.total <= sample(ready, second)",
            "local.inner.total > sample(ready, first)",
            "sample(local.inner.total, second) >= ready",
            "sample(local.inner.total, first) != sample(ready, second)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-independent-ieee-order",
        path: "tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch_cases.rs",
        required_patterns: &[
            "pub fn ieee_compare",
            "let unordered = nan(left) || nan(right)",
            "left & !sign == 0 && right & !sign == 0",
            "!v & mask",
            "Comparison::Ne => !equal",
            "Comparison::Ge => !unordered && !less",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-raw-operand-probe",
        path: "tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch_cases.rs",
        required_patterns: &[
            "counts[4] += 1",
            "trace.extend([105, value as i64, divisor])",
            "bodies.push((\"sample\", sample))",
            "for word in [96, 99, 101, 102, 103, 104, 105]",
            "comparison_pairs(kind)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-state-dependent-counts",
        path: "tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch.rs",
        required_patterns: &[
            "case.helper_counts(previous)",
            "assert_calls(&event.events, *row, Some(previous))",
            "ApplicationSessionPhase::Faulted",
            "\"sample\"",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-oracle-truth-table",
        path: "tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch.rs",
        required_patterns: &[
            "float_compare_argument_print_epoch_unsigned_oracle_keeps_ieee_truth_table",
            "cases::COMPARISONS",
            "cases::ieee_compare(kind, left, right, op)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-entry-rejection",
        path: "tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch.rs",
        required_patterns: &[
            "compiled.yir.functions.len() <= 32",
            "float_compare_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/float_compare_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-receipt",
        path: "docs/reference/nuis-native-float-compare-argument-print-v1.md",
        required_patterns: &[
            "Exact Comparison Operands",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-compare-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Float comparison argument print witnesses:",
            "Previous logical-helper checkpoint:",
            "Previous logical-helper task checkpoint:",
            "Previous logical-helper blocker checkpoint:",
            "Previous logical-helper action checkpoint:",
            "Previous logical-helper validation checkpoint:",
            "Previous logical-helper artifact checkpoint:",
        ],
    },
];
