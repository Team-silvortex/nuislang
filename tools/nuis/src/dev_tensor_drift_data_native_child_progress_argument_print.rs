use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/child_progress_argument_print.ns",
        required_patterns: &[
            "while outer < parents", "while child < trips",
            "initial_guard - (outer * trips + child)",
            "lazy_guard - (outer * trips + child)",
            "ready_guard - (outer * trips + child)",
            "let mut current = Pair { total: total, code: left, flag: selected }",
            "current.flag && (checked64(epoch_word,", "child = child + 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-source-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "child_progress_argument_print_accepts_ready_child_induction",
            "child_progress_argument_print_rejects_unready_and_escaping_entries",
            "module.functions.reverse()", "let alias = epoch_word",
            "for bad in [", "if selected { let epoch_word: i64 =",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-independent-model",
        path: "tools/nuisc/tests/native_application_bridge/child_progress_argument_print_cases.rs",
        required_patterns: &[
            "let mut completed = 0", "let mut code = case.child.base.left",
            "row.base.left = code", "row.base.trips = 1", "*guard -= completed",
            "code = case.child.base.right", "completed += 1",
            "&child_trace[2..child_trace.len() - 1]", "trace.push(77)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-late-fault-matrix",
        path: "tools/nuisc/tests/native_application_bridge/child_progress_argument_print_cases.rs",
        required_patterns: &[
            "for guard in 0..3", "for after in [4, 5]",
            "eager.child.base.selected = false", "eager.child.base.guards[0] = 7",
            "i64::from(zero.child.base.right) + 4294967296",
            "zero.child.base.guards[1] = 7", "tail.child.base.guards[3] = 0",
            "row.child.base.guards[1] = 5",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "child_progress_argument_print_native_preserves_completed_siblings_and_fault_prefixes",
            "child_progress_argument_print_reference_preserves_counts_state_and_faulted_close",
            "child_progress_argument_print_native_keeps_entry_exhaustion_before_publication",
            "child_progress_argument_print_oracle_pins_late_children_and_lazy_faults",
            "[4, 5, 4, 5, 4, 5, 7, 7, 9]", "row.words().len(), 11",
            "trace.ends_with(&[601, 0, 0])", "native_sequences(cases::PARENT)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-shared-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "child_progress_argument_print_fresh_cache_and_source_free_delivery_preserve_late_faults",
            "fn delivery(matrix: cases::Matrix, tag: &str)",
            "(matrix.failures)(kind)", "(matrix.cli_sequences)(kind)",
            "matrix.template.replace(\"f64\", kind.ty())", "cases::CHILD_PROGRESS",
            "compile_cache: hit", "materialize-artifact", "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-receipt",
        path: "docs/reference/nuis-native-child-progress-argument-print-v1.md",
        required_patterns: &[
            "Later Children In Later Parents", "22 raw sequences", "six finite sequences",
            "nine fault rows", "completed siblings", "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-progress-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Later-child epoch-local argument print witnesses:",
            "Previous parent-reentry checkpoint:", "Previous parent-reentry task checkpoint:",
            "Previous parent-reentry blocker checkpoint:", "Previous parent-reentry action checkpoint:",
            "Previous parent-reentry validation checkpoint:", "Previous parent-reentry artifact checkpoint:",
        ],
    },
];
