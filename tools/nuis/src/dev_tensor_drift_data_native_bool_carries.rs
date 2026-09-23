use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-source-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "Only seeded mutable locals may cross the backedge",
            "control_values::supported_type(ty, layouts)",
            "!locals.writable.contains(name)",
            "inferred != *existing",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-private-iteration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &[
            "carries.contains(&param.name)",
            "branches::fresh_name(\"__nuis_bool_seed\", &mut bindings)",
            "param.ty = scalar_type(\"i64\")",
            "NirExpr::CastBoolToI64(Box::new(input))",
            "scalar_carries::binding",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-scoped-projections",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries.rs",
        required_patterns: &[
            "NirExpr::CastI64ToBool(word) if projected_word(word, result, slot)",
            "fn matches_seed",
            "NirExpr::CastBoolToI64(value)",
            "seeds != 1",
            "instruction: \"cast_i64_to_bool\"",
            "const_bindings.remove(binding.name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-enclosing-capture",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values.rs",
        required_patterns: &[
            "Source admission still uses value_type",
            "NirExpr::CastBoolToI64(base)",
            "NirExpr::CastI64ToBool(base) => collect_inputs(base, inputs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-source-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_carries_tests.rs",
        required_patterns: &[
            "bool_carries_use_explicit_private_words_without_capture_name_collisions",
            "bool_carries_do_not_launder_const_parameters_or_forward_sibling_reads",
            "__nuis_bool_seed_0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-private-capture-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_values/tests.rs",
        required_patterns: &[
            "private_bool_transport_capture_does_not_widen_source_admission",
            "collect_inputs(&expr, &mut inputs)",
            "value_type(&expr, &scope, &ScalarHelpers::new(), &FlatLayouts::new()).is_none()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-seed-tests",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries_tests.rs",
        required_patterns: &[
            "bool_projection_requires_explicit_typed_seed_and_decode",
            "[\"missing\", \"raw\", \"expression\", \"type\", \"duplicate\"]",
            "Some((0, 1))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-execution",
        path: "tools/nuisc/tests/native_application_bridge/bool_carries.rs",
        required_patterns: &[
            "outer_bool_carries_match_seed_snapshot_order_and_lazy_call_oracle",
            "outer_bool_carries_do_not_hide_overwritten_checked_failures",
            "outer_bool_carries_preserve_full_preflight_before_any_iteration",
            "singleton_bool_carry_uses_canonical_seed_and_backedge_words",
            "outer_bool_carries_reject_raw_boolean_seeds_and_forged_cast_types",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-budgets",
        path: "tools/nuisc/tests/native_application_bridge/helper_entries.rs",
        required_patterns: &[
            "outer_bool_carries_share_both_budgets_without_refunds_or_resets",
            "check_bool_budgets(&source)",
            "execute(source, 8, 16, &[rejected])",
            "execute(source, 7, 17, &[rejected])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/bool_carries.rs",
        required_patterns: &[
            "outer_bool_carry_keeps_zero_trip_seed_and_pre_loop_snapshot",
            "outer_bool_and_flat_carries_keep_ordered_branch_updates",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "bool_carries_loops.ns",
            "native_outer_bool_carries_build_cache_and_standalone_relocation",
            "check_workflow(BOOL_CARRIES_SOURCE)",
            "fs::remove_file(project.0.join(\"main.ns\"))",
            "fs::remove_file(project.0.join(\"nuis.toml\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-bool-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Outer Bool Carries",
            "canonical 0/1 words by construction",
            "no new loop opcode",
            "no-forward-sibling-read rules",
            "### Literal Nested Counted Loops",
        ],
    },
];
