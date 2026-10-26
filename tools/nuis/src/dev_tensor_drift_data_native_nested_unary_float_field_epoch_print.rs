use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-shared-iterative-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_unary_float.rs",
        required_patterns: &[
            "while !matches!(leaf, NirExpr::FieldAccess { .. })",
            "expected != &scalar_type(&expected.name)",
            "op: NirBinaryOp::Xor", "rhs.as_ref() != &NirExpr::Int(mask)",
            "depth = depth.checked_add(3)?", "depth = depth.checked_add(1)?",
            "if depth > 64", "if !consumed", "(depth <= 64).then_some(leaf)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-original-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "unary_float::field_operand(value, expected, depth)?",
            "self.fields.project(leaf, scope, layouts, expected)",
            "missing.intersection(&self.seeds).cloned().collect()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-all-depths-and-corruption",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested_unary_float_print_epoch_tests.rs",
        required_patterns: &[
            "nested_unary_float_print_epoch_counts_every_layer_and_original_projection",
            "nested_unary_float_print_epoch_rejects_bad_operations_at_every_layer",
            "0..=24", "for bad_layer in 0..count", "usize::MAX",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-read-and-root-limits",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested_unary_float_print_epoch_tests.rs",
        required_patterns: &[
            "nested_unary_float_print_epoch_keeps_parameter_and_entry_permissions_local",
            "nested_unary_float_print_epoch_source_keeps_ordinary_reads_and_ancestors_closed",
            "nested_unary_float_print_epoch_retains_shared_root_work_budget",
            "locals.available.is_empty()", "iteration_prints::bounded",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-raw-nir-execution",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested_unary_float_print_epoch_tests.rs",
        required_patterns: &[
            "nested_unary_float_print_epoch_raw_nir_preserves_even_odd_versions_and_faults",
            "for leading in [false, true]", "for selected in [false, true]",
            "for zero in [false, true]", "count % 2 == 0", "NirExpr::UnpackF64Word(word)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-shared-test-source",
        path: "tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print_cases.rs",
        required_patterns: &[
            "pub fn source_with_negations", "for _ in 0..count",
            "pub negations: usize", "pub fn with_negations", "negations: 1",
            "self.negations % 2 == 0", "current ^ self.kind.mask()",
            "current ^= self.kind.mask()", "local ^= self.kind.mask()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-native-even-odd-witnesses",
        path: "tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.rs",
        required_patterns: &[
            "nested_unary_float_field_epoch_print_native_keeps_even_odd_raw_arguments_and_faults",
            "for negations in [2, 3, 6]", "kind.source_with_negations(negations)",
            "bad.with_negations(negations)", "case.kind.observers().iter().zip(case.observer_counts())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-native-budget-and-state",
        path: "tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.rs",
        required_patterns: &[
            "nested_unary_float_field_epoch_print_native_keeps_entry_exhaustion_atomic",
            "nested_unary_float_field_epoch_print_reference_keeps_counts_and_last_state",
            "case.row.initial = kind.nan()", "128", "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-cli-even-odd-delivery",
        path: "tools/nuis/tests/native_session_workflow/unary_float_field_epoch_print.rs",
        required_patterns: &[
            "nested_unary_float_field_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "for negations in [2, 3]", "cases::with_negations(rows, negations)",
            "fresh.host_objects == cached.host_objects", "for i in 0..2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/unary_float_print_epoch_tests.rs",
        required_patterns: &[
            "nested_unary_float_print_epoch_tests.rs", "mod nested;",
            "negate(ty, negate(wrong, field(ty)))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Nested typed unary-float argument print witnesses:",
            "Previous unary-float checkpoint:", "Previous unary-float task checkpoint:",
            "Previous unary-float blocker checkpoint:", "Previous unary-float action checkpoint:",
            "Previous unary-float validation checkpoint:", "Previous unary-float artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-unary-float-field-receipt",
        path: "docs/reference/nuis-native-nested-unary-float-field-epoch-print-v1.md",
        required_patterns: &[
            "Compositional Sign Proof", "even/odd", "nine failure rows",
            "finite CLI", "active/99",
        ],
    },
];
