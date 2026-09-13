use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-session-guarded-break-source-normalization",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_flow.rs",
        required_patterns: &[
            "let break_only = breaking.is_some() && !contains_continue(effects)",
            "let running_value = i64::from(!break_only)",
            "if breaking != Some(flag)",
            "rhs: Box::new(NirExpr::Int(running_value))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-guarded-break-scalar-source-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &[".filter(|plan| plan.has_store || plan.break_flag.is_some())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-guarded-break-parity-and-rejections",
        path: "tools/nuisc/tests/native_application_bridge/break_loops.rs",
        required_patterns: &[
            "guarded_break_scoped_calls_preserve_native_reference_and_typed_session_parity",
            "guarded_break_keeps_shared_layout_kinds_closure_and_general_call_rejections",
            "guarded_break_reference_fuel_failure_keeps_accepted_state",
            "scalar_break_source_preserves_provenance_and_rejects_unmodeled_updates",
            "assert_native_parity(BREAKS, true)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-guarded-break-execution-and-control",
        path: "tools/nuisc/tests/native_application_bridge/break_execution.rs",
        required_patterns: &[
            "guarded_break_native_calls_commit_carries_and_release_aggregates_before_exit",
            "guarded_break_rejects_nonzero_control_seeds_even_on_zero_trips",
            "guarded_break_rejects_invalid_returned_controls_without_exporting_state",
            "guarded_break_does_not_bypass_the_full_induction_preflight",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-guarded-break-source-nested-scope",
        path: "tools/nuisc/tests/native_application_bridge/break_source.rs",
        required_patterns: &[
            "scalar_break_source_keeps_current_skips_suffix_and_isolates_nested_exits",
            "ApplicationSession::open_registered",
            "assert_eq!(state_words(session.state()), result)",
            "for multi_state in [false, true]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-guarded-break-frontdoor-restoration",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "break_loops.ns",
            "native_guarded_break_build_cache_and_standalone_relocation",
            "loop_break_control_invalid",
            "check_workflow(BREAK_SOURCE)",
            "branch_loops.ns",
            "native_multi_state_branch_build_cache_and_standalone_relocation",
            "check_workflow(BRANCH_SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-flat-helper-return-contract",
        path: "crates/yir-lower-llvm/src/native_session/aggregates.rs",
        required_patterns: &[
            "pub(super) fn result_layout",
            "layout.fields.len() <= super::MAX_SCALAR_SLOTS",
            "result.ownership != YirValueOwnership::Owned",
            "flat_return_layout_has_a_slot_bound_not_precombined_arities",
            "flat_return_call_rejects_missing_layout_and_other_payload_families",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-flat-helper-call-closure",
        path: "crates/yir-lower-llvm/src/native_session/calls.rs",
        required_patterns: &[
            "super::aggregates::parse(node)?",
            "call.operands.len()",
            "super::aggregates::result_layout(function, nodes)? == layout",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-flat-helper-admission-proof",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_admission.rs",
        required_patterns: &[
            "flat_aggregate_calls_require_exact_call_result_and_parameter_contracts",
            "flat_aggregate_edges_reject_hidden_effects_foreign_values_and_recursive_helpers",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-flat-helper-nested-release-proof",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_execution.rs",
        required_patterns: &[
            "ordinary_flat_returns_preserve_exact_captures_order_and_nested_drop_balance",
            "multi_execution::execute_observed",
            "for slots in [2, 3, 7]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-flat-helper-selected-path-proof",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_guards.rs",
        required_patterns: &[
            "aggregate_early_return_skips_unselected_loop_preflight_but_selected_path_traps",
            "native_loop_preflight",
            "ApplicationSession::open_registered",
            "trap must not return callback state",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-multi-state-branch-lifecycle-proof",
        path: "tools/nuisc/tests/native_application_bridge/branch_loops.rs",
        required_patterns: &[
            "guarded_multi_state_exits_preserve_native_reference_and_typed_session_parity",
            "assert_native_parity(BRANCHES, true)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-checked-arithmetic-exact-kinds",
        path: "crates/yir-lower-llvm/src/function_lowering/strict_scalar.rs",
        required_patterns: &[
            "matches!(node.op.instruction.as_str(), \"div\" | \"rem\")",
            "native integer division/remainder",
            "require_value(node, operand, CpuCallScalarKind::I64, registers)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-checked-arithmetic-source-speculation",
        path: "tools/nuisc/src/lowering/speculation.rs",
        required_patterns: &[
            "pub(super) fn collect_checked_arithmetic",
            "while let Some(name) = pending.pop_first()",
            "NirBinaryOp::Div | NirBinaryOp::Rem",
            "purity_does_not_make_transitive_checked_arithmetic_speculatable",
            "reordered.functions.reverse()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-checked-arithmetic-execution",
        path: "tools/nuisc/tests/native_application_bridge/division_execution.rs",
        required_patterns: &[
            "checked_division_remainder_match_wide_oracle_and_skip_unselected_invalid_values",
            "selected_invalid_arithmetic_traps_even_for_unused_results_and_arguments",
            "invalid_constant_arithmetic_is_not_folded_away_or_hoisted_across_a_guard",
            "reference_arithmetic_failure_preserves_the_last_accepted_session_state",
            "i128::from(case.left)",
            "assert_eq!(cases.len(), 230)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-checked-arithmetic-admission",
        path: "tools/nuisc/tests/native_application_bridge/division_admission.rs",
        required_patterns: &[
            "generic_division_and_remainder_require_two_exact_i64_operands",
            "unoutlined_fallible_aggregate_guard_rejects_instead_of_speculating",
            "conditional fallible return requires guarded helper lowering",
            "assert_native_parity(DIVISION, true)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-checked-arithmetic-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "division_loops.ns",
            "native_checked_division_build_cache_and_standalone_relocation",
            "integer_divisor_invalid",
            "check_workflow(DIVISION_SOURCE)",
        ],
    },
];
