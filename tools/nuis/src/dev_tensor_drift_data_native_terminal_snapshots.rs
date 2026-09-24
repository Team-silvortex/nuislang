use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "private-snapshot-return-closed-scope-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_snapshot_scopes.rs",
        required_patterns: &[
            "Step::Branch(child, child + 1)",
            "NirStmt::Break | NirStmt::Continue => flow.push(Step::Escape)",
            "in_loop: true",
            "for index in (0..scopes.len()).rev()",
            "flow.escapes |= flows[*left].escapes || flows[*right].escapes",
            "scopes[index].returns = !flow.falls_through && !flow.escapes",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-snapshot-return-closed-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_terminal_snapshots_tests.rs",
        required_patterns: &[
            "terminal_branch_writes_project_old_parameter_and_local_snapshots",
            "terminal_snapshot_versions_keep_nested_returns_and_fallthrough_values_distinct",
            "terminal_snapshot_proof_rejects_live_joins_and_loop_backedges",
            "terminal_snapshots_preserve_whole_uses_and_computed_caller_transactions",
            "terminal_snapshots_keep_checked_constructor_work_on_selected_paths",
            "terminal_snapshot_scopes_require_returns_not_break_continue_or_zero_trip_bodies",
            "terminal_snapshot_candidates_still_require_exact_declared_value_types",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "private-snapshot-return-closed-source-composition",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn terminal_snapshot_source()",
            "let current = payload; let old = current;",
            "let fallback = current;",
            "return relay(second.f0) + current.f0;",
            "return fallback.f63 / fallback.f1 + current.f0;",
        ],
    },
];
