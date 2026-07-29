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

#[test]
fn linux_vulkan_shader_sample_executes_provider_output() {
    if !vulkan_host_available() {
        return;
    }
    let output_dir = temp_dir("linux_vulkan_shader_provider");
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/shader_vulkan_provider_demo");
    let project_text = project.display().to_string();
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&["build", &project_text, &output_dir_text]);
    assert_success(&build, "build Linux Vulkan shader provider project");
    assert_file_contains(
        &output_dir.join("nuis.domain.code-asset-contributions.toml"),
        "shader.vulkan.copy-u32.spirv",
        "Vulkan code asset contribution table",
    );
    assert_file_contains(
        &output_dir.join("nuis.domain.code-asset-contributions.toml"),
        "vulkan.discrete-or-integrated-gpu",
        "Vulkan lowering target contribution",
    );
    let spirv = fs::read(output_dir.join("nuis.shader.vulkan.copy-u32.spv"))
        .expect("Vulkan SPIR-V code asset");
    assert!(spirv
        .windows(b"nuis_vulkan_copy_u32".len())
        .any(|window| window == b"nuis_vulkan_copy_u32"));

    let run = run_nuis(&["run-artifact", &output_dir_text, "--json"]);
    assert_success(&run, "materialize Linux Vulkan provider request");
    let provider_samples =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml"))
            .expect("provider sample manifest");
    assert!(provider_samples.contains("provider_family = \"spirv:vulkan-gpu\""));
    assert!(provider_samples.contains("provider_adapter_binding_provider_family=spirv:vulkan-gpu"));
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
    assert!(executed_text
        .contains("\"first_output_payload_native_output_hash\":\"0x6ebcdd244b594ee4\""));

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
