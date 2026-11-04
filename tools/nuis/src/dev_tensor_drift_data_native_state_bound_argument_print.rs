use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-ready-comparisons",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "if left != right", "|| left == scalar_type(\"f32\")",
            "|| left == scalar_type(\"f64\")", "Some(scalar_type(\"bool\"))",
            "!locals.available.contains(name) && own != Some(name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-source",
        path: "tools/nuisc/tests/native_application_bridge/state_bound_argument_print.ns",
        required_patterns: &[
            "guard: i64, negative: bool", "if negative { return base + 1; } return base;",
            "child_count(trips, outer, bound_guard - outer, total < 0.0)",
            "total = current.total", "total: finish(-current.total",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-source-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs",
        required_patterns: &[
            "state_bound_argument_print_accepts_current_ready_carry",
            "state_bound_argument_print_keeps_scope_and_mutable_bound_refusals",
            "state_bound_argument_print_ready_float_comparisons_preserve_types_and_epochs",
            "child_trips = child_trips + 1", "module.functions.reverse()",
            "let alias = epoch_word",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-ieee-model",
        path: "tools/nuisc/tests/native_application_bridge/state_bound_argument_print_cases.rs",
        required_patterns: &[
            "let magnitude = bits as u64 & (sign - 1)", "magnitude <= infinity",
            "previous.unwrap_or(case.child.base.initial)", "active = !active",
            "for bits in [kind.nan(), 1, 0]", "bits as u64 ^ case.kind().mask()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-fault-matrix",
        path: "tools/nuisc/tests/native_application_bridge/state_bound_argument_print_cases.rs",
        required_patterns: &[
            "vec![row, seed(kind), seed(kind)]", "for at in [49, 50]",
            "eager.child.base.guards[0] = 51", "zero.child.base.guards[1] = 51",
            "becomes_zero.child.base.trips = 0", "becomes_zero.clock = GuardClock::StateBound(3)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-profile",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs",
        required_patterns: &[
            "pub const STATE_BOUND: Matrix", "StateBound(i64)",
            "pub fn child_counts_from(self, previous: Option<i64>)",
            "return state_bounds::counts(self, previous)", "zext i1 %arg3 to i64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-model-order",
        path: "tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs",
        required_patterns: &[
            "case.child_counts_from(previous).into_iter().enumerate()",
            "trace.push(i64::from(super::state_bounds::negative(",
            "if remaining == 0", "return Err(trace)", "current = next",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-native-controls",
        path: "tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_bound_argument_print_native_uses_changed_carry_and_accepted_state",
            "state_bound_argument_print_reference_preserves_state_dependent_counts_and_faults",
            "state_bound_argument_print_native_keeps_state_dependent_work_and_fault_order",
            "state_bound_argument_print_oracle_pins_backedges_decoys_and_ieee_classes",
            "check_calls(*row, Some(previous), &event.events)", "row.child_counts_from(previous)",
            "vec![0, 3, 0, 4]", "vec![0, 4, 0, 4]", "vec![0, 1, 0, 0]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs",
        required_patterns: &[
            "state_bound_argument_print_fresh_cache_and_source_free_delivery_use_accepted_state",
            "cases::STATE_BOUND", "compile_cache: hit", "materialize-artifact",
            "for i in 0..2", "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-receipt",
        path: "docs/reference/nuis-native-state-bound-argument-print-v1.md",
        required_patterns: &[
            "State-Dependent Child Bounds", "27 raw sequences", "eight finite sequences",
            "14 fault rows", "0/3/0/4", "0/4/0/4", "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-state-bound-argument-print-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "State-dependent child bound witnesses:", "Previous fallible-bound checkpoint:",
            "Previous fallible-bound task checkpoint:", "Previous fallible-bound blocker checkpoint:",
            "Previous fallible-bound action checkpoint:", "Previous fallible-bound validation checkpoint:",
            "Previous fallible-bound artifact checkpoint:",
        ],
    },
];
