use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-explicit-value-effect-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/validation.rs",
        required_patterns: &[
            "pub(super) enum LiteralPrints",
            "pub(super) fn contains_print",
            "let mut pending = body.iter().collect::<Vec<_>>()",
            "NirStmt::Print(NirExpr::Int(_))",
            "EffectTypes::Values(_, _, LiteralPrints::Allow)",
            "let EffectTypes::Buffer(catalog) = types else",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-profile-authority-forwarding",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "Profile::Pure => validation::LiteralPrints::Reject",
            "Profile::IterationLiteralPrints => validation::LiteralPrints::Allow",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-print-only-work-is-not-a-carry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/sequences.rs",
        required_patterns: &[
            "let has_prints = validation::contains_print(effects)",
            "has_prints && literal_prints == validation::LiteralPrints::Allow",
            "!has_exits && !print_work",
            "|| has_prints",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-scoped-recursive-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "NirStmt::Print(NirExpr::Int(_))\n                if literal_prints == validation::LiteralPrints::Allow",
            "literal_loops::validate(",
            "literal_prints,\n                    depth,",
            "if depth > 64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-scoped-outlining-and-neutral-result",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &[
            "validation::contains_print(body)",
            "literal_prints: validation::LiteralPrints",
            "self.literal_prints",
            "body.push(NirStmt::Expr(call))",
            "must not become a seed for the next trip",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-effect-plan-opt-in",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects.rs",
        required_patterns: &["validation::LiteralPrints::Allow", "Some(&self.roots)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-pure-route-opt-out",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &["validation::LiteralPrints::Reject"],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-isolation-and-rejection-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_tests.rs",
        required_patterns: &[
            "iteration_effects_direct_and_print_only_loops_keep_explicit_authority",
            "iteration_effects_direct_statements_reject_nonliteral_and_foreign_effects",
            "iteration_effects_direct_validation_does_not_grant_buffer_or_pure_authority",
            "validation::EffectTypes::Buffer(&catalog), false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-source-profile",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_statements_profile.rs",
        required_patterns: &[
            "while echo < trips { print(7); echo = echo + 1; }",
            "current = current + 0; let even = current % 2 == 0; let proof = 1 / root; print(10)",
            "print(19); let local = refine(current, 1 / (index - hole), index); print(20)",
            "pub const LOOP_LIMIT: u64 = 6",
            "assert_eq!(source.matches(original).count(), 1)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-live-state-and-cumulative-reservations",
        path: "tools/nuisc/tests/native_application_bridge/state_loop_statements.rs",
        required_patterns: &[
            "state_loop_statements_reference_executes_only_selected_prints_and_keeps_last_state",
            "state_loop_statements_native_observes_real_work_with_real_source_effects",
            "state_loop_statements_native_reserves_before_any_iteration_print",
            "ApplicationSessionPhase::Faulted",
            "for limit in [3, 4, 5]",
            "before_failure: vec![5, 7, 7, 7]",
            "input[5] = -1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-exact-grants-order-and-layout",
        path: "tools/nuisc/src/aot_application_effect_call_state_loop_statements_tests.rs",
        required_patterns: &[
            "native_state_loop_statements_require_exact_grants_and_transitive_effect_order",
            "bridge.loop_work_limit, 6",
            "missing grant {site} was accepted",
            "lacks dependency order",
            "State{result:bool;total:i64;marker:f64}",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-source-free-three-family-delivery",
        path: "tools/nuis/tests/native_session_workflow/effect_call_state_loop.rs",
        required_patterns: &[
            "state_loop_statements_build_cache_and_two_source_free_restores_keep_direct_prints",
            "statements::source_variants()",
            "statements::trace",
            "statements::LOOP_LIMIT",
            "native-session-policy-aot-bundle:v1.{loop_limit}.100.",
            "fresh.host_objects == cached.host_objects",
            "restore changed input artifact",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Stateful direct loop-statement effects follow-up:",
            "Previous iteration-effects checkpoint:",
            "Previous iteration-effects task checkpoint:",
            "Previous iteration-effects blocker checkpoint:",
            "Previous iteration-effects action checkpoint:",
            "Previous iteration-effects validation checkpoint:",
            "Previous iteration-effects artifact checkpoint:",
            "nuis-native-stateful-loop-statements-v1.md",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-loop-statements-scoped-acceptance",
        path: "docs/reference/nuis-native-stateful-loop-statements-v1.md",
        required_patterns: &[
            "15360 healthy accepted callbacks",
            "six loop-work units and 100 helper entries",
            "not post-abort memory inspection",
            "quoted no-op heredoc",
            "active/99",
        ],
    },
];
