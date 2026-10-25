use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-word-type-and-readiness",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "NirExpr::PackF32Word(inner) => (inner, \"f32\", \"i64\")",
            "NirExpr::UnpackF64Word(inner) => (inner, \"i64\", \"f64\")",
            "(expression_with_fields(\n                inner,\n                locals,\n                updates,\n                own,\n                catalog,\n                layouts,\n                depth + 1,\n                field_reads,\n            )? == scalar_type(source))",
            "NirBinaryOp::Xor if left == scalar_type(\"i64\") => Some(left)",
            "word_temporaries_tests.rs",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-word-negative-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/word_temporaries_tests.rs",
        required_patterns: &[
            "loop_word_temporaries_preserve_exact_width_and_ordered_self_or_sibling_reads",
            "loop_word_temporaries_reject_wrong_kinds_unknown_names_and_excess_depth",
            "locals.available.insert(\"sibling\".into())",
            "signed_word(\"missing\", true)",
            "for _ in 0..33",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-real-record-source",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_values.ns",
        required_patterns: &[
            "struct State { result: bool, total: i64, gain: f32, marker: f64 }",
            "let checked = 1 / proof",
            "narrow = -narrow",
            "wide = -wide",
            "return returned(choice, current, narrow, wide, index, outer, 1 / root)",
            "work(state.result, state.total + 1, state.gain, state.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-four-kinds-and-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_values_cases.rs",
        required_patterns: &[
            "Kind::Bool, Kind::F32, Kind::F64, Kind::Record",
            "Model actual source exits, never the generated return signal",
            "reserve(self.trips, &mut budget)",
            "0xff800123",
            "0xfff0000012345678",
            "pub fn after_prefix",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-unit-admission-and-real-zero-witnesses",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_values_tests.rs",
        required_patterns: &[
            "iteration_effects_return_values_admit_exact_types_without_pure_authority",
            "iteration_effects_return_values_oracle_observes_false_zero_raw_signs_and_current_records",
            "iteration_effects_return_values_reject_computed_effects_and_mismatched_return_types",
            "false is a real selected return",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-aot-layout-and-order",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_return_values_tests.rs",
        required_patterns: &[
            "native_state_loop_return_values_keep_exact_layout_grants_and_effect_order",
            "State{result:bool;total:i64;gain:f32;marker:f64}",
            "lacks dependency order",
            "sites.iter().filter(|s| *s != site).cloned()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-native-and-reference-proof",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_return_values.rs",
        required_patterns: &[
            "state_loop_return_values_reference_preserves_actual_state_raw_words_and_fault_retention",
            "state_loop_return_values_native_preserves_false_zeros_nan_payloads_and_typed_records",
            "state_loop_return_values_native_rejects_reached_bounds_without_waiving_typed_returns",
            "ApplicationSessionPhase::Faulted",
            "for limit in [4, 5]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-source-free-proof",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_values.rs",
        required_patterns: &[
            "state_loop_return_values_build_cache_and_source_free_restores_preserve_exact_payloads",
            "compile_cache: hit",
            "fresh.host_objects == cached.host_objects",
            "for _ in 0..2",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "to_bits(), expected.gain",
            "to_bits(), expected.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-complete-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful typed effectful loop returns follow-up:",
            "Previous scalar loop-return checkpoint:",
            "Previous scalar loop-return task checkpoint:",
            "Previous scalar loop-return blocker checkpoint:",
            "Previous scalar loop-return action checkpoint:",
            "Previous scalar loop-return validation checkpoint:",
            "Previous scalar loop-return artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-return-values-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-return-values-v1.md",
        required_patterns: &[
            "Word Readiness Repair",
            "False And Zero Returns",
            "Source-Free Delivery",
            "not post-abort memory inspection",
            "active/99",
        ],
    },
];
