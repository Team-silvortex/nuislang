use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-completed-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "scalar_helpers::pure_scalar_signature(callee, catalog)",
            "lazy_rhs && matches!(result.name.as_str(), \"bool\" | \"i32\" | \"i64\")",
            "params.get(index)",
            "Some(&scalar_type(\"bool\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-context",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "(expr, depth, logical, true, false)",
            "(lhs.as_ref(), depth + 1, computed_gate, false, lazy_rhs)",
            "(rhs.as_ref(), depth + 1, computed_gate, false, true)",
            "logical_argument(callee, index, lazy_rhs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-ordinary-closure",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "roots_with_fields(pending, computed_gate, |_, _| false, |_, _, _| false)",
            "record_root && bool_field(type_name, name)",
            "if !logical_root",
            "let mut logical_edges = 32usize",
            "let mut work = 4096usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/logical_helper_argument_print_epoch_tests.rs",
        required_patterns: &[
            "logical_helper_argument_print_epoch_requires_lazy_scope_and_original_reads",
            "logical_helper_argument_print_epoch_keeps_exact_parameters_and_effect_vetoes",
            "logical_helper_argument_print_epoch_shares_edge_work_and_capture_budgets",
            "logical_helper_argument_print_epoch_source_guards_nested_argument_calls",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-capture-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/logical_helper_argument_print_epoch_tests.rs",
        required_patterns: &[
            "for count in [16, 17]",
            "for count in [4096, 4097]",
            "for count in [31, 32]",
            "check(&value, &module, true, false).is_none()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-source",
        path: "tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch.ns",
        required_patterns: &[
            "fn accept(pick: bool, value: i64, divisor: i64) -> bool",
            "fn weight(pick: bool, value: i64, divisor: i64) -> i64",
            "current.inner.flag && accept(current.inner.flag && positive(",
            "positive(weight(local.inner.flag || (local.inner.tag % first > 0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-field-versions",
        path: "tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch.ns",
        required_patterns: &[
            "flag: selected == false",
            "flag: selected",
            "observe(-choose(",
            "local.inner.flag || accept(local.inner.flag || (positive(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-helper-oracle",
        path: "tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch_cases.rs",
        required_patterns: &[
            "counts[2] += 1",
            "counts[3] += 1",
            "trace.extend([103, i64::from(pick), value, divisor])",
            "trace.extend([104, i64::from(pick), value, divisor])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-native-bool-probes",
        path: "tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch_cases.rs",
        required_patterns: &[
            "[(\"accept\", 103), (\"weight\", 104)]",
            "zext i1 %arg0 to i64",
            "for word in [96, 99, 101, 102, 103, 104]",
            "pub fn helper_counts(self) -> [usize; 4]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-reference-counts",
        path: "tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch.rs",
        required_patterns: &[
            "[\"quotient\", \"positive\", \"accept\", \"weight\"]",
            "case.helper_counts()",
            "ApplicationSessionPhase::Faulted",
            "state_words(session.state())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-entry-rejection",
        path: "tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch.rs",
        required_patterns: &[
            "logical_helper_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
            "compiled.yir.functions.len() <= 32",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/logical_helper_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-receipt",
        path: "docs/reference/nuis-native-logical-helper-argument-print-v1.md",
        required_patterns: &[
            "Scoped Logical Parameters",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-helper-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Scoped logical helper argument print witnesses:",
            "Previous helper-call checkpoint:",
            "Previous helper-call task checkpoint:",
            "Previous helper-call blocker checkpoint:",
            "Previous helper-call action checkpoint:",
            "Previous helper-call validation checkpoint:",
            "Previous helper-call artifact checkpoint:",
        ],
    },
];
