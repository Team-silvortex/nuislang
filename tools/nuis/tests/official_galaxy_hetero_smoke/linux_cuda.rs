use super::*;

fn cuda_host_available() -> bool {
    Command::new("nvidia-smi")
        .arg("-L")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn output_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn sealed_linux_cuda_image_executes_and_replays_completion() {
    if !cuda_host_available() {
        return;
    }
    let output_dir = temp_dir("linux_cuda_final_image");
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/kernel_cuda_provider_demo");
    let project_text = project.display().to_string();
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&["build", &project_text, &output_dir_text]);
    assert_success(&build, "build Linux CUDA provider project");
    let run = run_nuis(&["run-artifact", &output_dir_text, "--json"]);
    assert_success(&run, "materialize Linux CUDA provider request");

    let pre_seal = run_nsdb(&[
        "execute-provider-samples",
        &output_dir_text,
        "--provider-family",
        "cuda:nvidia-gpu",
        "--json",
    ]);
    assert_success(&pre_seal, "execute pre-seal CUDA provider request");
    let pre_seal_text = output_text(&pre_seal);
    assert!(pre_seal_text
        .contains("\"final_image_dispatch_authority_status\":\"pre-seal-acquisition\""));
    assert!(pre_seal_text
        .contains("\"first_output_payload_native_output_hash\":\"0xdc51cb8047a381e1\""));

    let materialize = run_nsdb(&[
        "materialize-provider-samples",
        &output_dir_text,
        "--provider-family",
        "cuda:nvidia-gpu",
        "--json",
    ]);
    assert_success(&materialize, "materialize pre-seal CUDA completion");

    let rebuild = run_nuis(&["build", &project_text, &output_dir_text]);
    assert_success(&rebuild, "rebuild self-contained CUDA image");
    let seal = run_nsld(&["seal", &output_dir_text, "--json"]);
    assert_success(&seal, "seal Linux CUDA final image");
    let seal_text = output_text(&seal);
    assert!(seal_text.contains("\"completed\":true"));
    assert!(seal_text.contains("\"loader_provider_dispatch_status\":\"verified\""));

    let object = fs::read(output_dir.join("nuis.nsld.elf")).expect("Nsld ELF object");
    assert_eq!(&object[..4], b"\x7fELF");
    assert_eq!(object[4], 2, "ELF64 class");
    assert_eq!(
        u16::from_le_bytes(object[18..20].try_into().unwrap()),
        62,
        "x86-64 machine"
    );

    let post_seal = run_nsdb(&[
        "execute-provider-samples",
        &output_dir_text,
        "--provider-family",
        "cuda:nvidia-gpu",
        "--json",
    ]);
    assert_success(&post_seal, "execute sealed CUDA provider request");
    let post_seal_text = output_text(&post_seal);
    assert!(post_seal_text.contains("\"final_image_dispatch_authority_status\":\"verified\""));
    assert!(post_seal_text.contains("\"final_image_dispatch_matched_count\":1"));
    assert!(post_seal_text.contains(
        "\"first_output_payload_native_execution_status\":\"cuda-driver-kernel-completed\""
    ));
    assert!(post_seal_text
        .contains("\"first_output_payload_native_output_hash\":\"0xdc51cb8047a381e1\""));
    let provider_output = output_dir.join("nuis.nsdb.provider-output.cuda-nvidia-gpu.toml");
    for (needle, context) in [
        (
            "provider_request_order = \"kernel.cuda.vector-add.f32,kernel.cuda.scale.f32\"",
            "ordered CUDA request graph",
        ),
        ("native_output_count = \"2\"", "CUDA output count"),
        (
            "cuda_device_inventory_contract=nuis-cuda-device-inventory-v1",
            "CUDA request device-inventory contract",
        ),
        (
            "cuda_device_selection_contract=nuis-cuda-device-selection-v1",
            "CUDA request device-selection contract",
        ),
        (
            "cuda_device_selection_policy=capability-ranked-lowest-ordinal",
            "CUDA deterministic device-selection policy",
        ),
        (
            "native_output_0_request_id = \"kernel.cuda.vector-add.f32\"",
            "CUDA vector-add output identity",
        ),
        (
            "native_output_0_hash = \"0xdc51cb8047a381e1\"",
            "CUDA vector-add output hash",
        ),
        (
            "native_output_0_device = \"cuda:nvidia-gpu:ordinal-0:sm_",
            "CUDA vector-add selected device",
        ),
        (
            "native_output_1_request_id = \"kernel.cuda.scale.f32\"",
            "CUDA scale output identity",
        ),
        (
            "native_output_1_hash = \"0x40372e9bd3b02048\"",
            "CUDA scale output hash",
        ),
        (
            "native_output_1_device = \"cuda:nvidia-gpu:ordinal-0:sm_",
            "CUDA scale selected device",
        ),
        (
            "provider_edge_transport_receipt_count = \"1\"",
            "CUDA dependency receipt count",
        ),
        (
            "provider_edge_transport_receipt_0_staging_adapter_id = \"provider.output.transfer.v1\"",
            "CUDA direct-transfer adapter",
        ),
        (
            "provider_edge_transport_receipt_0_materialize_status = \"materialized\"",
            "CUDA dependency materialization",
        ),
        (
            "provider_edge_transport_receipt_0_consume_status = \"consumed\"",
            "CUDA dependency consumption",
        ),
        (
            "provider_edge_transport_receipt_0_release_status = \"released\"",
            "CUDA dependency release",
        ),
    ] {
        assert_file_contains(&provider_output, needle, context);
    }

    let refreshed = run_nsdb(&[
        "materialize-provider-samples",
        &output_dir_text,
        "--provider-family",
        "cuda:nvidia-gpu",
        "--json",
    ]);
    assert_success(&refreshed, "refresh post-seal CUDA completion evidence");
    let refreshed_text = output_text(&refreshed);
    assert!(refreshed_text.contains("\"materialized_record_count\":1"));
    assert!(refreshed_text.contains("\"first_provider_output_payload_attach_status\":\"attached\""));

    let final_output = run_nsld(&["final-executable-output", &output_dir_text, "--json"]);
    assert_success(&final_output, "replay sealed CUDA completion through Nsld");
    let final_output_text = output_text(&final_output);
    assert!(final_output_text.contains("\"object_output_family\":\"elf\""));
    assert!(final_output_text.contains("\"object_output_magic\":\"0x7f454c46\""));
    assert!(final_output_text.contains("\"final_output_nsdb_replay_ready\":true"));
    assert!(final_output_text.contains("\"completion_evidence_status\":\"verified\""));
    assert!(final_output_text.contains("\"completion_evidence_count\":2"));
    assert!(final_output_text.contains("\"completion_tokens\":\"provider-completion:0x"));
    assert!(final_output_text.contains("\"glm_release_tokens\":\"glm-release:0x"));

    fs::remove_dir_all(output_dir).unwrap();
}
