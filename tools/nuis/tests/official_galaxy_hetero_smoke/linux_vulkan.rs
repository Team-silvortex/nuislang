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
    final_request_id: &'a str,
    final_expected_hash: &'a str,
    transport_token: &'a str,
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
            final_request_id: "shader.vulkan.chain.xor-u32",
            final_expected_hash: "0x88201fb960ff6465",
            transport_token:
                "glm:provider-edge:shader.vulkan.chain.add-u32:output.values->shader.vulkan.chain.xor-u32:input.values",
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
    if let Some(graph) = &sample.graph {
        assert_file_contains(
            &provider_output,
            "native_output_count = \"2\"",
            "Vulkan graph native output count",
        );
        assert_file_contains(
            &provider_output,
            &format!(
                "native_output_1_request_id = \"{}\"",
                graph.final_request_id
            ),
            "Vulkan graph final request",
        );
        assert_file_contains(
            &provider_output,
            &format!("native_output_1_hash = \"{}\"", graph.final_expected_hash),
            "Vulkan graph final output hash",
        );
        assert_file_contains(
            &provider_output,
            "provider_edge_transport_receipt_count = \"1\"",
            "Vulkan graph transport receipt",
        );
        assert_file_contains(
            &provider_output,
            graph.transport_token,
            "Vulkan graph GLM transport token",
        );
        assert_file_contains(
            &provider_output,
            "compiled_code_asset_selection_count = \"2\"",
            "Vulkan graph code asset selection set",
        );
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

    let frontdoor =
        super::final_image::assemble_provider_complete_final_image(&project_text, &output_dir, 1);
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
