use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_VULKAN_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "nsdb-vulkan-host-probe",
        path: "tools/nsdb/src/provider_runner_vulkan.rs",
        required_patterns: &[
            "nuis-vulkan-host-probe-v1",
            "spirv.vulkan-gpu.bundle.v1",
            "spirv:vulkan-gpu",
            "libvulkan.so.1",
            "vkEnumerateInstanceVersion",
            "vkCreateInstance",
            "vkEnumeratePhysicalDevices",
            "probe-only-provider-runner",
            "NUIS_REQUIRE_VULKAN_DEVICE_PROBE",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsdb-vulkan-probe-only-execution-gate",
        path: "tools/nsdb/src/provider_execution_vulkan.rs",
        required_patterns: &[
            "vulkan-device-probe-runner",
            "probe-only",
            "registered SPIR-V asset",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "shader-nustar-vulkan-provider-bundle",
        path: "nustar-packages/shader.toml",
        required_patterns: &[
            "spirv.vulkan-gpu.bundle.v1",
            "spirv:vulkan-gpu",
            "provider_runner_vulkan::PROVIDER_BUNDLE",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "shader-nustar-vulkan-spirv-source",
        path: "nustar-packages/assets/shader/vulkan_copy_u32.nspv",
        required_patterns: &[
            "nuis-spirv-compute-source-v1",
            "operation = \"copy-u32\"",
            "entry = \"nuis_vulkan_copy_u32\"",
            "input_binding = 0",
            "output_binding = 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuisc-shader-spirv-emitter",
        path: "tools/nuisc/src/shader_spirv_emitter.rs",
        required_patterns: &[
            "nuis-spirv-compute-source-v1",
            "lower_registered_compute_source",
            "emit_copy_u32_module",
            "SPIRV_MAGIC",
            "SPIRV_VERSION_1_6",
            "rejects_entry_or_binding_drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "shader-nustar-vulkan-code-asset-registration",
        path: "nustar-packages/shader.toml",
        required_patterns: &[
            "shader.vulkan.copy-u32.spirv",
            "spirv-binary",
            "vulkan.discrete-or-integrated-gpu",
            "vulkan1.3-spirv1.6",
            "assets/shader/vulkan_copy_u32.nspv",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuisc-vulkan-spirv-aot-materialization",
        path: "tools/nuisc/src/aot_domain_artifact_writer.rs",
        required_patterns: &[
            "materializes_registered_vulkan_spirv_and_contribution_table",
            "shader.vulkan.copy-u32.spirv",
            "nuis.shader.vulkan.copy-u32.spv",
            "spirv-binary",
            "vulkan.discrete-or-integrated-gpu",
            "nuis_vulkan_copy_u32",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-shader-vulkan-device-sample-registration",
        path: "tools/nuis/src/artifact_device_sample_shader_vulkan.rs",
        required_patterns: &[
            "DeviceSampleInputRegistration",
            "shader.vulkan.copy-u32.spirv",
            "provider_code_asset_format=spirv-binary",
            "provider_adapter_binding_provider_family=spirv:vulkan-gpu",
            "select_compiled_code_asset_contribution",
            "render_selected_contribution_evidence",
            "validate_provider_request_evidence",
        ],
    },
];
