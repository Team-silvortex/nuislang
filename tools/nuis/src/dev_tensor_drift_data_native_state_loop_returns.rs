use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-explicit-profile-not-pure-reclassification",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_body.rs",
        required_patterns: &[
            "Profile::Pure => control_loops::returns::normalize",
            "Profile::IterationLiteralPrints",
            "control_loops::returns::normalize_effects",
            "fn validate_iteration_body",
            "normalized.as_deref().unwrap_or(&function.body)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-independent-storage-and-original-publication-order",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs",
        required_patterns: &[
            "fn prepare_effects",
            "scalar_helpers::validate_iteration_body(function, &plan.body, catalog, layouts)?",
            "rewrite(function, layouts, None, validation::LiteralPrints::Allow)",
            "literal_prints == validation::LiteralPrints::Allow",
            "output.push(binding(&self.value, self.result, value.clone()?))",
            "output.push(binding(&self.flag, &scalar_type(\"i64\"), NirExpr::Int(1)))",
            "output.push(self.propagate(in_loop))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-provenance-supplied-to-scoped-loops",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects.rs",
        required_patterns: &[
            "catalog.contains_key(name)",
            "control_loops::returns::prepare_effects",
            "return_signals.insert(function.name.clone(), plan.signal)",
            "&return_signals",
            "Some(&self.roots)",
            "iteration_effects_returns_tests.rs",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-source-prefix-and-fallible-payload",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_returns.ns",
        required_patterns: &[
            "print(30); return value + 10 * index + 100 * outer + 1 / proof",
            "if gate { print(12); return returned(current, index, outer, 1 / root); }",
            "if outer == parent { print(24); return returned(current, index, outer, 1 / root); }",
            "gate != state.result",
            "marker: state.marker",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-independent-source-oracle",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_returns_cases.rs",
        required_patterns: &[
            "It does",
            "not model the compiler's pending bit",
            "return returned(current, index, outer, self.root, observed, &mut trace)",
            "reserve(self.trips, &mut budget)",
            "pub fn after_prefix",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-unit-type-and-negative-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_returns_tests.rs",
        required_patterns: &[
            "iteration_effects_returns_use_explicit_profile_and_private_typed_signal",
            "iteration_effects_returns_reject_unproven_dependencies_and_nonterminal_returns",
            "iteration_effects_returns_keep_exact_bool_float_and_record_value_types",
            "iteration_effects_returns_oracle_distinguishes_return_prefix_and_shared_budget",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-native-and-reference-publication-proof",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_returns.rs",
        required_patterns: &[
            "state_loop_returns_reference_preserves_effect_prefix_and_faulted_state",
            "state_loop_returns_native_uses_actual_state_and_skips_parent_suffix_and_later_iterations",
            "state_loop_returns_native_reserves_full_reached_bounds_before_any_early_return",
            "for limit in [4, 5, 9]",
            "ApplicationSessionPhase::Faulted",
            "cases::RETURN_OBSERVER",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-grants-and-delivery-policy",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_returns_tests.rs",
        required_patterns: &[
            "native_state_loop_returns_require_grants_and_transitive_effect_order",
            "State{result:bool;total:i64;marker:f64}",
            "sites.iter().filter(|s| *s != site).cloned()",
            "lacks dependency order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-cache-and-source-free-proof",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop_returns.rs",
        required_patterns: &[
            "state_loop_returns_build_cache_and_two_source_free_restores_preserve_early_return",
            "compile_cache: hit",
            "fresh.host_objects == cached.host_objects",
            "for _ in 0..2",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
            "native_session_completed=1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-returns-v1.md",
        required_patterns: &[
            "Independent Return Storage",
            "Full Reached Bounds",
            "Source-Free Delivery",
            "bool/float/record return admission",
            "not complete this coordinate",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-returns-six-field-complete-predecessor-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful effectful loop returns follow-up:",
            "Previous parent-local return checkpoint:",
            "Previous parent-local return task checkpoint:",
            "Previous parent-local return blocker checkpoint:",
            "Previous parent-local return action checkpoint:",
            "Previous parent-local return validation checkpoint:",
            "Previous parent-local return artifact checkpoint:",
        ],
    },
];
