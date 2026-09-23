use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-elision",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "[NirStmt::Return(Some(value))] if is_atom(value)",
            "[] => next.as_ref().filter(|value| is_atom(value))",
            "NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::Var(_)",
            "value: condition",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-unit-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_tests.rs",
        required_patterns: &[
            "ready_scalar_and_record_returns_need_no_private_branch_or_continuation",
            "nontrivial_suffixes_remain_shared_and_branch_work_stays_guarded",
            "atomic_return_elision_never_moves_predicates_projections_or_constructors",
            "verify_nir_module(&normalized)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-composition-bound",
        path: "tools/nuisc/tests/native_application_bridge/control_composition.rs",
        required_patterns: &[
            "composed_counted_returns_fit_native_graph_without_relaxing_limits",
            "assert!(functions <= 51",
            "assert_native_parity(COMPOSED, true)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-entry-budgets",
        path: "tools/nuisc/tests/native_application_bridge/control_elision_budgets.rs",
        required_patterns: &[
            "atomic_returns_keep_one_predicate_evaluation_and_exact_remaining_call_budget",
            "scalar_elision_keeps_nontrivial_fallback_arguments_inside_the_guard",
            "ready_bool_and_flat_values_select_exact_snapshots_without_private_entries",
            "choose_packet",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-complete-fixture",
        path: "tools/nuisc/tests/native_application_bridge/control_composition.ns",
        required_patterns: &[
            "return total + checksum + index;",
            "fn rebalance(value: i64, width: i64) -> i64",
            "return packet.quotient + packet.remainder - tick.value + saved.remainder;",
            "if index == 1 { return value; }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "native_control_composition_build_cache_and_standalone_relocation",
            "check_workflow(CONTROL_COMPOSITION_SOURCE)",
            "control_composition.ns",
            "assert!(functions <= 51",
            "fs::read_to_string(relocated.join",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &[
            "mod control_composition;",
            "native_application_bridge/control_composition.rs",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "Ready-Value Control Elision",
            "It previously failed the 64-function limit; ready-value elision first reduced it",
            "including\nwhen both arms return the same value",
            "Single-Use Terminal Continuations",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-ready-value-control-tensor-evidence",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "57 reachable native functions",
            "predicates still run once",
            "reduce per-return flat-i64 aggregate allocation in native scalar helpers",
        ],
    },
];
