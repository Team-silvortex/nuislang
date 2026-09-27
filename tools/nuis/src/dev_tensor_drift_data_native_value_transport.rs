use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-record-codec-checked-snapshot",
        path: "crates/yir-lower-llvm/src/native_session/value_transport.rs",
        required_patterns: &[
            "not authority to change a function or external ABI",
            "pub(crate) struct PreparedNativeValue",
            "prepare_record(&self.layout, value, &mut words)",
            "value.type_name != layout.type_name",
            "Pure records must not inherit owned-variant conversion or zero-fill rules",
            "debug_assert_eq!(words.len(), self.slots)",
            "insertvalue {ty} {aggregate}, i64 {field}, {index}",
            "extractvalue {ty} {aggregate}, {slot}",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-codec-layout-and-snapshot-regressions",
        path: "crates/yir-lower-llvm/src/native_session/value_transport/tests.rs",
        required_patterns: &[
            "transport_counts_all_nested_scalar_leaves_and_rejects_resources",
            "prepared_values_order_nested_fields_and_own_their_snapshot",
            "prepare_rejects_nominal_field_and_scalar_kind_drift_before_emission",
            "input_and_output_decoding_keep_typed_independent_registers",
            "pure_record_transport_does_not_inherit_owned_variant_name_semantics",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-codec-host-roundtrip-probe",
        path: "crates/yir-lower-llvm/src/native_session/value_transport/tests/native.rs",
        required_patterns: &[
            "typed_record_arguments_round_trip_through_host_llvm",
            "requires a host clang driver; run explicitly with --ignored",
            "for optimization in [\"-O0\", \"-O2\"]",
            "[64 x i64] %first, [64 x i64] %second, i1 %which",
            ".prepare(&LlvmValueRef::Struct(first))",
            "0x7ff1_1234_5678_9abc",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-codec-return-rejection-transaction",
        path: "crates/yir-lower-llvm/src/native_session/aggregate_values/tests.rs",
        required_patterns: &[
            "rejected_value_return_leaves_ir_and_fresh_ids_unchanged",
            "assert_eq!(body, [\"entry:\"])",
            "assert_eq!((next_reg, next_block), (17, 23))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-codec-documented-integration-boundary",
        path: "docs/reference/nuis-native-scalar-value-returns-v1.md",
        required_patterns: &[
            "## Shared Pure-Value Codec",
            "This codec is not signature authority",
            "flatten whole-record inputs",
            "## Generated Record Inputs",
            "Scoped targets require the separate all-caller seed proof",
            "typed_record_arguments_round_trip_through_host_llvm --ignored",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-codec-tensor-evidence",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "A shared bounded pure-value record codec now serves native helper and callback returns",
            "retain the checked non-scoped record parameter contract",
            "nested record carries still lack exact field-path scoped seed and backedge maps",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-cpu-contract",
        path: "crates/yir-domain-cpu/src/value_parameters.rs",
        required_patterns: &[
            "param_value_struct", "ScalarStateLayout::parse(encoded)?",
            "value.type_name == layout.type_name", "value.fields.len() == layout.fields.len()",
            "does not match nominal layout",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-registered-verification",
        path: "crates/yir-verify/src/tests/function_parameters.rs",
        required_patterns: &[
            "registered_value_parameter_checks_index_type_ownership_domain_and_binding",
            "parameter_contract_is_domain_registered_not_a_cpu_opcode_switch",
            "disagrees with its function declaration", "invalid parameter",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-registered-host-forwarding",
        path: "crates/yir-runtime-host/src/application_outcome_pump/host_registry.rs",
        required_patterns: &[
            "fn function_parameter(", "fn validate_function_argument(",
            ".function_parameter(node, resource)", ".validate_function_argument(node, resource, value)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-runtime-argument-check",
        path: "crates/yir-exec/src/execution_engine.rs",
        required_patterns: &[
            "function.parameters.iter().zip(&arguments)",
            "domain.validate_function_argument(node, self.resources[&node.resource], value)?",
            "let saved_values = function",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-llvm-value-signature",
        path: "crates/yir-lower-llvm/src/call_parameters.rs",
        required_patterns: &[
            "enum CpuCallParameterKind", "Record(NativeValueLayout)",
            "layout.prepare(value)", "yir_domain_cpu::value_parameters::parse(node)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-private-plan-boundaries",
        path: "tools/nuisc/src/lowering/direct_calls/capture_params_tests.rs",
        required_patterns: &[
            "generated_record_plan_is_bounded_and_only_needed_after_scalar_compaction",
            "wide_record_scoped_helpers_keep_carry_and_seed_signature_mapping",
            "outlined.capture_plans.contains_key(name)",
            "generated_capture_transport_does_not_rewrite_user_signatures_or_namesakes",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-source-native-parity",
        path: "tools/nuisc/tests/native_application_bridge/typed_record_inputs.rs",
        required_patterns: &[
            "typed_record_inputs_replace_65_leaf_captures_without_widening_callback_abi",
            "typed_record_inputs_preserve_mixed_nested_snapshot_bits",
            "compiled.yir.nodes.reverse()", "source helper ABI remains flattened",
            "probe_allocs", "probe_drops",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-guarded-publication",
        path: "tools/nuisc/tests/native_application_bridge/typed_record_guards.rs",
        required_patterns: &[
            "typed_record_inputs_reject_layout_drift_before_execution_or_publication",
            "typed_record_inputs_keep_lazy_traps_and_shared_entry_limits_atomic",
            "does not match nominal layout", "vec![-700; 64]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-source-free-restoration",
        path: "tools/nuis/tests/native_session_workflow/record_inputs.rs",
        required_patterns: &[
            "native_record_inputs_build_cache_and_restore_without_sources",
            "compile_cache: hit", "materialize-artifact", "verify-artifact",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-record-input-llvm-boundaries-and-atomicity",
        path: "crates/yir-lower-llvm/src/tests/record_parameter_tests.rs",
        required_patterns: &[
            "record_parameters_use_value_signatures_without_task_invokers",
            "record_parameters_reject_deferred_task_transport",
            "record_parameters_cannot_be_coerced_into_scoped_scalar_arguments",
            "all_record_arguments_are_checked_before_any_call_packing",
            "assert_eq!(next_reg, 73)",
        ],
    },
];
