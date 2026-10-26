use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-signature",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pub(super) fn pure_float_parameters",
            "helper.pure_value",
            "actual == result",
            "helper.params.as_slice()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-plan",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "scalar_helpers::pure_scalar_signature(callee, catalog)?",
            "args.len() != params.len()",
            "work.saturating_sub(pending.len())",
            "args.iter().zip(params).rev()",
            "pending.push(Step::Exact(ty))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "pub(super) fn float_argument",
            "catalog: &ScalarHelpers",
            "ready(value, depth)",
            "project(value, scope, layouts, expected)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-typed-leaves",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "epoch.float_argument(",
            "scalar_helpers::pure_i64_parameter_type",
            "|leaf, depth|",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-compiler",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/float_call_print_epoch_tests.rs",
        required_patterns: &[
            "float_call_print_epoch_requires_complete_pure_exact_scalar_signatures",
            "float_call_print_epoch_composes_nested_calls_and_checks_every_parameter_read",
            "float_call_print_epoch_preserves_child_entry_layout_and_single_print_scope",
            "float_call_print_epoch_counts_call_depth_shared_work_and_parameter_order",
            "float_call_print_epoch_source_keeps_ordinary_reads_and_ancestor_calls_closed",
            "float_call_print_epoch_raw_nir_keeps_selected_faults_and_ieee_argument_values",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-source",
        path: "tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print.ns",
        required_patterns: &[
            "@noinline fn identity",
            "@noinline fn scale",
            "identity(ready - local.inner.total, first)",
            "identity(local.inner.total + ready, second)",
            "finish(current.inner.total, tail)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-oracle",
        path: "tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print_cases.rs",
        required_patterns: &[
            "calculation!(calculate_f32",
            "calculation!(calculate_f64",
            "trace.extend([97, word(v), guard])",
            "trace.extend([98, word(v), word(factor), guard])",
            "0i64.checked_div(guard)",
            "right == 0.0",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-raw-state",
        path: "tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print_cases.rs",
        required_patterns: &[
            "trace.extend([96, self.base.operand, self.first, self.second])",
            "trace.extend([95, current as i64])",
            "current ^= self.kind().mask()",
            "local ^= self.kind().mask()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-failures",
        path: "tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print_cases.rs",
        required_patterns: &[
            "[(0, 1), (1, 0), (0, 0)]",
            "skipped.first = 0",
            "skipped.second = 0",
            "zero.base.base.row.trips = 0",
            "ENTRY_LIMIT: u64 = 512",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-native",
        path: "tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print.rs",
        required_patterns: &[
            "float_call_field_epoch_print_native_keeps_ieee_arguments_and_float_fault_prefixes",
            "float_call_field_epoch_print_native_keeps_entry_exhaustion_atomic",
            "ApplicationSessionPhase::Faulted",
            "case.base.base.row",
            "calls(events, \"identity\")",
            "calls(events, \"scale\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-delivery",
        path: "tools/nuis/tests/native_session_workflow/float_call_field_epoch_print.rs",
        required_patterns: &[
            "float_call_field_epoch_print_fresh_cache_and_source_free_delivery_keep_operand_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "\"materialize-artifact\"",
            "fs::remove_file(project.0.join(name))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-receipt",
        path: "docs/reference/nuis-native-float-call-field-epoch-print-v1.md",
        required_patterns: &[
            "Completed Pure Calls",
            "Nonfloat parameters",
            "512 helper entries",
            "128-entry",
            "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-float-call-field-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Pure float-call argument print witnesses:",
            "Previous binary-float checkpoint:",
            "Previous binary-float task checkpoint:",
            "Previous binary-float blocker checkpoint:",
            "Previous binary-float action checkpoint:",
            "Previous binary-float validation checkpoint:",
            "Previous binary-float artifact checkpoint:",
        ],
    },
];
