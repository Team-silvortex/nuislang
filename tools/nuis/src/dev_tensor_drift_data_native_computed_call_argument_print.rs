use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-fallback-closure",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "if let Some((_, result)) = scalar_helpers::pure_float_signature(callee, catalog)",
            "return expression(value, locals, updates, own, catalog, layouts, depth)",
            "A rejected plan must not regain epoch reads through generic fallback",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-typed-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "struct Proof",
            "total: bool",
            "control_values::binary_type(op, left.ty, right.ty)?",
            "proofs[0].ty == *expected",
            "let mut work = 4096usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-exact-call",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "scalar_helpers::pure_scalar_signature(callee, catalog)?",
            "args.len() != params.len()",
            "pending.push(Step::Exact(ty))",
            "pending.push(Step::Value(arg, child, lazy_rhs))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-lazy-boundary",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "(!right.total || !right.ready_predicate)",
            "right.captures.len() > 31",
            "total: false",
            "!matches!(op, NirBinaryOp::Div | NirBinaryOp::Rem)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-shape-scope",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs",
        required_patterns: &[
            "computed_call_expression_roots",
            "scalar_helpers::pure_scalar_signature(callee, catalog)",
            "params.get(index)",
            "Some(&scalar_type(\"bool\"))",
            "roots_with_fields(pending, computed_gate, false, |_, _| false, |_, _, _| false)",
            "let mut logical_edges = 32usize",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-entry-field",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_field_reads.rs",
        required_patterns: &[
            "pub(super) fn project_type",
            "Shape::from_layouts(&entry.ty, layouts).as_ref() != Some(entry)",
            ".filter(|ty| ty == expected)",
            "leaf.fields.is_empty()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-signature",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pub(super) fn pure_float_signature",
            "Some((pure_float_parameters(callee, result, catalog)?, result))",
            "helper.pure_value",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-compiler",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/computed_call_argument_print_epoch_tests.rs",
        required_patterns: &[
            "computed_call_argument_print_epoch_reuses_exact_integer_and_boolean_types",
            "computed_call_argument_print_epoch_certifies_fallible_rhs_without_widening_ordinary_reads",
            "computed_call_argument_print_epoch_scopes_logical_shape_and_shared_edge_budget",
            "computed_call_argument_print_epoch_bounds_work_depth_and_original_leaf_order",
            "computed_call_argument_print_epoch_raw_source_keeps_ieee_values_and_selected_failures",
            "computed_call_argument_print_epoch_ready_float_fallback_keeps_rejected_reads_closed",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-source",
        path: "tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch.ns",
        required_patterns: &[
            "current.inner.tag / first",
            "local.inner.tag % first",
            "current.inner.tag - current.inner.tag",
            "local.inner.flag != false",
            "flag: selected",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-integer-oracle",
        path: "tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "tag.checked_div(self.first)",
            "tag.checked_rem(self.first)",
            "0i64.checked_rem(self.first)",
            "i64::MIN.wrapping_sub(100)",
            "trace.extend([99, word(value) as i64, i64::from(pick), guard])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-typed-native-probes",
        path: "tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "zext i1 %arg1 to i64",
            "trace.extend([96, self.operand, self.first, self.second])",
            "trace.extend([95, current as i64])",
            "for word in [96, 99]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-native-order",
        path: "tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch.rs",
        required_patterns: &[
            "computed_call_argument_print_epoch_native_keeps_ieee_arguments_and_integer_fault_prefixes",
            "calls(events, \"choose\")",
            "ApplicationSessionPhase::Faulted",
            "kind.last_state()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-budget",
        path: "tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch.rs",
        required_patterns: &[
            "computed_call_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/computed_call_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-receipt",
        path: "docs/reference/nuis-native-computed-call-argument-print-v1.md",
        required_patterns: &[
            "Typed Argument Proof",
            "Fallible logical RHS",
            "Predecessor Lineage",
            "512 helper entries",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-call-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Computed integer/bool call argument print witnesses:",
            "Previous float-call checkpoint:",
            "Previous float-call task checkpoint:",
            "Previous float-call blocker checkpoint:",
            "Previous float-call action checkpoint:",
            "Previous float-call validation checkpoint:",
            "Previous float-call artifact checkpoint:",
        ],
    },
];
