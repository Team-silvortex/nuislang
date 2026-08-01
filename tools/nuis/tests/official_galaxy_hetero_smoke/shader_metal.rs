use super::*;

struct MetalSampleSmoke<'a> {
    project_name: &'a str,
    assets: &'a [MetalGeneratedAsset<'a>],
    registration_id: &'a str,
    execution_contract: &'a str,
    expected_hash: &'a str,
    graph: Option<MetalGraphSmoke<'a>>,
}

struct MetalGeneratedAsset<'a> {
    asset_id: &'a str,
    generated_file: &'a str,
    operation: &'a str,
    entry: &'a str,
}

struct MetalGraphSmoke<'a> {
    final_request_id: &'a str,
    final_expected_hash: &'a str,
    transport_token: &'a str,
}

fn output_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn shader_metal_generated_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_provider_demo",
        assets: &[MetalGeneratedAsset {
            asset_id: "shader.metal.copy-u32.msl",
            generated_file: "nuis.shader.metal.copy-u32.metal",
            operation: "copy-u32",
            entry: "nuis_metal_copy_u32",
        }],
        registration_id: "official.shader.metal-copy-u32",
        execution_contract: "nuis-metal-u32-copy-provider-runner-v1",
        expected_hash: "0x6ebcdd244b594ee4",
        graph: None,
    });
}

#[test]
fn shader_metal_generated_add_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_add_provider_demo",
        assets: &[MetalGeneratedAsset {
            asset_id: "shader.metal.add-u32.msl",
            generated_file: "nuis.shader.metal.add-u32.metal",
            operation: "add-u32",
            entry: "nuis_metal_add_u32",
        }],
        registration_id: "official.shader.metal-add-u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0xdce2c1aca0f32707",
        graph: None,
    });
}

#[test]
fn shader_metal_generated_add_pair_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_add_pair_provider_demo",
        assets: &[MetalGeneratedAsset {
            asset_id: "shader.metal.add-pair-u32.msl",
            generated_file: "nuis.shader.metal.add-pair-u32.metal",
            operation: "add-pair-u32",
            entry: "nuis_metal_add_pair_u32",
        }],
        registration_id: "official.shader.metal-add-pair-u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0xbada6f73928b9f42",
        graph: None,
    });
}

#[test]
fn shader_metal_generated_fan_out_msl_writes_two_provider_outputs() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_fan_out_provider_demo",
        assets: &[MetalGeneratedAsset {
            asset_id: "shader.metal.add-xor-pair-u32.msl",
            generated_file: "nuis.shader.metal.add-xor-pair-u32.metal",
            operation: "add-xor-pair-u32",
            entry: "nuis_metal_add_xor_pair_u32",
        }],
        registration_id: "official.shader.metal-add-xor-pair-u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0xbada6f73928b9f42",
        graph: None,
    });
}

#[test]
fn shader_metal_generated_sub_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_sub_provider_demo",
        assets: &[MetalGeneratedAsset {
            asset_id: "shader.metal.sub-u32.msl",
            generated_file: "nuis.shader.metal.sub-u32.metal",
            operation: "sub-u32",
            entry: "nuis_metal_sub_u32",
        }],
        registration_id: "official.shader.metal-sub-u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0x88201fb960ff6465",
        graph: None,
    });
}

#[test]
fn shader_metal_generated_mul_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_mul_provider_demo",
        assets: &[MetalGeneratedAsset {
            asset_id: "shader.metal.mul-u32.msl",
            generated_file: "nuis.shader.metal.mul-u32.metal",
            operation: "mul-u32",
            entry: "nuis_metal_mul_u32",
        }],
        registration_id: "official.shader.metal-mul-u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0x02f6f7f591bff68f",
        graph: None,
    });
}

#[test]
fn shader_metal_generated_xor_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_xor_provider_demo",
        assets: &[MetalGeneratedAsset {
            asset_id: "shader.metal.xor-u32.msl",
            generated_file: "nuis.shader.metal.xor-u32.metal",
            operation: "xor-u32",
            entry: "nuis_metal_xor_u32",
        }],
        registration_id: "official.shader.metal-xor-u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0x88201fb960ff6465",
        graph: None,
    });
}

#[test]
fn shader_metal_generated_chain_msl_sample_executes_provider_graph() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_chain_provider_demo",
        assets: &[
            MetalGeneratedAsset {
                asset_id: "shader.metal.add-u32.msl",
                generated_file: "nuis.shader.metal.add-u32.metal",
                operation: "add-u32",
                entry: "nuis_metal_add_u32",
            },
            MetalGeneratedAsset {
                asset_id: "shader.metal.xor-u32.msl",
                generated_file: "nuis.shader.metal.xor-u32.metal",
                operation: "xor-u32",
                entry: "nuis_metal_xor_u32",
            },
        ],
        registration_id: "official.shader.metal-u32-chain",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0xdce2c1aca0f32707",
        graph: Some(MetalGraphSmoke {
            final_request_id: "shader.metal.chain.xor-u32",
            final_expected_hash: "0x88201fb960ff6465",
            transport_token:
                "glm:provider-edge:shader.metal.chain.add-u32:output.values->shader.metal.chain.xor-u32:input.values",
        }),
    });
}

fn run_metal_sample_smoke(sample: MetalSampleSmoke<'_>) {
    let output_dir = temp_dir(sample.project_name);
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains")
        .join(sample.project_name);
    let project_text = project.display().to_string();
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&["build", &project_text, &output_dir_text]);
    assert_success(&build, "build generated Metal shader provider project");
    let contribution_table = output_dir.join("nuis.domain.code-asset-contributions.toml");
    for asset in sample.assets {
        assert_file_contains(
            &contribution_table,
            asset.asset_id,
            "Metal code asset contribution table",
        );
    }
    assert_file_contains(
        &contribution_table,
        "target = \"msl2.4\"",
        "Metal native IR contribution target",
    );
    for asset in sample.assets {
        let msl = fs::read_to_string(output_dir.join(asset.generated_file))
            .expect("generated Metal MSL code asset");
        assert!(msl.contains("nuis-module-lowering-plan"));
        assert!(msl.contains("msl:metal-gpu"));
        assert!(msl.contains(&format!("kernel void {}(", asset.entry)));
    }

    let run = run_nuis(&["run-artifact", &output_dir_text, "--json"]);
    assert_success(&run, "materialize generated Metal provider request");
    let provider_samples =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml"))
            .expect("provider sample manifest");
    assert!(provider_samples.contains("provider_family = \"metal:apple-silicon-gpu\""));
    assert!(provider_samples.contains("provider_sample_registration_package=official.shader"));
    assert!(provider_samples.contains(&format!(
        "provider_sample_registration_id={}",
        sample.registration_id
    )));
    for asset in sample.assets {
        assert!(provider_samples.contains(&format!("kernel_operation={}", asset.operation)));
        assert!(provider_samples.contains(&format!("code_asset_id={}", asset.asset_id)));
    }
    assert!(provider_samples.contains("code_asset_target=msl2.4"));
    assert!(provider_samples.contains("adapter_binding_provider_family=metal:apple-silicon-gpu"));
    assert!(provider_samples.contains(
        "provider_code_asset_contribution_selection_set_contract=nuis-provider-code-asset-contribution-selection-set-v1"
    ));
    if matches!(
        sample.registration_id,
        "official.shader.metal-add-pair-u32" | "official.shader.metal-add-xor-pair-u32"
    ) {
        assert!(provider_samples.contains("input_binding_contract=nuis-provider-input-binding-v2"));
        assert!(provider_samples.contains("buffer_layout=tensor-row-major"));
        assert!(provider_samples.contains("buffer_shape=2x2"));
        assert!(provider_samples.contains("input_binding_1_layout=tensor-row-major"));
        assert!(provider_samples.contains("input_binding_1_row_stride_bytes=8"));
    }

    let executed = run_nsdb(&[
        "execute-provider-samples",
        &output_dir_text,
        "--provider-family",
        "metal:apple-silicon-gpu",
        "--json",
    ]);
    assert_success(&executed, "execute generated Metal provider request");
    let executed_text = output_text(&executed);
    assert!(executed_text.contains("\"status\":\"provider-output-payloads-ready\""));
    assert!(executed_text.contains(&format!(
        "\"first_output_payload_native_execution_contract\":\"{}\"",
        sample.execution_contract
    )));
    assert!(executed_text
        .contains("\"first_output_payload_native_output_kind\":\"provider-tensor-u32\""));
    assert!(
        executed_text.contains("\"first_output_payload_comparison_status\":\"comparison-passed\"")
    );
    assert!(executed_text.contains(&format!(
        "\"first_output_payload_native_output_hash\":\"{}\"",
        sample.expected_hash
    )));

    let provider_output = output_dir.join("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml");
    assert_file_contains(
        &provider_output,
        &format!(
            "native_output_0_execution_contract = \"{}\"",
            sample.execution_contract
        ),
        "generated Metal provider output payload",
    );
    assert_file_contains(
        &provider_output,
        "native_output_0_kind = \"provider-tensor-u32\"",
        "generated Metal native output kind",
    );
    assert_file_contains(
        &provider_output,
        "compiled_code_asset_selection_status = \"verified\"",
        "generated Metal compiled code asset selection",
    );
    if let Some(first_asset) = sample.assets.first() {
        assert_file_contains(
            &provider_output,
            &format!(
                "compiled_code_asset_asset_id = \"{}\"",
                first_asset.asset_id
            ),
            "generated Metal compiled code asset identity",
        );
    }
    if let Some(graph) = &sample.graph {
        assert_file_contains(
            &provider_output,
            "native_output_count = \"2\"",
            "Metal graph native output count",
        );
        assert_file_contains(
            &provider_output,
            &format!(
                "native_output_1_request_id = \"{}\"",
                graph.final_request_id
            ),
            "Metal graph final request",
        );
        assert_file_contains(
            &provider_output,
            &format!("native_output_1_hash = \"{}\"", graph.final_expected_hash),
            "Metal graph final output hash",
        );
        assert_file_contains(
            &provider_output,
            "provider_edge_transport_receipt_count = \"1\"",
            "Metal graph transport receipt",
        );
        assert_file_contains(
            &provider_output,
            graph.transport_token,
            "Metal graph GLM transport token",
        );
        assert_file_contains(
            &provider_output,
            "compiled_code_asset_selection_count = \"2\"",
            "Metal graph code asset selection set",
        );
    }
    if sample.registration_id == "official.shader.metal-add-xor-pair-u32" {
        assert_file_contains(
            &provider_output,
            "native_output_0_output_binding_count = \"2\"",
            "Metal fan-out output binding count",
        );
        assert_file_contains(
            &provider_output,
            "native_output_0_output_binding_buffers = \"output.values,output.xor\"",
            "Metal fan-out output buffers",
        );
        assert_file_contains(
            &provider_output,
            "native_output_0_comparison_collection_statuses = \"comparison-passed,comparison-passed\"",
            "Metal fan-out comparison collection",
        );
        assert_file_contains(
            &provider_output,
            "native_output_0_worker_additional_output_hashes = \"0x73bb5b39fe3ab738\"",
            "Metal fan-out secondary output hash",
        );
    }

    let materialized = run_nsdb(&[
        "materialize-provider-samples",
        &output_dir_text,
        "--provider-family",
        "metal:apple-silicon-gpu",
        "--json",
    ]);
    assert_success(
        &materialized,
        "materialize generated Metal completion evidence",
    );
    let materialized_text = output_text(&materialized);
    assert!(materialized_text.contains("\"status\":\"ready\""));
    assert!(materialized_text.contains("\"materialized_record_count\":1"));
    assert!(
        materialized_text.contains("\"first_provider_output_payload_attach_status\":\"attached\"")
    );

    let frontdoor = super::final_image::assemble_provider_complete_final_image(
        &project_text,
        &output_dir,
        1,
        1,
    );
    assert!(
        frontdoor.contains("\"nsld_final_executable_output_nsdb_first_provider_family\":\"metal:apple-silicon-gpu\"")
            && frontdoor.contains("\"closure_summary_first_provider_family\":\"metal:apple-silicon-gpu\"")
            && frontdoor.contains(
                "\"nsld_final_executable_output_nsdb_first_provider_output_contract\":\"nuis-provider-output-payload-handoff-v1\""
            )
            && frontdoor.contains(
                "\"nsld_final_executable_output_object_package_provider_dispatch_identity_status\":\"verified\""
            )
            && frontdoor.contains(
                "\"nsld_final_executable_output_debugger_api_provider_dispatch_identity_status\":\"verified\""
            ),
        "generated Metal sealed final image should project through provider-neutral frontdoors\n{frontdoor}"
    );

    fs::remove_dir_all(output_dir).unwrap();
}
