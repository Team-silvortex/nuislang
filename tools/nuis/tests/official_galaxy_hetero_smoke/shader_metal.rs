use super::*;

struct MetalSampleSmoke<'a> {
    project_name: &'a str,
    asset_id: &'a str,
    generated_file: &'a str,
    registration_id: &'a str,
    operation: &'a str,
    entry: &'a str,
    execution_contract: &'a str,
    expected_hash: &'a str,
}

fn output_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn shader_metal_generated_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_provider_demo",
        asset_id: "shader.metal.copy-u32.msl",
        generated_file: "nuis.shader.metal.copy-u32.metal",
        registration_id: "official.shader.metal-copy-u32",
        operation: "copy-u32",
        entry: "nuis_metal_copy_u32",
        execution_contract: "nuis-metal-u32-copy-provider-runner-v1",
        expected_hash: "0x6ebcdd244b594ee4",
    });
}

#[test]
fn shader_metal_generated_add_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_add_provider_demo",
        asset_id: "shader.metal.add-u32.msl",
        generated_file: "nuis.shader.metal.add-u32.metal",
        registration_id: "official.shader.metal-add-u32",
        operation: "add-u32",
        entry: "nuis_metal_add_u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0xdce2c1aca0f32707",
    });
}

#[test]
fn shader_metal_generated_sub_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_sub_provider_demo",
        asset_id: "shader.metal.sub-u32.msl",
        generated_file: "nuis.shader.metal.sub-u32.metal",
        registration_id: "official.shader.metal-sub-u32",
        operation: "sub-u32",
        entry: "nuis_metal_sub_u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0x88201fb960ff6465",
    });
}

#[test]
fn shader_metal_generated_mul_msl_sample_executes_provider_output() {
    run_metal_sample_smoke(MetalSampleSmoke {
        project_name: "shader_metal_mul_provider_demo",
        asset_id: "shader.metal.mul-u32.msl",
        generated_file: "nuis.shader.metal.mul-u32.metal",
        registration_id: "official.shader.metal-mul-u32",
        operation: "mul-u32",
        entry: "nuis_metal_mul_u32",
        execution_contract: "nuis-metal-u32-canonical-provider-runner-v1",
        expected_hash: "0x02f6f7f591bff68f",
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
    assert_file_contains(
        &contribution_table,
        sample.asset_id,
        "Metal code asset contribution table",
    );
    assert_file_contains(
        &contribution_table,
        "target = \"msl2.4\"",
        "Metal native IR contribution target",
    );
    let msl = fs::read_to_string(output_dir.join(sample.generated_file))
        .expect("generated Metal MSL code asset");
    assert!(msl.contains("nuis-module-lowering-plan"));
    assert!(msl.contains("msl:metal-gpu"));
    assert!(msl.contains(&format!("kernel void {}(", sample.entry)));

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
    assert!(provider_samples.contains(&format!("provider_kernel_operation={}", sample.operation)));
    assert!(provider_samples.contains(&format!("provider_code_asset_id={}", sample.asset_id)));
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
    assert_file_contains(
        &provider_output,
        &format!("compiled_code_asset_asset_id = \"{}\"", sample.asset_id),
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
