use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_grouped_argument_print.ns",
        required_patterns: &[
            "&& (confirm_continue(child, permit",
            "|| rescue_continue(child, rescue",
            "if (break_at(child, stop_at",
            "&& confirm_break(child, seal",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_grouped_argument_print_accepts_explicit_logical_groups",
            "state_grouped_argument_print_keeps_step_type_and_scope_refusals",
            "function `rescue_continue` argument 1 expects `i64`",
            "function `confirm_break` argument 2 expects `i64`",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-independent-model",
        path: "tools/nuisc/tests/native_application_bridge/state_grouped_argument_print_cases.rs",
        required_patterns: &[
            "pub fn continue_index",
            "pub fn first_break",
            "pub fn call_counts",
            "counts[4] += usize::from(a && !b)",
            "call(607, self.rescue, self.rescue_guard)?",
            "call(608, self.seal, self.seal_guard)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-dispatch",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.grouped()",
            "case.short_circuit()",
            "exits.evaluate(",
            "Err(()) => return Err(trace)",
            "decisions.map_or(child_index == cut, |(_, stopping)| stopping)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_grouped_argument_print.rs",
        required_patterns: &[
            "state_grouped_argument_print_native_observes_regrouped_faults",
            "state_grouped_argument_print_native_keeps_changed_work_and_fault_priority",
            "state_grouped_argument_print_oracle_pins_truth_tables_and_abi",
            "row.words().len(), 25",
            "(matrix.failures)(kind).len(), 20",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-helper-counts",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "row.grouped_calls_from(previous)",
            "calls(\"] rescue_continue(\")",
            "calls(\"] confirm_break(\")",
            "cases::GuardClock::StateGrouped { .. }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_grouped_argument_print_fresh_cache_and_source_free_delivery_preserve_grouping",
            "cases::STATE_GROUPED",
            "compile_cache: hit",
            "for i in 0..2",
            "materialize-artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs",
        required_patterns: &[
            "mod state_grouped_argument_print;",
            "pub(super) const STATE_GROUPED_ARGUMENT_PRINT",
            "state_grouped_argument_print::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-receipt",
        path: "docs/reference/nuis-native-state-grouped-argument-print-v1.md",
        required_patterns: &[
            "Grouped Exit Conditions",
            "31 raw sequences",
            "twelve finite sequences",
            "20 fault rows",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-grouped-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded grouped-exit witnesses:",
            "Previous state-short-circuit checkpoint:",
            "Previous state-short-circuit task checkpoint:",
            "Previous state-short-circuit blocker checkpoint:",
            "Previous state-short-circuit action checkpoint:",
            "Previous state-short-circuit validation checkpoint:",
            "Previous state-short-circuit artifact checkpoint:",
        ],
    },
];
