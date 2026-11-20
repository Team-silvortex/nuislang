use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-resource-record-condition-entry",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs",
        required_patterns: &["if let NirStmt::If { condition, .. } = statement", "statement, &ty, condition, template, scope, live, context, self"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-condition-requeue",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_operands.rs",
        required_patterns: &["NirStmt::If { condition, .. } => condition", "if prefix.is_empty()", "prefix.push(rewritten)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-condition-source-tests",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_condition_tests.rs",
        required_patterns: &["resource_predicate_record_conditions_preserve_arms_expressions_and_requeue_once", "resource_predicate_record_conditions_reject_scope_type_owner_and_budget_drift_atomically", "resource_predicate_record_conditions_keep_ready_pure_and_unrelated_routes"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-condition-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_conditions.rs",
        required_patterns: &["record_conditions_preserve_values_completed_reads_and_binding_forms_for_120_events", "record_conditions_keep_both_arms_local_owners_and_shared_fallthrough", "record_conditions_keep_branch_local_errors_and_failed_allocations_atomic", "record_conditions_keep_borrowed_only_registered_identity", "record_conditions_share_scalar_paths_without_widening_capture_bounds", "record_conditions_do_not_yet_admit_local_record_operands_in_returned_state"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-condition-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["case.strip_prefix(\"condition_operand_\")", "format!(\"condition_operand_{name}\")", "if ({value}) != 0", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-condition-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["local_record_condition_survives_cache_nsb_and_binary_only_launch", "replace(\"if valid {\"", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-condition-checked-error-identity",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &["fn checked_error_arm", "outcome.err_variant", "ty.generic_args", "not a scalar"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-condition-checked-error-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields_tests.rs",
        required_patterns: &["resource_predicate_checked_error_arms_use_registered_identity_without_live_owners", "super::super::checked_error_arm"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-scalar-admission",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_operands.rs",
        required_patterns: &["fn split_scalar", "capture_scalar(ty).then_some(())?", "bounded(value)?", "if prefix.is_empty()", "liveness::prepare(value, ty, template, scope, live, context, locals)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-scalar-integration",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs",
        required_patterns: &["if capture_scalar(ty)", "return operands::split_scalar(", "statement, ty, value, template, scope, live, context, self"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-scalar-source-tests",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_scalar_initializers_tests.rs",
        required_patterns: &["resource_predicate_local_record_scalar_declarations_preserve_types_shadowing_and_requeue_once", "resource_predicate_local_record_scalar_rejection_is_atomic_for_type_scope_owner_and_budget_drift", "resource_predicate_local_record_scalars_leave_ready_pure_and_unrelated_routes_unchanged"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-scalar-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_scalar_initializers.rs",
        required_patterns: &["local_record_scalar_initializers_preserve_truth_tables_reads_and_declarations_for_120_events", "local_record_scalar_initializers_keep_branch_local_shadows_and_outer_fallthrough", "local_record_scalar_initializers_keep_errors_and_failed_allocations_atomic", "local_record_scalar_initializers_preserve_independent_borrowed_only_error_identity", "local_record_scalar_initializers_share_paths_without_widening_capture_bounds", "local_record_scalar_initializers_also_admit_direct_resource_conditions"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-scalar-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["case.strip_prefix(\"scalar_operand_\")", "case.strip_prefix(\"bool_operand_\")", "format!(\"scalar_operand_{name}\")", "format!(\"bool_operand_{name}\")", "standalone_bool_probe_requires_one_complete_word_call", "bool_probe_operand(value).is_some()", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-scalar-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["local_record_bool_initializer_survives_cache_nsb_and_binary_only_launch", "local_record_i64_initializer_survives_cache_nsb_and_binary_only_launch", "let code: i64 = word({predicate})", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-operand-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_operands.rs",
        required_patterns: &["fn for_proof", "filter(capture_scalar)", "Type witnesses are only used by admission, never emitted or constant-folded", "Rebuild syntax without evaluating, extracting or reordering any operand"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-operand-planning",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_liveness.rs",
        required_patterns: &["operands::for_proof(value, self.locals, &self.context.calls)", "self.freeze(value, &ty, path(value))", "operands::children(value, |child| self.operands(child))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-operand-source-tests",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_operands_tests.rs",
        required_patterns: &["resource_predicate_record_operands_share_exact_scalar_snapshots_with_late_record_paths", "resource_predicate_record_operands_preserve_helper_wrappers_and_checked_arguments", "resource_predicate_record_operands_reject_invalid_trees_atomically"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-operand-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_operands.rs",
        required_patterns: &["record_operands_preserve_truth_tables_completed_reads_and_nominal_paths_for_120_events", "record_operands_share_used_leaves_without_widening_capture_limits", "record_operands_admit_standalone_scalar_declarations", "assert_eq!(library.live_blobs(), 0)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-operand-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["let operand_local = case", ".strip_prefix(\"operand_\")", "super::resource_record_operands::TRAPS", "format!(\"operand_{name}\")", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-operand-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_record_operands_survive_cache_nsb_and_binary_only_launch", "valid: !prior.extent.invalid &&", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-liveness-planning",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_liveness.rs",
        required_patterns: &["fn prepare", "struct Planner", "if self.branched", "Calls, resource reads and checked arithmetic stay at their source position", "self.leaves.get(&key)", "self.leaves.insert(key, saved.clone())", "fn path"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-liveness-integration",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs",
        required_patterns: &["liveness::prepare(value, ty, template, scope, live, context, locals)", "preserved.contains(name)", "&operands::for_proof(value, locals, &context.calls)", "bounded_tree(value, true)?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-liveness-source-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_liveness_tests.rs",
        required_patterns: &["resource_predicate_record_liveness_hoists_only_completed_leaves_before_eager_and_lazy_fields", "resource_predicate_record_liveness_deduplicates_overlapping_nominal_paths", "resource_predicate_record_liveness_never_hoists_a_new_helper_call_or_its_arguments"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-liveness-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_liveness.rs",
        required_patterns: &["record_liveness_preserves_late_fields_completed_reads_and_nested_paths_for_120_events", "record_liveness_shares_overlapping_paths_across_nested_and_multiple_lazy_fields", "record_liveness_keeps_branch_errors_and_failed_allocation_atomic", "record_liveness_without_held_owners_keeps_registered_identity", "record_liveness_reuses_repeated_paths_without_widening_guard_input_bounds", "record_liveness_admits_local_scalar_projection_inside_resource_predicates"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-liveness-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["case.strip_prefix(\"late_\")", "format!(\"late_{name}\")", "late_init_read_take", "late_init_div_take", "after: {after}, inner: prior", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-liveness-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_record_liveness_survives_cache_nsb_and_binary_only_launch", "load_at(pixels, 99) > 0), extent: prior.extent", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-local-field-scope",
        path: "tools/nuisc/src/lowering/guarded_owned_returns.rs",
        required_patterns: &["let mut local_records = resource_predicates::LocalRecords::default()", "local_records.split(", "local_records.advance(&statement, &context.calls)", "if capture_scalar(&ty)", "resource_predicates::checked_error_arm"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-local-field-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs",
        required_patterns: &["struct LocalRecords", "self.0.remove(name)", "self.0.clear()", "calls.is_record(ty) && snapshot_layout(ty, calls)", "locals.value_type(value, calls).as_ref() == Some(ty)", "liveness::prepare"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-local-field-scope-tests",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_local_fields_tests.rs",
        required_patterns: &["resource_predicate_record_locals_project_exact_visible_types_without_captures", "resource_predicate_record_locals_invalidate_rebindings_and_control_flow", "resource_predicate_record_locals_reject_nominal_or_late_drift_before_names"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-local-field-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_local_fields.rs",
        required_patterns: &["record_local_fields_preserve_completed_reads_and_nominal_projections_for_120_events", "record_local_fields_share_one_completed_value_and_continue_through_scalar_locals", "record_local_fields_keep_branch_errors_and_failed_allocation_nonpublishing", "record_local_fields_without_held_owners_keep_independent_registered_identity", "record_local_fields_after_an_earlier_lazy_field_preserve_scalar_liveness"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-local-field-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["case.strip_prefix(\"bound_\")", "format!(\"bound_{name}\")", "bound_init_read_take", "bound_init_div_take", "inner: prior, after:", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-local-field-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_record_local_fields_survive_cache_nsb_and_binary_only_launch", "extent: prior.extent, valid: color >= 0 &&", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-field-value-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs",
        required_patterns: &["fn snapshot_layout", "visited > 4096 || depth >= 64", "helper_value_type(value, scope, live, calls).as_ref() == Some(ty)", "Execute a record-valued helper once", "fn project", "ty: Some(ty.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-field-value-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_field_values_tests.rs",
        required_patterns: &["resource_predicate_record_fields_snapshot_one_call_before_later_lazy_fields", "resource_predicate_record_fields_reject_invalid_helpers_and_late_fields_atomically", "resource_predicate_record_fields_keep_original_budgets_and_ready_calls", "resource_predicate_record_fields_bound_complete_snapshot_layouts_before_expansion"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-field-value-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_field_values.rs",
        required_patterns: &["record_field_helpers_execute_once_and_preserve_field_order_for_120_events", "record_field_helpers_compose_multiple_calls_and_later_scalar_continuations", "record_field_helpers_keep_early_errors_and_allocation_failures_atomic", "record_field_helpers_without_held_owners_keep_registered_error_identity", "record_field_locals_before_later_resource_logic_use_local_scope_proof", "call [4 x i64] @nuis_fn_packet("],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-field-value-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["case.strip_prefix(\"member_\")", "format!(\"member_{name}\")", "member_field_before_read_take", "member_field_after_div_take", "hidden: 100 / value", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-field-value-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_record_helper_fields_survive_cache_nsb_and_binary_only_launch", "let validation: Validation = Validation { extent: status(", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-literal-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs",
        required_patterns: &["bounded_tree(value, true)?", "calls.record_fields(ty)?", "!seen.insert(field)", "predicate_kind(value, scope, live, calls) == Some(kind)", "Freeze every leaf in source order", "*target = value"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-literal-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals_tests.rs",
        required_patterns: &["resource_predicate_record_literals_preserve_nominal_bindings_and_source_order", "resource_predicate_record_literals_reject_whole_constructor_drift_before_names", "resource_predicate_record_literals_keep_shared_budgets_and_ready_routes"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-literal-native-admission",
        path: "crates/yir-lower-llvm/src/native_resource_session/mixed_admission.rs",
        required_patterns: &["Kind::Bool | Kind::Record(_, _) if kind.is_record_value()", "NativeValueLayout::parse(&encoded)?", "require_layout(name, &fields, layout)?", "resource_record_literals_native_admission_shares_slot_and_depth_bounds", "resource_record_literals_native_rejection_is_atomic_and_never_grants_owned_state"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-literal-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_literals.rs",
        required_patterns: &["record_literals_keep_nested_fields_declaration_kinds_and_read_order_for_120_events", "record_literals_keep_early_returns_and_projected_scalar_continuations", "record_literals_keep_error_and_allocation_failure_state_atomic", "record_literals_without_held_owners_keep_independent_registered_names", "record_literals_with_record_valued_helper_fields_stage_scalar_snapshots"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-literal-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["case.strip_prefix(\"literal_\")", "format!(\"literal_{name}\")", "literal_field_before_read_take", "literal_field_after_div_take", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-literal-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_record_literals_survive_cache_nsb_and_binary_only_launch", "let validation: Validation = Validation { extent: Validity", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-initializer-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &["bounded(value).is_some()", "context.calls.is_record(ty)", "arguments::contains_argument_logic(value)", "helper_value_type(value, scope, live, &context.calls).as_ref() == Some(ty)", "fn requeue_declaration", "*target = value"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-initializer-layout",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/pure_value_calls.rs",
        required_patterns: &["fn is_record", "ty == &scalar_type(&ty.name) && self.layouts.contains_key(&ty.name)", "collect_capture_values", "may_loop"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-initializer-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_record_initializers_tests.rs",
        required_patterns: &["resource_predicate_record_initializers_preserve_nominal_bindings_and_scalar_argument_order", "resource_predicate_record_initializers_reject_invalid_types_owners_and_calls_before_names", "resource_predicate_record_initializers_keep_whole_expression_budgets_and_ready_routes", "resource_predicate_record_catalog_never_admits_hidden_resources_or_recursive_layouts"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-initializer-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_record_initializers.rs",
        required_patterns: &["record_local_initializers_preserve_nested_values_argument_order_and_owners_for_120_events", "record_local_initializers_keep_early_returns_and_scalar_projected_continuations", "record_local_initializers_keep_application_errors_and_allocation_failures_atomic", "record_local_initializers_do_not_widen_native_value_record_slots", "record_local_initializers_without_held_owners_keep_registered_error_identity", "record_local_literal_short_circuit_fields_use_ordered_staging"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-initializer-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &["let record_local = case.strip_prefix(\"record_\")", "let local: ProbeNumber = record_value({value})", "format!(\"record_{name}\")", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-record-initializer-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["resource_record_initializers_survive_cache_nsb_and_binary_only_launch", "let validation: Validation = decision(color >= 0 &&", "let valid: bool = !validation.extent.invalid", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-buffer-length-lowering",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs",
        required_patterns: &["NirExpr::BytesLen(value) | NirExpr::BufferLen(value)", "| NirExpr::BufferLen(_)", "NirExpr::BufferLen(value) => borrowed_buffer(value, scope).then_some(Kind::I64)", "fn borrowed_buffer", "p.ty.is_ref", "p.ty.generic_args.is_empty()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-buffer-length-proof",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/resource_scalar_initializers_tests.rs",
        required_patterns: &["resource_predicate_buffer_length_stages_in_order_without_walking_lazy_children", "resource_predicate_buffer_length_rejects_nonborrowed_or_hidden_invalid_operands_atomically", "resource_predicate_buffer_length_keeps_original_budgets_and_ready_route"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-buffer-length-execution",
        path: "tools/nuisc/tests/native_resource_session/resource_buffer_lengths.rs",
        required_patterns: &["buffer_length_compositions_keep_scalar_types_read_order_and_owners_for_120_events", "buffer_length_inline_conditions_preserve_borrowed_and_owned_return_paths", "buffer_length_error_codes_and_failed_results_preserve_committed_state", "buffer_length_native_admission_rechecks_input_kind_and_arity", "buffer_length_short_circuit_in_value_record_locals_uses_typed_snapshots", "length_and_skip", "length_and_take", "length_eager_left_take", "length_eager_right_take"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-buffer-length-traps",
        path: "tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs",
        required_patterns: &[".chain(super::resource_buffer_lengths::TRAPS)", "format!(\"field_{name}\")", "format!(\"error_{name}\")", "Some(4 | 5)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-buffer-length-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &["buffer_length_return_fields_survive_cache_nsb_and_binary_only_launch", "cursor: cursor + buffer_len(pixels) * word", "bytes_len(snapshot) == buffer_len(pixels) * 8", "compile_cache: hit"],
    },
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
        required_patterns: &["direct_resource_error_codes_preserve_values_committed_state_and_all_owners", "direct_resource_error_result_allocation_failure_is_not_an_application_code", "direct_resource_error_codes_keep_registered_identity_without_held_bytes", "returned_resource_buffer_length_composition_uses_guarded_initializers"],
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
