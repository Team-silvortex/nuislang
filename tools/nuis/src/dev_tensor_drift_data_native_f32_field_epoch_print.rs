use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-argument-kind",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "expected != &scalar_type(&expected.name)",
            "!matches!(expected.name.as_str(), \"bool\" | \"i32\" | \"f32\" | \"f64\")",
            "self.fields.project(value, scope, layouts, expected)",
            "missing.intersection(&self.seeds).cloned().collect()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-shared-completed-signature",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pub(super) fn pure_i64_parameter_type",
            "!helper.pure_value || helper.result != scalar_type(\"i64\")",
            "helper.params.get(index).cloned()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/f32_field_print_epoch_tests.rs",
        required_patterns: &[
            "f32_field_print_epoch_requires_exact_parameter_leaf_and_layout",
            "f32_field_print_epoch_keeps_word_cast_record_and_ordinary_reads_closed",
            "f32_field_print_epoch_source_keeps_conditions_compounds_and_ancestors_closed",
            "f32_field_print_epoch_raw_nir_keeps_ieee_versions_and_skips_poison",
            "f32_field_print_epoch_raw_nir_zero_trips_skip_but_selected_calls_fail",
            "NirExpr::UnpackF32Word",
            "assert!(locals.available.is_empty())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/f32_field_epoch_print.ns",
        required_patterns: &[
            "struct State { total: f32 }",
            "fn observe(value: f32, divisor: i64) -> i64",
            "if value != value",
            "total: -current.inner.total",
            "total: -local.inner.total",
            "let checked = 0 / divisor; return value",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-actual-callee-bits",
        path: "tools/nuisc/tests/native_application_bridge/f32_field_epoch_print_cases.rs",
        required_patterns: &[
            "bitcast float %arg0 to i32",
            "zext i32 %observed_f32_bits to i64",
            "trace.extend([94, i64::from(bits)])",
            "trace.extend([95, i64::from(current)])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-independent-ieee-oracle",
        path: "tools/nuisc/tests/native_application_bridge/f32_field_epoch_print_cases.rs",
        required_patterns: &[
            "previous.unwrap_or(self.initial) as u32",
            "current ^= 0x8000_0000",
            "local ^= 0x8000_0000",
            "value.is_nan()",
            "0x7fc01234",
            "0xff800001",
            "94, 95",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-counts-not-payload-markers",
        path: "tools/nuisc/tests/native_application_bridge/f32_field_epoch_print.rs",
        required_patterns: &[
            "cases::OBSERVERS.iter().zip(case.observer_counts())",
            "Value::F32(f32::from_bits(value as u32))",
            "vec![previous as u64]",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-budget-negative",
        path: "tools/nuisc/tests/native_application_bridge/f32_field_epoch_print.rs",
        required_patterns: &[
            "f32_field_epoch_print_native_keeps_entry_exhaustion_atomic",
            "case.initial = 0x7fc0_1234",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-existing-ieee-codecs",
        path: "tools/nuisc/tests/native_application_bridge/f32_scoped_record_carries.rs",
        required_patterns: &[
            "typed_f32_scoped_record_carries_preserve_all_seed_and_backedge_bits",
            "typed_f32_scoped_record_carries_reject_wrong_word_kinds",
            "typed_f32_scoped_record_carries_transport_wide_private_word_inputs",
            "0x7f80_0001",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-finite-cli-delivery",
        path: "tools/nuis/tests/native_session_workflow/f32_field_epoch_print.rs",
        required_patterns: &[
            "f32_field_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "for rows in cases::cli_sequences()",
            "cases::decimal(next)",
            "fresh.host_objects == cached.host_objects",
            "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-ieee-scalar-inequality",
        path: "crates/yir-lower-llvm/src/scalar_equality_lowering.rs",
        required_patterns: &[
            "fcmp une double",
            "fcmp une float",
            "fcmp oeq double",
            "fcmp oeq float",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-ieee-loop-inequality",
        path: "crates/yir-lower-llvm/src/loop_scalar.rs",
        required_patterns: &[
            "CpuLoopScalarKind::F32",
            "CpuLoopScalarKind::F64",
            "\"ne\" => \"une\"",
            "\"eq\" => \"oeq\"",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-ieee-backend-regressions",
        path: "crates/yir-lower-llvm/src/tests/float_comparison_tests.rs",
        required_patterns: &[
            "scalar_float_comparisons_preserve_ieee_nan_equality_rules",
            "loop_float_comparisons_preserve_ieee_nan_equality_rules",
            "CpuLoopScalarKind::F32",
            "CpuLoopScalarKind::F64",
            "(\"ne\", \"une\")",
            "icmp ne i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-ieee-native-regressions",
        path: "tools/nuisc/tests/native_application_bridge/float_ieee_comparisons.rs",
        required_patterns: &[
            "f32_ieee_native_equality_matches_reference_for_nan_zeros_and_infinities",
            "f64_ieee_native_equality_matches_reference_for_nan_zeros_and_infinities",
            "u64::from(a == b)",
            "u64::from(a != b)",
            "typed_record_inputs::check_flattened",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Typed-f32 field argument print witnesses:",
            "Previous i32-field checkpoint:",
            "Previous i32-field task checkpoint:",
            "Previous i32-field blocker checkpoint:",
            "Previous i32-field action checkpoint:",
            "Previous i32-field validation checkpoint:",
            "Previous i32-field artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-f32-field-epoch-print-receipt",
        path: "docs/reference/nuis-native-f32-field-epoch-print-v1.md",
        required_patterns: &[
            "Raw Callee Words",
            "nine failure rows",
            "finite CLI",
            "active/99",
        ],
    },
];
