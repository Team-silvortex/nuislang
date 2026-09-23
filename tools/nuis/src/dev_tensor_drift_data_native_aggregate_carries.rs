use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-source-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "control_values::supported_type(ty, layouts)",
            "!locals.writable.contains(name)",
            "inferred != *existing",
            "own != Some(name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-iteration-transport",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &[
            "sequences::carry_names(body)",
            "scalar_carries::Plan::new(&carries, scope, Some(self.layouts))",
            "inputs.extend(carries.iter().cloned())",
            "scalar_carries::projected_call",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-projection-contract",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries.rs",
        required_patterns: &[
            "pub(super) struct Projection",
            "slot != words.len()",
            "type_name != &ty.name",
            "projected_word(value, result, slot + offset)",
            "&param.ty == binding.ty",
            "seeds != 1",
            "control && !is_scalar_i64(binding.ty)",
            "New value nodes preserve pre-loop snapshots",
            "format!(\"carry{slot}\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-scoped-integration",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering.rs",
        required_patterns: &[
            "scalar_carries::validate_seeds",
            "scalar_carries::bind_result",
            "flattened.len() != width",
            "index + offset, input",
            "scoped_call_i64_carries",
            "scoped_break_controls",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-source-tests",
        path:
            "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/aggregate_carries_tests.rs",
        required_patterns: &[
            "outer_flat_carries_admit_seeded_locals_and_keep_buffer_authority_closed",
            "outer_flat_carries_reject_nominal_mutability_scope_and_order_drift",
            "outer_flat_carry_width_is_layout_driven_not_a_native_arity_table",
            "[1, 3, 7, 65]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-projection-tests",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries_tests.rs",
        required_patterns: &[
            "flat_projection_slots_follow_layouts_not_capture_order",
            "flat_projection_rejects_noncanonical_reconstruction_without_evaluation",
            "flat_projection_requires_exact_nominal_single_seed_and_never_a_break_record",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-execution",
        path: "tools/nuisc/tests/native_application_bridge/aggregate_carries.rs",
        required_patterns: &[
            "outer_flat_carries_match_zero_trip_seeds_snapshots_and_ordered_oracle",
            "outer_flat_carries_keep_overwritten_checked_failures_and_skipped_guards",
            "outer_flat_carries_stay_behind_complete_induction_preflight",
            "outer_flat_carries_reject_transport_layout_slot_and_seed_drift",
            "singleton_flat_carry_uses_value_transport_and_retains_immutable_scalar_captures",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-shared-budgets",
        path: "tools/nuisc/tests/native_application_bridge/helper_entries.rs",
        required_patterns: &[
            "outer_flat_carries_share_both_budgets_without_refunds_or_resets",
            "execute(&source, 8, 15, &[rejected])",
            "execute(&source, 7, 16, &[rejected])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-default-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/aggregate_carries.rs",
        required_patterns: &[
            "outer_flat_carries_preserve_seeds_snapshots_and_order_on_default_native_entry",
            "single_word_record_self_copy_is_not_mistaken_for_scalar_or_owned_resource_loop",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "aggregate_carries_loops.ns",
            "native_outer_flat_carries_build_cache_and_standalone_relocation",
            "check_workflow(AGGREGATE_CARRIES_SOURCE)",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-outer-flat-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Outer Flat-I64 Carries",
            "zero-trip seeds",
            "not a finite arity table",
            "### Literal Nested Counted Loops",
            "no new loop opcode",
        ],
    },
];
