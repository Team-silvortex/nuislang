use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-scalar-input-alias-bounded-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scalar_aliases.rs",
        required_patterns: &[
            "normalized(function, layouts, controls, 65_536)",
            "depth >= 64 || !walk::supported_expr(expr)",
            "self.writes.get(name) == Some(&1) && !self.parameters.contains(name)",
            "declared.is_none_or(|ty| ty == &origin.ty)",
            "parameters.extend(controls.iter().cloned())",
            "validate_expansion(&result, &mut context.remaining)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-input-alias-candidate-integration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs",
        required_patterns: &[
            "let mut candidate = module.functions[index].clone()",
            "Scoped helpers still own named control identities and transport seeds",
            "scalar_aliases::normalize(&mut candidate, layouts, &control_names)",
            "record_views::normalize(&mut candidate, layouts)",
            "valid_caller(&module.functions[*i].body, &name, &plan)",
            "caller_records::valid(&module.functions[*i], &name, &plan, layouts)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-input-alias-negative-and-differential-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scalar_aliases_tests.rs",
        required_patterns: &[
            "private_scalar_aliases_do_not_promote_computed_local_codec_or_changed_versions",
            "private_scalar_aliases_keep_branch_and_loop_locals_lexical",
            "private_scalar_aliases_keep_registered_control_identities",
            "private_scalar_aliases_remain_transactional_when_any_caller_is_computed",
            "private_scalar_aliases_preserve_guarded_returns_and_unused_checked_work",
            "private_scalar_aliases_reduce_64_input_fields_without_widening_any_caller",
            "private_scalar_aliases_reject_exhausted_and_deep_work_without_partial_rewrite",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-input-alias-full-width-execution",
        path: "tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs",
        required_patterns: &[
            "typed_sparse_scalar_aliases_unlock_full_width_private_record_copies",
            "&aliases::scalar_copy_source(false), Some(&[2, 3])",
            "typed_sparse_scalar_aliases_preserve_unused_calls_at_the_selected_branch",
            "&aliases::scalar_copy_source(true), Some(&[3, 3])",
            "typed_sparse_scalar_aliases_keep_call_backed_record_transport_conservative",
            "signature.contains(\"[64 x i64] %arg1\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-input-alias-source-free-restoration",
        path: "tools/nuis/tests/native_session_workflow/capture_fields.rs",
        required_patterns: &[
            "native_scalar_alias_copies_build_cache_and_restore_without_sources",
            "&aliases::scalar_copy_source(false), Some(&[1, 2])",
            "&aliases::scalar_copy_source(true), Some(&[2, 2])",
            "&aliases::aggregate_transport_source(), Some(&[0, 2])",
            "fs::remove_file(project.0.join(\"main.ns\"))",
            "assert_eq!(fs::read_to_string(restored.join(&llvm_name)).unwrap(), llvm)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scalar-input-alias-documented-boundaries",
        path: "docs/reference/nuis-native-scalar-loop-snapshots-v1.md",
        required_patterns: &[
            "## Scalar Input Aliases",
            "Scoped transport helpers are not",
            "Ten unit tests include a two-storage-order",
            "or 3/3 when an unused selected field still evaluates an opaque scalar call",
            "This is bounded input-alias closure",
        ],
    },
];
