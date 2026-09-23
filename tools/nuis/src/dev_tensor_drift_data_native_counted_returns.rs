use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-counted-return-normalization",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs",
        required_patterns: &[
            "control_values::supported_type(result, layouts)",
            "control_values::zero_value(result, layouts)",
            "branches::collect_bindings(&function.body, &mut names)",
            "reserve_references(&function.body, &mut names)?",
            "if depth > 32",
            "output.push(self.propagate(in_loop))",
            "value.clone()?",
            "NirStmt::Return(Some(NirExpr::Var(self.value.clone())))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "control_loops::returns::normalize(function, layouts)?",
            "normalized.as_deref().unwrap_or(&function.body)",
            "if returned",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-outlining",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &[
            "if control_catalog.contains_key(&function.name)",
            "control_loops::returns::normalize(function, &layouts)",
            "admitted counted return flow",
            "scalar_control::outline(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-source-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns_tests.rs",
        required_patterns: &[
            "counted_returns_admit_exact_values_without_widening_buffer_helpers",
            "counted_returns_propagate_child_exit_and_reserve_future_source_names",
            "counted_returns_reject_wrong_types_effects_writes_and_unreachable_suffixes",
            "counted_returns_do_not_bind_unknown_source_or_nir_references",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-native-oracles",
        path: "tools/nuisc/tests/native_application_bridge/counted_returns.rs",
        required_patterns: &[
            "counted_returns_preserve_typed_snapshots_induction_and_parent_child_scope",
            "counted_returns_skip_child_preflight_but_never_waive_entered_loop_bounds",
            "counted_returns_evaluate_selected_fallible_payload_before_publication",
            "break 'outer;",
            "aggregate_loop_probe::execute_probed(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-budgets",
        path: "tools/nuisc/tests/native_application_bridge/counted_return_budgets.rs",
        required_patterns: &[
            "counted_returns_keep_shared_reservations_entries_and_atomic_callback_output",
            "counted_return_payload_traps_leave_callback_output_untouched",
            "Returning on the first iteration does not refund",
            "failed.remaining_loop = 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/counted_returns.rs",
        required_patterns: &[
            "counted_returns_run_through_ordinary_entry_and_typed_helpers",
            "counted_returns_skip_suffixes_but_preserve_selected_return_failure",
            "Some(14)",
            "Some(11)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-frontdoor-fixture",
        path: "tools/nuisc/tests/native_application_bridge/counted_returns.ns",
        required_patterns: &[
            "return total + checksum + index;",
            "if child == 1",
            "if index == 1 { return value; }",
            "fn decompose(value: i64, width: i64) -> Parts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "native_counted_returns_build_cache_and_standalone_relocation",
            "check_workflow(COUNTED_RETURNS_SOURCE)",
            "counted_returns.ns",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-counted-return-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "Returns From Counted Value Loops",
            "The return expression executes at its source location",
            "unchanged 64-function",
            "Hygienic Continuation Bindings",
        ],
    },
];
