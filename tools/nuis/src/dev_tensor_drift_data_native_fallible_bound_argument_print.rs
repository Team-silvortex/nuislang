use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/fallible_bound_argument_print.ns",
        required_patterns: &[
            "fn child_count(base: i64, round: i64, guard: i64)",
            "let checked = 0 / guard;", "round % 2 == 0",
            "child_count(trips, outer, bound_guard - outer)",
            "while child < child_trips", "parents: i64, bound_guard: i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-source-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "fallible_bound_argument_print_accepts_reached_producer_checks",
            "fallible_bound_argument_print_rejects_unready_and_mutated_bounds",
            "let mut child_trips =", "child_trips = child_trips + 1",
            "module.functions.reverse()", "let alias = epoch_word",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-fault-matrix",
        path: "tools/nuisc/tests/native_application_bridge/fallible_bound_argument_print_cases.rs",
        required_patterns: &[
            "GuardClock::FallibleBound(0)", "GuardClock::FallibleBound(-1)",
            "(0..4)", "all_zero.child.base.trips = 0",
            "all_zero.clock = GuardClock::FallibleBound(1)",
            "rows.extend(bounds::failures(kind).into_iter().map(guarded))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-model-order",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "GuardClock::FallibleBound(guard)", "let remaining = guard - parent as i64",
            "trace.push(remaining)", "if remaining == 0", "return Err(trace)",
            "for child_index in 0..trips", "let mut active = false", "active = !active",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-profile-observers",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "pub const FALLIBLE_BOUND: Matrix", "FallibleBound(i64)",
            "words.push(guard)", "args.push_str(&format!(\",{guard}\"))",
            "call void @word_print(i64 %arg2)",
            "LOOP_LIMIT: u64 = 12", "ENTRY_LIMIT: u64 = 192",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "fallible_bound_argument_print_native_preserves_producer_fault_prefixes",
            "fallible_bound_argument_print_reference_preserves_last_state_and_faulted_close",
            "fallible_bound_argument_print_native_orders_producer_before_child_reservation",
            "fallible_bound_argument_print_oracle_pins_reached_zero_bounds_and_abi_isolation",
            "prefix.ends_with(&[76, 602, 3, 3, 2])", "(matrix.failures)(kind).len(), 14",
            "assert_eq!(row.words().len(), 12)", "assert_eq!(old.words().len(), 11)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-shared-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "fallible_bound_argument_print_fresh_cache_and_source_free_delivery_preserve_producer_faults",
            "cases::FALLIBLE_BOUND", "compile_cache: hit", "materialize-artifact",
            "fresh.host_objects == cached.host_objects", "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-fault-discrimination",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "!stderr.contains(\"native helper entry limit exhausted\")",
            "FailureDiagnostic::Source", "LoopRejection::Work", "LoopRejection::Induction",
            "assert_eq!(actual, expected, \"reversed={reversed} mode={mode}\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-receipt",
        path: "docs/reference/nuis-native-fallible-bound-argument-print-v1.md",
        required_patterns: &[
            "Fallible Bound Producers", "24 raw sequences", "eight finite sequences",
            "14 fault rows", "11 units", "Predecessor Lineage",
            "not a failed-buffer memory snapshot",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-fallible-bound-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Fallible child bound witnesses:", "Previous parent-bound checkpoint:",
            "Previous parent-bound task checkpoint:", "Previous parent-bound blocker checkpoint:",
            "Previous parent-bound action checkpoint:", "Previous parent-bound validation checkpoint:",
            "Previous parent-bound artifact checkpoint:",
        ],
    },
];
