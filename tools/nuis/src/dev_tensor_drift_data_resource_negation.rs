use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-resource-error-code-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields.rs",
        required_patterns: &["*type_name == outcome.err_variant", "!admitted(code, scope, live, &context.calls, Kind::I64)", "!arguments::contains_logic(code)", "value: code.clone()", "split_cleanup_binding"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-error-code-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields_tests.rs",
        required_patterns: &["resource_predicate_error_code_requeues_one_intact_i64_and_preserves_identity", "resource_predicate_error_code_rejection_is_atomic_for_kinds_identity_owners_and_budgets", "resource_predicate_error_codes_preserve_terminal_cleanup_and_ready_routes"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-owned-continuation-routing",
        path: "tools/nuisc/src/lowering/bootstrap.rs",
        required_patterns: &[".difference(&owned_return_arms.functions)", "guarded_owned_helpers: owned_return_arms.owners.clone()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-error-code-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_error_codes.rs",
        required_patterns: &["direct_resource_error_codes_preserve_values_committed_state_and_all_owners", "direct_resource_error_result_allocation_failure_is_not_an_application_code", "direct_resource_error_codes_keep_registered_identity_without_held_bytes", "returned_resource_buffer_length_composition_still_needs_a_separate_initializer"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-error-code-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["let error_return = case.strip_prefix(\"error_\")", "format!(\"error_{name}\")", "error_bare_i64_and_skip", "error_bare_i64_and_take", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-error-code-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/checked.rs",
        required_patterns: &["direct_resource_error_codes_survive_cache_nsb_and_binary_only_launch", "return Result.Err(error_code(bytes_len(saved) == 48 || load_at(pixels, 99) > 0", "resource return helpers exceed the source closure bound", "delivery(&with_error_codes([1, 4, 7]))", "check_failure", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-return-field-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields.rs",
        required_patterns: &["MixedResourceLayout::parse_outcome", "bounded_tree(value, true)?", "available.remove(name)", "split_cleanup_binding", "Stage every field in source order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-return-field-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields_tests.rs",
        required_patterns: &["resource_predicate_return_fields_stage_all_fields_in_source_order", "resource_predicate_return_fields_reject_invalid_whole_records_before_names", "resource_predicate_return_fields_share_original_aggregate_budgets", "resource_predicate_return_fields_validate_checked_wrapper_identity_and_type_arguments", "resource_predicate_return_fields_preserve_terminal_cleanup_bindings_only"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-return-field-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_return_fields.rs",
        required_patterns: &["returned_resource_fields_keep_source_order_and_owners_for_120_events", "returned_resource_fields_failed_result_allocation_preserves_state_and_releases_staged_bytes", "returned_resource_fields_follow_registered_layouts_not_canvas_or_result_names", "returned_resource_fields_do_not_read_moved_owners_even_under_false_gates", "returned_resource_error_codes_use_guarded_initializers_before_cleanup"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-return-field-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["let returned = case.strip_prefix(\"field_\")", "integer.filter(|_| returned.is_none())", "format!(\"field_{name}\")", "before-short-circuit-event", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-return-field-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["returned_resource_fields_survive_cache_nsb_and_binary_only_launch", "cursor: cursor + word(color >= 0 &&", "delivery(&source)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-inline-condition-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &["if let NirStmt::If { condition, .. } = statement", "!admitted(condition, scope, live, &context.calls, Kind::Bool)", "!arguments::contains_logic(condition)", "*target = NirExpr::Var(name.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-inline-condition-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_condition_tests.rs",
        required_patterns: &["resource_predicate_inline_condition_requeues_one_intact_bool_and_original_arms", "resource_predicate_inline_condition_proof_rejects_invalid_trees_atomically", "resource_predicate_inline_conditions_keep_pure_and_ready_routes_unchanged"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-inline-condition-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_conditions.rs",
        required_patterns: &["inline_resource_conditions_preserve_selected_values_and_owners_for_120_events", "inline_resource_conditions_keep_both_arms_local_owners_and_one_suffix", "inline_resource_condition_snapshot_preserves_writes_and_source_names", "inline_resource_condition_errors_and_failed_results_preserve_state_and_release_owners", "inline_resource_condition_snapshots_share_the_existing_guard_input_budget", "returned_resource_state_fields_use_ordered_short_circuit_initializers"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-inline-condition-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["let inline = case.strip_prefix(\"inline_\")", "let source = if inline.is_some()", "super::resource_conditions::TRAPS.iter().copied()", "before-short-circuit-event", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-inline-condition-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["inline_resource_conditions_survive_cache_nsb_and_binary_only_launch", "if !(color >= 0 &&", "delivery(&source)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-i64-initializer-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &["if !capture_scalar(ty)", "let expected = if ty.name == \"bool\"", "predicate_kind(value, scope, live, calls) == Some(expected)", "arguments::extract(value, template, scope, live, context)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-i64-initializer-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_scalar_initializers_tests.rs",
        required_patterns: &["resource_predicate_i64_initializers_preserve_order_types_and_lazy_subtrees", "resource_predicate_i64_admission_rejects_invalid_types_before_allocating_names", "resource_predicate_i64_keeps_plain_scalar_initializers_on_the_existing_route"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-i64-initializer-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_i64_initializers.rs",
        required_patterns: &["direct_i64_resource_arguments_preserve_values_and_owners_for_120_events", "direct_i64_resource_arguments_keep_full_width_values", "direct_i64_initializers_preserve_old_scope_writes_and_temporary_hygiene", "direct_i64_errors_and_failed_results_preserve_state_and_release_owners", "direct_i64_arguments_retain_the_source_closure_depth_bound", "direct_inline_resource_conditions_reach_guarded_lowering"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-i64-initializer-traps",
        path: "tools/nuisc/tests/native_resource_session/resource_i64_initializers.rs",
        required_patterns: &["i64_and_skip", "i64_or_take", "i64_not_or_skip", "i64_xor_take", "i64_nested_skip", "i64_body_take", "i64_conditional_skip", "i64_left_take", "i64_right_take", "i64_eager_left_take", "i64_eager_right_take", "i64_index_skip", "i64_pure_take"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-i64-initializer-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["direct_i64_resource_arguments_survive_cache_nsb_and_binary_only_launch", "let choice: i64 = word(color >= 0 &&", "delivery(&source)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-wrapper-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &["logical_parts(value).is_none() && arguments::contains_logic(value)", "reads || arguments::contains_argument_logic(value)", "let left = predicate_kind(lhs, scope, live, calls)?", "left != predicate_kind(rhs, scope, live, calls)?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-wrapper-source-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates_tests.rs",
        required_patterns: &["resource_wrapper_snapshots_are_ordered_and_leave_lazy_children_intact", "resource_wrapper_rejection_is_atomic_for_bad_types_owners_and_budget", "neither lazy child may be hoisted", "pure predicates retain scalar-helper routing"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-wrapper-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_wrapped_predicates.rs",
        required_patterns: &["wrapped_resource_predicates_keep_truth_tables_and_owners_for_120_events", "resource_wrapper_snapshots_preserve_writes_shadowing_and_name_hygiene", "resource_wrapper_errors_and_failed_results_preserve_state_and_release_owners", "resource_wrappers_reach_direct_i64_initializers"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-wrapper-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["wrap_eq_and_skip", "wrap_xor_or_take", "wrap_both_skip", "wrap_right_take", "wrap_eq_right_take", "wrap_ne_right_take", "wrap_eager_left_take", "wrap_index_skip", "wrap_mandatory_div_take"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-wrapper-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["wrapped_resource_predicates_survive_cache_nsb_and_binary_only_launch", "== (width > 0)) ^ (height < 0)", "delivery(&source)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-scalar-select-admission",
        path: "crates/yir-lower-llvm/src/native_resource_session/admission.rs",
        required_patterns: &["\"select\" if mixed.is_some()", "Kind::Bool, Kind::I64, Kind::I64", "Kind::Bool, Kind::Bool, Kind::Bool", "select requires ready i64 or bool arms"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-scalar-select-execution",
        path: "tools/nuisc/tests/native_resource_session/scalar_selection.rs",
        required_patterns: &["ready_scalar_selection_preserves_types_state_and_owners_for_120_events", "scalar_selection_rechecks_both_arms_and_never_selects_resources", "resource_comparison_wrappers_use_guarded_lowering", "unavailable_select_arm"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-scalar-select-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["select_and_skip", "select_or_take", "select_not_or_skip", "select_body_and_take", "select_bool_and_skip", "select_bool_and_take", "select_conditional_and_skip", "select_conditional_or_take"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-scalar-select-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["scalar_selection_survives_cache_nsb_and_binary_only_launch", "fn word(flag: bool) -> i64", "delivery(&source)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-logical-argument-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicate_arguments.rs",
        required_patterns: &["contains_logic", "logical_parts(value).is_some()", "args.iter().map(|arg| self.operand(arg)).collect()", "fresh_left"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-logical-argument-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates_tests.rs",
        required_patterns: &["resource_argument_snapshots_preserve_order_and_prove_before_allocating_names", "resource_argument_budgets_count_hidden_logic_and_pending_arguments"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-logical-argument-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_argument_predicates.rs",
        required_patterns: &[
            "resource_logical_arguments_preserve_120_events_per_declaration_form",
            "resource_logical_argument_errors_and_failed_returns_preserve_state_and_owners",
            "resource_logical_argument_snapshots_keep_write_order_and_source_names",
            "load_at(pixels, 3) == word(valid && bytes_len(saved) == 48)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-logical-argument-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["argument_and_skip", "argument_outer_or_take", "argument_left_take", "argument_right_take", "argument_first_take", "argument_second_take"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-logical-argument-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_logical_arguments_survive_cache_nsb_and_binary_only_launch", "!decision(color >= 0 &&", "delivery(&source)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-helper-read-source-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/pure_value_calls.rs",
        required_patterns: &["collect_capture_values", "scalar_helpers::typed_call_type", "pub(in crate::lowering) fn field_type", "may_loop"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-helper-read-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_helper_predicates.rs",
        required_patterns: &[
            "resource_helper_predicates_preserve_selected_values_and_owners_for_120_events",
            "resource_helper_predicate_errors_and_failed_returns_never_publish_or_leak",
            "resource_helper_predicates_reject_effects_hidden_logic_and_resource_signatures",
            "resource_helper_predicate_reads_observe_writes_before_the_selected_call",
            "resource_helper_predicates_keep_one_typed_bounded_native_closure",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-helper-read-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["helper_and_skip", "helper_or_take", "helper_not_and_take", "helper_body_and_take", "helper_bytes_or_skip"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-helper-read-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_helper_reads_survive_cache_nsb_and_binary_only_launch", "nonempty(bytes_len(snapshot))", "nonempty(load_at(pixels, 99))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-nested-read-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["nested_resource_reads_survive_cache_nsb_and_binary_only_launch", "delivery(&source)", "load_at(pixels, 99) > 0"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-nested-read-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &["needs_guard(left)", "logical_edges > 32", "identity_handoff(edges, live)", "edges.cleanup.is_empty()", "fn fresh_left"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-nested-read-execution",
        path: "tools/nuisc/tests/native_resource_session/nested_resource_predicates.rs",
        required_patterns: &[
            "nested_resource_predicates_preserve_truth_tables_and_owners_for_120_events",
            "nested_resource_predicate_errors_and_failed_returns_never_publish_or_leak",
            "nested_resource_predicates_share_suffixes_and_preserve_closure_bounds",
            "nested_resource_predicate_graph_rechecks_owner_edges_and_predicate_types",
            "nested_resource_left_snapshots_do_not_collide_with_source_bindings",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-nested-read-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["nested_inner_and_skip", "nested_inner_or_skip", "nested_left_take", "nested_both_take", "nested_not_or_take"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-selected-read-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &[
            "continuations::share(",
            "visited > 4096 || depth >= 64",
            "live.contains(name).then_some(Kind::I64)",
            "disjunction ^ inverted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-selected-read-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates_tests.rs",
        required_patterns: &[
            "resource_predicate_proof_preserves_exact_types_and_live_owners",
            "resource_predicate_proof_bounds_original_depth_and_work",
            "nested_resource_predicates_prove_both_subtrees_and_count_all_lazy_edges",
            "resource_predicate_identity_handoff_never_bypasses_cleanup_or_partial_owners",
            "resource_predicate_calls_require_completed_pure_exact_signatures",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-selected-read-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_predicates.rs",
        required_patterns: &[
            "resource_predicates_preserve_selected_reads_shadowing_and_owners_for_120_events",
            "resource_predicate_error_and_failed_return_release_transferred_bytes",
            "resource_predicate_reads_observe_prior_writes_and_keep_the_result_snapshot",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-selected-read-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["read_and_skip", "read_or_take", "read_not_and_take", "read_not_or_skip"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-negation-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "fn negated_predicate",
            "fn computed_predicate_root",
            "eager_bool || negation",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-negation-lowering",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical.rs",
        required_patterns: &[
            "!prefix::computed_predicate_root(condition)",
            "self.logical_tree_mode(inner, scope, force)",
            "changed.then_some(value)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-negation-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_negated_logical_tests.rs",
        required_patterns: &[
            "negated_logical_roots_share_guards_without_widening_legacy_profiles",
            "negated_logical_roots_count_wrapper_work_depth_and_all_lazy_edges",
            "negated_logical_roots_reject_effects_resources_hidden_logic_and_bad_types_atomically",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-negation-execution",
        path: "tools/nuisc/tests/native_resource_session/boolean_negated_short_circuit.rs",
        required_patterns: &[
            "negated_short_circuit_shadows_preserve_120_events_per_declaration_form",
            "negated_short_circuit_errors_and_allocation_failures_do_not_publish_or_leak",
            "negated_resource_wrappers_reuse_owned_branches",
        ],
    },
];
