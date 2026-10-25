use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-original-path-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_field_reads.rs",
        required_patterns: &[
            ".intersection(writable)",
            "Shape::from_layouts(scope.get(name)?, layouts)?",
            "shape.ty == scalar_type(&shape.ty.name)",
            "Shape::from_layouts(&entry.ty, layouts).as_ref() != Some(entry)",
            "path.into_iter().rev()",
            "leaf.ty == *expected",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-no-record-readiness",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "field_reads::Fields::enter(scope, writable, updates, layouts)",
            "self.fields.contains(name, scope)",
            "missing.intersection(&self.seeds).cloned().collect()",
            ".project(value, scope, layouts, &scalar_type(\"i64\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-scoped-checker",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "expression_with_fields(value, locals, updates, own, catalog, layouts, depth, None)",
            "field_reads: Option<&print_epoch::Epoch>",
            "field_reads.and_then(|epoch| epoch.field(value, &locals.scope, layouts))",
            "view.to_mut().available.extend(reads)",
            "Some(print_epoch)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-negative-and-raw-controls",
        path:
            "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/field_print_epoch_tests.rs",
        required_patterns: &[
            "field_print_epoch_checks_exact_entry_shape_and_original_i64_path",
            "field_print_epoch_rejects_nonowned_malformed_and_unready_child_roots",
            "field_print_epoch_never_launders_whole_records_or_ordinary_readiness",
            "field_print_epoch_source_keeps_assignment_condition_and_ancestor_boundaries",
            "field_print_epoch_raw_nir_keeps_old_new_fields_and_skips_poisoned_guards",
            "assert!(locals.available.is_empty())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/field_epoch_print.ns",
        required_patterns: &[
            "struct Outer { inner: Pair }",
            "print(current.inner.total)",
            "print(current.inner.tag)",
            "print(observe(local.inner.total, divisor - index - child))",
            "tag: local.inner.tag + 2",
            "finish(current.inner.total, tail)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/field_epoch_print_cases.rs",
        required_patterns: &[
            "let mut tag = 100i64",
            "trace.extend([71, current, tag])",
            "tag = tag.wrapping_add(1)",
            "tag = tag.wrapping_add(2)",
            "0i64.checked_div(divisor)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-native-equivalence",
        path: "tools/nuisc/tests/native_application_bridge/field_epoch_print.rs",
        required_patterns: &[
            "field_epoch_print_native_keeps_current_versions_and_selected_fault_prefixes",
            "field_epoch_print_reference_preserves_callee_counts_and_last_state",
            "check_sequences_with_i64_prints",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/field_epoch_print.rs",
        required_patterns: &[
            "field_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "project_for_source_with_i64_prints",
            "fresh.host_objects == cached.host_objects",
            "for i in 0..2",
            "trace.starts_with(&String::from_utf8(failed.stdout).unwrap())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Original-path record-field print witnesses:",
            "Previous child-epoch checkpoint:",
            "Previous child-epoch task checkpoint:",
            "Previous child-epoch blocker checkpoint:",
            "Previous child-epoch action checkpoint:",
            "Previous child-epoch validation checkpoint:",
            "Previous child-epoch artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-field-epoch-print-receipt",
        path: "docs/reference/nuis-native-field-epoch-print-v1.md",
        required_patterns: &[
            "Original Field Path",
            "nine failure rows",
            "64 -> 72 -> -2",
            "active/99",
        ],
    },
];
