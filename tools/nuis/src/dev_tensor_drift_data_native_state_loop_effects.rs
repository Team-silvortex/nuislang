use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-isolated-literal-profile",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "enum Profile",
            "IterationPrints",
            "Profile::Pure",
            "NirStmt::Print(value) if matches!(profile, Profile::IterationPrints)",
            "pub(super) fn collect_iteration_calls",
            "while let Some(name) = ready.pop_first()",
        ],
    },
    // The original blanket veto is now an explicit validated profile.
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-private-plan-and-dependent-return-veto",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects.rs",
        required_patterns: &[
            "catalog: ScalarHelpers",
            "roots: BTreeSet<String>",
            "control_loops::returns::prepare_effects",
            "catalog.contains_key(name)",
            "retained.difference(&outlined.functions)",
            "Some(&self.roots)",
            "scalar_control::outline",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-pipeline-routing-not-pure-seeding",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &[
            "iteration_effects::Plan::collect(module, &layouts, &control_catalog)",
            "iteration_effects.contains(&function.name)",
            "iteration_effects.outline(module, &layouts, &mut names, &mut helpers, &mut outlined)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-only-selected-control-roots",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &[
            "selected: Option<&BTreeSet<String>>",
            "!selected.contains(&function.name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-guarded-scalar-control-keeps-literals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "NirStmt::Print(_) => output.push(stmt)",
            "| NirStmt::Print(value)",
            "self.guarded.insert(name.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-no-provisional-or-pure-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_tests.rs",
        required_patterns: &[
            "iteration_effects_catalog_is_separate_and_retains_real_prints",
            "iteration_effects_unvalidated_dependencies_never_gain_provisional_authority",
            "iteration_effects_reject_cycles_types_and_unscoped_effects",
            "gained pure authority",
            "discovery mutated source",
            "print(true)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-real-literals-and-checked-argument-prefix",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_effects_profile.rs",
        required_patterns: &[
            "print(10)",
            "print(19)",
            "print(20)",
            "pub fn before_failure",
            "trace.extend([10, 19])",
            "row.entering, row.proof, row.index",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-reference-native-and-reservation",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_effects.rs",
        required_patterns: &[
            "state_loop_effects_reference_executes_only_selected_prints_and_keeps_last_state",
            "state_loop_effects_native_observes_real_work_with_real_source_effects",
            "state_loop_effects_native_reserves_before_any_iteration_print",
            "ApplicationSessionPhase::Faulted",
            "for limit in [0, 2, 3]",
            "input[4] = 1",
            "input[5] = -1",
            "effects::before_failure(call, bad)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-exact-print-grants-and-order",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_effects_tests.rs",
        required_patterns: &[
            "native_state_loop_effects_require_exact_grants_and_transitive_effect_order",
            "missing grant {site} was accepted",
            "lacks dependency order",
            "bridge.loop_work_limit, 3",
            "bridge.helper_entry_limit, 100",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-source-free-shared-delivery-and-pure-replay",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop.rs",
        required_patterns: &[
            "state_loop_effects_build_cache_and_two_source_free_restores_keep_selected_prints",
            "cases::Invocation::trace",
            "effects::trace,\n        cases::LOOP_LIMIT,",
            "trace(calls[0], false)",
            "restore changed input artifact",
            "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &[
            "native_application_bridge/state_loop_effects.rs",
            "mod state_loop_effects;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &[
            "aot_application_effect_call_state_loop_effects_tests.rs",
            "mod state_loop_effects;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &[
            "dev_tensor_drift_data_native_state_loop_effects.rs",
            "state_loop_effects::CHECKS",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful literal-print iteration effects follow-up:",
            "Previous state-loop checkpoint:",
            "Previous state-loop task checkpoint:",
            "Previous state-loop blocker checkpoint:",
            "Previous state-loop action checkpoint:",
            "Previous state-loop validation checkpoint:",
            "Previous state-loop artifact checkpoint:",
            "nuis-native-stateful-loop-effects-v1.md",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-effects-scoped-receipt",
        path: "docs/reference/nuis-native-stateful-loop-effects-v1.md",
        required_patterns: &[
            "15360 healthy accepted callbacks",
            "source prints 10/19/20",
            "quoted no-op heredoc",
            "## Next Boundary",
            "not general I/O",
        ],
    },
];
