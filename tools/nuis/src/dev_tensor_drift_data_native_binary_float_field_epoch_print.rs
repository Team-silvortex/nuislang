use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-shared-typed-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "expected != &scalar_type(&expected.name)", "NirBinaryOp::Div",
            "let mut work = 4096usize", "pending.push(Step::Value(rhs, child, right))",
            "pending.push(Step::Value(lhs, child, lazy_rhs))", "unary_float::operand(value, &ty)?",
            "depth.checked_add(3)?", "proofs[0].ty == *expected",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-certificate-and-ready-read",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "pub(super) fn float_argument", "binary_float::validate",
            "self.fields.project(value, scope, layouts, expected)", "ready(value, depth)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-shared-parameter-validation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "scalar_helpers::pure_i64_parameter_type(callee, index, catalog)?",
            "epoch.float_argument(", "expression(", "|leaf, depth|",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-shared-type-division",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values/typed_tests.rs",
        required_patterns: &[
            "binary_type(NirBinaryOp::Div", "Some(scalar_type(\"f32\"))",
            "NirBinaryOp::Rem", "scalar_type(\"f32\"), scalar_type(\"f64\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-compiler-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/binary_float_print_epoch_tests.rs",
        required_patterns: &[
            "binary_float_print_epoch_checks_all_ops_and_exact_ready_leaf_types",
            "binary_float_print_epoch_keeps_ordinary_reads_ancestors_and_stale_roots_closed",
            "binary_float_print_epoch_bounds_depth_work_and_checks_left_before_right",
            "binary_float_print_epoch_source_does_not_leak_into_conditions_or_initializers",
            "binary_float_print_epoch_raw_nir_keeps_ieee_values_and_selected_poison_lazy",
            "binary_float_print_epoch_checked_division_rejects_both_zero_signs_only_when_reached",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-checked-zero-owner",
        path: "crates/yir-lower-llvm/src/checked_float_division.rs",
        required_patterns: &[
            "fcmp oeq {ty}", "float_divisor_invalid_", "float_divisor_ready_",
            "call void @llvm.trap()", "unreachable",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-scalar-division-owner",
        path: "crates/yir-lower-llvm/src/scalar_lowering.rs",
        required_patterns: &[
            "checked_float_division::check(&rhs, \"float\", body, next_reg)",
            "checked_float_division::check(&rhs, \"double\", body, next_reg)",
            "\"div_f32\" =>", "\"div_f64\" =>",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-division-unit-controls",
        path: "crates/yir-lower-llvm/src/tests/float_division_tests.rs",
        required_patterns: &[
            "scalar_float_division_checks_both_zero_signs_before_generic_and_typed_divide",
            "loop_float_division_reuses_checked_zero_guard_with_unique_labels",
            "labels.len(), 4", "fcmp oeq",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-source-program",
        path: "tools/nuisc/tests/native_application_bridge/binary_float_field_epoch_print.ns",
        required_patterns: &[
            "tail: i64, ready: f64", "current.inner.total + ready",
            "ready - local.inner.total", "local.inner.total * ready",
            "finish(current.inner.total, tail)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-independent-ieee-oracle",
        path: "tools/nuisc/tests/native_application_bridge/binary_float_field_epoch_print_cases.rs",
        required_patterns: &[
            "calculation!(calculate_f32", "calculation!(calculate_f64",
            "denominator == 0.0", "result.is_nan()", "current ^= self.kind().mask()",
            "local ^= self.kind().mask()", "kind.last_state()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-input-result-state-probes",
        path: "tools/nuisc/tests/native_application_bridge/binary_float_field_epoch_print_cases.rs",
        required_patterns: &[
            "bitcast float %arg7 to i32", "bitcast double %arg7 to i64",
            "trace.extend([96, self.operand])", "trace.extend([94, bits as i64])",
            "trace.extend([95, current as i64])", "self.base.observer_counts()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-native-atomicity",
        path: "tools/nuisc/tests/native_application_bridge/binary_float_field_epoch_print.rs",
        required_patterns: &[
            "binary_float_field_epoch_print_native_keeps_ieee_arguments_and_float_fault_prefixes",
            "binary_float_field_epoch_print_native_keeps_entry_exhaustion_atomic",
            "binary_float_field_epoch_print_reference_keeps_counts_and_last_state",
            "case.base.row.initial = kind.nan()", "128", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-cli-delivery",
        path: "tools/nuis/tests/native_session_workflow/binary_float_field_epoch_print.rs",
        required_patterns: &[
            "binary_float_field_epoch_print_fresh_cache_and_source_free_delivery_keep_operand_checks",
            "cases::source(kind)", "fresh.host_objects == cached.host_objects", "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Binary typed float argument print witnesses:", "Previous nested-unary checkpoint:",
            "Previous nested-unary task checkpoint:", "Previous nested-unary blocker checkpoint:",
            "Previous nested-unary action checkpoint:", "Previous nested-unary validation checkpoint:",
            "Previous nested-unary artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-receipt",
        path: "docs/reference/nuis-native-binary-float-field-epoch-print-v1.md",
        required_patterns: &[
            "Typed Binary Plan", "Checked Division", "thirteen failure rows",
            "finite CLI", "active/99", "NaN payload",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-binary-float-field-registration-base",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &[
            "dev_tensor_drift_data_native_iteration_base.rs", "use base::CHECKS",
            "binary_float_field_epoch_print::CHECKS",
        ],
    },
];
