use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_CFFI_OBJECT_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "cffi-owned-object-yir-escape-gate",
        path: "tools/nuisc/src/pipeline_ffi_owned_object.rs",
        required_patterns: &[
            "validate_owned_return_object_yir",
            "exactly one direct free",
            "only owned_object_size/owned_object_read_i64",
            "EdgeKind::Lifetime",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "cffi-owned-object-native-closure",
        path: "examples/ns/ffi/owned_return_object_demo.ns",
        required_patterns: &[
            "host_owned_object_make",
            "owned_object_size(object)",
            "owned_object_read_i64(object, 1)",
            "free(object)",
            "host_owned_object_live_count",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "cffi-owned-object-project-closure",
        path: "examples/projects/ffi/owned_return_object_demo/main.ns",
        required_patterns: &[
            "host_owned_object_make",
            "owned_object_size(object)",
            "owned_object_read_i64(object, 1)",
            "free(object)",
            "host_owned_object_live_count",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "cffi-owned-object-regressions",
        path: "tools/nuisc/tests/ffi_owned_object_compile.rs",
        required_patterns: &[
            "lowers_registered_owned_object_with_static_reads_and_exact_cleanup",
            "rejects_owned_object_without_exact_once_release",
            "rejects_owned_object_raw_buffer_fallback",
            "aot_owned_object_returns_to_zero_live_allocations",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "cffi-memory-capability-project-nsld-roundtrip",
        path: "tools/nuisc/src/lib_tests_execution.rs",
        required_patterns: &[
            "compile_command_carries_borrowed_utf8_capability_into_nsld_plan",
            "compile_command_carries_owned_object_authority_into_nsld_plan",
            "memory_capability_count=1",
            "kind=borrowed_utf8",
            "length=nul_terminated",
            "mutability=read_only",
            "lifetime=call",
            "link_plan.host_ffi.memory_capability_count",
            "link_plan.host_ffi.validation.link_allowed",
            "build_link_plan_from_manifest",
            "host_owned_object_destroy",
            "size/read policy",
            "missing or drifted destructor",
            "policy drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "cffi-project-index-registry-authority",
        path: "tools/nuisc/src/project/rendering_host_ffi.rs",
        required_patterns: &[
            "HostFfiRegistryView::try_from_manifest",
            "memory_capabilities(abi, symbol, signature_hash)",
            "collect_destructor_authorities",
            "@nustar-memory-authority",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "cffi-project-index-owned-shape-validation",
        path: "tools/nuisc/src/host_ffi_index.rs",
        required_patterns: &[
            "OwnedReturnUtf8",
            "OwnedReturnObject",
            "ref_FfiObject",
            "OWNED_OBJECT_DESTRUCTOR_SIGNATURE",
            "missing or drifted destructor",
        ],
    },
];
