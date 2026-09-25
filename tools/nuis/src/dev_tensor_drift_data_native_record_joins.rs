use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "private-record-join-copy-family-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_join_plan.rs",
        required_patterns: &[
            "binding.valid &= ty.is_some()",
            "binding.ty.as_ref() == ty",
            "(scope != binding.scope && !info.returns) || (info.in_loop && rebound)",
            "!control_values::supported_type(ty, layouts)",
            "path.len() == 1 || field_type(ty, &path[1..], layouts).is_none()",
            "inputs.insert(param.name.clone(), paths.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-record-join-input-reconstruction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_joins.rs",
        required_patterns: &[
            "plan::collect(function, layouts)",
            "control_values::zero_value(&param.ty, layouts)",
            "__nuis_capture_join_input",
            "names.get(name)",
            "function.body.splice(0..0, seeds)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-record-join-reconstruction-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_join_tests.rs",
        required_patterns: &[
            "fallthrough_record_joins_project_fields_without_changing_public_inputs",
            "fallthrough_record_joins_keep_nested_reaching_values_and_old_copies",
            "fallthrough_record_joins_retain_loop_reads_without_rewriting_backedges",
            "fallthrough_record_joins_preserve_selected_unused_constructor_work_and_calls",
            "fallthrough_record_joins_project_copy_families_and_nominal_prefixes",
            "fallthrough_record_joins_leave_whole_uses_and_computed_callers_unchanged",
            "fallthrough_record_joins_require_exact_types_and_preserve_name_hygiene",
            "fallthrough_record_joins_preserve_scalar_kinds",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-record-join-source-composition",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn join_source()",
            "return current.f0 + old.f3;",
            "3 => \"999\"",
            "fields(\"old.f63 / old.f1 + 27\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-record-loop-entry-reconstruction-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_loop_join_tests.rs",
        required_patterns: &[
            "loop_record_inputs_preserve_zero_trips_and_per_trip_snapshots",
            "loop_record_inputs_preserve_nested_writes_and_control_exits",
            "loop_record_inputs_keep_checked_unused_constructor_work_per_trip",
            "loop_record_inputs_follow_cyclic_copies_and_nominal_prefixes",
            "loop_record_inputs_reject_whole_escapes_and_computed_callers_transactionally",
            "loop_record_reconstruction_retains_original_bodies_and_exact_type_vetoes",
            "divisor - i + 1",
            "assert_eq!(&helper.body[2..], &before.body[1..])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-record-loop-snapshot-source-composition",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn loop_join_source()",
            "let current = current; let before = current;",
            "before.marker + current.marker",
            "divisor: 0, marker: 0, unused: 7 }, 0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-record-loop-snapshot-native-lifecycle",
        path: "tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs",
        required_patterns: &[
            "typed_sparse_captures_preserve_loop_record_snapshots",
            "let source = aliases::loop_join_source()",
            "assert_eq!(slots, [0, 1, 2, 4])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-record-loop-snapshot-source-free-restoration",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_sparse_captures_loop_snapshots_build_cache_and_restore_without_sources",
            "check_sparse_workflow_with_iteration(",
            "&aliases::loop_join_source()",
        ],
    },
];
