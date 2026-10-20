use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-checked-fallback",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs",
        required_patterns: &[
            "coalesced_effects(function, layouts, catalog, &plan).unwrap_or(plan)",
            "fn coalesced_effects",
            "catalog.get(&function.name)?",
            "effect_prefix::verify_shared",
            "storage::candidate(function, layouts, catalog)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-separate-certificate-and-source-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs",
        required_patterns: &[
            "verify_independent(function, independent, layouts)?",
            "scalar_helpers::validate_iteration_body(function, &independent.body",
            "scalar_helpers::validate_iteration_body(function, &shared.body",
            "shared.signal.name() != independent.signal.name()",
            "if scope.contains_key(storage)",
            "if !mutable || ty != *result",
            "None if !initialized => initialized = true",
            "Some(previous) if initialized && previous == result",
            "NirStmt::Let { name, .. } if in_loop && name == storage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-admission-and-read-set-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_tests.rs",
        required_patterns: &[
            "effect_return_storage_reuses_real_record_carries_without_pure_or_name_authority",
            "effect_return_storage_revalidates_new_inner_writes_and_keeps_independent_fallback",
            "effect_return_storage_never_uses_parameters_constants_or_noncarried_locals",
            "effect_return_storage_rejects_pending_hoisting_and_unvalidated_source_reads",
            "effect_return_storage_does_not_grant_a_return_an_assignments_self_read",
            "assert_eq!(cases::variants().len(), 8)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-real-carry-reduction-and-reference-baseline",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs",
        required_patterns: &[
            "effect_return_storage_removes_four_real_yir_carry_words_not_just_a_local_binding",
            "effect_return_storage_matches_independent_reference_results_prints_and_selected_traps",
            "effect_return_storage_evaluates_every_record_field_before_overwriting_its_old_value",
            "assert_eq!(after + 4, before",
            "Ok(yir_core::Value::Int(942))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-raw-native-state-and-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_reference_keeps_actual_state_raw_words_and_fault_retention",
            "shared_return_storage_native_keeps_once_only_prefixes_false_zeros_nan_and_local_exits",
            "shared_return_storage_native_still_reserves_the_full_reached_loop_bounds",
            "for limit in [4, 5]",
            "huge.trips = i64::MAX",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs",
        required_patterns: &[
            "shared_return_storage_fresh_cache_and_source_free_delivery_keep_payloads_and_faults",
            "fresh.host_objects == cached.host_objects",
            "for restore in 0..2",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-storage-v1.md",
        required_patterns: &[
            "Original Read Authority",
            "Actual Carry Reduction",
            "four transport words",
            "not a physical-memory or speedup measurement",
            "active/99",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-storage-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified effectful return storage sharing follow-up:",
            "Previous independent effect-prefix checkpoint:",
            "Previous independent effect-prefix task checkpoint:",
            "Previous independent effect-prefix blocker checkpoint:",
            "Previous independent effect-prefix action checkpoint:",
            "Previous independent effect-prefix validation checkpoint:",
            "Previous independent effect-prefix artifact checkpoint:",
        ],
    },
];
