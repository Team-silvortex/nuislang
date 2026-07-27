use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_CUDA_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "nuisc-kernel-code-asset-registry",
        path: "tools/nuisc/src/kernel_code_asset.rs",
        required_patterns: &[
            "nuis-kernel-code-asset-registry-v1",
            "cuda.nvidia-gpu",
            "nuis.domain.kernel.cuda.ptx",
            "nuis_kernel_vector_add_f32",
            "registered_kernel_code_assets",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-kernel-cuda-provider-request",
        path: "tools/nuis/src/artifact_device_sample_kernel.rs",
        required_patterns: &[
            "provider_kernel_input_buffers=input.left,input.right",
            "provider_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1",
            "provider_input_binding_count=2",
            "provider_output_binding_0_role=output.result",
            "provider_adapter_binding_provider_family=cuda:nvidia-gpu",
            "validate_provider_request_evidence",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuisc-kernel-code-asset-materialization",
        path: "tools/nuisc/src/aot_domain_artifact_writer.rs",
        required_patterns: &[
            "write_registered_domain_code_asset",
            "domain_code_asset_",
            "materializes_registered_cuda_ptx_without_external_compiler",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsdb-cuda-persistent-worker-driver-runner",
        path: "tools/nsdb/src/provider_execution_cuda.rs",
        required_patterns: &[
            "prepare_vector_add_worker_invocation",
            "validate_provider_code_asset",
            "worker_descriptor_argument",
            "nuis-cuda-ptx-driver-provider-execution-v1",
            "cuda-driver-kernel-completed",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsdb-cuda-driver-abi-adapter",
        path: "tools/nsdb/provider-runners/cuda_vector_add.c",
        required_patterns: &[
            "libcuda.so.1",
            "cuModuleLoad",
            "cuLaunchKernel",
            "NUIS_PROVIDER_OUTPUT_FD",
            "nuis-cuda-ptx-driver-provider-runner-v1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsdb-cuda-real-device-worker-closure-test",
        path: "tools/nsdb/src/provider_sample_execute_tests.rs",
        required_patterns: &[
            "executes_registered_cuda_ptx_through_persistent_worker",
            "cuda:nvidia-gpu",
            "nuis-provider-worker-process-adapter-v5",
            "comparison-passed",
            "graph_output_release_count",
        ],
    },
];
