use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print.ns",
        required_patterns: &[
            "fn child_count(base: i64, round: i64)", "round % 2 == 0",
            "return base + round - 1", "let child_trips = child_count(trips, outer)",
            "while child < child_trips", "initial_guard - (outer * 16 + child)",
            "lazy_guard - (outer * 16 + child)", "ready_guard - (outer * 16 + child)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-source-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "parent_bound_argument_print_accepts_ready_per_parent_counts",
            "parent_bound_argument_print_rejects_scope_escape_and_child_mutated_bounds",
            "let mut child_trips =", "child_trips = child_trips + 1",
            "module.functions.reverse()", "let alias = epoch_word",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-independent-model",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "let mut active = false", "active = !active", "count += 2",
            "let mut code = case.child.base.left", "parent_guard += 16",
            "guard_offset += 1", "for at in [49, 50]",
            "row.child.base.guards = [32, 32, 32, 1]", "zero.child.base.guards[1] = 52",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-profile-observers",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "pub const PARENT_BOUND: Matrix", "template: include_str!(\"parent_bound_argument_print.ns\")",
            "pub fn child_counts(self)", "return bounds::counts(self)",
            "return bounds::execute(self, previous, native)", "call void @word_print(i64 602)",
            "LOOP_LIMIT: u64 = 12", "ENTRY_LIMIT: u64 = 192",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "parent_bound_argument_print_native_preserves_skip_resume_and_fault_prefixes",
            "parent_bound_argument_print_reference_preserves_dynamic_counts_and_faulted_close",
            "parent_bound_argument_print_native_keeps_entry_and_exact_work_budgets",
            "parent_bound_argument_print_oracle_pins_bounds_reseeding_and_skipped_poison",
            "vec![0, 3, 0, 5]", "prefix.ends_with(&[76, 602, 3, 3])",
            "check_i64_loop_work_limit_rejection", "children + row.parents + 1",
            "(matrix.sequences)(kind).len(), 24", "(matrix.cli_sequences)(kind).len(), 8",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-budget-diagnostics",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "native helper entry limit exhausted", "native loop work limit exhausted",
            "native_loop_work_rejected", "nuis_probe_work_diagnostic_{index}",
            "pub(super) fn check_i64_loop_work_limit_rejection",
            "expected_work_prefix: Option<&[i64]>", "assert_eq!(actual, prefix",
            "assert!(actual.len() < trace.len())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-shared-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "parent_bound_argument_print_fresh_cache_and_source_free_delivery_preserve_resumed_work",
            "cases::PARENT_BOUND", "fn delivery(matrix: cases::Matrix, tag: &str)",
            "(matrix.cli_sequences)(kind)", "compile_cache: hit", "materialize-artifact",
            "fresh.host_objects == cached.host_objects", "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-receipt",
        path: "docs/reference/nuis-native-parent-bound-argument-print-v1.md",
        required_patterns: &[
            "Parent-Dependent Child Bounds", "24 raw sequences", "eight finite sequences",
            "nine fault rows", "12 units", "11 units", "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-bound-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Parent-dependent child bound witnesses:",
            "Previous child-progress checkpoint:", "Previous child-progress task checkpoint:",
            "Previous child-progress blocker checkpoint:", "Previous child-progress action checkpoint:",
            "Previous child-progress validation checkpoint:", "Previous child-progress artifact checkpoint:",
        ],
    },
];
