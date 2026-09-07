use crate::provider_runtime_result_stream::{
    persist_provider_runtime_results, ProviderRuntimeResult,
};
use std::{fs, time::SystemTime};

pub(super) fn result() -> ProviderRuntimeResult {
    ProviderRuntimeResult {
        arguments: yir_core::provider_runtime_ipc::DispatchArguments::parse("test.v1|count:u64:4")
            .unwrap(),
        source_yir_fnv1a64: "0x0000000000000000".to_owned(),
        provider_family: "test.device".to_owned(),
        request_id: "draw".to_owned(),
        module: "shader".to_owned(),
        instruction: "draw_instanced".to_owned(),
        node: "draw".to_owned(),
        resource: "gpu".to_owned(),
        element_type: "u8".to_owned(),
        layout: "image-2d-row-major:pixel-format=rgba8".to_owned(),
        shape: vec![1, 1],
        row_stride_bytes: 4,
        payload: vec![1, 2, 3, 255],
        completion_wire: yir_core::ProviderPhysicalCompletion::new(
            "shader.clock.frame.v1",
            "test.clock",
            "test.fence",
            1,
        )
        .unwrap()
        .to_wire(),
    }
}

#[test]
fn rejected_replacement_preserves_previous_runtime_evidence() {
    let dir = temp_output_dir("preserve");
    fs::create_dir_all(&dir).unwrap();
    let manifest = persist_provider_runtime_results(&dir, &[result()]).unwrap();
    let old_manifest = fs::read(&manifest).unwrap();
    let payload = dir.join("nuis.runtime.provider-result.0000.bin");
    let old_payload = fs::read(&payload).unwrap();
    for invalid_kind in 0..7 {
        let mut invalid = result();
        match invalid_kind {
            0 => invalid.completion_wire = "bad-completion".to_owned(),
            1 => invalid.payload.clear(),
            2 => invalid.request_id = "x".repeat(257),
            3 => invalid.request_id = "bad\nrecord".to_owned(),
            4 => invalid.module = "bad\rmodule".to_owned(),
            5 => invalid.layout = "bad\0layout".to_owned(),
            _ => invalid.shape = vec![1; 9],
        }
        assert!(persist_provider_runtime_results(&dir, &[result(), invalid]).is_err());
        assert_eq!(fs::read(&manifest).unwrap(), old_manifest);
        assert_eq!(fs::read(&payload).unwrap(), old_payload);
        assert!(!dir.join("nuis.runtime.provider-result.0001.bin").exists());
    }
    fs::remove_dir_all(dir).unwrap();
}

fn temp_output_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("nsdb-runtime-result-{label}-{nonce}"))
}

#[test]
fn missing_provider_manifest_has_no_runtime_targets() {
    let output_dir = temp_output_dir("missing");
    assert!(
        crate::provider_runtime_result_stream::provider_runtime_result_targets(&output_dir, None)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn present_provider_manifest_with_wrong_protocol_fails_closed() {
    let output_dir = temp_output_dir("protocol");
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        "protocol = \"wrong\"\nschema = \"nsdb-yir-device-provider-sample-v1\"\n",
    )
    .unwrap();

    let error =
        crate::provider_runtime_result_stream::provider_runtime_result_targets(&output_dir, None)
            .unwrap_err();
    assert!(error.contains("rejected manifest protocol"));
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn malformed_runtime_binding_fails_instead_of_disappearing() {
    let output_dir = temp_output_dir("binding");
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"

[[device_provider_samples]]
trace_id = "render"
provider = "render"
provider_family = "metal"
materialization_status = "provider-sample-materialized"
input_evidence = "provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=1;provider_request_0_runtime_result_binding_contract=unsupported"
"#,
    )
    .unwrap();

    let error =
        crate::provider_runtime_result_stream::provider_runtime_result_targets(&output_dir, None)
            .unwrap_err();
    assert!(error.contains("runtime result binding") && error.contains("malformed"));
    fs::remove_dir_all(output_dir).unwrap();
}
