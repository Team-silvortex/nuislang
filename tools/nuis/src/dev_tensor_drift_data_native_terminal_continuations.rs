use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-single-terminal-generated-uses",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "has_single_continuation_use(&then_body, &else_body)",
            "pending.extend([then_body.as_slice(), else_body.as_slice()])",
            "pending.push(tail)",
            "if uses > 1",
            "uses == 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-terminal-lowering-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_tests.rs",
        required_patterns: &[
            "single_use_terminal_values_stay_inside_guards_without_forwarders",
            "terminal_fallthroughs_count_generated_uses_not_runtime_branch_paths",
            "nontrivial_terminal_expression_is_never_duplicated_by_return_chains",
            "assert_eq!(helpers.len(), 32)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-terminal-execution-budgets",
        path: "tools/nuisc/tests/native_application_bridge/terminal_continuation_budgets.rs",
        required_patterns: &[
            "terminal_forwarding_folds_single_use_boolean_and_record_returns",
            "shared_terminal_work_keeps_one_body_and_guarded_call_sequence",
            "terminal_forwarding_preserves_nested_prefix_and_shared_suffix_order",
            "execute(&source, 0, 7, &[open(&[2], None, &with_prefix[..7])])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-terminal-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/terminal_continuations.rs",
        required_patterns: &[
            "terminal_continuations_keep_typed_results_and_lazy_failures_in_ordinary_entry",
            "Some(45)",
            "pair(false, 0)",
            "fallback(false, 4)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-terminal-probe-registration",
        path: "tools/nuisc/tests/native_application_bridge/helper_entries.rs",
        required_patterns: &[
            "mod terminal_continuation_budgets;",
            "terminal_continuation_budgets.rs",
            "execute(&unused, 0, 8, &[open(&[10, 0], Some(10), PAIR)])",
            "execute(&unused, 0, 7, &[open(&[10, 0], None, &PAIR[..7])])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-terminal-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "Single-Use Terminal Continuations",
            "exactly one generated continuation use",
            "54 reachable native functions instead of 57",
            "Single-Use Statement Continuations",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-terminal-tensor-evidence",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "54 reachable native functions",
            "Generated-use counting is iterative",
            "project field demand through fallthrough record joins while preserving branch-selected values and loop backedges",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-statement-scope-preserving-fold",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs",
        required_patterns: &[
            "_ if single_use =>",
            "hygiene::prepare_arms(",
            "append_single_use_tail(&mut then_body, &mut else_body, tail)",
            "body.extend(tail)",
            "pending.push((body, index + 1))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-statement-lowering-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_tests.rs",
        required_patterns: &[
            "single_use_statement_suffixes_stay_guarded_and_keep_unused_calls",
            "single_use_statement_suffix_merges_into_shared_inner_tail_once",
            "statement_suffixes_keep_boundaries_for_multiple_uses",
            "assert_eq!(helpers.len(), 32)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-statement-execution-budgets",
        path: "tools/nuisc/tests/native_application_bridge/terminal_continuation_budgets.rs",
        required_patterns: &[
            "statement_suffixes_keep_unused_calls_argument_order_and_failure_sentinels",
            "statement_suffixes_keep_boolean_and_record_snapshots",
            "statement_suffix_name_collisions_keep_separate_lexical_bindings",
            "statement_suffix_merges_once_into_the_inner_shared_body",
            "execute(&unused_trap, 0, 6, &[open(&[2], None, &calls[..4])])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-statement-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/terminal_continuations.rs",
        required_patterns: &[
            "statement_continuations_preserve_scope_snapshots_and_unused_failures",
            "Some(46)",
            "let unused = checked(4, divisor - 2);",
            "truth(false, 0)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-statement-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "Single-Use Statement Continuations",
            "63 to 32 private helpers",
            "51 reachable native functions, down from 54",
            "Scope-colliding single-use continuations",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-single-statement-tensor-evidence",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "51 reachable native functions",
            "disjoint lexical bindings",
            "63 to 32 private helpers",
            "project field demand through fallthrough record joins while preserving branch-selected values and loop backedges",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuation-local-hygiene",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs",
        required_patterns: &[
            "!scope.contains_key(*name)",
            "rename_expr(value, &visible)",
            "visible.entry(name.clone()).or_insert_with",
            "fresh_name(\"__nuis_scalar_local\", reserved)",
            "NirExpr::CastBoolToI64(base)",
            "NirExpr::CastI64ToBool(base)",
            "pending.push((then_body, visible.clone()))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuation-hygiene-unit-evidence",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_tests.rs",
        required_patterns: &[
            "single_use_collisions_rename_locals_without_touching_fields_or_callees",
            "sibling_constants_keep_distinct_bindings_when_the_outer_suffix_folds",
            "__nuis_scalar_local_1",
            "assert_eq!(constants.len(), 2)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuation-hygiene-work-budgets",
        path: "tools/nuisc/tests/native_application_bridge/terminal_continuation_budgets.rs",
        required_patterns: &[
            "statement_suffix_name_collisions_keep_separate_lexical_bindings",
            "execute(&source, 0, 4, &[open(&[2], None, &calls[..4])])",
            "hygienic_continuations_preserve_loop_updates_and_work_reservations",
            "hygienic_continuations_keep_bool_and_record_backedges",
            "execute(&outer, 2, 4, &[exhausted])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuation-hygiene-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/terminal_continuations.rs",
        required_patterns: &[
            "hygienic_continuations_keep_sibling_constants_symbols_and_future_names",
            "Some(69)",
            "let __nuis_scalar_local_1 = 3;",
            "result(4, divisor + 2)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuation-hygiene-full-composition",
        path: "tools/nuisc/tests/native_application_bridge/control_composition.ns",
        required_patterns: &[
            "let result = Parts { quotient: quotient(value, 1), remainder: 0 };",
            "let unused = result.quotient + result.remainder;",
            "let result: i64 = counted(value, delta, delta + 4, delta + 1);",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuation-hygiene-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "Hygienic Continuation Bindings",
            "five entries instead of six",
            "initializers retain their original scope",
            "per-return flat-i64 aggregate allocation",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-continuation-hygiene-tensor-evidence",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Scope-aware local hygiene",
            "Private names reserve future source bindings",
            "five actual entries instead of six",
            "project field demand through fallthrough record joins while preserving branch-selected values and loop backedges",
        ],
    },
];
