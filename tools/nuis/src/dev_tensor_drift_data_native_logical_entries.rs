use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-single-edge-whole-root-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_entry.rs",
        required_patterns: &[
            "single_edge(condition, scope)",
            "op: NirBinaryOp::And | NirBinaryOp::Or",
            "predicate(lhs, scope)",
            "conditional_values::prefix::expression_roots(vec![(condition, 0, true)])",
            "== Some(scalar_type(\"bool\"))",
            "this grants no logical internal-arm authority",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-shared-guarded-value-root-consumption",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs",
        required_patterns: &[
            "if !in_loop && ty.as_ref().is_none_or(|ty| ty == &scalar_type(\"bool\"))",
            "if let Some(guarded) = self.gated_condition(value, scope)",
            "mod logical;",
            "let mut args = vec![condition]",
            "(atom_gate && prefix::expression_roots(vec![(condition, 0, true)]))",
            "Branch calls, projections and arithmetic stay inside the helper",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-once-only-reference-and-partial-composition",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_entry_logical_tests.rs",
        required_patterns: &[
            "conditional_return_logical_entries_preserve_selected_rhs_once_and_complete_arm_order",
            "assert_eq!(cases, 144)",
            "conditional_return_logical_entries_compose_replay_stored_signals_and_intermediate_suffixes",
            "assert_eq!(cases, 216)",
            "conditional_return_logical_entries_keep_real_zero_exits_fallible_parent_tails_and_current_gates",
            "assert_eq!(cases, 52)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-structure-whole-root-bounds-and-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_entry_logical_tests.rs",
        required_patterns: &[
            "conditional_return_logical_entries_store_original_value_root_without_replaying_helper_inputs",
            "assert_eq!(value, condition)",
            "conditional_return_logical_entries_share_whole_root_4096_node_and_depth_limits_before_typing",
            "4093 argument nodes + call + logical root + gate = exactly 4096",
            "for count in [61, 62]",
            "conditional_return_logical_entries_retain_nested_derived_gate_effect_type_and_capture_vetoes",
            "matching operand types",
            "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-default-source-binaries-and-selected-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_entry_logical_native_tests.rs",
        required_patterns: &[
            "conditional_return_logical_entries_execute_default_aot_selected_skipped_rhs_and_earlier_exits",
            "conditional_return_logical_entries_execute_default_aot_partial_signals_and_real_zero_exits",
            "use super::super::super::tests::native::run",
            "Some(\"99\\n0\")",
            "Some(\"99\\n77\\n10\")",
            "run(&source, expected)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-retained-original-veto-now-executes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_entry_tests.rs",
        required_patterns: &[
            "Keep the former single-edge rejection as independent positive evidence",
            "gate && helper(produce(entry))",
            "execute(&logical, Some(11), &[99, 11], 2)",
            "helper(produce(entry)) || gate",
            "assert_eq!(module, before)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-documentation-independent-scope",
        path: "docs/reference/nuis-native-scalar-loop-snapshots-v1.md",
        required_patterns: &[
            "## Single-Edge Logical Outer Entries",
            "Eight compiler tests cover 412 runtime source cases and 824 ordinary/reversed",
            "Internal arm condition and prefix grammars are unchanged",
            "no entry predicate or RHS call",
            "4096/4097-node and depth-63/64 probes",
            "Invalid bool/record operands retain their front-end type diagnostic",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-logical-entry-tensor-receipt-and-residual-boundary",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Single-edge logical-entry follow-up:",
            "Single-edge logical-entry acceptance:",
            "412 runtime source cases and 824 ordinary/reversed reference executions",
            "Previous intermediate-suffix checkpoint:",
            "conditional_return_logical_entries",
            "active/90",
            "nested/derived-gate logical entry conditions",
        ],
    },
];
