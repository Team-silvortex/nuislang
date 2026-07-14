use super::*;

#[test]
fn run_artifact_json_reports_ready_after_final_pipeline_exists() {
    let project_root = write_temp_project_fixture(
        "run_artifact_json_ready_nsld_smoke",
        r#"
name = "run_artifact_json_ready_nsld_smoke"
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
    let output_dir = temp_dir("run_artifact_json_ready_nsld_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);

    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"nsld_prepared_artifact_chain_ready\":true"));
    assert!(json.contains("\"nsld_final_executable_tail_ready\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"ready\""));
    assert!(json.contains(
        "\"run_artifact_prelaunch_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_path\":\""));
    assert!(json.contains("nuis.host-entrypoint.sh"));
    assert!(json.contains(
        "\"run_artifact_prelaunch_reason\":\"nsld final executable pipeline materialized a verified host entrypoint stub\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_tail_stage_records\":[{\"stage\":\"final-executable-writer-input\",\"file\":\"nuis.nsld.final-executable-writer-input.toml\",\"present\":true"
    ));
    assert!(json.contains("\"nsld_next_action_source\":\"nuis-summary\""));
    assert!(json.contains("\"nsld_next_action\":\"inspect-final-executable-output\""));
    assert!(json.contains("\"nsld_next_action_command\":\"nsld final-executable-output "));
    assert!(json.contains(
        "\"nsld_next_action_reason\":\"final executable output boundary is blocked by `final-executable-output:ownership-unknown`\""
    ));
    assert!(json.contains("\"nsld_artifact_chain_next_action_available\":false"));
    assert!(json.contains("\"nsld_artifact_chain_next_action_source\":null"));
    assert!(json.contains("\"nsld_artifact_chain_next_action_command_id\":null"));
    assert!(json.contains("\"nsld_artifact_chain_next_action_command_resolved\":null"));
    assert!(json.contains("\"nsld_drive_recommended_mode\":\"dry-run\""));
    assert!(json.contains("\"nsld_drive_recommended_mutates_artifacts\":false"));
    assert!(json.contains(
        "\"nsld_drive_recommended_reason\":\"artifact-chain has no mutating next action; inspect the final executable output boundary blocked by `final-executable-output:ownership-unknown`\""
    ));
    assert!(json.contains("\"nsld_final_executable_pipeline_valid\":true"));
    assert!(json.contains("\"nsld_final_executable_pipeline_final_executable_emitted\":true"));
    assert!(json.contains("\"nsld_final_executable_pipeline_launcher_manifest_ready\":true"));
    assert!(json.contains("\"nsld_final_executable_pipeline_launcher_dry_run_ready\":true"));
    assert!(json.contains("\"nsld_final_executable_pipeline_would_enter_lifecycle_hook\":true"));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_execution_handoff_contract\":\"nsld-final-output-handoff-v1\""
    ));
    assert!(json.contains("\"nsld_final_executable_pipeline_execution_handoff_ready\":true"));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_execution_handoff_target\":\"entrypoint-materializer\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_execution_handoff_decision_code\":\"handoff-entrypoint-materializer\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_entrypoint_materialization_kind\":\"host-shell-entrypoint-plan\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_entrypoint_materialization_path\":\"nuis.host-entrypoint.sh\""
    ));
    assert!(
        json.contains("\"nsld_final_executable_pipeline_entrypoint_materialization_ready\":true")
    );
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_entrypoint_materialization_first_blocker\":null"
    ));
    assert!(
        json.contains("\"nsld_final_executable_pipeline_entrypoint_materialization_present\":true")
    );
    assert!(json
        .contains("\"nsld_final_executable_pipeline_entrypoint_materialization_hash\":\"0xabcd\""));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_entrypoint_materialization_runner_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""
    ));
    assert!(json.contains("\"nsld_final_executable_pipeline_scheduler_metadata_present\":true"));
    assert!(json.contains("\"nsld_final_executable_pipeline_scheduler_metadata_hash\":\"0x1234\""));
    assert!(json.contains("\"nsld_final_executable_pipeline_required_stage_path_count\":10"));
    assert!(
        json.contains("\"nsld_final_executable_pipeline_required_stage_path_present_count\":10")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_first_missing_required_stage_path\":null")
    );
    assert!(json.contains("\"nsld_self_owned_image_ready\":"));
    assert!(json.contains("\"nsld_self_owned_image_status\":"));
    assert!(json.contains("\"nsld_entrypoint_materialization_status\":"));
    assert!(json.contains("\"nsld_self_owned_image_path\":"));
    assert!(json.contains("\"nsld_final_executable_output_ready\":"));
    assert!(json.contains("\"nsld_final_executable_output_boundary_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_materialization_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_contract\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_ready\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_target\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_evidence_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_first_blocker\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_decision_code\":"));
    assert!(json.contains("\"nsld_final_executable_output_recommended_next_action\":"));
    assert!(json.contains("\"nsld_final_executable_output_path_present\":"));
    assert!(json.contains("\"nsld_final_executable_output_nsld_owned\":null"));
    assert!(json.contains("\"nsld_final_executable_output_blockers\":["));
}

#[test]
fn run_artifact_accepts_ready_nsld_entrypoint_when_legacy_binary_is_missing() {
    let project_root = write_temp_project_fixture(
        "run_artifact_nsld_entrypoint_without_legacy_binary",
        r#"
name = "run_artifact_nsld_entrypoint_without_legacy_binary"
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
    let output_dir = temp_dir("run_artifact_nsld_entrypoint_without_legacy_binary_outputs");
    let manifest_path = output_dir.join("nuis.build.manifest.toml");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    let legacy_binary =
        resolve_run_artifact_binary_path(&manifest_path).expect("legacy binary path resolves");
    fs::remove_file(&legacy_binary).expect("remove legacy binary");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);

    handle_run_artifact(manifest_path, false).expect("run-artifact accepts nsld handoff");
}

#[test]
fn run_artifact_json_blocks_nsld_prelaunch_when_entrypoint_stub_is_missing() {
    let project_root = write_temp_project_fixture(
        "run_artifact_json_missing_entrypoint_smoke",
        r#"
name = "run_artifact_json_missing_entrypoint_smoke"
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
    let output_dir = temp_dir("run_artifact_json_missing_entrypoint_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);
    fs::remove_file(output_dir.join("nuis.host-entrypoint.sh")).expect("remove entrypoint stub");

    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"nsld_final_executable_tail_ready\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"blocked\""));
    assert!(json.contains(
        "\"run_artifact_prelaunch_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_path\":\""));
    assert!(json.contains("nuis.host-entrypoint.sh"));
    assert!(json.contains(
        "\"run_artifact_prelaunch_reason\":\"nsld final executable pipeline reports an entrypoint, but the host entrypoint stub is missing on disk\""
    ));
}

#[test]
fn run_artifact_json_blocks_nsld_prelaunch_when_entrypoint_protocol_is_missing() {
    let project_root = write_temp_project_fixture(
        "run_artifact_json_bad_entrypoint_protocol_smoke",
        r#"
name = "run_artifact_json_bad_entrypoint_protocol_smoke"
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
    let output_dir = temp_dir("run_artifact_json_bad_entrypoint_protocol_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);
    fs::write(
        output_dir.join("nuis.host-entrypoint.sh"),
        "#!/bin/sh\nset -eu\nexec nuis-host-runner --manifest 'manifest.toml'\n",
    )
    .expect("write invalid entrypoint stub");

    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"nsld_final_executable_tail_ready\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"blocked\""));
    assert!(json.contains(
        "\"run_artifact_prelaunch_reason\":\"nsld final executable pipeline reports an entrypoint, but the host entrypoint stub does not declare `nuis-nsld-host-entrypoint-v1`\""
    ));
}
