use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs",
        required_patterns: &[
            "pub(super) fn guarded_declaration",
            "ready_values::atom_type(join.condition, scope)",
            "layouts.scalar(&join.ty.name)",
            "conditional_values::prefix::expression_roots",
            "control_values::value_type(join.yes, scope, catalog, layouts)",
            "control_values::value_type(join.no, scope, catalog, layouts)",
            "!observed(name, suffix)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-shared-factory",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "scalar_joins::guarded_declaration(",
            "self.extract(join.condition.clone(), yes, no, scope, false)",
            "Branch calls, projections and arithmetic stay inside the helper.",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_guarded_tests.rs",
        required_patterns: &[
            "guarded_scalar_join_requires_completed_pure_dependencies_and_is_idempotent",
            "guarded_scalar_join_bounds_syntax_and_keeps_scope_gate_and_effect_vetoes",
            "guarded_scalar_join_uses_shared_scalar_types_for_both_arms",
            "guarded_scalar_join_keeps_checked_work_inside_selected_helper_arms",
            "reordered.functions.reverse()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/guarded_scalar_join.ns",
        required_patterns: &[
            "let selected: i64 = if choice { quotient(value, yes) } else { value % no };",
            "unused: 1 / tail",
            "while index < trips",
            "packet(choice, seed.total + value + 0 / value, yes, no, tail).value",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/guarded_scalar_join_cases.rs",
        required_patterns: &[
            "seed.wrapping_add(self.value)",
            "input.checked_div(self.yes)",
            "input.checked_rem(self.no)",
            "1i64.checked_div(self.tail)",
            "i64::MIN + 2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-native-witness",
        path: "tools/nuisc/tests/native_application_bridge/guarded_scalar_join.rs",
        required_patterns: &[
            "guarded_scalar_join_native_keeps_selected_checks_and_exact_failed_prefixes",
            "guarded_scalar_join_reference_preserves_live_state_and_blocks_partial_publication",
            "probe::check_sequences_with_limits",
            "typed_effect_returns::reordered",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-delivery",
        path: "tools/nuis/tests/native_session_workflow/guarded_scalar_join.rs",
        required_patterns: &[
            "guarded_scalar_join_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-current-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Certified guarded-scalar-expression witnesses:",
            "Previous typed-scalar checkpoint:",
            "Previous typed-scalar task checkpoint:",
            "Previous typed-scalar blocker checkpoint:",
            "Previous typed-scalar action checkpoint:",
            "Previous typed-scalar validation checkpoint:",
            "Previous typed-scalar artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-guarded-scalar-join-receipt",
        path: "docs/reference/nuis-native-guarded-scalar-joins-v1.md",
        required_patterns: &[
            "Guarded Expression Contract",
            "4,096-work/64-depth",
            "Six failure categories",
            "strict scalar-write validator is unchanged",
            "active/99",
        ],
    },
];
