use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.ns",
        required_patterns: &[
            "while outer < parents", "total = finish(total, 1)",
            "let mut current = Pair { total: total, code: left, flag: selected }",
            "while child < trips", "initial_guard - outer", "lazy_guard - outer",
            "ready_guard - outer", "print(76)", "print(77)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-source-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "parent_reentry_argument_print_accepts_dynamic_parent_entries",
            "parent_reentry_argument_print_rejects_unready_and_escaping_entries",
            "module.functions.reverse()", "for count in [0, 2, 3]",
            "let alias = epoch_word", "if selected { let epoch_word: i64 =",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-independent-model",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "LOOP_LIMIT: u64 = 12", "ENTRY_LIMIT: u64 = 192",
            "words.push(self.parents)", "for parent in 0..self.parents",
            "let mut row = self.child", "*guard -= parent", "row.base.guards[3] = 1",
            "&child_trace[2..child_trace.len() - 1]", "trace.push(77)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-native-state",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "parent_reentry_argument_print_native_preserves_eager_initializers_and_fault_prefixes",
            "parent_reentry_argument_print_reference_preserves_counts_state_and_faulted_close",
            "check_sequences_with_i64_prints", "ApplicationSessionPhase::Faulted",
            "children + row.parents + 1",
            "parent_reentry_argument_print_native_keeps_entry_exhaustion_before_publication",
            "[(2, 2, 8), (3, 3, 64)]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-oracle-pins",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "parent_reentry_argument_print_oracle_pins_reseeding_skips_and_late_faults",
            "cases::sequences(kind).len(), 22", "cases::cli_sequences(kind).len(), 6",
            "cases::failures(kind).len(), 6", "(4294967295, 3)", "(4294967303, 3)",
            "trace.ends_with(&[601, 0, 0])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "parent_reentry_argument_print_fresh_cache_and_source_free_delivery_preserve_initializers",
            "fresh.host_objects == cached.host_objects", "fresh.lifecycle, cached.lifecycle",
            "compile_cache: hit", "for i in 0..2", "materialize-artifact",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "trace.starts_with(&String::from_utf8(failed.stdout).unwrap())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-receipt",
        path: "docs/reference/nuis-native-parent-reentry-argument-print-v1.md",
        required_patterns: &[
            "Repeated Parent Entry", "22 raw sequences", "six finite sequences",
            "six fault rows", "12 shared loop-work units", "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-parent-reentry-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Repeated parent epoch-local argument print witnesses:",
            "Previous child-owned checkpoint:", "Previous child-owned task checkpoint:",
            "Previous child-owned blocker checkpoint:", "Previous child-owned action checkpoint:",
            "Previous child-owned validation checkpoint:", "Previous child-owned artifact checkpoint:",
        ],
    },
];
