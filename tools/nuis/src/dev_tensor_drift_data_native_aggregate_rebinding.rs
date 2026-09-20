use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-source-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "ty != &scalar_type(\"bool\") && control_values::supported_type(ty, layouts)",
            "!locals.writable.contains(name)",
            "!control_values::supported_type(existing, layouts)",
            "inferred != *existing",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-transport-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_carries.rs",
        required_patterns: &[
            "pub(super) struct Plan",
            "pub(super) fn needs_struct",
            "fields.is_some()",
            "Private branch transport stays flat-i64",
            "type_name: ty.name.clone()",
            "fields.iter().map(|field| (field.clone(), word()))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-branch-integration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/branches.rs",
        required_patterns: &[
            "carries.retain(|name| scope.contains_key(name))",
            "scalar_carries::Plan::new(&carries, scope, types.layouts())",
            "scalar_carries::projected_call",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-total-guard-projections",
        path: "tools/nuisc/src/lowering/direct_calls/control_boundaries.rs",
        required_patterns: &[
            "fn is_flat_parameter_field",
            "let NirExpr::Var(name) = base.as_ref()",
            "definition.where_bounds.is_empty()",
            "direct_call_scalar_kind(&entry.ty) == Some(DirectCallScalarKind::I64)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-source-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/aggregate_calls_tests.rs",
        required_patterns: &[
            "iteration_flat_values_reject_effects_cycles_and_nominal_rebinding_drift",
            "flat_rebinding_never_grants_constant_or_parameter_write_authority",
            "iteration_flat_value_layout_width_is_not_a_native_slot_table",
            "[1, 3, 7, 65]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-execution",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_rebinding.rs",
        required_patterns: &[
            "flat_rebindings_preserve_snapshots_nominal_joins_and_balanced_cleanup",
            "overwritten_flat_rebindings_keep_checked_failures_and_zero_trip_guards",
            "flat_rebindings_stay_behind_complete_induction_preflight",
            "flat_rebinding_fields_keep_source_order_before_failure_or_overwrite",
            "flat_rebinding_private_transport_rejects_layout_and_projection_drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-shared-budgets",
        path: "tools/nuisc/tests/native_application_bridge/helper_entries.rs",
        required_patterns: &[
            "flat_rebinding_guards_share_both_budgets_without_refunds_or_resets",
            "execute(&source, 8, 17, &[rejected])",
            "execute(&source, 7, 18, &[rejected])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-default-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/aggregate_rebinding.rs",
        required_patterns: &[
            "flat_rebinding_snapshots_and_joins_run_through_default_native_entry",
            "overwritten_flat_result_keeps_native_arithmetic_failure",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_rebinding_loops.ns",
            "native_iteration_flat_rebindings_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_REBINDING_SOURCE)",
            "fs::remove_file(project.0.join(\"main.ns\"))",
            "fs::remove_file(project.0.join(\"nuis.toml\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-flat-rebinding-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Iteration-Local Flat Rebinding",
            "declared field order",
            "Outer aggregate carries",
            "### Outer Flat-I64 Carries",
        ],
    },
];
