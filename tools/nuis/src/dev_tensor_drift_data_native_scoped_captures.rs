use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "scoped-capture-loop-written-input-protection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped.rs",
        required_patterns: &[
            "protected_inputs(",
            "branches::collect_bindings(body, &mut written)",
            "protected.entry(callee.clone()).or_default()",
            "written.contains(&path[0]) && !carried.contains(&index)",
            "inputs.protected.insert(index)",
            "inputs.carried.extend(carried)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-control-identity-preservation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs",
        required_patterns: &[
            "if !scoped.contains(&name) {",
            "Scoped helpers retain the outliner's control identities",
            "aliases::normalize(&mut candidate, layouts)",
            "valid_caller(&module.functions[*i].body, &name, &plan)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-invariant-field-argument-admission",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/arguments.rs",
        required_patterns: &[
            "invariant.remove(name.as_str())",
            "while let NirExpr::FieldAccess { base, .. } = root",
            "invariant.contains(name.as_str())",
            "scoped_arguments_only_hoist_ready_invariant_field_paths",
            "scoped_arguments_exclude_nested_writes_and_unknown_roots",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-driver-map-after-projection",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering.rs",
        required_patterns: &[
            "arguments::invariant_bindings(body, bindings)",
            "arguments::ready(arg, &invariant_inputs)",
            "scalar_carries::argument_index(&result, param, arg)",
            "scalar_carries::validate_field_seed_origins(",
            "action_args.push(\"$current\".to_owned())",
            "action_args.push(\"$carry\".to_owned())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-projection-and-control-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped_tests.rs",
        required_patterns: &[
            "scoped_projection_keeps_rebound_record_seeds_by_argument_identity",
            "scoped_projection_unions_writes_across_callers_and_nested_scopes",
            "scoped_projection_keeps_computed_and_whole_uses_transactional",
            "scoped_field_inputs_preserve_zero_trips_and_induction_carry_maps",
            "scoped_field_inputs_preserve_bool_record_and_break_carry_slots",
            "scoped_field_inputs_keep_nested_break_controls_local",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-wide-source-not-narrow-wrapper",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn wide_loop_source()",
            "let saved = fallback; let second = saved;",
            "let result = relay(second.f63) / second.f1;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-carry-field-seed-map-contract",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries.rs",
        required_patterns: &[
            "fn seed_range",
            "position(|name| name == field)",
            "seed_range(binding, param, arg).is_some()",
            "function.params.len() != args.len()",
            "std::mem::replace(slot, true)",
            "covered.iter().any(|covered| !covered)",
            "Some((offset + field, width))",
            "fn validate_field_seed_origins",
            "fn initial_record_type",
            "actual.as_deref() != Some(binding.ty.name.as_str())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-carry-field-seed-map-regressions",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carry_seed_maps_tests.rs",
        required_patterns: &[
            "carried_field_seeds_follow_result_layout_not_parameter_order",
            "carried_field_seeds_require_complete_unique_exact_typed_coverage",
            "carried_field_seeds_preserve_bool_and_break_word_identities",
            "carried_field_seeds_mix_whole_and_field_mapped_records",
            "carried_field_seeds_execute_updated_values_zero_trips_and_old_snapshots",
            "carried_field_seeds_reject_incomplete_and_computed_source_maps",
            "carried_field_seeds_preserve_nominal_origins_through_calls_and_fields",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-carry-field-seed-source-composition",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn field_seed_source()",
            "update_fields(current.marker, result, current.value, current.unused, current.divisor)",
            "let result = words.carry4;",
            "repeat_update(select_input(payload), 2)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-carry-field-seed-native-lifecycle",
        path: "tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs",
        required_patterns: &[
            "typed_sparse_captures_map_record_fields_to_scoped_backedges",
            "assert_eq!(slots, [2, 4, 0, 3, 1])",
            "aliases::field_seed_source()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-carry-field-seed-source-free-restoration",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_sparse_captures_field_seeds_build_cache_and_restore_without_sources",
            "check_sparse_workflow(&aliases::field_seed_source(), Some(&[1, 2]), 30)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-carry-field-seed-ordinary-native-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/scoped_field_seeds.rs",
        required_patterns: &[
            "scoped_field_seed_backedges_execute_in_the_ordinary_native_entry",
            "scoped_field_seed_backedges_preserve_selected_arithmetic_failure",
            "Some(171)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-wire-contract",
        path: "crates/yir-core/src/loop_carry_contract/scoped_scalars.rs",
        required_patterns: &[
            "SCOPED_I64_SEEDS_MARKER",
            "encode_scoped_i64_seeds",
            "count.checked_add(2)",
            "count != seeds.len()",
            "explicit && *slot != Some(input)",
            "explicit_seeds_are_independent_of_iteration_operands",
            "explicit_seed_dependencies_and_glm_exclude_transport_metadata",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-glm-inputs",
        path: "crates/yir-core/src/glm.rs",
        required_patterns: &[
            "parse_scoped_i64_carries(&op.args)",
            "carries.dependencies()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-domain-dependencies",
        path: "crates/yir-domain-cpu/src/describe_loops_control.rs",
        required_patterns: &[
            "parse_scoped_i64_carries(&node.op.args)",
            "carry.dependencies()?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-registered-execution",
        path: "crates/yir-domain-cpu/src/tests_loop_effect_execution.rs",
        required_patterns: &[
            "separate_seeds_keep_unpassed_state_and_break_control",
            "state.values.remove(\"second\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-llvm-storage",
        path: "crates/yir-lower-llvm/src/loop_owned_struct_lowering.rs",
        required_patterns: &[
            "operand: Option<String>",
            "multi.seeds.iter().enumerate()",
            "None, CpuCallScalarKind::I64",
            "slot.operand",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-source-proof",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carry_seed_storage_tests.rs",
        required_patterns: &[
            "partial_record_arguments_keep_independent_complete_initial_state",
            "partial_record_arguments_require_real_origins_and_unambiguous_slot_maps",
            "separate_seed_storage_retains_unpassed_initializer_failures_on_zero_trips",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-source-lowering",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering.rs",
        required_patterns: &[
            "separate_seeds: true",
            "scalar_carries::lower_initial_seeds",
            "encode_scoped_i64_seeds",
            "seeds[index..index + width]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-native-lifecycle",
        path: "tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs",
        required_patterns: &[
            "typed_sparse_captures_separate_initial_state_from_partial_iteration_arguments",
            "assert_eq!(call.seeds.len(), 5)",
            "assert_eq!(call.operands.len(), 4)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-independent-seed-artifact-restoration",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_sparse_captures_partial_field_seeds_build_cache_and_restore_without_sources",
            "aliases::partial_field_seed_source()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-generated-record-shared-seed-proof",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering.rs",
        required_patterns: &[
            "fn projectable_record_seed_inputs(",
            "scalar_carries::projected_bindings(name, ty, tail, definitions)",
            "scalar_carries::validate_seeds(&carries, breaking, function, args, callee)",
            "carry.whole_record_seed(param, arg)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-generated-record-parameter-snapshots",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_snapshots.rs",
        required_patterns: &[
            "fn normalize_scoped(",
            "candidates.retain(|name| function.params.iter().any(|p| &p.name == name))",
            "normalize_candidates(function, &candidates)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-generated-record-partial-demand-only",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs",
        required_patterns: &[
            "snapshots::normalize_scoped(&mut candidate, layouts)",
            "inputs.protected.contains(index)",
            "paths.is_empty() && protected.is_some_and(|inputs| inputs.carried.contains(&index))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-generated-record-transactional-projection-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped_carries_tests.rs",
        required_patterns: &[
            "generated_scoped_record_inputs_project_only_demanded_initial_fields",
            "generated_scoped_projection_rejects_unproven_or_empty_carry_maps_transactionally",
            "generated_scoped_projection_requires_all_callers_to_prove_the_backedge",
            "generated_scoped_snapshots_leave_fallthrough_and_nested_loop_writes_whole",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-generated-record-execution-and-width-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped_carries_tests.rs",
        required_patterns: &[
            "generated_partial_carries_execute_with_complete_seed_storage",
            "generated_projection_retains_all_initial_and_per_trip_checked_work",
            "generated_projection_scales_state_width_independently_of_helper_arity",
            "generated_projection_preserves_break_continue_and_bool_word_identities",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-generated-record-ordinary-native-execution",
        path: "tools/nuisc/tests/control_flow_syntax_native/scoped_field_seeds.rs",
        required_patterns: &[
            "generated_record_capture_projection_preserves_real_native_results_and_traps",
            "Some(154)",
            "scoped_projected_record_carries.ns",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-generated-record-source-free-signature-restoration",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_sparse_captures_loop_snapshots_build_cache_and_restore_without_sources",
            "iteration_arity: Option<usize>",
            "line.contains(\" @nuis_fn___nuis_scalar_iteration_\")",
            "assert_eq!(fs::read_to_string(restored.join(&llvm_name)).unwrap(), llvm)",
        ],
    },
];
