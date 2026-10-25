use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-typed-leaf",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "pub(super) fn argument",
            "!matches!(expected.name.as_str(), \"bool\" | \"i32\" | \"f32\" | \"f64\")",
            ".project(value, scope, layouts, &scalar_type(\"i64\"))",
            "missing.intersection(&self.seeds).cloned().collect()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-completed-signature",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pub(super) fn pure_i64_parameter_type",
            "!helper.pure_value || helper.result != scalar_type(\"i64\")",
            "helper.params.get(index).cloned()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-position-not-ambient",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "if depth < 64",
            "scalar_helpers::pure_i64_parameter_type(callee, index, catalog)?",
            "epoch.argument(arg, &locals.scope, layouts, &expected)",
            "scalar_helpers::typed_call_type(callee, &types, catalog)",
            "expression_with_fields(value, locals, updates, own, catalog, layouts, depth, None)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_field_print_epoch_tests.rs",
        required_patterns: &[
            "bool_field_print_epoch_requires_exact_completed_pure_i64_parameter_position",
            "bool_field_print_epoch_grants_one_argument_without_record_or_bool_readiness",
            "bool_field_print_epoch_source_keeps_conditions_initializers_and_ancestors_closed",
            "bool_field_print_epoch_raw_nir_keeps_true_false_old_new_and_skipped_poison",
            "bool_field_print_epoch_raw_nir_zero_trips_skip_but_selected_calls_fail",
            "assert!(locals.available.is_empty())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/bool_field_epoch_print.ns",
        required_patterns: &[
            "fn observe(flag: bool, value: i64, divisor: i64) -> i64",
            "flag: selected",
            "observe(current.inner.flag, current.inner.total, 1)",
            "observe(local.inner.flag, local.inner.total, divisor - index - child)",
            "flag: current.inner.flag == false",
            "flag: local.inner.flag == false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/bool_field_epoch_print_cases.rs",
        required_patterns: &[
            "fn observed(",
            "let mut flag = self.selected",
            "flag = !flag",
            "0i64.checked_div(divisor)",
            "if flag { 0 } else { 10 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-native-equivalence",
        path: "tools/nuisc/tests/native_application_bridge/bool_field_epoch_print.rs",
        required_patterns: &[
            "bool_field_epoch_print_native_keeps_current_versions_and_selected_fault_prefixes",
            "bool_field_epoch_print_reference_preserves_callee_counts_and_last_state",
            "check_sequences_with_i64_prints",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/bool_field_epoch_print.rs",
        required_patterns: &[
            "bool_field_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "project_for_source_with_i64_prints",
            "fresh.host_objects == cached.host_objects",
            "for i in 0..2",
            "compile_cache: hit",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Typed-bool field argument print witnesses:",
            "Previous field-epoch checkpoint:",
            "Previous field-epoch task checkpoint:",
            "Previous field-epoch blocker checkpoint:",
            "Previous field-epoch action checkpoint:",
            "Previous field-epoch validation checkpoint:",
            "Previous field-epoch artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-epoch-print-receipt",
        path: "docs/reference/nuis-native-bool-field-epoch-print-v1.md",
        required_patterns: &[
            "Exact Argument Position",
            "nine failure rows",
            "64 -> 72 -> -2",
            "active/99",
        ],
    },
];
