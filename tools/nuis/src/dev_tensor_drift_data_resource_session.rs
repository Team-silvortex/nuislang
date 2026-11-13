use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-resource-inferred-shadowing-types",
        path: "tools/nuisc/src/resource_session_bindings/types.rs",
        required_patterns: &[
            "pub(super) fn validate",
            "requires resolved i64 on both sides",
            "seen.len() != shadows.len()",
            "visible.get(shadow.previous.as_str())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-inferred-shadowing-runtime",
        path: "tools/nuisc/tests/native_resource_session/inferred_shadowing.rs",
        required_patterns: &[
            "inferred_and_aliased_shadows_preserve_all_four_paths_for_120_events_each",
            "inferred_outer_locals_and_nested_generic_aliases_keep_lexical_identity",
            "aliases_and_inference_do_not_grant_resource_or_non_i64_shadowing",
            "inferred_suffix_allocation_failure_keeps_state_and_cleans_up",
            "inferred_record_fields_resolve_without_granting_native_record_helper_calls",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-inferred-shadowing-receipt",
        path: "docs/reference/nuis-native-resource-inferred-shadowing-v1.md",
        required_patterns: &["active/95", "120 events each", "## Remaining Boundaries"],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-scalar-shadowing-source",
        path: "tools/nuisc/src/resource_session_bindings.rs",
        required_patterns: &[
            "pub(crate) fn normalize",
            "AstStmt::AssignLocal",
            "collect_names",
            "__nuis_resource_local_",
            "Cow::Borrowed(module)",
            "lower_project_ast_to_nir(&self.ast, local_modules)",
            "types::validate(&nir, &self.shadows)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-scalar-shadowing-runtime",
        path: "tools/nuisc/tests/native_resource_session/scalar_shadowing.rs",
        required_patterns: &[
            "paired_scalar_shadowing_keeps_outer_state_and_selected_early_returns",
            "nested_and_empty_else_scopes_restore_each_outer_binding",
            "captured_assignments_and_owned_shadowing_remain_rejected_on_both_edges",
            "shadowed_errors_keep_the_last_committed_state_and_release_owners",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-scalar-shadowing-receipt",
        path: "docs/reference/nuis-native-resource-scalar-shadowing-v1.md",
        required_patterns: &["active/95", "120 events", "## Remaining Boundaries"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-owned-continuation-source",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/branches.rs",
        required_patterns: &[
            "pub(super) fn pair",
            "continuations::share",
            "continuations::check_captures",
            "regions::terminal_return_arm",
            "MAX_RETURN_HELPER_FUNCTIONS",
            "std::mem::take(else_body)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-owned-continuation-runtime",
        path: "tools/nuisc/tests/native_resource_session/paired_continuations.rs",
        required_patterns: &[
            "paired_arms_preserve_snapshots_and_share_one_suffix_for_120_events",
            "paired_suffix_allocation_failure_preserves_state_on_both_continuing_edges",
            "paired_join_rejects_rebinding_consumption_and_drift_on_either_owner_edge",
            "buffer_predicate_is_not_retested_and_branch_locals_need_no_shared_owner",
            "nested_pairs_mix_terminal_and_continuing_arms_without_extra_suffix_work",
            "paired_shared_suffixes_obey_the_existing_closure_depth_budget",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-owned-continuation-receipt",
        path: "docs/reference/nuis-native-paired-owned-continuation-v1.md",
        required_patterns: &["active/95", "120 events", "## Remaining Boundaries"],
    },
    DevTensorDriftCheckSpec {
        id: "native-owned-continuation-source-sharing",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/continuations.rs",
        required_patterns: &[
            "__nuis_owned_continue_",
            "helper.body = body",
            "owned continuation cannot rebind captured",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-owned-continuation-terminal-contract",
        path: "crates/yir-core/src/guarded_owned_call.rs",
        required_patterns: &[
            "pub fn parse_tail",
            "terminal_continuation_has_no_condition_and_keeps_exact_owner_trailers",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-owned-continuation-runtime-proof",
        path: "tools/nuisc/tests/native_resource_session/continuations.rs",
        required_patterns: &[
            "nonterminal_join_preserves_owners_and_runs_one_shared_suffix",
            "continuation_allocation_failure_is_forwarded_without_unpacking_or_publication",
            "nested_open_arms_rejoin_in_order_without_repeating_the_root_suffix",
            "continuation_rejects_consumed_owners_rebinding_and_malformed_tail_contracts",
            "shared_suffix_checked_arithmetic_is_skipped_by_a_real_inner_return",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-owned-continuation-receipt",
        path: "docs/reference/nuis-native-owned-continuation-v1.md",
        required_patterns: &[
            "active/95",
            "tail_call_owned_struct_return",
            "## Remaining Boundaries",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-owned-shared-bounds",
        path: "crates/yir-core/src/native_resource_session.rs",
        required_patterns: &[
            "MAX_RETURN_HELPER_DEPTH: usize = 8",
            "MAX_RETURN_HELPER_FUNCTIONS: usize = 32",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-owned-helper-queue",
        path: "tools/nuisc/src/lowering/guarded_owned_returns.rs",
        required_patterns: &[
            "context.pending.pop_front()",
            "outline_body(&mut helper, depth",
            "branches::arm",
            "filter(|p| owned_bytes(&p.ty))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-owned-terminal-region",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/regions.rs",
        required_patterns: &[
            "MAX_RETURN_HELPER_DEPTH",
            "ends_in_return(then_body) && ends_in_return(else_body)",
            "transfers: live.iter().cloned().collect()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-owned-execution-proof",
        path: "tools/nuisc/tests/native_resource_session/nested_returns.rs",
        required_patterns: &[
            "nested_owned_exits_preserve_each_snapshot_and_each_surviving_owner",
            "nested_result_failures_propagate_without_running_parent_continuations",
            "nested_error_releases_all_ancestor_owners_and_stays_failed_after_close",
            "nested_edges_and_callee_consumption_are_revalidated_at_every_level",
            "source_and_native_admission_share_depth_and_function_budgets",
            "nonterminal_outer_resource_join_uses_a_shared_owned_continuation",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-nested-owned-receipt",
        path: "docs/reference/nuis-native-nested-owned-return-v1.md",
        required_patterns: &[
            "active/95",
            "## Remaining Boundaries",
            "120 events",
            "32 functions",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-transferred-bytes-contract",
        path: "crates/yir-core/src/guarded_owned_call.rs",
        required_patterns: &[
            "transfer_bytes_v1=",
            "pub transfers:",
            "transfer_and_cleanup_form_a_bounded_disjoint_canonical_partition",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-transferred-bytes-preflight",
        path: "crates/yir-lower-llvm/src/guarded_owned_call_tests.rs",
        required_patterns: &[
            "transfers_are_typed_pointer_arguments_and_cleanup_is_inside_the_taken_block",
            "invalid_transfer_preflight_emits_no_cleanup_call_or_branch",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-transferred-bytes-native-proof",
        path: "tools/nuisc/tests/native_resource_session/transferred_bytes.rs",
        required_patterns: &[
            "selected_bytes_transfer_preserves_snapshots_and_fallthrough_ownership",
            "transfer_partition_signature_and_helper_consumption_are_revalidated",
            "transferred_bytes_are_released_when_selected_result_allocation_fails",
            "selected_helper_can_consume_transferred_bytes_before_an_error_return",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-transferred-bytes-receipt",
        path: "docs/reference/nuis-native-transferred-bytes-return-v1.md",
        required_patterns: &[
            "active/95",
            "transfer_bytes_v1=N",
            "## Remaining Boundaries",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-bytes-cleanup-contract",
        path: "crates/yir-core/src/guarded_owned_call.rs",
        required_patterns: &[
            "cleanup_bytes_v1=",
            "pub cleanup:",
            "cleanup_trailer_is_bounded_canonical_and_disjoint_from_captures",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-bytes-source-cleanup",
        path: "tools/nuisc/src/lowering/guarded_owned_returns/owners.rs",
        required_patterns: &[
            "extract_cleanup",
            "mentions_owner",
            "remaining.remove(owner)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-bytes-path-proof",
        path: "tools/nuisc/tests/native_resource_session/retained_bytes.rs",
        required_patterns: &[
            "caller_bytes_cleanup_is_taken_only_and_fallthrough_keeps_reads_and_transfer",
            "caller_bytes_are_released_before_selected_helper_allocation_failure",
            "caller_cleanup_contract_rejects_missing_extra_duplicate_and_consumed_owners",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-bytes-receipt",
        path: "docs/reference/nuis-native-retained-bytes-return-v1.md",
        required_patterns: &["active/90", "cleanup_bytes_v1=N", "## Remaining Boundaries"],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-resource-source-arms",
        path: "tools/nuisc/src/lowering/guarded_owned_returns.rs",
        required_patterns: &[
            "guard_call_owned_struct_return",
            "push_lifetime_edge",
            "host_entries.contains",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-resource-helper-admission",
        path: "crates/yir-lower-llvm/src/native_resource_session/admission/guarded_helpers.rs",
        required_patterns: &[
            "recursive resource return helper",
            "live caller owners",
            "capture drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-resource-native-proof",
        path: "tools/nuisc/tests/native_resource_session/early_ok.rs",
        required_patterns: &[
            "early_ok_selects_one_resource_path_and_releases_all_branch_allocations",
            "early_ok_allocation_failures_do_not_execute_on_unselected_paths_or_publish_state",
            "early_ok_serializes_helpers_and_rejects_owner_or_capture_drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-resource-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/early_ok.rs",
        required_patterns: &[
            "shared_resource_continuation_survives_cache_nsb_and_binary_only_launch",
            "materialize-artifact",
            ".env(\"PATH\", \"\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-resource-receipt",
        path: "docs/reference/nuis-native-guarded-resource-return-v1.md",
        required_patterns: &["active/85", "## Remaining Boundaries", "120 events"],
    },
    DevTensorDriftCheckSpec {
        id: "native-checked-resource-layout",
        path: "crates/yir-core/src/native_resource_session/outcome.rs",
        required_patterns: &["parse_outcome", "Result.Err(i64)", "error_slot"],
    },
    DevTensorDriftCheckSpec {
        id: "native-checked-resource-errors",
        path: "tools/nuisc/tests/native_resource_session/checked.rs",
        required_patterns: &[
            "checked_dimensions_return_nuis_errors_before_any_state_is_published",
            "checked_event_and_close_errors_preserve_both_halves_and_remain_failed",
            "checked_allocation_failure_is_not_absence_or_an_application_error",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-checked-resource-delivery",
        path: "tools/nuis/tests/native_resource_session_workflow/checked.rs",
        required_patterns: &[
            "checked_canvas_errors_and_success_survive_source_free_delivery",
            "materialize-artifact",
            ".env(\"PATH\", \"\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-checked-resource-inactive-owner",
        path: "crates/yir-lower-llvm/src/tests/owned_return_contract_tests.rs",
        required_patterns: &[
            "inactive_variant_bytes_are_absent_but_runtime_allocations_stay_checked",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-checked-resource-receipt",
        path: "docs/reference/nuis-native-checked-resource-state-v1.md",
        required_patterns: &[
            "nuis-native-checked-resource-session-v1",
            "active/80",
            "## Remaining Boundaries",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-mixed-resource-state-layout",
        path: "crates/yir-core/src/native_resource_session/mixed.rs",
        required_patterns: &[
            "1..=16 i64 fields",
            "mixed callback State parameter field order drift",
            "mixed callback State layout drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-mixed-resource-state-lifetime-proof",
        path: "tools/nuisc/tests/native_resource_session/mixed.rs",
        required_patterns: &[
            "mixed_failure_never_publishes_either_half_and_close_cannot_erase_failure",
            "mixed_aggregate_allocation_failure_releases_its_bytes_and_never_opens",
            "mixed_transport_is_not_a_canvas_specific_field_or_arity_combination",
            "mixed_descriptor_rejects_identity_and_version_before_callbacks",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-mixed-resource-state-artifact-proof",
        path: "tools/nuis/tests/native_resource_session_workflow/mixed.rs",
        required_patterns: &[
            "mixed_canvas_nuis_owned_extent_control_and_source_free_artifact",
            "--resource-frame-fields",
            "materialize-artifact",
            ".env(\"PATH\", \"\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-mixed-resource-state-receipt",
        path: "docs/reference/nuis-native-mixed-resource-state-v1.md",
        required_patterns: &[
            "nuis-native-mixed-resource-session-v1",
            "active/75",
            "## Remaining Boundaries",
            "No fresh Linux/GPU execution",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-frame-accepted-state-consumer",
        path: "crates/yir-runtime-host/src/native_resource_session/frame.rs",
        required_patterns: &[
            "session.completion_status()?",
            "frame.ppm(session.state())",
            "unsigned packed RGBA8 cells",
            "native-cpu-resource-session",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-frame-source-free-pixel-proof",
        path: "tools/nuis/tests/native_resource_session_workflow/frame.rs",
        required_patterns: &[
            "native_canvas_exact_pixels_preflight_faults_and_source_free_delivery",
            "verify_frame_export_capability",
            "materialize-artifact",
            ".env(\"PATH\", \"\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-frame-nuis-caller",
        path: "examples/projects/domains/native_canvas_session_demo/main.ns",
        required_patterns: &[
            "canvas_open",
            "canvas_event",
            "canvas_close",
            "store_at(pixels, index, color)",
            "return copy_bytes(pixels)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-frame-receipt",
        path: "docs/reference/nuis-native-resource-frame-export-v1.md",
        required_patterns: &[
            "nuis-native-resource-frame-rgba8-v1",
            "active/65",
            "## Remaining Boundaries",
            "No fresh Linux/GPU execution",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-contract",
        path: "crates/yir-core/src/native_resource_session.rs",
        required_patterns: &[
            "nuis-native-resource-session-v1",
            "MAX_BUFFER_WORDS: usize = 4096",
            "parameters[0].ownership != YirValueOwnership::Borrowed",
            "r.ownership == YirValueOwnership::Owned",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-root-preservation",
        path: "tools/nuisc/src/pipeline_resource_session.rs",
        required_patterns: &[
            "compile_resource_session_source",
            "lower_prepared_pipeline_with_host_entries",
            "registration.bind(artifacts.yir())?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-bounded-admission",
        path: "crates/yir-lower-llvm/src/native_resource_session/admission.rs",
        required_patterns: &[
            "body.len() > 256",
            "copies > 8",
            "!owners.contains(arg)",
            "resource callback leaks an owned Bytes value",
            "resource callback has an external ordering or lifetime dependency",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-native-lifetime-proof",
        path: "tools/nuisc/tests/native_resource_session.rs",
        required_patterns: &[
            "owned_buffer_bytes_callbacks_execute_and_release_on_every_completion",
            "native_resource_failed_completion_releases_bytes_without_publishing_or_retrying",
            "resource_callback_rejects_signature_result_and_linear_ownership_drift",
            "resource_callbacks_support_empty_maximal_and_reordered_scopes_with_private_bytes",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-trap-process-boundary",
        path: "tools/nuisc/tests/native_resource_session/trap.rs",
        required_patterns: &[
            "native_resource_trap_is_a_process_failure_not_recoverable_completion",
            "store_at(buffer, 99, value)",
            "resource-before-trap:10,-7",
            "output.status.signal()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-receipt-and-open-boundary",
        path: "docs/reference/nuis-native-resource-session-v1.md",
        required_patterns: &[
            "## Ownership And Failure",
            "not a secret or unforgeable capability",
            "## Not Yet Delivered",
            "active/35",
            "No fresh Linux/GPU execution",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-project-registration",
        path: "tools/nuisc/src/project/application_sessions.rs",
        required_patterns: &[
            "parse_resource_sessions",
            "ResourceSessionRegistration::from_fields",
            "duplicate resource session ID",
            "validate_registrations(module)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-nsb-export-checkpoint",
        path: "tools/nuisc/src/aot_resource_session.rs",
        required_patterns: &[
            "native-resource-session-aot-bundle:",
            "resource session LLVM checkpoint identity mismatch",
            "native_resource_registration",
            "native_resource_close_export",
            "unexpected resource session bundle claim",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-static-entry-identity",
        path: "crates/yir-runtime-host/src/native_resource_session/entry.rs",
        required_patterns: &[
            "NativeResourceDescriptorV1",
            "native resource artifact graph identity mismatch",
            "native resource registration/export identity mismatch",
            "ResourceScript::parse",
            "session.completion_status()?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-static-binary-producer",
        path: "tools/yir-pack-aot/src/host_resource_session.rs",
        required_patterns: &[
            "ensure_runtime_host_staticlib_built",
            "nuis_native_resource_script_main",
            "static_native_resource_session",
            "native_resource_event_export",
            "single_binary=true",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-source-free-workflow",
        path: "tools/nuis/tests/native_resource_session_workflow.rs",
        required_patterns: &[
            "resource_build_cache_nsb_relocation_and_binary_only_launch",
            "compile_cache: hit",
            "materialize-artifact",
            "binary-only",
            ".env(\"PATH\", \"\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-resource-session-artifact-receipt",
        path: "docs/reference/nuis-native-resource-session-artifacts-v1.md",
        required_patterns: &[
            "active/60",
            "resource_sessions",
            "native-resource-session-aot-bundle:bytes",
            "not authentication",
            "## Remaining Boundaries",
        ],
    },
];
