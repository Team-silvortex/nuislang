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

#[test]
fn release_check_summary_json_mirrors_replay_ready_self_contained_closure() {
    let project_root = write_temp_project_fixture(
        "release_check_self_contained_replay_ready",
        r#"
name = "release_check_self_contained_replay_ready"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 0;
  }
}
"#,
    );
    let output_dir = temp_dir("release_check_self_contained_replay_ready_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        Some("nuis-self-contained-image".to_owned()),
    )
    .expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);
    write_nsdb_payload_handoff_placeholder(&output_dir);

    let json = render_release_check_summary_json(&project_root, &output_dir);
    assert!(json.contains("\"kind\":\"release_check_summary\""));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_ready\":true"));
    assert!(json
        .contains("\"nsld_final_executable_output_nsdb_replay_status\":\"replay-evidence-ready\""));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_checkpoint_count\":1"));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replayable_checkpoint_count\":1"));
    assert!(json.contains(
        "\"nsld_final_executable_output_nsdb_replay_next_action\":\"replay-nsdb-payload-execution\""
    ));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_first_blocker\":null"));
}

#[test]
fn release_check_summary_json_blocks_pending_hetero_closure() {
    let project_root = write_temp_project_fixture(
        "release_check_pending_hetero_closure",
        r#"
name = "release_check_pending_hetero_closure"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
    );
    let output_dir = temp_dir("release_check_pending_hetero_closure_outputs");
    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        Some("nuis-self-contained-image".to_owned()),
    )
    .expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);
    write_nsdb_payload_handoff_placeholder(&output_dir);
    let handoff_path = output_dir.join("nuis.nsdb.payload-execution-handoff.toml");
    let handoff = fs::read_to_string(&handoff_path).expect("read nsdb handoff");
    fs::write(
        &handoff_path,
        handoff.replace(
            "record_count = 1\n",
            "record_count = 1\nhetero_execution_closure_protocol = \"nuis-hetero-execution-closure-v1\"\nhetero_execution_closure_status = \"host-runner-pending\"\nhetero_execution_closure_ready = \"false\"\nhetero_execution_closure_first_blocker = \"host-runner-backend-artifact-payload:not-observed\"\nhetero_execution_closure_next_action = \"run-host-runner-payload-probe\"\n",
        ),
    )
    .expect("write pending closure handoff");

    let json = render_release_check_summary_json(&project_root, &output_dir);

    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_ready\":false"));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_status\":\"blocked\""));
    assert!(json.contains(
        "\"nsld_final_executable_output_nsdb_replay_first_blocker\":\"hetero-execution-closure:host-runner-backend-artifact-payload:not-observed\""
    ));
}
