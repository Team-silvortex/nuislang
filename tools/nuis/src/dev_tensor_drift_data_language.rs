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
        id: "language-named-enum-pattern-tagged-field-projection",
        path: "tools/nuisc/src/frontend/match_pattern_lowering.rs",
        required_patterns: &[
            "let is_enum_variant =",
            "NirExpr::VariantIs",
            "NirExpr::VariantFieldAccess",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-carry-preparation",
        path: "tools/nuisc/src/lowering/loop_preparation_pattern.rs",
        required_patterns: &[
            "prepare_dynamic_pattern_plan",
            "prepare_dynamic_pattern_carries",
            "PreviousCarry(0)",
            "tail_recursive_prev_carry_binding(source_index + 1)",
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
        required_patterns: &[
            "pattern_payload_initial",
            "for payload in &transition.payloads",
            "&payload.initial",
        ],
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
    DevTensorDriftCheckSpec {
        id: "language-post-flow-previous-control-domain-contract",
        path: "crates/yir-domain-cpu/src/loop_metadata.rs",
        required_patterns: &[
            "validate_post_flow_control_kind",
            "other.starts_with(\"prev_carry\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-post-flow-previous-control-llvm-contract",
        path: "crates/yir-lower-llvm/src/loop_flow_control_lowering.rs",
        required_patterns: &[
            "emit_post_loop_flow_control_expr",
            "resolve_previous_carry_operand",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-post-flow-previous-control-execution-routing",
        path: "crates/yir-domain-cpu/src/tests_loop_describe.rs",
        required_patterns: &[
            "execution_path_routes_previous_state_validation_to_async_post_flow_only",
            "cpu.loop_while_scalar_async_post_flow_cond_chain",
            "cpu.loop_while_scalar_async_flow_cond_chain",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-previous-control-native-regression",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "dynamic_while_let_flow_control_reads_the_previous_payload",
            "if payload == 3",
            "assert_eq!(status.code(), Some(50))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-multi-field-native-regression",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "dynamic_while_let_carries_ordered_multi_field_payloads",
            "value: payload, step: stride",
            "assert_eq!(status.code(), Some(34))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-payload-admission-contract",
        path: "tools/nuisc/src/lowering/loop_preparation_pattern.rs",
        required_patterns: &[
            "DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2",
            "PreparedDynamicPatternPayloadTransport",
            "`i64` and `bool` payload carries",
            "GLM-owned payload carry contract",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-payload-admission-regression",
        path: "tools/nuisc/src/lowering/tests_loops_terminal.rs",
        required_patterns: &[
            "admits_bool_dynamic_while_let_payload_through_the_v2_transport",
            "rejects_i32_dynamic_while_let_payload_before_carry_lowering",
            "typed-scalar payload carry contract",
            "rejects_owned_text_dynamic_while_let_payload_before_carry_lowering",
            "GLM-owned payload carry contract",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-bool-native-regression",
        path: "tools/nuisc/tests/control_flow_syntax_native.rs",
        required_patterns: &[
            "dynamic_while_let_preserves_bool_payloads_across_native_backedges",
            "consume(Phase.Active { ready: false })",
            "assert_eq!(status.code(), Some(64))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-yir-carry-contract",
        path: "crates/yir-core/src/dynamic_pattern_carry.rs",
        required_patterns: &[
            "DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER",
            "split_dynamic_pattern_payload_carry_trailer",
            "validate_dynamic_pattern_payload_carry_context",
            "BoolAsI64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-yir-carry-emission",
        path: "tools/nuisc/src/lowering/loop_flow_nodes_post.rs",
        required_patterns: &[
            "encode_dynamic_pattern_payload_carry_trailer",
            "carry_index: index + 1",
            "payload.transport.yir_codec()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-yir-carry-domain-validation",
        path: "crates/yir-domain-cpu/src/loop_metadata.rs",
        required_patterns: &[
            "split_dynamic_pattern_payload_args",
            "validate_dynamic_pattern_payload_carry_context",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "language-dynamic-pattern-yir-carry-llvm-validation",
        path: "crates/yir-lower-llvm/src/function_lowering/loop_post_flow_cond_chain.rs",
        required_patterns: &[
            "split_dynamic_pattern_payload_carry_trailer",
            "validate_dynamic_pattern_payload_carry_context",
            "slot.codec.render()",
        ],
    },
];
