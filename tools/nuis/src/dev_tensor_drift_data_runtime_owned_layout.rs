use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_OWNED_LAYOUT_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "yir-canonical-owned-struct-layout-contract",
        path: "crates/yir-core/src/owned_struct_layout.rs",
        required_patterns: &[
            "OWNED_VARIANT_UNION_LAYOUT_PREFIX",
            "MAX_LAYOUT_BYTES",
            "MAX_LAYOUT_DEPTH",
            "MAX_LAYOUT_FIELDS",
            "OwnedStructScalarLayout",
            "OwnedStructFieldLayout",
            "parse_owned_struct_layout",
            "parses_flat_nested_and_variant_union_layouts",
            "rejects_trailing_or_malformed_layouts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "cpu-canonical-owned-struct-layout-consumer",
        path: "crates/yir-domain-cpu/src/execute_values.rs",
        required_patterns: &[
            "parse_owned_struct_layout(layout_source)",
            "default_owned_layout_value",
            "default_owned_field_value",
            "Value::VariantUnion(VariantUnionValue",
            "must bind an owned variant union layout",
            "owned variant union layout does not match",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "runtime-owned-blob-aggregate-source-owner",
        path: "crates/nuis-runtime/src/c_shim_owned_blob_runtime.rs",
        required_patterns: &[
            "pub fn append_c_shim_owned_blob_runtime",
            "nuis_scheduler_owned_blob_copy_v1",
            "nuis_scheduler_owned_aggregate_alloc_v1",
            "nuis_scheduler_owned_aggregate_set_scalar_v1",
            "nuis_scheduler_owned_aggregate_finish_v1",
            "nuis_scheduler_owned_aggregate_require_v1",
            "nuis_scheduler_owned_aggregate_drop_v1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "yir-pack-aot-owned-aggregate-host-runtime",
        path: "tools/yir-pack-aot/src/main.rs",
        required_patterns: &[
            "use nuis_runtime::append_c_shim_owned_blob_runtime",
            "host_text_runtime_source",
            "append_c_shim_owned_blob_runtime(&mut out)",
            "append_c_shim_owned_blob_runtime(&mut owned_blob_runtime)",
            "const char *action_class",
            "const char *event_name",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "shader-result-enum-owned-layout-aot-regression",
        path: "tools/nuis/src/main_tests/artifact_runtime_run_artifact.rs",
        required_patterns: &[
            "build_report_json_exposes_shader_result_enum_bundle_summary",
            "shader_result_enum_demo",
            "\\\"ready_to_run\\\":true",
            "\\\"runtime_host_yir_ok\\\":true",
            "\\\"link_plan_final_driver\\\":\\\"yir-pack-aot\\\"",
        ],
    },
];
