use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-declaration-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs",
        required_patterns: &["pub(super) fn declaration", "scope.contains_key(name)", "ready_values::atom_type(condition, scope)", "ready_values::atom_type(yes, scope)", "ready_values::atom_type(no, scope)", "!observed(name, suffix)?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-scope-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs",
        required_patterns: &["let mut work = 4096usize", "if depth >= 64", "work.checked_sub(1)?", "if bound == name", "An initializer sees the preceding scope"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-source-phase",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &["pub(super) fn outline_declarations", "fn declarations", "scalar_joins::declaration(stmt, scope, suffix)", ".unwrap_or_else(|| join.condition.clone())", "self.extract(condition, yes, no, scope, false)", "Iteration-local joins keep the loop outliner's own authority"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-pipeline",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &["mod scalar_joins", "let declarations =", "conditional_values::outline_declarations(", "selections.extend(declarations)", "effectful_selections::outline(module, &mut names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_tests.rs",
        required_patterns: &["source_join_normalization_is_idempotent_and_keeps_strict_write_admission", "fresh_typed_joins_share_ready_value_proof_without_mutating_source", "joins_reject_outer_writes_checks_calls_and_incomplete_branches", "branch_locals_are_not_exported_through_later_or_nested_redeclarations", "typed_joins_preserve_selected_raw_scalar_words_in_reversed_yir"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-compiler-witness",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_scalar_join_tests.rs",
        required_patterns: &["effect_return_storage_scalar_join_admits_typed_values_without_pure_effect_authority", "effect_return_storage_scalar_join_full_helpers_fit_native_bound", "effect_return_storage_scalar_join_matches_independent_execution_and_carry_width", "effect_return_storage_scalar_join_preserves_caller_certificate_controls", "module.functions.len() <= 64", "assert!(!pure.contains_key(name)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_scalar_join_helpers.ns",
        required_patterns: &["fn record_marker", "fn record_total", "let selected_root: i64 = if choice { root } else { 0 };", "let selected_root: i64 = if choice { 0 } else { root };", "record_gate(choice) && record_rhs(selected_root)", "record_gate(choice) || record_rhs(selected_root)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs",
        required_patterns: &["scalar_join_payload_full_exit_stress_fits_unchanged_function_bound", "scalar_join_payload_reference_preserves_lazy_fields_and_faulted_state", "scalar_join_payload_native_preserves_typed_selections_and_exact_fault_prefixes", "scalar_join_payload_native_keeps_complete_loop_reservations", "check_native(cases::scalar_join_variants())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs",
        required_patterns: &["scalar_join_payload_fresh_cache_and_source_free_delivery_preserve_typed_values", "delivery(cases::scalar_join_variants()[0])", "for restore in 0..2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Certified typed-scalar-declaration witnesses:", "Previous ready-value closure checkpoint:", "Previous ready-value closure task checkpoint:", "Previous ready-value closure blocker checkpoint:", "Previous ready-value closure action checkpoint:", "Previous ready-value closure validation checkpoint:", "Previous ready-value closure artifact checkpoint:"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-scalar-join-receipt",
        path: "docs/reference/nuis-native-typed-scalar-joins-v1.md",
        required_patterns: &["Source Declaration Contract", "4,096/64", "31 failure categories", "64-function profile", "strict scalar-write validator is unchanged", "active/99"],
    },
];
