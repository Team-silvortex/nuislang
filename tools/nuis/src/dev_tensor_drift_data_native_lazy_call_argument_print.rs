use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-scalar-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_field_captures.rs",
        required_patterns: &[
            "pub(super) struct Captures",
            "self.params.len() >= 31",
            "self.work.checked_sub(1)?",
            "depth > 64",
            "self.paths.get(&path)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-original-path-types",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_field_captures.rs",
        required_patterns: &[
            "self.scope.get(root)?",
            ".get(&ty.name)?",
            "ty != scalar_type(&ty.name)",
            "fields.next().is_some()",
            "self.args.push(value.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-projected-outline",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs",
        required_patterns: &[
            "if let NirStmt::Print(value) = stmt",
            "field_captures::collect(&rhs, scope, layouts)",
            "projected.value, projected.params, projected.args",
            "args.extend(captures)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-guard-body",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs",
        required_patterns: &[
            "let rhs_scope = params",
            "predicate_inner(rhs, &rhs_scope",
            "NirStmt::Return(Some(NirExpr::Bool(false)))",
            "guarded.insert(name.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-word-wrappers",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs",
        required_patterns: &[
            "NirExpr::PackF32Word(v)",
            "NirExpr::UnpackF32Word(v)",
            "NirExpr::PackF64Word(v)",
            "NirExpr::UnpackF64Word(v)",
            "NirExpr::CastI32ToI64(v)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-capture-union",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "captures: BTreeSet<Vec<String>>",
            "(!right.total || !right.ready_predicate)",
            "right.captures.len() > 31",
            "left.captures.append(&mut right.captures)",
            "captures.append(&mut proof.captures)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-capture-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/logical_field_captures_tests.rs",
        required_patterns: &[
            "lazy_field_captures_keep_all_scalar_kinds_order_dedup_and_hygiene",
            "lazy_field_captures_bound_complete_union_and_reject_unsafe_or_unknown_paths",
            "lazy_field_capture_helpers_keep_checks_after_guard_even_when_all_fields_are_used",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-epoch-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/lazy_call_argument_print_epoch_tests.rs",
        required_patterns: &[
            "lazy_call_argument_print_epoch_bounds_complete_rhs_capture_union",
            "lazy_call_argument_print_epoch_raw_nir_keeps_both_short_circuit_edges_under_sign_wrappers",
            "lazy_call_argument_print_epoch_reaches_only_selected_zero_and_overflow_faults",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-source",
        path: "tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch.ns",
        required_patterns: &[
            "observe(-choose(",
            "current.inner.flag && current.inner.tag / second",
            "local.inner.flag || ((local.inner.tag / first",
            "local.inner.tag % (first - 2)",
            "flag: selected == false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "if flag",
            "tag.checked_div(self.second)?",
            "tag.checked_rem(self.first.wrapping_sub(2))?",
            "flag = !row.selected",
            "flag = row.selected",
            "0xaaaaaaaaaaaaaaab",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-native-input-state",
        path: "tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "trace.extend([99, word(value) as i64, i64::from(pick), guard])",
            "zext i1 %arg1 to i64",
            "trace.extend([96, self.operand, self.first, self.second])",
            "trace.extend([95, current as i64])",
            "for word in [96, 99]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-native-failure",
        path: "tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch.rs",
        required_patterns: &[
            "lazy_call_argument_print_epoch_native_keeps_ieee_arguments_and_integer_fault_prefixes",
            "calls(events, \"choose\")",
            "ApplicationSessionPhase::Faulted",
            "state_words(session.state())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-entry-budget",
        path: "tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch.rs",
        required_patterns: &[
            "lazy_call_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/lazy_call_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-receipt",
        path: "docs/reference/nuis-native-lazy-call-argument-print-v1.md",
        required_patterns: &[
            "Scalar RHS Captures",
            "31 scalar captures",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-lazy-call-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Lazy checked call argument print witnesses:",
            "Previous computed-call checkpoint:",
            "Previous computed-call task checkpoint:",
            "Previous computed-call blocker checkpoint:",
            "Previous computed-call action checkpoint:",
            "Previous computed-call validation checkpoint:",
            "Previous computed-call artifact checkpoint:",
        ],
    },
];
