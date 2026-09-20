use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-local-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "iteration-local declarations gain bool write authority",
            "inferred != *existing",
            "Only bindings that exist before the branch survive its join",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-word-transport",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_carries.rs",
        required_patterns: &[
            "Private branch transport stays flat-i64",
            "NirExpr::CastBoolToI64",
            "NirExpr::CastI64ToBool",
            "ty: Some(ty)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-guard-seeds",
        path: "tools/nuisc/src/lowering/direct_calls/control_boundaries.rs",
        required_patterns: &[
            "let word_seed =",
            "NirExpr::CastBoolToI64(inner)",
            "parameter(inner, DirectCallScalarKind::Bool)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-effect-separation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/validation.rs",
        required_patterns: &[
            "EffectTypes::Buffer(_) => inferred == scalar_type(\"i64\")",
            "control_values::supported_type(&inferred, layouts)",
            "existing != &inferred",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-scope-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries_tests.rs",
        required_patterns: &[
            "temporary_scope_and_definite_initialization_are_fail_closed",
            "bool_write_authority_does_not_escape_the_iteration_or_widen_buffer_effects",
            "bool_branch_transport_uses_canonical_words_without_new_loop_state",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-execution",
        path: "tools/nuisc/tests/native_application_bridge/bool_rebinding.rs",
        required_patterns: &[
            "bool_rebindings_preserve_snapshots_nested_joins_and_lazy_calls",
            "bool_rebindings_do_not_hide_unused_arithmetic_traps",
            "bool_rebindings_keep_complete_induction_preflight",
            "bool_branch_transport_rejects_forged_cast_types_and_dependencies",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-shared-budgets",
        path: "tools/nuisc/tests/native_application_bridge/helper_entries.rs",
        required_patterns: &[
            "bool_rebinding_guards_share_both_budgets_without_refunds_or_resets",
            "execute(&source, 8, 17, &[rejected])",
            "execute(&source, 7, 18, &[rejected])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-default-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/bool_rebinding.rs",
        required_patterns: &[
            "bool_rebinding_snapshots_and_joins_run_through_default_native_entry",
            "overwritten_bool_result_keeps_native_arithmetic_failure",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "native_iteration_bool_rebindings_build_cache_and_standalone_relocation",
            "bool_rebinding_loops.ns",
            "check_workflow(BOOL_REBINDING_SOURCE)",
            "fs::remove_file(project.0.join(\"main.ns\"))",
            "fs::remove_file(project.0.join(\"nuis.toml\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-rebinding-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Iteration-Local Bool Rebinding",
            "canonical i64 0/1 words",
            "Outer bool carries",
            "Iteration-Local Flat Rebinding",
        ],
    },
];
