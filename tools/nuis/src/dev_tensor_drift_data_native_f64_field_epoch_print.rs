use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-exact-argument-kind",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "expected != &scalar_type(&expected.name)",
            "!matches!(expected.name.as_str(), \"bool\" | \"i32\" | \"f32\" | \"f64\")",
            "self.fields.project(value, scope, layouts, expected)",
            "missing.intersection(&self.seeds).cloned().collect()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-shared-completed-signature",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pub(super) fn pure_i64_parameter_type",
            "!helper.pure_value || helper.result != scalar_type(\"i64\")",
            "helper.params.get(index).cloned()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/f64_field_print_epoch_tests.rs",
        required_patterns: &[
            "f64_field_print_epoch_requires_exact_parameter_leaf_and_layout",
            "f64_field_print_epoch_keeps_word_cast_record_and_ordinary_reads_closed",
            "f64_field_print_epoch_source_keeps_conditions_compounds_and_ancestors_closed",
            "f64_field_print_epoch_raw_nir_keeps_ieee_versions_and_skips_poison",
            "f64_field_print_epoch_raw_nir_zero_trips_skip_but_selected_calls_fail",
            "NirExpr::UnpackF64Word",
            "bits as i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/f64_field_epoch_print.ns",
        required_patterns: &[
            "struct State { total: f64 }",
            "fn observe(value: f64, divisor: i64) -> i64",
            "total: -current.inner.total",
            "total: -local.inner.total",
            "let checked = 0 / divisor; return value",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-actual-callee-bits",
        path: "tools/nuisc/tests/native_application_bridge/f64_field_epoch_print_cases.rs",
        required_patterns: &[
            "bitcast double %arg0 to i64",
            "trace.extend([94, bits as i64])",
            "trace.extend([95, current as i64])",
            "0xbff8_0000_0000_0000_u64 as i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-independent-full-word-oracle",
        path: "tools/nuisc/tests/native_application_bridge/f64_field_epoch_print_cases.rs",
        required_patterns: &[
            "previous.unwrap_or(self.initial) as u64",
            "current ^= 0x8000_0000_0000_0000",
            "local ^= 0x8000_0000_0000_0000",
            "0x7ff812345678abcd",
            "0xfff0000000000001",
            "0x3ff000000000005e",
            "0x3ff000000000005f",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-counts-not-payload-markers",
        path: "tools/nuisc/tests/native_application_bridge/f64_field_epoch_print.rs",
        required_patterns: &[
            "Value::F64(f64::from_bits(value as u64))",
            "cases::OBSERVERS.iter().zip(case.observer_counts())",
            "vec![previous as u64]",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-budget-negative",
        path: "tools/nuisc/tests/native_application_bridge/f64_field_epoch_print.rs",
        required_patterns: &[
            "f64_field_epoch_print_native_keeps_entry_exhaustion_atomic",
            "case.initial = 0x7ff8_1234_5678_abcd",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-existing-word-codecs",
        path: "tools/nuisc/tests/native_application_bridge/f64_scoped_record_carries.rs",
        required_patterns: &[
            "typed_f64_scoped_record_carries_preserve_all_seed_and_backedge_bits",
            "typed_f64_scoped_record_carries_reject_wrong_word_kinds",
            "0x4000_0000_0000_0001",
            "0xffff_ffff_ffff_ffff",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-finite-cli-delivery",
        path: "tools/nuis/tests/native_session_workflow/f64_field_epoch_print.rs",
        required_patterns: &[
            "f64_field_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "for rows in cases::cli_sequences()",
            "cases::decimal(next)",
            "fresh.host_objects == cached.host_objects",
            "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Typed-f64 field argument print witnesses:",
            "Previous f32-field checkpoint:",
            "Previous f32-field task checkpoint:",
            "Previous f32-field blocker checkpoint:",
            "Previous f32-field action checkpoint:",
            "Previous f32-field validation checkpoint:",
            "Previous f32-field artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f64-field-epoch-print-receipt",
        path: "docs/reference/nuis-native-f64-field-epoch-print-v1.md",
        required_patterns: &[
            "Full 64-Bit Words",
            "nine failure rows",
            "finite CLI",
            "active/99",
        ],
    },
];
