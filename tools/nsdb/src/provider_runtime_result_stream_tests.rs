use std::{fs, time::SystemTime};

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
