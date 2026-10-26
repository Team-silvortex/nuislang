use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-exact-private-sign",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_unary_float.rs",
        required_patterns: &[
            "expected != &scalar_type(&expected.name)",
            "NirExpr::UnpackF32Word(word)", "1_i64 << 31",
            "NirExpr::UnpackF64Word(word)", "i64::MIN",
            "op: NirBinaryOp::Xor", "rhs.as_ref() != &NirExpr::Int(mask)",
            "depth.checked_add(3)?", "depth.checked_add(1)?", "depth <= 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-original-leaf-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "unary_float::field_operand(value, expected, depth)?",
            "self.fields.project(leaf, scope, layouts, expected)",
            "missing.intersection(&self.seeds).cloned().collect()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-shared-argument-position",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "scalar_helpers::pure_i64_parameter_type(callee, index, catalog)?",
            "epoch.unary_argument(", "depth + 1", "scalar_helpers::typed_call_type",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/unary_float_print_epoch_tests.rs",
        required_patterns: &[
            "unary_float_print_epoch_requires_exact_private_sign_and_original_field",
            "unary_float_print_epoch_retains_signature_scope_and_ordinary_read_vetoes",
            "unary_float_print_epoch_source_keeps_compounds_and_ancestors_closed",
            "unary_float_print_epoch_raw_nir_keeps_width_sign_payload_and_source_order",
            "unary_float_print_epoch_zero_trips_skip_but_selected_calls_fail",
            "usize::MAX", "std::mem::swap(lhs, rhs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.ns",
        required_patterns: &[
            "print(observe(-current.inner.total, 1))",
            "print(observe(-local.inner.total, divisor - index - child))",
            "total: -current.inner.total", "total: -local.inner.total",
            "finish(current.inner.total, tail)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print_cases.rs",
        required_patterns: &[
            "pub const KINDS: [Kind; 2]", "current ^ self.kind.mask()",
            "current ^= self.kind.mask()", "local ^= self.kind.mask()",
            "trace.extend([94, negated as i64])", "trace.extend([95, current as i64])",
            "self.row.observer_counts()", "0x3f80005e", "0x3f80005f",
            "0x3ff000000000005e", "0x3ff000000000005f",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-shared-raw-probes",
        path: "tools/nuisc/tests/native_application_bridge/word_session_probe.rs",
        required_patterns: &[
            "check_sequences_with_i64_prints", "SequenceFailure",
            "check_i64_helper_entry_limit_rejection",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-raw-state-and-order",
        path: "tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.rs",
        required_patterns: &[
            "Value::F32(f32::from_bits(value as u32))",
            "Value::F64(f64::from_bits(value as u64))", "for reversed in [false, true]",
            "ApplicationSessionPhase::Faulted", "case.kind.observers().iter().zip(case.observer_counts())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-budget-rejection",
        path: "tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.rs",
        required_patterns: &[
            "case.row.initial = kind.nan()", "128",
            "unary_float_field_epoch_print_native_keeps_entry_exhaustion_atomic",
            "assert!(compiled.yir.functions.len() <= 32)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-cli-delivery",
        path: "tools/nuis/tests/native_session_workflow/unary_float_field_epoch_print.rs",
        required_patterns: &[
            "for kind in cases::KINDS", "for rows in cases::cli_sequences(kind)",
            "fresh.host_objects == cached.host_objects", "for i in 0..2",
            "unary_float_field_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-checkpoint-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Typed unary-float argument print witnesses:",
            "Previous f64-field checkpoint:", "Previous f64-field task checkpoint:",
            "Previous f64-field blocker checkpoint:", "Previous f64-field action checkpoint:",
            "Previous f64-field validation checkpoint:", "Previous f64-field artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-unary-float-field-receipt",
        path: "docs/reference/nuis-native-unary-float-field-epoch-print-v1.md",
        required_patterns: &[
            "Exact Sign Operation", "both widths", "nine failure rows",
            "finite CLI", "active/99",
        ],
    },
];
