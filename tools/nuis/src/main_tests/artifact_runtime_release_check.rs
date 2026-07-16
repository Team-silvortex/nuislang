use super::*;

#[test]
fn release_check_summary_json_mirrors_generated_payload_decoder_manifest() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_packet_bridge_demo");
    let output_dir = temp_dir("release_check_summary_shader_packet_bridge_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    let run_json = render_run_artifact_json(&output_dir);
    assert!(run_json.contains("\"payload_decoder_manifest_persisted\":true"));

    let json = render_release_check_summary_json(&project_root, &output_dir);
    assert!(json.contains("\"kind\":\"release_check_summary\""));
    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"release_check_payload_decoder_manifest_available\":true"));
    assert!(json.contains("\"release_check_payload_decoder_manifest_status\":\"ready\""));
    assert!(json.contains("\"release_check_payload_decoder_manifest_record_count\":1"));
    assert!(json.contains("\"release_check_payload_decoder_manifest_invalid_record_count\":0"));
    assert!(json.contains("\"release_check_device_sample_handoff_available\":true"));
    assert!(json
        .contains("\"release_check_device_sample_handoff_status\":\"provider-handoff-pending\""));
    assert!(json.contains("\"release_check_device_sample_handoff_record_count\":1"));
    assert!(json.contains(
        "\"release_check_device_sample_handoff_first_provider_family\":\"metal:apple-silicon-gpu\""
    ));
    assert!(json.contains(
        "\"release_check_device_sample_handoff_first_validation_status\":\"pending-provider-execution\""
    ));
    assert!(json.contains("\"nsld_drive_protocol\":\"nsld-drive-command-set-v1\""));
}
