use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_break_argument_print.ns",
        required_patterns: &[
            "if child == cut { print(78); break; }", "print(79);",
            "print(80);", "print(child);", "bound_guard: i64, cut: i64",
            "child_count(trips, outer, bound_guard - outer, total < 0.0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-source-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_break_argument_print_accepts_post_update_break_and_observed_index",
            "state_break_argument_print_keeps_suffix_owner_and_bound_refusals",
            "break; print(81);", "child_trips = child_trips + 1",
            ".replace(exit, \"\")", "let __nuis_buffer_break: i64 = 0; print(71);",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-work-model",
        path: "tools/nuisc/tests/native_application_bridge/state_break_argument_print_cases.rs",
        required_patterns: &[
            "pub requested: i64", "pub completed: i64", "cut >= 0 && cut < requested",
            "for _ in 0..completed", "active = !active", "[49, 49, 49, 1]",
            "before_break.child.base.guards[2] = 48", "GuardClock::StateBreak(3, 0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-index-model",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "for child_index in 0..trips", "if child_index == cut",
            "observed_index = child_index;", "trace.push(78)", "trace.push(79)",
            "trace.extend([80, observed_index])", "observed_index = child_index + 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-profile",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "pub const STATE_BREAK: Matrix", "StateBreak(i64, i64)",
            "pub fn completed_child_counts_from", "work.requested", "work.completed",
            "words.push(cut)", "observers: STATE_BOUND.observers",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-helper-counts",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "mod state_break_argument_print", "row.completed_child_counts_from(previous)",
            "completed.into_iter().sum()", "done <= bound",
            "children + row.parents + 1", "check_calls(*row, Some(previous), &event.events)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/state_break_argument_print.rs",
        required_patterns: &[
            "state_break_argument_print_native_preserves_exits_indices_and_faults",
            "state_break_argument_print_reference_counts_completed_not_reserved_children",
            "state_break_argument_print_native_reserves_before_early_exit_without_refund",
            "state_break_argument_print_oracle_pins_completion_indices_and_skipped_poison",
            "vec![0, 1, 0, 1]", "prefix.ends_with(&[76, 602, 3, 3, 2, 1])",
            "(matrix.failures)(kind).len(), 15", "row.words().len(), 13",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_break_argument_print_fresh_cache_and_source_free_delivery_preserve_early_exits",
            "cases::STATE_BREAK", "compile_cache: hit", "materialize-artifact", "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-normalization-origin",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_flow.rs",
        required_patterns: &[
            "entry_prologue: Vec<NirStmt>", "pub(super) fn entry_blocks",
            "std::ptr::eq(body, self.effects.as_slice())", "body.starts_with(&self.entry_prologue)",
            "let entry_prologue = body[..prologue_len].to_vec()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-normalization-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_plan_tests.rs",
        required_patterns: &[
            "epoch_local_plan_normalized_entry_requires_original_generated_prologue",
            "!check(&flow.effects.clone(), Some(&flow))", "!check(&flow.effects[1..], Some(&flow))",
            "!check(&forged.effects, Some(&forged))", "flow.effects.clear()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-fault-classification",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "enum FailureDiagnostic", "FailureDiagnostic::Source", "FailureDiagnostic::Loop(expected)",
            "check_sequences_with_loop_rejections", "LoopRejection::Work", "LoopRejection::Induction",
            "native loop induction rejected", "assert_eq!(failures.len(), expected.len())",
            "assert_eq!(actual, expected,", "assert_eq!(actual, prefix,",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-receipt",
        path: "docs/reference/nuis-native-state-break-argument-print-v1.md",
        required_patterns: &[
            "State-Bounded Early Breaks", "27 raw sequences", "eight finite sequences",
            "15 fault rows", "reserved work", "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-break-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-bounded early-break witnesses:", "Previous state-bound checkpoint:",
            "Previous state-bound task checkpoint:", "Previous state-bound blocker checkpoint:",
            "Previous state-bound action checkpoint:", "Previous state-bound validation checkpoint:",
            "Previous state-bound artifact checkpoint:",
        ],
    },
];
