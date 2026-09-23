use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-descendant-writes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/sequences.rs",
        required_patterns: &[
            "NirStmt::While { body, .. } => pending.extend(body.iter().rev())",
            "updates.contains(&prepared.binding_name)",
            "contains_loop(effects)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-source-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops.rs",
        required_patterns: &[
            "update_name(iteration.step, &locals.scope, &locals.writable)",
            "changed.insert(prepared.binding_name.clone())",
            "expression(input, locals, updates, None, catalog, layouts, 0)",
            "child.available.retain(|name| !changed.contains(name))",
            "child_updates.extend(changed.iter().cloned())",
            "locals.available.extend(changed)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-shared-source-depth",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "NirStmt::While { condition, body } if depth < 32",
            "literal_loops::validate(condition, body, locals, updates, catalog, layouts, depth)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-local-capture",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/validation.rs",
        required_patterns: &[
            "if let EffectTypes::Values(..) = types",
            "let mut child_carries = Vec::new()",
            "locals.contains_key(&name) && !carries.contains(&name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-shared-outlining",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs",
        required_patterns: &[
            "|| contains_loop(body)",
            "fn outline_iteration(",
            "body.len() > 1 || present(body, scope)",
            "Child effects were checked as scoped expressions",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-branch-outlining",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/branches.rs",
        required_patterns: &[
            "validation::EffectTypes::Values(catalog, layouts)",
            "control_loops::outline_iteration(",
            "validation::EffectTypes::Buffer(catalog)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-admission-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops_tests.rs",
        required_patterns: &[
            "literal_loops_use_existing_scopes_and_keep_child_declarations_local",
            "literal_loops_cannot_launder_headers_write_authority_or_future_siblings",
            "literal_loop_scope_and_combined_branch_loop_depth_are_bounded",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-execution",
        path: "tools/nuisc/tests/native_application_bridge/literal_loops.rs",
        required_patterns: &[
            "literal_nested_loops_share_bool_flat_carries_and_independent_snapshots",
            "literal_nested_loops_preflight_only_selected_invocations",
            "literal_nested_loops_keep_selected_and_overwritten_arithmetic_failures",
            "CallProbe",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-budgets",
        path: "tools/nuisc/tests/native_application_bridge/literal_loops_budget.rs",
        required_patterns: &[
            "literal_nested_loops_share_exact_loop_and_helper_entry_budgets",
            "literal_nested_zero_trip_and_counter_only_loops_do_not_invent_helper_entries",
            "literal_child_single_bool_guard_uses_scoped_not_metadata_predicates",
            "exhausted.remaining_loop = 2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-ordinary-entry",
        path: "tools/nuisc/tests/control_flow_syntax_native/literal_loops.rs",
        required_patterns: &[
            "literal_nested_loops_keep_persistent_and_iteration_local_induction",
            "literal_child_predicates_remain_scoped_even_for_a_single_scalar_update",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-frontdoor",
        path: "tools/nuis/tests/native_session_workflow.rs",
        required_patterns: &[
            "native_literal_nested_loops_build_cache_and_standalone_relocation",
            "check_workflow(LITERAL_LOOPS_SOURCE)",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-literal-loop-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Literal Nested Counted Loops",
            "Every selected invocation performs its own complete induction preflight",
            "Neither counter resets at nesting boundaries",
            "Guarded exits in these leading-step bodies now use the contract below",
        ],
    },
];
