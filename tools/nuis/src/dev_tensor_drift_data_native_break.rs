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
            "ordinary_flat_returns_preserve_declared_names_without_a_loop_carry_schema",
            "names.insert(name)",
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
            "fallible_aggregate_returns_use_guarded_helpers_instead_of_speculation",
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
    DevTensorDriftCheckSpec {
        id: "native-session-flat-value-control-catalog",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values/tests.rs",
        required_patterns: &[
            "value_catalog_keeps_transitive_types_effects_cycles_and_buffer_admission_separate",
            "aggregate_control_outlining_shares_suffixes_and_captures_existing_records",
            "assert_eq!(helpers.len(), 32 * 3)",
            "scalar_helpers::collect_with_layouts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-shared-value-operators-and-neutral-guards",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values.rs",
        required_patterns: &[
            "pub(super) fn binary_type",
            "NirBinaryOp::Div",
            "NirBinaryOp::Rem",
            "pub(super) fn zero_value",
            "NirExpr::Bool(false)",
            "NirExpr::Int(0)",
            "NirExpr::StructLiteral",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-fallible-aggregate-execution",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_division.rs",
        required_patterns: &[
            "fallible_flat_branches_match_oracles_and_release_real_aggregate_temporaries",
            "fallible_flat_branches_trap_only_on_reached_operands_including_prefixes",
            "flat_branch_literal_failures_stay_behind_the_selected_guard",
            "registered_flat_state_branches_keep_declared_field_names_and_declaration_order_parity",
            "counted_loop_bearing_aggregate_branches_gain_guarded_lowering",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-counted-value-source-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops.rs",
        required_patterns: &[
            "prepare_counted_while(",
            "!loop_bindings.contains(name)",
            "!updates.contains(input) && scope.get(input)? == &scalar_type(\"i64\")",
            "!updates.insert(name.clone())",
            "!nonfallible_i64(value, scope)",
            "prepare_chained_while(",
            "chained.carries.len() != tail.len()",
            "PreparedCarryUpdateKind::Linear",
            "counted_control_catalog_is_transitive_without_widening_buffer_admission",
            "counted_control_catalog_rejects_mutation_type_effect_and_invariant_drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-counted-value-outlining-selection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &["helper.may_loop || checked_arithmetic.contains(&function.name)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-counted-value-transitive-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "collect_profile(module, &control_values::FlatLayouts::new(), false)",
            "control_loops::contains_loop(&function.body)",
            "helper.may_loop |= helper",
            ".any(|name| admitted[name].may_loop)",
            "control_loops::validate(condition, body, locals, loop_bindings)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-counted-value-execution",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_counted.rs",
        required_patterns: &[
            "guarded_counted_values_match_reference_and_independent_oracle",
            "guarded_counted_preflight_and_arithmetic_trap_only_when_selected",
            "loop_only_branches_preserve_preflight_without_checked_arithmetic",
            "loop_carried_updates_inside_flat_branches_gain_guarded_lowering",
            "counted_flat_branch_helpers_compose_with_typed_lifecycle",
            "aggregate_loop_probe::execute(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-guarded-loop-execution-probe",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_loop_probe.rs",
        required_patterns: &[
            "probe_trap_evidence",
            "multi_execution::ALLOCATION_PROBE",
            "call void @llvm.trap()",
            "ApplicationSession::open_registered(",
            "dynamic_loop_guard::run_bounded(",
            "assert_eq!(actual, oracle)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-carried-value-source-policy",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/carries_tests.rs",
        required_patterns: &[
            "ordered_carries_share_preparation_without_widening_buffer_catalog",
            "carry_catalog_is_not_a_finite_backend_profile_table",
            "carries_reject_seed_order_effect_and_header_mutation_drift",
            "carry_updates_require_exact_i64_declarations_and_values",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-carried-value-execution",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_carried.rs",
        required_patterns: &[
            "guarded_carried_values_match_reference_and_ordered_wrapping_oracle",
            "variable_carry_widths_and_loop_only_guards_retain_every_slot",
            "guarded_carries_trap_before_updates_and_never_speculate_skipped_calls",
            "conditional_carry_updates_inside_flat_branches_still_fail_closed",
            "carried_flat_branch_helpers_compose_with_typed_lifecycle",
            "aggregate_loop_probe::execute(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-carried-value-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_carried_loops.ns",
            "native_guarded_carried_aggregate_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_CARRIED_SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-reference-integer-wrapping",
        path: "crates/yir-domain-cpu/src/execute_scalar.rs",
        required_patterns: &[
            "wrapping_add(",
            "wrapping_sub(",
            "wrapping_mul(",
            "wrapping_neg()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-reference-integer-wrapping-regression",
        path: "crates/yir-domain-cpu/src/tests_scalar_logic.rs",
        required_patterns: &[
            "scalar_integer_arithmetic_wraps_independently_of_host_build_profile",
            "invalid_integer_divisors_report_errors_without_panicking",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-counted-value-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_counted_loops.ns",
            "native_guarded_counted_aggregate_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_COUNTED_SOURCE)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-session-fallible-aggregate-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_division_loops.ns",
            "native_fallible_aggregate_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_DIVISION_SOURCE)",
        ],
    },
];
