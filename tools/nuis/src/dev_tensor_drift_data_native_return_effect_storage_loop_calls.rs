use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-authority-and-structure",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_loop_call_tests.rs",
        required_patterns: &[
            "effect_return_storage_loop_calls_preserve_exact_work_without_new_catalog_authority",
            "effect_return_storage_loop_calls_reject_type_valid_changed_iteration_effects",
            "effect_return_storage_loop_calls_keep_candidate_fallback_and_original_read_rejection",
            "effect_return_storage_loop_oracle_distinguishes_assignment_events_from_observed_words",
            "denied.remove(callee)", "args.swap(1, 2)", "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-yir-comparison",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_loop_helper_calls_keep_real_reduction_and_selected_exits",
            "compare_source_cases(cases::SourceKind::LoopHelperCalls)",
            "maximum_carry_width(&shared) + 4", "match kind.execute(call, false, None)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-real-updates-and-failures",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_loop_calls.rs",
        required_patterns: &[
            "pub fn with_loop_calls", "if delta == \"outer\" { 3 } else { 1 }",
            "root + 11 - index", "after + 12 - index",
            "call.execute_with_assignment_effects", "trace.extend([810, 811])",
            "trace.extend([814, 812])", "call.root = if callee { 1 } else { -9 }",
            "call.after = if callee { -10 } else { 1 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-nuis-helper-definitions",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_loop_helpers.ns",
        required_patterns: &[
            "@noinline fn loop_seed", "@noinline fn loop_delta",
            "@noinline fn loop_gate", "@noinline fn loop_record",
            "let checked = 0 / root", "seed.total + delta + checked",
            "gain: seed.gain, marker: seed.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-semantic-oracle-sites",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs",
        required_patterns: &[
            "pub fn execute_with_assignment_effects",
            "&mut |_, _, _| Some(())", "assignment(8, outer, &mut trace)?",
            "assignment(10, index, &mut trace)?", "assignment(9, outer, &mut trace)?",
            "assignment(25, outer, &mut trace)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-native-state-and-later-failure",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_loop_helper_calls_reference_keeps_state_bits_and_selected_exits",
            "shared_return_storage_loop_helper_calls_native_keeps_raw_words_and_once_only_work",
            "shared_return_storage_loop_helper_calls_native_preserves_full_reached_reservations",
            "shared_return_storage_loop_helper_calls_later_argument_failure_retains_actual_state",
            "shared_return_storage_loop_helper_calls_later_callee_failure_retains_actual_state",
            "[10, 810, 811, 814, 812]", "[10, 810, 811]",
            "kind.execute(bad, true, None).unwrap_err()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_loop_helper_calls_fresh_cache_and_source_free_delivery_keeps_exits_and_faults",
            "delivery(cases::SourceKind::LoopHelperCalls)",
            "failures.push(cases::loop_calls::failure(variant, callee))",
            "for restore in 0..2", "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-loop-calls-v1.md",
        required_patterns: &[
            "Real Reached-Loop Updates", "Semantic Oracle Events", "Later Iteration Failures",
            "active/99", "not measured physical memory or runtime speedup",
            "Source-Free Delivery", "Remaining Frontier",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-loop-calls-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified reached-loop record-helper assignment witnesses:",
            "Previous effectful prefix-helper checkpoint:",
            "Previous effectful prefix-helper task checkpoint:",
            "Previous effectful prefix-helper blocker checkpoint:",
            "Previous effectful prefix-helper action checkpoint:",
            "Previous effectful prefix-helper validation checkpoint:",
            "Previous effectful prefix-helper artifact checkpoint:",
        ],
    },
];
