use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_LANGUAGE_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "language-unbounded-loop-entry-lowering",
        path: "tools/nuisc/src/lowering/loop_preparation_entries.rs",
        required_patterns: &[
            "PreparedLoopEntryCondition::Unbounded",
            "parse_unbounded_loop_step_binding",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-unbounded-loop-domain-contract",
        path: "crates/yir-domain-cpu/src/describe_post_control.rs",
        required_patterns: &["validate_post_flow_loop_compare_kind"],
    },
    DevTensorDriftCheckSpec {
        id: "language-unbounded-loop-llvm-backedge",
        path: "crates/yir-lower-llvm/src/function_lowering/loop_post_flow_chain.rs",
        required_patterns: &["cmp_kind == \"always\"", "br label %{loop_body}"],
    },
    DevTensorDriftCheckSpec {
        id: "language-unbounded-loop-native-regression",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "state_carrying_unbounded_loop_runs_as_a_native_binary",
            "assert_eq!(status.code(), Some(6))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-while-let-syntax-normalization",
        path: "tools/nuisc/src/frontend/parser_statements.rs",
        required_patterns: &[
            "parse_while_let_stmt_after_keyword",
            "`while let _ = ...` is irrefutable",
            "body: vec![AstStmt::Break]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-invariant-pattern-loop-entry-lowering",
        path: "tools/nuisc/src/lowering/loop_flow_nodes_post.rs",
        required_patterns: &[
            "PreparedLoopEntryCondition::InvariantPattern",
            "invariant_true",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-invariant-pattern-loop-llvm-gate",
        path: "crates/yir-lower-llvm/src/function_lowering/loop_post_flow_chain.rs",
        required_patterns: &[
            "invariant_true\" | \"pattern_exit",
            "icmp ne i64 {limit}, 0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-invariant-pattern-loop-lazy-false-gate",
        path: "crates/yir-lower-llvm/src/function_lowering/invariant_pattern_gate.rs",
        required_patterns: &[
            "lower_false_invariant_post_flow_loop",
            "gate_is_false",
            "type_name: \"LoopChain\"",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-while-let-native-regressions",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "invariant_while_let_payload_runs_as_a_native_binary",
            "invariant_while_let_mismatch_skips_the_native_loop",
            "invariant_while_let_accepts_runtime_enum_arguments",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-while-let-grammar-reference",
        path: "docs/grammar/nuislang.bnf",
        required_patterns: &[
            "<while_condition> ::= <expression> | \"let\" <pattern> \"=\" <expression>",
            "<while_stmt> ::= \"while\" <while_condition> <block>",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-terminal-pattern-transition-preparation",
        path: "tools/nuisc/src/lowering/loop_preparation_pattern.rs",
        required_patterns: &[
            "prepare_terminal_pattern_transition",
            "PreparedTerminalPatternTransition",
            "variant_parent(type_name) != variant_parent(variant)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-terminal-pattern-transition-llvm-backedge",
        path: "crates/yir-lower-llvm/src/function_lowering/loop_post_flow_chain.rs",
        required_patterns: &["cmp_kind == \"pattern_exit\"", "br label %{loop_exit}"],
    },
    DevTensorDriftCheckSpec {
        id: "language-terminal-pattern-transition-native-regression",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "terminal_while_let_variant_transition_runs_as_a_native_binary",
            "consume(Phase.Active(2)) + consume(Phase.Done)",
            "assert_eq!(status.code(), Some(21))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-carry-preparation",
        path: "tools/nuisc/src/lowering/loop_preparation_pattern.rs",
        required_patterns: &[
            "prepare_dynamic_pattern_plan",
            "prepare_dynamic_pattern_carries",
            "PreviousCarry(0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-carry-domain-contract",
        path: "crates/yir-domain-cpu/src/loop_metadata.rs",
        required_patterns: &["strip_prefix(\"pattern_carry\")"],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-carry-llvm-backedge",
        path: "crates/yir-lower-llvm/src/function_lowering/loop_post_flow_cond_chain.rs",
        required_patterns: &[
            "strip_prefix(\"pattern_carry\")",
            "resolve_source_for_async_post_flow",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-lazy-payload-gate",
        path: "tools/nuisc/src/lowering/loop_flow_nodes_post.rs",
        required_patterns: &["pattern_payload_initial", "transition.initial_payload"],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-native-regression",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "dynamic_while_let_variant_state_runs_across_native_backedges",
            "Phase.Active(payload - 1)",
            "assert_eq!(status.code(), Some(26))",
        ],
    },
];
