use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-shared-parameter-type",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pub(super) fn pure_i64_parameter_type",
            "!helper.pure_value || helper.result != scalar_type(\"i64\")",
            "helper.params.get(index).cloned()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-exact-argument-policy",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "pub(super) fn argument",
            "expected != &scalar_type(&expected.name)",
            "!matches!(expected.name.as_str(), \"bool\" | \"i32\" | \"f32\" | \"f64\")",
            "self.fields.project(value, scope, layouts, expected)",
            "missing.intersection(&self.seeds).cloned().collect()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-local-position-and-depth",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "if depth < 64",
            "field_reads.and_then(|epoch|",
            "scalar_helpers::pure_i64_parameter_type(callee, index, catalog)?",
            ".argument(arg, &locals.scope, layouts, &expected)",
            "scalar_helpers::typed_call_type(callee, &types, catalog)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_field_print_epoch_tests.rs",
        required_patterns: &[
            "i32_field_print_epoch_requires_exact_completed_parameter_and_leaf_kinds",
            "i32_field_print_epoch_lends_no_cast_record_or_ordinary_readiness",
            "i32_field_print_epoch_source_keeps_conditions_initializers_and_ancestors_closed",
            "i32_field_print_epoch_raw_nir_preserves_wrap_sign_zero_and_old_new_versions",
            "i32_field_print_epoch_raw_nir_zero_trips_skip_but_selected_prints_fail",
            "value.wrapping_add(1)",
            "assert!(locals.available.is_empty())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/i32_field_epoch_print.ns",
        required_patterns: &[
            "struct State { total: i32 }",
            "fn observe(value: i32, divisor: i64) -> i64",
            "total: current.inner.total + i32_from_i64(delta)",
            "total: local.inner.total + i32_from_i64(delta)",
            "State { total: i32_from_i64(initial) }",
            "return State { total: finish(current.inner.total, tail) }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-independent-signed-oracle",
        path: "tools/nuisc/tests/native_application_bridge/i32_field_epoch_print_cases.rs",
        required_patterns: &[
            "previous.unwrap_or(self.initial) as i32",
            "i64::from(value.signum())",
            "current.wrapping_add(self.delta as i32)",
            "local.wrapping_add(self.delta as i32)",
            "i64::from(current.wrapping_add(checked as i32))",
            "pub fn sequences()",
            "4294967295",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-reference-native-words",
        path: "tools/nuisc/tests/native_application_bridge/i32_field_epoch_print.rs",
        required_patterns: &[
            "i32_field_epoch_print_native_keeps_current_versions_and_selected_fault_prefixes",
            "i32_field_epoch_print_reference_preserves_callee_counts_and_last_state",
            "i32_field_epoch_print_native_keeps_entry_exhaustion_atomic",
            "for rows in cases::sequences()",
            "vec![previous as u64]",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-budget-policy-isolation",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "pub(super) fn check_helper_entry_limit_rejection",
            "pub(super) fn check_i64_helper_entry_limit_rejection",
            "if computed",
            "LiteralPrintPolicy::new(sites)",
            "native helper entry limit exhausted",
            "assert!(actual.len() < trace.len())",
            "trace.starts_with(&actual)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-existing-word-guards",
        path: "tools/nuisc/tests/native_application_bridge/i32_scoped_record_carries.rs",
        required_patterns: &[
            "typed_i32_scoped_record_carries_preserve_signed_words_zero_trips_and_snapshots",
            "typed_i32_scoped_record_carries_transport_wide_signed_backedges",
            "typed_i32_scoped_record_carries_reject_wrong_cast_kinds",
            "narrow as i64 as u64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/i32_field_epoch_print.rs",
        required_patterns: &[
            "i32_field_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "for rows in cases::sequences()",
            "total: {next}i32",
            "fresh.host_objects == cached.host_objects",
            "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Typed-i32 field argument print witnesses:",
            "Previous bool-field checkpoint:",
            "Previous bool-field task checkpoint:",
            "Previous bool-field blocker checkpoint:",
            "Previous bool-field action checkpoint:",
            "Previous bool-field validation checkpoint:",
            "Previous bool-field artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-i32-field-epoch-print-receipt",
        path: "docs/reference/nuis-native-i32-field-epoch-print-v1.md",
        required_patterns: &[
            "Shared Typed Argument",
            "nine failure rows",
            "signed State words",
            "active/99",
        ],
    },
];
