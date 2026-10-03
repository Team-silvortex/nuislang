use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-literal-snapshot-bounded-typed-origins",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_literals.rs",
        required_patterns: &[
            "pub(super) enum Literal",
            "self.budget.charge(text.len())?",
            ".filter(|v| v.is_finite())",
            "Literal::F32(v.to_bits())",
            "Literal::F64(v.to_bits())",
            "value.ty != scalar_type(from) || value.words.len() != 1",
            "Literal::I32(*value as i32)",
            "Literal::I64(i64::from(*value))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-snapshot-return-proof-integration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_facts.rs",
        required_patterns: &[
            "Literal(Literal)",
            "self.literal(Literal::I64(*value))",
            "self.literal(Literal::Bool(*value))",
            "self.float_literal(expr)",
            "self.integer_cast(expr, env, depth + 1)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-snapshot-negative-and-budget-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_literal_tests.rs",
        required_patterns: &[
            "return_invariant_literal_storage_identity_keeps_types_rounding_and_signed_zero",
            "return_invariant_literals_check_intermediate_changed_values_and_opaque_work",
            "return_invariant_literals_do_not_recover_delayed_or_mismatched_snapshot_values",
            "return_invariant_literal_exhaustion_does_not_publish_snapshot_or_advance_clock",
            "assert_eq!(clock, 17)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-snapshot-differential-return-storage",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_literal_execution_tests.rs",
        required_patterns: &[
            "return_invariant_literals_match_independent_returns_exits_and_selected_failures",
            "normalize(function, &layouts)",
            "nested_tests::execute(&independent)",
            "assert_eq!(actual, expected",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-snapshot-full-width-bridge-execution",
        path: "tools/nuisc/tests/native_application_bridge/sparse_typed_return_literals.rs",
        required_patterns: &[
            "typed_sparse_literal_returns_reduce_private_carries_without_widening_native_limits",
            "typed_sparse_literal_returns_execute_all_words_with_exact_visible_exits",
            "typed_sparse_literal_returns_execute_mixed_constant_storage_without_heap_aggregation",
            "[60, 60]",
            "[width - 4, width]",
            "[width - 3, width + 1]",
            "65 carried words exceed the 64-word native limit",
            "typed_record_inputs::check_compact(&source, cases, width - 7)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-snapshot-source-free-cache-restoration",
        path: "tools/nuis/tests/native_session_workflow/literal_snapshots.rs",
        required_patterns: &[
            "native_literal_snapshots_build_cache_and_restore_all_words_without_sources",
            "native_literal_snapshots_keep_selected_checked_work_before_publication",
            "compile_cache: hit",
            "fs::remove_file(project.0.join(\"main.ns\"))",
            "literal snapshot restore {cycle}",
            "assert_eq!(observed.len(), 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-snapshot-documented-scope",
        path: "docs/reference/nuis-native-scalar-loop-snapshots-v1.md",
        required_patterns: &[
            "## Typed Literal Origins",
            "signed zero and scalar kinds remain distinct",
            "60/64 words, rather than 61/65",
            "This proves stored identity, not permission to erase",
            "No source/public/callback/FFI ABI",
        ],
    },
];
