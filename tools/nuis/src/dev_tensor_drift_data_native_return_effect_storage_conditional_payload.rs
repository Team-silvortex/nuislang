use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-scalar-lowering",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &["let condition = logical_conditions::predicate(", "self.guarded", "value: condition"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-shared-predicates",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs",
        required_patterns: &["pub(super) fn predicate", "__nuis_scalar_predicate", "guarded.insert(name.clone())", "negate(call)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-loop-reuse",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &["use super::super::logical_conditions as conditions", "conditions::outline("],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs",
        required_patterns: &["return_effect_storage_conditional_payload_tests.rs", "mod conditional_payload_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_conditional_payload_tests.rs",
        required_patterns: &[
            "effect_return_storage_conditional_payload_requires_transitive_predicate_authority",
            "effect_return_storage_conditional_payload_keeps_existing_scalar_writes_outside_admission",
            "denied.functions.retain(|function| function.name != leaf)",
            "scalar_helpers::collect_iteration_calls(&denied, &layouts)",
            "assert!(!pure.contains_key(effectful))", "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-execution",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_conditional_payload_tests.rs",
        required_patterns: &[
            "effect_return_storage_conditional_payload_oracle_covers_lazy_truth_table_and_fault_stages",
            "effect_return_storage_conditional_payload_matches_independent_execution_and_carry_width",
            "effect_return_storage_conditional_payload_rejects_caller_reordering_and_guard_bypasses",
            "effect_return_storage_conditional_payload_detects_eager_and_pruned_rhs",
            "for mutation in [6, 7]", "assert_eq!(cases::failures(variant).len(), 31)",
            "assert_ne!(trace, expected", "Alternative source programs test oracle sensitivity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_conditional_cases.rs",
        required_patterns: &[
            "pub(super) fn payload", "let choice = seeds[calls].result",
            "trace.extend([991, 997])", "trace.extend([992, 993, 994, 995, 997])",
            "if before == 0", "if after == 0", "trace.push(996)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-nuis",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_conditional_helpers.ns",
        required_patterns: &[
            "fn record_gate", "fn record_rhs", "fn record_select",
            "record_gate(choice) && record_rhs(selected_root)",
            "record_gate(choice) || record_rhs(selected_root)",
            "record_select(choice, root, 0)", "record_select(choice, 0, root)",
            "marker: record_marker(-seed.marker, before, seed.result)",
            "total: record_total(seed.total + delta + checked, after, seed.result)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-family",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs",
        required_patterns: &[
            "pub fn conditional_payload_variants", "pub conditional_payload: bool",
            "return conditional::payload", "source = conditional::source(source)",
            "pub const LOOP_LIMIT: u64 = 18", "pub const ENTRY_LIMIT: u64 = 192",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-native",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &[
            "conditional_payload_reference_preserves_lazy_fields_and_faulted_state",
            "conditional_payload_native_keeps_short_circuit_order_and_exact_fault_prefixes",
            "conditional_payload_native_reserves_before_conditional_field_work",
            "conditional_payload_full_exit_stress_fits_unchanged_function_bound",
            "check_native(cases::conditional_payload_variants())",
            "for limit in [required - 1, required]", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &[
            "conditional_payload_fresh_cache_and_source_free_delivery_preserve_lazy_fields",
            "delivery(cases::conditional_payload_variants()[0])", "for restore in 0..2",
            "for (prefix, bad) in cases::failures(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-conditional-payload-v1.md",
        required_patterns: &[
            "Lazy Callee Fields", "Authority And Controls", "Native And Delivery Gates",
            "All six lazy truth paths", "31 failure categories", "active/99",
            "scalar branch assignments", "Four fewer logical YIR carry words",
            "64", "not native acceptance of the entire accumulated stress fixture",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-ready-value-compaction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/ready_values.rs",
        required_patterns: &[
            "pub(super) fn atom_type", "pub(super) fn total_predicate", "pub(super) fn updates",
            "atom_type(rhs, scope).as_ref() != Some(&ty)",
            "atom_type(value, scope).as_ref() != Some(ty)",
            "value: NirExpr::Var(name.clone())", "Keep source order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-ready-value-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/ready_values_tests.rs",
        required_patterns: &[
            "ready_predicates_do_not_grant_speculation_to_calls_checks_or_projections",
            "ready_updates_preserve_sequential_reads_and_reject_partial_or_nominal_work",
            "ready_logical_rhs_keeps_effectful_left_and_guards_nested_calls",
            "ready_update_selects_execute_in_order_on_both_paths_and_reversed_yir",
            "ready_rhs_never_erases_effectful_left_even_when_result_is_constant",
            "for reversed in [false, true]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-ready-value-branches",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/branches.rs",
        required_patterns: &["ready_values::updates(&arm, scope, &predicate, selected)", "outlined.extend(updates)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-ready-value-predicates",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs",
        required_patterns: &["ready_values::total_predicate(&rhs, scope)", "lhs: Box::new(predicate(*lhs, scope, names, helpers, guarded))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-full-helper-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_conditional_payload_tests.rs",
        required_patterns: &["effect_return_storage_conditional_payload_full_helpers_fit_native_function_bound", "module.functions.len() <= 64", "only the two effectful field predicates require helpers"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-ready-value-receipt",
        path: "docs/reference/nuis-native-return-ready-values-v1.md",
        required_patterns: &[
            "75 to 63 NIR function definitions", "31 failure categories",
            "64/65", "18/192", "Typed conditional scalar bindings",
            "active/99", "No source branch is deleted to satisfy native admission",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-ready-value-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified full conditional-return closure:",
            "Previous conditional-field payload checkpoint:",
            "Previous conditional-field payload task checkpoint:",
            "Previous conditional-field payload blocker checkpoint:",
            "Previous conditional-field payload action checkpoint:",
            "Previous conditional-field payload validation checkpoint:",
            "Previous conditional-field payload artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-conditional-payload-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified conditional-callee-field witnesses:",
            "Previous unprojected-field payload checkpoint:", "Previous unprojected-field payload task checkpoint:",
            "Previous unprojected-field payload blocker checkpoint:", "Previous unprojected-field payload action checkpoint:",
            "Previous unprojected-field payload validation checkpoint:", "Previous unprojected-field payload artifact checkpoint:",
        ],
    },
];
