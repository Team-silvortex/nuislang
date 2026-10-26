use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-completed-signature",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pub(super) fn pure_scalar_signature",
            "helper.pure_value",
            "helper.result == scalar_type(&helper.result.name)",
            "helper.params.as_slice(), &helper.result",
            "actual == result",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-rhs-context",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "Step::Value(value, depth, false)",
            "Step::Value(value, depth, lazy_rhs)",
            "lazy_rhs || matches!(op, NirBinaryOp::And | NirBinaryOp::Or)",
            "lazy_rhs && matches!(result.name.as_str(), \"bool\" | \"i64\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-exact-arguments",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "scalar_helpers::pure_scalar_signature(callee, catalog)?",
            "args.len() != params.len()",
            "pending.push(Step::Exact(ty))",
            "pending.push(Step::Value(arg, child, lazy_rhs))",
            "captures.append(&mut proof.captures)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-compiler-signature",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/helper_call_argument_print_epoch_tests.rs",
        required_patterns: &[
            "helper_call_argument_print_epoch_uses_completed_exact_primitive_signatures",
            "helper_call_argument_print_epoch_scopes_nested_calls_to_lazy_rhs_only",
            "helper_call_argument_print_epoch_keeps_auxiliary_logical_argument_roots_closed",
            "helper_call_argument_print_epoch_bounds_complete_call_capture_union_and_depth",
            "helper_call_argument_print_epoch_source_keeps_skipped_calls_under_sign_wrappers",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-source",
        path: "tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch.ns",
        required_patterns: &[
            "fn quotient(value: i64, divisor: i64) -> i64",
            "fn positive(value: i64, divisor: i64) -> bool",
            "positive(quotient(current.inner.tag, second), first)",
            "positive(quotient(local.inner.tag, first), second)",
            "positive(local.inner.tag % first, second)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-source-wrappers",
        path: "tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch.ns",
        required_patterns: &[
            "observe(-choose(",
            "flag: selected == false",
            "finish(current.inner.total, tail)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-independent-entry-oracle",
        path: "tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "trace.extend([101, value, divisor])",
            "trace.extend([102, value, divisor])",
            "quotient(value, divisor, trace, native, counts)?",
            "counts[0] += 1",
            "counts[1] += 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-skipped-work",
        path: "tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "if flag",
            "quotient(tag, self.second, trace, native, counts)?",
            "positive(v, self.first, trace, native, counts)?",
            "tag.checked_rem(self.first.wrapping_sub(2))?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-raw-probes",
        path: "tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch_cases.rs",
        required_patterns: &[
            "[(\"quotient\", 101), (\"positive\", 102)]",
            "zext i1 %arg1 to i64",
            "trace.extend([95, current as i64])",
            "for word in [96, 99, 101, 102]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-structural-counts",
        path: "tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch.rs",
        required_patterns: &[
            "[\"quotient\", \"positive\"]",
            "case.helper_counts()",
            "calls(events, callee)",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-entry-exhaustion",
        path: "tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch.rs",
        required_patterns: &[
            "helper_call_argument_print_epoch_native_keeps_entry_exhaustion_atomic",
            "check_i64_helper_entry_limit_rejection",
            "128",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-delivery",
        path: "tools/nuis/tests/native_session_workflow/helper_call_argument_print_epoch.rs",
        required_patterns: &[
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-receipt",
        path: "docs/reference/nuis-native-helper-call-argument-print-v1.md",
        required_patterns: &[
            "Completed Auxiliary Calls",
            "31-scalar",
            "512 helper entries",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-call-argument-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Pure helper RHS call argument print witnesses:",
            "Previous lazy-call checkpoint:",
            "Previous lazy-call task checkpoint:",
            "Previous lazy-call blocker checkpoint:",
            "Previous lazy-call action checkpoint:",
            "Previous lazy-call validation checkpoint:",
            "Previous lazy-call artifact checkpoint:",
        ],
    },
];
