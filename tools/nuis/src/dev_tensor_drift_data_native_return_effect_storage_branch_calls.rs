use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-authority-and-exact-selection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_branch_call_tests.rs",
        required_patterns: &[
            "effect_return_storage_branch_calls_keep_catalog_authority_and_immutable_source",
            "effect_return_storage_branch_calls_reject_typed_hoisting_swaps_duplicates_and_argument_changes",
            "effect_return_storage_branch_calls_preserve_candidate_fallback_and_original_read_rejection",
            "effect_return_storage_branch_source_and_oracle_preserve_semantic_roles_under_mirroring",
            "denied.remove(&callee)", "std::mem::swap(then_body, else_body)",
            "args.swap(1, 2)", "body.insert(index, hoisted)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-independent-yir-comparison",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_branch_helper_calls_keep_multifield_values_and_unselected_poison_inert",
            "compare_source_cases(cases::SourceKind::BranchHelperCalls)",
            "maximum_carry_width(&shared) + 4", "match kind.execute(call, false, None)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-semantic-source-adapter-and-poison",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_branch_calls.rs",
        required_patterns: &[
            "pub fn with_branches", "call.contains(\"loop_delta(index + 1,\")",
            "0 / ({safe})", "if {selector} % 2 != 0", "if {selector} % 2 == 0",
            "let base = if index % 2 == 0 { 910 } else { 920 }",
            "call.root.checked_add(21)", "call.after.checked_add(22)",
            "let index = if odd { 3 } else { 2 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-distinct-nuis-field-transforms",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_branch_helpers.ns",
        required_patterns: &[
            "@noinline fn branch_even_record", "@noinline fn branch_odd_seed",
            "@noinline fn branch_odd_record", "result: choice != true",
            "gain: -seed.gain", "return State { marker: -seed.marker, gain: seed.gain",
            "total: delta + seed.total + checked, result: seed.result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-semantic-events-and-raw-aliases",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs",
        required_patterns: &[
            "BranchHelperCalls", "return branch_calls::execute(call, observed, budget)",
            "Self::LoopHelperCalls | Self::BranchHelperCalls",
            "loop effects require semantic oracle hooks", "[(8, 9), (10, 8)]",
            "Self::BranchHelperCalls => 128", "_ => 100",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-native-state-failures-and-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_branch_helper_calls_reference_keeps_multifield_bits_and_selection",
            "shared_return_storage_branch_helper_calls_native_keeps_unselected_poison_inert",
            "shared_return_storage_branch_helper_calls_native_preserves_full_reached_reservations",
            "shared_return_storage_branch_helper_calls_selected_argument_failures_keep_actual_state",
            "shared_return_storage_branch_helper_calls_selected_callee_failures_keep_actual_state",
            "shared_return_storage_branch_helper_calls_entry_limit_100_still_rejects_long_callback",
            "cases::branch_calls::failure(variant, callee, odd)",
            "vec![10, base, base + 1, base + 4, base + 2]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-entry-budget-diagnostics-and-rejection",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "native helper entry limit exhausted", "check_helper_entry_limit_rejection",
            "trace.starts_with(&actual)", "actual.len() < trace.len()",
            "reversed={reversed} status={:?} stderr={} stdout_tail={}",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_branch_helper_calls_fresh_cache_and_source_free_delivery_keeps_selection",
            "delivery(cases::SourceKind::BranchHelperCalls)",
            "failures.push(cases::branch_calls::failure(variant, callee, odd))",
            "for restore in 0..2", "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-branch-calls-v1.md",
        required_patterns: &[
            "Branch-Selected Multi-Field Updates", "Unselected Poison And Semantic Roles",
            "Selected Later Failures", "not measured physical memory or runtime speedup",
            "active/99", "Source-Free Delivery", "Remaining Frontier",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-branch-calls-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified branch-selected multi-field record-helper witnesses:",
            "Previous reached-loop helper checkpoint:",
            "Previous reached-loop helper task checkpoint:",
            "Previous reached-loop helper blocker checkpoint:",
            "Previous reached-loop helper action checkpoint:",
            "Previous reached-loop helper validation checkpoint:",
            "Previous reached-loop helper artifact checkpoint:",
        ],
    },
];
