use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-authority-and-exact-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_bound_call_tests.rs",
        required_patterns: &[
            "effect_return_storage_branch_bound_calls_keep_catalog_authority_and_immutable_source",
            "effect_return_storage_branch_bound_calls_reject_typed_bypasses_reordering_and_changed_arguments",
            "effect_return_storage_branch_bound_calls_preserve_fallback_and_original_read_rejection",
            "denied.remove(callee)", "if callee == \"extent\"", "assert_eq!(module, before)",
            "args[0] = NirExpr::Var(\"carry\".into())", "body.swap(index - 1, index)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-independent-state-dependent-execution",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_branch_bound_calls_use_completed_record_and_keep_real_carry_reduction",
            "compare_source_cases(cases::SourceKind::BranchBoundCalls)",
            "calls.push(cases::bound_calls::state_dependent(variant))",
            "cases::bound_calls::failure(variant, stage)", "maximum_carry_width(&shared) + 4",
            "(-9223372036854775807 - 1)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-real-nuis-record-and-width",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_helpers.ns",
        required_patterns: &[
            "@noinline fn bound_seed", "let parity = seed.total % 2",
            "let scale = 1 + parity * parity", "total: trips / scale + checked",
            "@noinline fn bound_extent", "return extent(seed.total + checked, divisor)",
            "print(930)", "print(931)", "print(932)", "print(933)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-semantic-hook-and-actual-state-chain",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_calls.rs",
        required_patterns: &[
            "bound_extent(bound_seed(carry, trips, root + 31 - outer), divisor, after + 32 - outer)",
            "call.execute_with_hooks(", "let parity = state.total % 2",
            "previous = Some(execute(*call, false, None).unwrap().0)",
            "pub fn state_dependent", "pub fn failure_prefix", "numerator.checked_div(call.divisor)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-reached-only-oracle-order",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs",
        required_patterns: &[
            "pub fn execute_with_hooks", "self.trips.checked_div(self.divisor)",
            "let width = bound(current, outer, &mut trace)?",
            "if !reserve(width, &mut budget)", "assignment(9, outer, &mut trace)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-mode-isolation-and-entry-configuration",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs",
        required_patterns: &[
            "BranchBoundCalls", "Self::BranchBoundCalls => 160", "Self::BranchHelperCalls => 128",
            "_ => 100", "return bound_calls::execute(call, observed, budget)",
            "rows = rows.into_iter().map(bound_calls::sequence).collect()", "pub fn next(self, call: Call)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-native-once-only-work-and-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_tests.rs",
        required_patterns: &[
            "branch_bound_calls_reference_keeps_actual_state_and_selected_work",
            "branch_bound_calls_native_keeps_once_only_evaluation_and_state_bits",
            "branch_bound_calls_native_preserves_reached_reservations",
            "branch_bound_calls_native_reserves_computed_width_before_any_child_effect",
            "trace.ends_with(&[width, call.divisor, 16, 7])", "for limit in [4, 5]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-selected-failure-and-last-state",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_tests.rs",
        required_patterns: &[
            "branch_bound_calls_argument_callee_and_division_failures_keep_last_state",
            "failure_prefix(variant, stage == 3)", "for stage in 0..4",
            "&[6, 930, 931, 932, 933, 16]", "ApplicationSessionPhase::Faulted",
            "assert!(session.completion_status().is_err())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "branch_bound_calls_fresh_cache_and_source_free_delivery_keep_computed_work_and_faults",
            "delivery(cases::SourceKind::BranchBoundCalls)", "kind.next(*call).text(\"event\")",
            "failures.push(cases::bound_calls::failure(variant, stage))",
            "for restore in 0..2", "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-bound-calls-v1.md",
        required_patterns: &[
            "Completed Records Determine Work", "Reached-Only Evaluation And Reservations",
            "Independent State And Failure Boundaries", "Native And Source-Free Delivery",
            "not measured physical memory or runtime speedup", "active/99", "Remaining Frontier",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bound-calls-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified completed-record child-bound witnesses:",
            "Previous branch-selected helper checkpoint:",
            "Previous branch-selected helper task checkpoint:",
            "Previous branch-selected helper blocker checkpoint:",
            "Previous branch-selected helper action checkpoint:",
            "Previous branch-selected helper validation checkpoint:",
            "Previous branch-selected helper artifact checkpoint:",
        ],
    },
];
