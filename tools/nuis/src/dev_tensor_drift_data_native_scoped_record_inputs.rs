use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-shared-contract",
        path: "crates/yir-core/src/loop_carry_contract/scoped_record.rs",
        required_patterns: &[
            "pub struct ScopedRecordInput",
            "ScalarStateLayout::parse(encoded)?",
            "operands.len() != layout.fields().len()",
            "ScalarKind::I64",
            "reserved separator",
            "pub fn scoped_input_leaves",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-map-regressions",
        path: "crates/yir-core/src/loop_carry_contract/scoped_record_tests.rs",
        required_patterns: &[
            "scoped_record_maps_preserve_slot_order_and_seed_dependencies",
            "scoped_record_maps_reject_duplicate_missing_and_mismatched_seeds",
            "scoped_record_descriptor_is_bounded_and_resource_free",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-all-caller-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_layouts.rs",
        required_patterns: &[
            "generated.contains(&function.name)",
            "scoped_record_seeds(module, &scoped)",
            "plan.supports_scoped(function, seeds)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-private-planner",
        path: "tools/nuisc/src/lowering/direct_calls/capture_params.rs",
        required_patterns: &[
            "fn supports_scoped(",
            "seeds.contains_key(&index)",
            "Slot::Bools(_) => false",
            "fn lower_scoped_arguments(",
            "ScopedRecordInput::encode(layout, inputs)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-runtime-current-values",
        path: "crates/yir-domain-cpu/src/execute_scoped_loop/record_inputs.rs",
        required_patterns: &[
            "ScopedRecordInput::parse(input)?",
            "Leaf::Carry(index)",
            "Value::Int(value)",
            "self.layout.unpack(&words)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-llvm-atomicity",
        path: "crates/yir-lower-llvm/src/scoped_record_args/tests.rs",
        required_patterns: &[
            "scoped_record_layout_and_exact_leaf_kinds_are_checked_before_emission",
            "scoped_call_validates_all_record_arguments_before_packing",
            "assert_eq!(body, [\"entry:\"])",
            "assert_eq!(next_reg, 73)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-source-execution",
        path: "tools/nuisc/tests/native_application_bridge/typed_scoped_record_inputs.rs",
        required_patterns: &[
            "typed_scoped_record_inputs_preserve_full_seeds_and_per_trip_updates",
            "typed_scoped_record_inputs_keep_break_mapping_independent_of_record_width",
            "typed_scoped_record_inputs_preserve_independent_boolean_carry_and_break",
            "typed_scoped_record_inputs_keep_iteration_failures_and_entry_limits_atomic",
            "typed_scoped_record_inputs_reject_descriptor_and_parameter_drift",
            "wide_scoped_record_branch_helpers_keep_the_remaining_argument_boundary_explicit",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-source-free-workflow",
        path: "tools/nuis/tests/native_session_workflow/scoped_record_inputs.rs",
        required_patterns: &[
            "native_scoped_record_inputs_cache_and_restore_the_full_mapping_without_sources",
            "compile_cache: hit",
            "verify-artifact",
            "materialize-artifact",
            "fs::remove_file(project.0.join(\"main.ns\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-scoped-record-input-documented-boundary",
        path: "docs/reference/nuis-native-scalar-value-returns-v1.md",
        required_patterns: &[
            "## Scoped Record Inputs",
            "$value_record:<layout>|<leaf0>|...",
            "Only multi-carry scoped actions admit it",
            "complete seed range",
            "remaining reproduced boundary is a wide generated branch helper",
        ],
    },
];
