use super::*;

fn vulkan_host_available() -> bool {
    Command::new("vulkaninfo")
        .arg("--summary")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn output_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

struct VulkanSampleSmoke<'a> {
    project_name: &'a str,
    assets: &'a [VulkanGeneratedAsset<'a>],
    registration_id: &'a str,
    expected_hash: &'a str,
    graph: Option<VulkanGraphSmoke<'a>>,
}

struct VulkanGeneratedAsset<'a> {
    asset_id: &'a str,
    generated_file: &'a str,
    entry: &'a str,
}

struct VulkanGraphSmoke<'a> {
    native_output_count: usize,
    final_output_index: usize,
    final_request_id: &'a str,
    final_expected_hash: &'a str,
    compiled_selection_count: usize,
    edges: &'a [VulkanGraphEdgeSmoke<'a>],
}

struct VulkanGraphEdgeSmoke<'a> {
    consumer_request_index: usize,
    transport_token: &'a str,
    dependency_output_buffer: &'a str,
    dependency_input_shape: &'a str,
    dependency_byte_length: usize,
}

#[test]
fn linux_vulkan_shader_sample_executes_provider_output() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.copy-u32.spirv",
            generated_file: "nuis.shader.vulkan.copy-u32.spv",
            entry: "nuis_vulkan_copy_u32",
        }],
        registration_id: "official.shader.vulkan-copy-u32",
        expected_hash: "0x6ebcdd244b594ee4",
        graph: None,
    });
}

#[test]
fn linux_vulkan_add_shader_sample_executes_provider_output() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_add_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.add-u32.spirv",
            generated_file: "nuis.shader.vulkan.add-u32.spv",
            entry: "nuis_vulkan_add_u32",
        }],
        registration_id: "official.shader.vulkan-add-u32",
        expected_hash: "0xdce2c1aca0f32707",
        graph: None,
    });
}

#[test]
fn linux_vulkan_add_pair_shader_sample_executes_provider_output() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_add_pair_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.add-pair-u32.spirv",
            generated_file: "nuis.shader.vulkan.add-pair-u32.spv",
            entry: "nuis_vulkan_add_pair_u32",
        }],
        registration_id: "official.shader.vulkan-add-pair-u32",
        expected_hash: "0xbada6f73928b9f42",
        graph: None,
    });
}

#[test]
fn linux_vulkan_fan_out_shader_writes_two_provider_outputs() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_fan_out_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.add-xor-pair-u32.spirv",
            generated_file: "nuis.shader.vulkan.add-xor-pair-u32.spv",
            entry: "nuis_vulkan_add_xor_pair_u32",
        }],
        registration_id: "official.shader.vulkan-add-xor-pair-u32",
        expected_hash: "0xbada6f73928b9f42",
        graph: None,
    });
}

#[test]
fn linux_vulkan_padded_fan_out_shader_writes_independent_output_layouts() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_padded_fan_out_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.add-xor-pair-u32.spirv",
            generated_file: "nuis.shader.vulkan.add-xor-pair-u32.spv",
            entry: "nuis_vulkan_add_xor_pair_u32",
        }],
        registration_id: "official.shader.vulkan-add-xor-pair-padded-u32",
        expected_hash: "0xbada6f73928b9f42",
        graph: None,
    });
}

#[test]
fn linux_vulkan_reduced_fan_out_shader_bounds_secondary_output() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_reduced_fan_out_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.add-xor-pair-reduced-u32.spirv",
            generated_file: "nuis.shader.vulkan.add-xor-pair-reduced-u32.spv",
            entry: "nuis_vulkan_add_xor_pair_reduced_u32",
        }],
        registration_id: "official.shader.vulkan-add-xor-pair-reduced-u32",
        expected_hash: "0xbada6f73928b9f42",
        graph: None,
    });
}

#[test]
fn linux_vulkan_reduced_fan_out_shader_feeds_independent_typed_downstream_inputs() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_reduced_output_fan_out_provider_demo",
        assets: &[
            VulkanGeneratedAsset {
                asset_id: "shader.vulkan.add-xor-pair-reduced-u32.spirv",
                generated_file: "nuis.shader.vulkan.add-xor-pair-reduced-u32.spv",
                entry: "nuis_vulkan_add_xor_pair_reduced_u32",
            },
            VulkanGeneratedAsset {
                asset_id: "shader.vulkan.copy-u32.spirv",
                generated_file: "nuis.shader.vulkan.copy-u32.spv",
                entry: "nuis_vulkan_copy_u32",
            },
            VulkanGeneratedAsset {
                asset_id: "shader.vulkan.xor-u32.spirv",
                generated_file: "nuis.shader.vulkan.xor-u32.spv",
                entry: "nuis_vulkan_xor_u32",
            },
        ],
        registration_id: "official.shader.vulkan-reduced-output-fan-out-u32",
        expected_hash: "0xbada6f73928b9f42",
        graph: Some(VulkanGraphSmoke {
            native_output_count: 3,
            final_output_index: 2,
            final_request_id: "shader.vulkan.reduced-fan-out.xor-reduced-u32",
            final_expected_hash: "0xa8c7f832281a39c5",
            compiled_selection_count: 3,
            edges: &[
                VulkanGraphEdgeSmoke {
                    consumer_request_index: 1,
                    transport_token: "glm:provider-edge:shader.vulkan.reduced-fan-out.add-xor-pair-u32:output.values->shader.vulkan.reduced-fan-out.copy-sum-u32:input.values",
                    dependency_output_buffer: "output.values",
                    dependency_input_shape: "2x2",
                    dependency_byte_length: 16,
                },
                VulkanGraphEdgeSmoke {
                    consumer_request_index: 2,
                    transport_token: "glm:provider-edge:shader.vulkan.reduced-fan-out.add-xor-pair-u32:output.xor->shader.vulkan.reduced-fan-out.xor-reduced-u32:input.values",
                    dependency_output_buffer: "output.xor",
                    dependency_input_shape: "2x1",
                    dependency_byte_length: 8,
                },
            ],
        }),
    });
}

#[test]
fn linux_vulkan_producer_fans_out_to_cuda_and_vulkan_consumers() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_cuda_fan_out_provider_demo",
        assets: &[
            VulkanGeneratedAsset {
                asset_id: "shader.vulkan.add-xor-pair-reduced-u32.spirv",
                generated_file: "nuis.shader.vulkan.add-xor-pair-reduced-u32.spv",
                entry: "nuis_vulkan_add_xor_pair_reduced_u32",
            },
            VulkanGeneratedAsset {
                asset_id: "kernel.cuda.copy-u32.ptx",
                generated_file: "nuis.domain.kernel.copy-u32.cuda.ptx",
                entry: "nuis_kernel_copy_u32",
            },
            VulkanGeneratedAsset {
                asset_id: "shader.vulkan.xor-u32.spirv",
                generated_file: "nuis.shader.vulkan.xor-u32.spv",
                entry: "nuis_vulkan_xor_u32",
            },
        ],
        registration_id: "official.shader.vulkan-cuda-reduced-output-fan-out-u32",
        expected_hash: "0xbada6f73928b9f42",
        graph: Some(VulkanGraphSmoke {
            native_output_count: 3,
            final_output_index: 2,
            final_request_id: "shader.vulkan-cuda-fan-out.xor-reduced-u32",
            final_expected_hash: "0xa8c7f832281a39c5",
            compiled_selection_count: 3,
            edges: &[
                VulkanGraphEdgeSmoke {
                    consumer_request_index: 1,
                    transport_token: "glm:provider-edge:shader.vulkan-cuda-fan-out.add-xor-pair-u32:output.values->kernel.cuda.fan-out.copy-sum-u32:input.values",
                    dependency_output_buffer: "output.values",
                    dependency_input_shape: "2x2",
                    dependency_byte_length: 16,
                },
                VulkanGraphEdgeSmoke {
                    consumer_request_index: 2,
                    transport_token: "glm:provider-edge:shader.vulkan-cuda-fan-out.add-xor-pair-u32:output.xor->shader.vulkan-cuda-fan-out.xor-reduced-u32:input.values",
                    dependency_output_buffer: "output.xor",
                    dependency_input_shape: "2x1",
                    dependency_byte_length: 8,
                },
            ],
        }),
    });
}

#[test]
fn linux_vulkan_sub_shader_sample_executes_provider_output() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_sub_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.sub-u32.spirv",
            generated_file: "nuis.shader.vulkan.sub-u32.spv",
            entry: "nuis_vulkan_sub_u32",
        }],
        registration_id: "official.shader.vulkan-sub-u32",
        expected_hash: "0x88201fb960ff6465",
        graph: None,
    });
}

#[test]
fn linux_vulkan_mul_shader_sample_executes_provider_output() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_mul_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.mul-u32.spirv",
            generated_file: "nuis.shader.vulkan.mul-u32.spv",
            entry: "nuis_vulkan_mul_u32",
        }],
        registration_id: "official.shader.vulkan-mul-u32",
        expected_hash: "0x02f6f7f591bff68f",
        graph: None,
    });
}

#[test]
fn linux_vulkan_xor_shader_sample_executes_provider_output() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_xor_provider_demo",
        assets: &[VulkanGeneratedAsset {
            asset_id: "shader.vulkan.xor-u32.spirv",
            generated_file: "nuis.shader.vulkan.xor-u32.spv",
            entry: "nuis_vulkan_xor_u32",
        }],
        registration_id: "official.shader.vulkan-xor-u32",
        expected_hash: "0x88201fb960ff6465",
        graph: None,
    });
}

#[test]
fn linux_vulkan_chain_shader_sample_executes_provider_graph() {
    run_vulkan_sample_smoke(VulkanSampleSmoke {
        project_name: "shader_vulkan_chain_provider_demo",
        assets: &[
            VulkanGeneratedAsset {
                asset_id: "shader.vulkan.add-u32.spirv",
                generated_file: "nuis.shader.vulkan.add-u32.spv",
                entry: "nuis_vulkan_add_u32",
            },
            VulkanGeneratedAsset {
                asset_id: "shader.vulkan.xor-u32.spirv",
                generated_file: "nuis.shader.vulkan.xor-u32.spv",
                entry: "nuis_vulkan_xor_u32",
            },
        ],
        registration_id: "official.shader.vulkan-u32-chain",
        expected_hash: "0xdce2c1aca0f32707",
        graph: Some(VulkanGraphSmoke {
            native_output_count: 2,
            final_output_index: 1,
            final_request_id: "shader.vulkan.chain.xor-u32",
            final_expected_hash: "0x88201fb960ff6465",
            compiled_selection_count: 2,
            edges: &[VulkanGraphEdgeSmoke {
                consumer_request_index: 1,
                transport_token:
                    "glm:provider-edge:shader.vulkan.chain.add-u32:output.values->shader.vulkan.chain.xor-u32:input.values",
                dependency_output_buffer: "output.values",
                dependency_input_shape: "4",
                dependency_byte_length: 16,
            }],
        }),
    });
}

fn run_vulkan_sample_smoke(sample: VulkanSampleSmoke<'_>) {
    if !vulkan_host_available() {
        return;
    }
    let output_dir = temp_dir(sample.project_name);
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/shader_vulkan_provider_demo");
    let project = project.with_file_name(sample.project_name);
    let project_text = project.display().to_string();
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&["build", &project_text, &output_dir_text]);
    assert_success(&build, "build Linux Vulkan shader provider project");
    for asset in sample.assets {
        assert_file_contains(
            &output_dir.join("nuis.domain.code-asset-contributions.toml"),
            asset.asset_id,
            "Vulkan code asset contribution table",
        );
    }
    assert_file_contains(
        &output_dir.join("nuis.domain.code-asset-contributions.toml"),
        "vulkan.discrete-or-integrated-gpu",
        "Vulkan lowering target contribution",
    );
    for asset in sample.assets {
        let spirv =
            fs::read(output_dir.join(asset.generated_file)).expect("Vulkan SPIR-V code asset");
        assert!(spirv
            .windows(asset.entry.len())
            .any(|window| window == asset.entry.as_bytes()));
    }

    let run = run_nuis(&["run-artifact", &output_dir_text, "--json"]);
    assert_success(&run, "materialize Linux Vulkan provider request");
    let provider_samples =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml"))
            .expect("provider sample manifest");
    assert!(provider_samples.contains("provider_family = \"spirv:vulkan-gpu\""));
    assert!(provider_samples.contains(&format!(
        "provider_sample_registration_id={}",
        sample.registration_id
    )));
    for asset in sample.assets {
        assert!(provider_samples.contains(&format!("code_asset_id={}", asset.asset_id)));
    }
    assert!(provider_samples.contains("adapter_binding_provider_family=spirv:vulkan-gpu"));
    assert!(provider_samples.contains(
        "provider_code_asset_contribution_selection_set_contract=nuis-provider-code-asset-contribution-selection-set-v1"
    ));
    if matches!(
        sample.registration_id,
        "official.shader.vulkan-add-pair-u32"
            | "official.shader.vulkan-add-xor-pair-u32"
            | "official.shader.vulkan-add-xor-pair-padded-u32"
            | "official.shader.vulkan-add-xor-pair-reduced-u32"
            | "official.shader.vulkan-reduced-output-fan-out-u32"
            | "official.shader.vulkan-cuda-reduced-output-fan-out-u32"
    ) {
        assert!(provider_samples.contains("input_binding_contract=nuis-provider-input-binding-v2"));
        assert!(provider_samples.contains("buffer_layout=tensor-row-major"));
        assert!(provider_samples.contains("buffer_shape=2x2"));
        assert!(provider_samples.contains("input_binding_1_layout=tensor-row-major"));
        assert!(provider_samples.contains("input_binding_1_row_stride_bytes=8"));
    }
    if sample.registration_id == "official.shader.vulkan-add-xor-pair-padded-u32" {
        assert!(provider_samples.contains("output_binding_1_row_stride_bytes=12"));
        assert!(provider_samples.contains("output_binding_1_byte_length=24"));
    }
    if matches!(
        sample.registration_id,
        "official.shader.vulkan-add-xor-pair-reduced-u32"
            | "official.shader.vulkan-reduced-output-fan-out-u32"
            | "official.shader.vulkan-cuda-reduced-output-fan-out-u32"
    ) {
        assert!(provider_samples.contains("output_binding_1_shape=2x1"));
        assert!(provider_samples.contains("output_binding_1_row_stride_bytes=8"));
        assert!(provider_samples.contains("output_binding_1_byte_length=8"));
    }

    let executed = run_nsdb(&[
        "execute-provider-samples",
        &output_dir_text,
        "--provider-family",
        "spirv:vulkan-gpu",
        "--json",
    ]);
    assert_success(&executed, "execute Linux Vulkan provider request");
    let executed_text = output_text(&executed);
    assert!(executed_text.contains("\"status\":\"provider-output-payloads-ready\""));
    assert!(
        executed_text.contains(
            "\"first_output_payload_native_execution_contract\":\"nuis-vulkan-spirv-provider-execution-v1\""
        )
    );
    assert!(executed_text.contains(
        "\"first_output_payload_native_execution_status\":\"vulkan-compute-dispatch-completed\""
    ));
    assert!(
        executed_text.contains("\"first_output_payload_comparison_status\":\"comparison-passed\"")
    );
    assert!(executed_text.contains(&format!(
        "\"first_output_payload_native_output_hash\":\"{}\"",
        sample.expected_hash
    )));

    let provider_output = output_dir.join("nuis.nsdb.provider-output.spirv-vulkan-gpu.toml");
    assert_file_contains(
        &provider_output,
        "output_payload_status = \"native-api-output-ready\"",
        "Vulkan provider output payload",
    );
    assert_file_contains(
        &provider_output,
        "native_output_0_execution_contract = \"nuis-vulkan-spirv-provider-execution-v1\"",
        "Vulkan native execution contract",
    );
    assert_file_contains(
        &provider_output,
        "native_output_0_device = \"vulkan:spirv-gpu:device-",
        "Vulkan selected device evidence",
    );
    assert_file_contains(
        &provider_output,
        "compiled_code_asset_selection_status = \"verified\"",
        "Vulkan compiled code asset selection",
    );
    if matches!(
        sample.registration_id,
        "official.shader.vulkan-add-xor-pair-u32"
            | "official.shader.vulkan-add-xor-pair-padded-u32"
            | "official.shader.vulkan-add-xor-pair-reduced-u32"
            | "official.shader.vulkan-reduced-output-fan-out-u32"
            | "official.shader.vulkan-cuda-reduced-output-fan-out-u32"
    ) {
        assert_file_contains(
            &provider_output,
            "native_output_0_output_binding_count = \"2\"",
            "Vulkan fan-out output binding count",
        );
        assert_file_contains(
            &provider_output,
            "native_output_0_output_binding_buffers = \"output.values,output.xor\"",
            "Vulkan fan-out output buffers",
        );
        assert_file_contains(
            &provider_output,
            "native_output_0_comparison_collection_statuses = \"comparison-passed,comparison-passed\"",
            "Vulkan fan-out comparison collection",
        );
        assert_file_contains(
            &provider_output,
            &format!(
                "native_output_0_worker_additional_output_hashes = \"{}\"",
                match sample.registration_id {
                    "official.shader.vulkan-add-xor-pair-padded-u32" => "0x9adad3c97291d1e8",
                    "official.shader.vulkan-add-xor-pair-reduced-u32"
                    | "official.shader.vulkan-reduced-output-fan-out-u32"
                    | "official.shader.vulkan-cuda-reduced-output-fan-out-u32" => {
                        "0x279d73758e81abdd"
                    }
                    _ => "0x73bb5b39fe3ab738",
                }
            ),
            "Vulkan fan-out secondary output hash",
        );
    }
    if let Some(graph) = &sample.graph {
        assert_file_contains(
            &provider_output,
            &format!("native_output_count = \"{}\"", graph.native_output_count),
            "Vulkan graph native output count",
        );
        assert_file_contains(
            &provider_output,
            &format!(
                "native_output_{}_request_id = \"{}\"",
                graph.final_output_index, graph.final_request_id
            ),
            "Vulkan graph final request",
        );
        assert_file_contains(
            &provider_output,
            &format!(
                "native_output_{}_hash = \"{}\"",
                graph.final_output_index, graph.final_expected_hash
            ),
            "Vulkan graph final output hash",
        );
        assert_file_contains(
            &provider_output,
            &format!(
                "provider_edge_transport_receipt_count = \"{}\"",
                graph.edges.len()
            ),
            "Vulkan graph transport receipt",
        );
        for (receipt_index, edge) in graph.edges.iter().enumerate() {
            assert_file_contains(
                &provider_output,
                edge.transport_token,
                "Vulkan graph GLM transport token",
            );
            assert!(provider_samples.contains(&format!(
                "provider_request_{}_dependency_0_producer_output_buffer={}",
                edge.consumer_request_index, edge.dependency_output_buffer
            )));
            assert!(provider_samples.contains(&format!(
                "provider_request_{}_input_binding_0_shape={}",
                edge.consumer_request_index, edge.dependency_input_shape
            )));
            assert!(provider_samples.contains(&format!(
                "provider_request_{}_input_binding_0_byte_length={}",
                edge.consumer_request_index, edge.dependency_byte_length
            )));
            assert_file_contains(
                &provider_output,
                &format!(
                    "provider_edge_transport_receipt_{receipt_index}_byte_length = \"{}\"",
                    edge.dependency_byte_length
                ),
                "Vulkan graph transport byte length",
            );
            assert_file_contains(
                &provider_output,
                &format!(
                    "native_output_{}_output_binding_shapes = \"{}\"",
                    edge.consumer_request_index, edge.dependency_input_shape
                ),
                "Vulkan graph typed consumer output shape",
            );
        }
        assert_file_contains(
            &provider_output,
            &format!(
                "compiled_code_asset_selection_count = \"{}\"",
                graph.compiled_selection_count
            ),
            "Vulkan graph code asset selection set",
        );
        if sample.registration_id == "official.shader.vulkan-cuda-reduced-output-fan-out-u32" {
            assert!(provider_samples
                .contains("provider_request_1_adapter_binding_provider_family=cuda:nvidia-gpu"));
            assert!(provider_samples
                .contains("provider_request_2_adapter_binding_provider_family=spirv:vulkan-gpu"));
            assert_file_contains(
                &provider_output,
                "native_output_1_execution_contract = \"nuis-cuda-ptx-driver-provider-execution-v1\"",
                "cross-provider CUDA consumer execution",
            );
            assert_file_contains(
                &provider_output,
                "native_output_1_device = \"cuda:nvidia-gpu:ordinal-",
                "cross-provider CUDA selected device",
            );
            assert_file_contains(
                &provider_output,
                "native_output_2_execution_contract = \"nuis-vulkan-spirv-provider-execution-v1\"",
                "cross-provider Vulkan consumer execution",
            );
            assert_file_contains(
                &provider_output,
                "provider_edge_transport_receipt_0_staging_adapter_id = \"provider.output.transfer.v1\"",
                "cross-provider transferable staging adapter",
            );
        }
    }

    let materialized = run_nsdb(&[
        "materialize-provider-samples",
        &output_dir_text,
        "--provider-family",
        "spirv:vulkan-gpu",
        "--json",
    ]);
    assert_success(
        &materialized,
        "materialize Linux Vulkan completion evidence",
    );
    let materialized_text = output_text(&materialized);
    assert!(materialized_text.contains("\"status\":\"ready\""));
    assert!(materialized_text.contains("\"materialized_record_count\":1"));
    assert!(
        materialized_text.contains("\"first_provider_output_payload_attach_status\":\"attached\"")
    );
    let materialized_provider_samples =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();
    let expected_dispatch_count =
        if sample.registration_id == "official.shader.vulkan-cuda-reduced-output-fan-out-u32" {
            2
        } else {
            1
        };
    assert_eq!(
        materialized_provider_samples
            .matches("[[provider_dispatch]]")
            .count(),
        expected_dispatch_count
    );
    assert!(materialized_provider_samples.contains(&format!(
        "selected_provider_bundle_count = {expected_dispatch_count}"
    )));
    if expected_dispatch_count == 2 {
        assert!(materialized_provider_samples
            .contains("provider_bundle_id = \"cuda.nvidia-gpu.bundle.v1\""));
        assert!(materialized_provider_samples
            .contains("runner_adapter_id = \"cuda.nvidia-gpu.real-device\""));
    }

    let frontdoor = super::final_image::assemble_provider_complete_final_image(
        &project_text,
        &output_dir,
        1,
        expected_dispatch_count,
    );
    if expected_dispatch_count == 2 {
        let replay = nsdb::payload_execution_replay_summary(&output_dir);
        let completion = &replay.provider_completions[0];
        assert_eq!(completion.request_completion_count, 3);
        let handoff =
            fs::read_to_string(output_dir.join("nuis.nsdb.payload-execution-handoff.toml"))
                .unwrap();
        for (index, request_id, family, dispatch_id) in [
            (
                0,
                "shader.vulkan-cuda-fan-out.add-xor-pair-u32",
                "spirv:vulkan-gpu",
                "dispatch0000",
            ),
            (
                1,
                "kernel.cuda.fan-out.copy-sum-u32",
                "cuda:nvidia-gpu",
                "dispatch0001",
            ),
            (
                2,
                "shader.vulkan-cuda-fan-out.xor-reduced-u32",
                "spirv:vulkan-gpu",
                "dispatch0000",
            ),
        ] {
            assert!(handoff.contains(&format!(
                "request_completion_{index}_request_id = \"{request_id}\""
            )));
            assert!(handoff.contains(&format!(
                "request_completion_{index}_provider_family = \"{family}\""
            )));
            assert!(handoff.contains(&format!(
                "request_completion_{index}_dispatch_id = \"{dispatch_id}\""
            )));
            assert!(handoff.contains(&format!(
                "request_completion_{index}_selected_set_hash = \"{}\"",
                completion.dispatch_selected_set_hash
            )));
            assert!(frontdoor.contains(&format!("\"request_id\":\"{request_id}\"")));
            assert!(frontdoor.contains(&format!("\"provider_family\":\"{family}\"")));
            assert!(frontdoor.contains(&format!("\"dispatch_id\":\"{dispatch_id}\"")));
        }
        assert!(frontdoor.contains(
            "\"nsld_final_executable_output_object_package_provider_request_completion_receipt_count\":3"
        ));
        assert!(frontdoor.contains(
            "\"closure_summary_debugger_api_provider_request_completion_receipt_count\":3"
        ));

        let request_id = "kernel.cuda.fan-out.copy-sum-u32";
        let request_cursor = output_dir.join("nuis.nsdb.request-replay-cursor.toml");
        let request_cursor_text = request_cursor.display().to_string();
        let request_replay = run_nsdb(&[
            "replay",
            &output_dir_text,
            "--request-id",
            request_id,
            "--json",
        ]);
        assert_success(&request_replay, "Nsdb CUDA request-addressable replay");
        let request_replay_text = output_text(&request_replay);
        for expected in [
            "\"debugger_transcript_request_selector_contract\":\"nsdb-provider-request-replay-selector-v1\"",
            "\"debugger_transcript_request_selector_status\":\"request-resolved\"",
            "\"debugger_transcript_request_selector_ready\":true",
            "\"debugger_transcript_request_id\":\"kernel.cuda.fan-out.copy-sum-u32\"",
            "\"debugger_transcript_request_provider_family\":\"cuda:nvidia-gpu\"",
            "\"debugger_transcript_request_dispatch_id\":\"dispatch0001\"",
            "\"debugger_transcript_request_frame_count\":3",
            "\"debugger_transcript_control_mode\":\"request\"",
            "\"debugger_transcript_control_status\":\"request-selected\"",
            "\"debugger_transcript_stop_reason\":\"request-selected\"",
        ] {
            assert!(
                request_replay_text.contains(expected),
                "request replay should contain `{expected}`\n{request_replay_text}"
            );
        }
        assert!(request_replay_text.contains(&format!(
            "\"debugger_transcript_request_selected_set_hash\":\"{}\"",
            completion.dispatch_selected_set_hash
        )));
        for field in [
            "debugger_transcript_request_frame_id",
            "debugger_transcript_request_completion_clock",
            "debugger_transcript_request_output_hash",
            "debugger_transcript_request_completion_token",
            "debugger_transcript_selected_frame_id",
        ] {
            assert!(
                request_replay_text.contains(&format!("\"{field}\":\"")),
                "request replay should materialize `{field}`\n{request_replay_text}"
            );
        }
        assert_eq!(
            request_replay_text
                .matches("\"frame_scope\":\"provider-request\"")
                .count(),
            3
        );
        let nuis_request = run_nuis(&[
            "debug-request",
            &output_dir_text,
            "--request-id",
            request_id,
            "--save-cursor",
            &request_cursor_text,
            "--json",
        ]);
        assert_success(&nuis_request, "Nuis CUDA request debugger frontdoor");
        let nuis_request_text = output_text(&nuis_request);
        assert!(
            nuis_request_text
                .contains("\"debugger_transcript_request_selector_status\":\"request-resolved\"")
                && nuis_request_text.contains(
                    "\"debugger_transcript_request_provider_family\":\"cuda:nvidia-gpu\""
                )
                && nuis_request_text
                    .contains("\"debugger_transcript_control_status\":\"request-selected\""),
            "Nuis should preserve the typed Nsdb request selection\n{nuis_request_text}"
        );
        let cuda_frame =
            json_string_values(&nuis_request_text, "debugger_transcript_request_frame_id")
                .into_iter()
                .next()
                .expect("Nuis CUDA request replay frame ID");
        let final_frame = json_string_values(
            &nuis_request_text,
            "debugger_transcript_resume_next_frame_id",
        )
        .into_iter()
        .next()
        .expect("Nuis final Vulkan successor frame ID");
        assert!(cuda_frame.starts_with("frame:request:"));
        assert!(final_frame.starts_with("frame:request:"));
        assert_ne!(cuda_frame, final_frame);
        assert_file_contains(
            &request_cursor,
            &format!("after_frame_id = \"{cuda_frame}\""),
            "Nuis request cursor CUDA stop",
        );
        assert_file_contains(
            &request_cursor,
            &format!("next_frame_id = \"{final_frame}\""),
            "Nuis request cursor final Vulkan successor",
        );

        let cursor_report = run_nuis(&["build-report", "--json", &output_dir_text]);
        assert_success(&cursor_report, "Nuis selected request cursor projection");
        let cursor_report_text = output_text(&cursor_report);
        assert!(
            cursor_report_text.contains(
                "\"nsld_final_executable_output_debugger_cursor_status\":\"cursor-resume-ready\""
            ) && cursor_report_text
                .contains("\"closure_summary_debugger_cursor_status\":\"cursor-resume-ready\"")
                && cursor_report_text.contains("nuis.nsdb.request-replay-cursor.toml"),
            "Nuis build-report should follow the selected request cursor\n{cursor_report_text}"
        );

        let resumed_request = run_nuis(&[
            "debug-resume",
            &output_dir_text,
            "--break-at",
            &final_frame,
            "--json",
        ]);
        assert_success(&resumed_request, "Nuis request-frame cursor resume");
        let resumed_request_text = output_text(&resumed_request);
        assert!(
            resumed_request_text
                .contains("\"debugger_transcript_resume_input_status\":\"cursor-accepted\"")
                && resumed_request_text.contains(&format!(
                    "\"debugger_transcript_selected_frame_id\":\"{final_frame}\""
                ))
                && resumed_request_text
                    .contains("\"debugger_transcript_replayed_checkpoint_count\":0")
                && resumed_request_text.contains("\"debugger_transcript_replayed_frame_count\":1"),
            "Nuis should resume only the final Vulkan request frame\n{resumed_request_text}"
        );

        let missing = run_nsdb(&[
            "replay",
            &output_dir_text,
            "--request-id",
            "missing.request",
            "--json",
        ]);
        assert_success(&missing, "Nsdb unknown request fail-closed replay");
        let missing_text = output_text(&missing);
        assert!(
            missing_text
                .contains("\"debugger_transcript_request_selector_status\":\"request-not-found\"")
                && missing_text.contains("\"debugger_transcript_request_selector_ready\":false")
                && missing_text
                    .contains("\"debugger_transcript_control_status\":\"request-not-found\"")
                && missing_text
                    .contains("request-replay-selector:request-not-found:missing.request"),
            "unknown requests must not fall back to whole-transcript replay\n{missing_text}"
        );
    }
    assert!(
        frontdoor.contains("\"nsld_final_executable_output_nsdb_first_provider_family\":\"spirv:vulkan-gpu\"")
            && frontdoor.contains("\"closure_summary_first_provider_family\":\"spirv:vulkan-gpu\"")
            && frontdoor.contains(
                "\"nsld_final_executable_output_object_package_first_provider_bundle_id\":\"spirv.vulkan-gpu.bundle.v1\""
            )
            && frontdoor.contains(
                "\"closure_summary_object_package_first_provider_bundle_id\":\"spirv.vulkan-gpu.bundle.v1\""
            )
            && frontdoor.contains(
                "\"nsld_final_executable_output_nsdb_first_provider_output_contract\":\"nuis-provider-output-payload-handoff-v1\""
            )
            && frontdoor.contains(
                "\"nsld_final_executable_output_object_package_provider_dispatch_identity_status\":\"verified\""
            )
            && frontdoor.contains(
                "\"nsld_final_executable_output_debugger_api_provider_dispatch_identity_status\":\"verified\""
            ),
        "Vulkan sealed final image should project through Nuis object-package/debugger frontdoors without a cursor lineage prerequisite\n{frontdoor}"
    );

    fs::remove_dir_all(output_dir).unwrap();
}
