use super::*;

fn output_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn shader_metal_generated_msl_sample_executes_provider_output() {
    let output_dir = temp_dir("shader_metal_generated_msl_provider");
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/shader_metal_provider_demo");
    let project_text = project.display().to_string();
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&["build", &project_text, &output_dir_text]);
    assert_success(&build, "build generated Metal shader provider project");
    assert_file_contains(
        &output_dir.join("nuis.domain.code-asset-contributions.toml"),
        "shader.metal.copy-u32.msl",
        "Metal code asset contribution table",
    );
    assert_file_contains(
        &output_dir.join("nuis.domain.code-asset-contributions.toml"),
        "target = \"msl2.4\"",
        "Metal native IR contribution target",
    );
    let msl = fs::read_to_string(output_dir.join("nuis.shader.metal.copy-u32.metal"))
        .expect("generated Metal MSL code asset");
    assert!(msl.contains("nuis-module-lowering-plan"));
    assert!(msl.contains("msl:metal-gpu"));
    assert!(msl.contains("kernel void nuis_metal_copy_u32"));

    let run = run_nuis(&["run-artifact", &output_dir_text, "--json"]);
    assert_success(&run, "materialize generated Metal provider request");
    let provider_samples =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml"))
            .expect("provider sample manifest");
    assert!(provider_samples.contains("provider_family = \"metal:apple-silicon-gpu\""));
    assert!(provider_samples.contains("provider_sample_registration_package=official.shader"));
    assert!(
        provider_samples.contains("provider_sample_registration_id=official.shader.metal-copy-u32")
    );
    assert!(provider_samples.contains("provider_code_asset_id=shader.metal.copy-u32.msl"));
    assert!(provider_samples.contains("provider_code_asset_target=msl2.4"));
    assert!(provider_samples
        .contains("provider_adapter_binding_provider_family=metal:apple-silicon-gpu"));
    assert!(provider_samples.contains(
        "provider_code_asset_contribution_selection_set_contract=nuis-provider-code-asset-contribution-selection-set-v1"
    ));

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
    assert!(executed_text.contains(
        "\"first_output_payload_native_execution_contract\":\"nuis-metal-u32-copy-provider-runner-v1\""
    ));
    assert!(executed_text
        .contains("\"first_output_payload_native_output_kind\":\"provider-tensor-u32\""));
    assert!(
        executed_text.contains("\"first_output_payload_comparison_status\":\"comparison-passed\"")
    );
    assert!(executed_text
        .contains("\"first_output_payload_native_output_hash\":\"0x6ebcdd244b594ee4\""));

    let provider_output = output_dir.join("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml");
    assert_file_contains(
        &provider_output,
        "native_output_0_execution_contract = \"nuis-metal-u32-copy-provider-runner-v1\"",
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
    assert_file_contains(
        &provider_output,
        "compiled_code_asset_asset_id = \"shader.metal.copy-u32.msl\"",
        "generated Metal compiled code asset identity",
    );

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

    let frontdoor =
        super::final_image::assemble_provider_complete_final_image(&project_text, &output_dir, 1);
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
