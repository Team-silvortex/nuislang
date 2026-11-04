use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-source-binding-sites",
        path: "tools/nuisc/tests/native_application_bridge/local_scalar_argument_print.ns",
        required_patterns: &[
            "let parameter_word = checked64(",
            "let ready_word = checked64(",
            "let branch_word = ready_word + 1",
            "let branch_word = ready_word - 1",
            "current.flag && (checked64(i64_from_i32(current.code) ^ parameter_word, lazy_guard)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-source-readiness-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/local_scalar_argument_print_tests.rs",
        required_patterns: &[
            "local_scalar_argument_print_accepts_ready_initializers_and_branch_local_uses",
            "local_scalar_argument_print_does_not_lend_epoch_reads_to_initializers",
            "local_scalar_argument_print_rejects_forward_reads_and_branch_escape",
            "local_scalar_argument_print_keeps_child_epoch_reads_print_only",
            "unknown value `{name}`",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-no-ambient-epoch-initializer-grant",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "mod local_scalar_argument_print_tests;",
            "let inferred = match expression(value, locals, updates, own, catalog, layouts, 0)",
            "None if literal_prints == validation::LiteralPrints::Scalars",
            "owner.is_some_and(|owner| owner.matches(body, print_epoch))",
            "certified_consumer = Some(print_epoch.initializer_pair(",
            "expression_with_fields(value, locals, updates, own, catalog, layouts, depth, None)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/local_scalar_argument_print_cases.rs",
        required_patterns: &[
            "left % 2 != right % 2",
            "value.rem_euclid(1i128 << 64)",
            "Equal codes cancel in the ready word",
            "next.initial = kind.nan()",
            "row.guards[guard] = 0",
            "call void @word_print(i64 %arg1)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-native-reference-publication",
        path: "tools/nuisc/tests/native_application_bridge/local_scalar_argument_print.rs",
        required_patterns: &[
            "local_scalar_argument_print_native_preserves_eager_initializers_and_fault_prefixes",
            "local_scalar_argument_print_reference_preserves_counts_state_and_faulted_close",
            "local_scalar_argument_print_native_rejects_unproved_epoch_initializers",
            "cases::sequences(kind).len(), 14",
            "cases::failures(kind).len(), 8",
            "ApplicationSessionPhase::Faulted",
            "a later lazy gate must not skip a local initializer",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/local_scalar_argument_print.rs",
        required_patterns: &[
            "local_scalar_argument_print_fresh_cache_and_source_free_delivery_preserve_initializers",
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-receipt",
        path: "docs/reference/nuis-native-local-scalar-argument-print-v1.md",
        required_patterns: &[
            "Existing Support, Not A New Grant",
            "Fourteen raw sequences",
            "Eight fault rows",
            "scoped epoch-local initializer plan",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-local-scalar-functional-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Source-local scalar argument print witnesses:",
            "Previous i64-XOR checkpoint:",
            "Previous i64-XOR task checkpoint:",
            "Previous i64-XOR blocker checkpoint:",
            "Previous i64-XOR action checkpoint:",
            "Previous i64-XOR validation checkpoint:",
            "Previous i64-XOR artifact checkpoint:",
        ],
    },
];
