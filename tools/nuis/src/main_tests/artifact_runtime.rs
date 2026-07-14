use super::*;

#[test]
fn build_output_self_check_accepts_built_output_dir() {
    let project_root = write_temp_project_fixture(
        "build_output_self_check_smoke",
        r#"
name = "build_output_self_check_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 42;
  }
}
"#,
    );
    let output_dir = temp_dir("build_output_self_check_outputs");
    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let doctor = run_build_output_self_check(&output_dir).expect("self-check passes");
    assert!(doctor.ready_to_run);
    assert_eq!(doctor.source_kind, "output_dir");
    assert_eq!(doctor.recommended_next_step, "run_artifact");
}

#[test]
fn build_output_self_check_reports_missing_artifact_file() {
    let project_root = write_temp_project_fixture(
        "build_output_self_check_missing_artifact",
        r#"
name = "build_output_self_check_missing_artifact"
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
    let output_dir = temp_dir("build_output_self_check_missing_artifact_outputs");
    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    fs::remove_file(output_dir.join("nuis.compiled.artifact")).expect("remove artifact");

    let error = match run_build_output_self_check(&output_dir) {
        Ok(_) => panic!("self-check should fail"),
        Err(error) => error,
    };
    assert!(error.contains("build self-check could not verify manifest"));
    assert!(error.contains("next step: verify_build_manifest"));
    assert!(error.contains("nuis.compiled.artifact"));
    let json = render_artifact_doctor_json(&output_dir);
    assert!(json.contains("\"self_check_ready\":false"));
    assert!(json.contains("\"artifact_diagnostic_code\":\"manifest_invalid\""));
    assert!(json.contains("\"self_check_code\":\"manifest_verify_failed\""));
    assert!(json.contains("\"project_checks_code\":\"unavailable\""));
    assert!(json.contains("\"self_check_error\":\"build self-check could not verify manifest"));
}

#[test]
fn build_output_self_check_reports_missing_binary_as_incomplete_output() {
    let project_root = write_temp_project_fixture(
        "build_output_self_check_missing_binary",
        r#"
name = "build_output_self_check_missing_binary"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
    );
    let output_dir = temp_dir("build_output_self_check_missing_binary_outputs");
    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    fs::remove_file(output_dir.join("build_output_self_check_missing_binary"))
        .expect("remove binary");

    let error = match run_build_output_self_check(&output_dir) {
        Ok(_) => panic!("self-check should fail"),
        Err(error) => error,
    };
    assert!(error.contains("build self-check could not verify manifest"));
    assert!(error.contains("next step: verify_build_manifest"));
    assert!(error.contains("nuis verify-build-manifest"));
}

#[test]
fn build_report_json_exposes_lifecycle_and_domain_unit_summary() {
    let project_root = write_temp_project_fixture(
        "build_report_smoke",
        r#"
name = "build_report_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  fn main() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return text_handle_helper() + handle;
  }
}
"#,
    );
    let output_dir = temp_dir("build_report_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"kind\":\"build_report\""));
    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"artifact_diagnostic_code\":\"ready_to_run\""));
    assert!(json.contains("\"self_check_ready\":true"));
    assert!(json.contains("\"self_check_code\":\"ok\""));
    assert!(json.contains("\"project_checks_available\":true"));
    assert!(json.contains("\"project_checks_code\":\"ok\""));
    assert!(json.contains("\"abi_checks_ok\":true"));
    assert!(json.contains("\"registry_checks_ok\":true"));
    assert!(json.contains("\"lowering_checks_ok\":true"));
    assert!(json.contains("\"abi_checks\":[{"));
    assert!(json.contains("\"registry_checks\":[{"));
    assert!(json.contains("\"lowering_checks\":[{"));
    assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
    assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
    assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
    assert!(json.contains("\"packaging_mode\":\"native-cpu-llvm\""));
    assert!(json.contains("\"lifecycle_bootstrap_entry\":\"nuis.bootstrap.lifecycle.v1\""));
    assert!(json.contains("\"lifecycle_tick_policy\":\"owned-pump.active-wait-drain\""));
    assert!(json.contains("\"domain_units_count\":1"));
    assert!(json.contains("\"domain_units\":[{"));
    assert!(json.contains("\"domain_family\":\"cpu\""));
    assert!(json.contains("\"artifact_roundtrip_verified\":true"));
    assert!(json.contains("\"lifecycle_contract_consistent\":true"));
    assert!(json.contains("\"heterogeneous_domain_count\":0"));
    assert!(json.contains("\"bridge_registry_units\":0"));
    assert!(json.contains("\"host_bridge_plan_units\":0"));
    assert!(json.contains("\"runtime_load_attempted\":true"));
    assert!(json.contains("\"runtime_load_ok\":true"));
    assert!(json.contains("\"runtime_loaded_lifecycle_entry\":\"nuis.bootstrap.lifecycle.v1\""));
    assert!(json.contains("\"runtime_loaded_domain_units\":1"));
    assert!(json.contains("\"runtime_loaded_heterogeneous_units\":0"));
    assert!(json.contains("\"runtime_loaded_payload_blobs\":0"));
    assert!(json.contains("\"runtime_execution_attempted\":true"));
    assert!(json.contains("\"runtime_execution_ok\":true"));
    assert!(json.contains("\"runtime_execution_domains\":0"));
    assert!(json.contains("\"runtime_execution_plan_phases\":0"));
    assert!(json.contains("\"runtime_execution_trace_events\":0"));
    assert!(json.contains("\"link_plan_domain_unit_records\":[{"));
}

#[test]
fn build_report_json_exposes_self_contained_nsb_packaging_route() {
    let project_root = write_temp_project_fixture(
        "build_report_self_contained_nsb_smoke",
        r#"
name = "build_report_self_contained_nsb_smoke"
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
    let output_dir = temp_dir("build_report_self_contained_nsb_outputs");

    handle_build(
        project_root,
        output_dir.clone(),
        false,
        None,
        None,
        Some("nuis-self-contained-image".to_owned()),
    )
    .expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"ready_to_run\":false"));
    assert!(json.contains("\"recommended_next_step\":\"nsld_drive\""));
    assert!(json.contains("\"packaging_mode\":\"nuis-self-contained-image\""));
    assert!(json.contains("\"link_plan_final_stage\":\"nuis-self-contained-image\""));
    assert!(json.contains("\"link_plan_final_driver\":\"nsld-internal-image-writer\""));
    assert!(json.contains("\"link_plan_final_link_mode\":\"self-contained\""));
    assert!(json.contains("\"link_plan_final_output\":\""));
    assert!(json.contains("\"binary_path\":\""));
    assert!(json.contains(".nsb"));
    assert!(json.contains("\"nsld_drive_recommended_mode\":\"apply-next\""));
    assert!(json.contains("\"nsld_artifact_chain_next_action_command_id\":\"emit-inputs\""));
}

#[test]
fn run_artifact_json_reports_prelaunch_summary_for_built_output() {
    let project_root = write_temp_project_fixture(
        "run_artifact_json_smoke",
        r#"
name = "run_artifact_json_smoke"
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
    let output_dir = temp_dir("run_artifact_json_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"kind\":\"run_artifact\""));
    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"binary_resolved\":true"));
    assert!(json.contains("\"binary_path\":\""));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"host-binary\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"ready\""));
    assert!(json.contains("\"run_artifact_prelaunch_evidence_status\":\"host-binary-ready\""));
    assert!(json.contains("\"run_artifact_prelaunch_command\":\""));
    assert!(json.contains("\"run_artifact_prelaunch_runner_command_present\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_path\":null"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_present\":false"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_protocol\":null"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_protocol_valid\":null"));
    assert!(json.contains(
        "\"run_artifact_prelaunch_reason\":\"legacy host binary path is resolved and can be executed directly\""
    ));
    assert!(json.contains("\"heterogeneous_domain_count\":0"));
    assert!(json.contains("\"bridge_registry_units\":0"));
    assert!(json.contains("\"host_bridge_plan_units\":0"));
    assert!(json.contains("\"link_plan_available\":true"));
    assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
    assert!(json.contains("\"nsld_drive_dry_run_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_dry_run_json_command\":\"nsld drive "));
    assert!(json.contains(" --json\""));
    assert!(json.contains("\"nsld_drive_apply_next_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_apply_next_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --json\""));
    assert!(json.contains(" --apply\""));
    assert!(json.contains("\"nsld_drive_apply_until_clean_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --until-clean --json\""));
    assert!(json.contains(" --apply --until-clean\""));
    assert!(json.contains("\"nsld_drive_command_set\":{"));
    assert!(json.contains("\"protocol\":\"nsld-drive-command-set-v1\""));
    assert!(json.contains("\"recommended_first_json_command\":\"nsld drive "));
    assert!(json.contains("\"dry_run_mutates_artifacts\":false"));
    assert!(json.contains("\"apply_next_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_until_clean_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_next_json_command\":\"nsld drive "));
    assert!(json.contains("\"apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_recommended_available\":true"));
    assert!(json.contains("\"nsld_drive_recommended_mode\":\"apply-next\""));
    assert!(json.contains("\"nsld_drive_recommended_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_recommended_mutates_artifacts\":true"));
    assert!(json.contains("\"nsld_next_action_source\":\"nuis-summary\""));
    assert!(json.contains("\"nsld_next_action\":\"prepare\""));
    assert!(json.contains("\"nsld_next_action_command\":\"nsld prepare "));
    assert!(json.contains(
        "\"nsld_next_action_reason\":\"prepared artifact chain is missing `link-inputs`\""
    ));
    assert!(json.contains("\"nsld_artifact_chain_next_action_available\":true"));
    assert!(json.contains("\"nsld_artifact_chain_next_action_source\":\"required\""));
    assert!(json.contains("\"nsld_artifact_chain_next_action_command_id\":\"emit-inputs\""));
    assert!(
        json.contains("\"nsld_artifact_chain_next_action_command\":\"nsld emit-inputs <input>\"")
    );
    assert!(
        json.contains("\"nsld_artifact_chain_next_action_command_resolved\":\"nsld emit-inputs ")
    );
    assert!(json.contains(
        "\"nsld_artifact_chain_next_action_reason\":\"first missing required artifact stage `link-inputs`\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_command\":\"nsld emit-final-executable-pipeline "
    ));
    assert!(json.contains("\"nsld_final_executable_tail_ready\":false"));
    assert!(json.contains("\"nsld_final_executable_pipeline_final_executable_emitted\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_launcher_manifest_ready\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_execution_handoff_contract\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_execution_handoff_ready\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_execution_handoff_status\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_execution_handoff_target\":null"));
    assert!(
        json.contains("\"nsld_final_executable_pipeline_execution_handoff_evidence_status\":null")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_execution_handoff_first_blocker\":null")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_execution_handoff_decision_code\":null")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_entrypoint_materialization_kind\":null")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_entrypoint_materialization_path\":null")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_entrypoint_materialization_ready\":null")
    );
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_entrypoint_materialization_first_blocker\":null"
    ));
    assert!(
        json.contains("\"nsld_final_executable_pipeline_entrypoint_materialization_present\":null")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_entrypoint_materialization_hash\":null")
    );
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_entrypoint_materialization_runner_command\":null"
    ));
    assert!(json.contains("\"nsld_final_executable_pipeline_scheduler_metadata_payload_id\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_required_stage_path_count\":null"));
    assert!(json.contains("\"nsld_self_owned_image_ready\":"));
    assert!(json.contains("\"nsld_self_owned_image_status\":"));
    assert!(json.contains("\"nsld_entrypoint_materialization_status\":"));
    assert!(json.contains("\"nsld_self_owned_image_path\":"));
    assert!(json.contains("\"nsld_final_executable_output_ready\":"));
    assert!(json.contains("\"nsld_final_executable_output_boundary_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_materialization_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_contract\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_ready\":false"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_target\":"));
    assert!(json.contains("\"nsld_final_executable_output_execution_handoff_evidence_status\":"));
    assert!(json.contains(
        "\"nsld_final_executable_output_execution_handoff_first_blocker\":\"final-executable-output:ownership-unknown\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_output_execution_handoff_decision_code\":\"inspect-output-boundary\""
    ));
    assert!(json.contains("\"nsld_final_executable_output_recommended_next_action\":"));
    assert!(json.contains("\"nsld_final_executable_output_path_present\":"));
    assert!(json.contains("\"nsld_final_executable_output_nsld_owned\":null"));
    assert!(json.contains("\"nsld_final_executable_output_blockers\":["));
}

#[test]
fn run_artifact_json_recommends_final_pipeline_after_prepared_chain_exists() {
    let project_root = write_temp_project_fixture(
        "run_artifact_json_prepared_nsld_smoke",
        r#"
name = "run_artifact_json_prepared_nsld_smoke"
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
    let output_dir = temp_dir("run_artifact_json_prepared_nsld_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);

    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"nsld_prepared_artifact_chain_ready\":true"));
    assert!(json.contains("\"nsld_prepared_artifact_next_missing_stage\":null"));
    assert!(json.contains(
        "\"nsld_prepared_artifact_stage_records\":[{\"stage\":\"link-inputs\",\"file\":\"nuis.nsld.link-inputs.toml\",\"present\":true"
    ));
    assert!(json.contains("\"nsld_next_action_source\":\"nuis-summary\""));
    assert!(json.contains("\"nsld_next_action\":\"emit-final-executable-pipeline\""));
    assert!(json.contains("\"nsld_next_action_command\":\"nsld emit-final-executable-pipeline "));
    assert!(json.contains(
        "\"nsld_next_action_reason\":\"final executable tail is missing `final-executable-writer-input`\""
    ));
    assert!(json.contains("\"nsld_artifact_chain_next_action_available\":true"));
    assert!(json.contains("\"nsld_artifact_chain_next_action_source\":\"optional\""));
    assert!(json.contains(
        "\"nsld_artifact_chain_next_action_command_id\":\"emit-final-executable-pipeline\""
    ));
    assert!(json.contains(
        "\"nsld_artifact_chain_next_action_command\":\"nsld emit-final-executable-pipeline <input>\""
    ));
    assert!(json.contains(
        "\"nsld_artifact_chain_next_action_command_resolved\":\"nsld emit-final-executable-pipeline "
    ));
    assert!(json.contains(
        "\"nsld_artifact_chain_next_action_reason\":\"first missing optional artifact stage `final-executable-writer-input`\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_tail_next_missing_stage\":\"final-executable-writer-input\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_tail_stage_records\":[{\"stage\":\"final-executable-writer-input\",\"file\":\"nuis.nsld.final-executable-writer-input.toml\",\"present\":false"
    ));
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
fn artifact_doctor_json_prefers_ready_nsld_entrypoint_closure() {
    let project_root = write_temp_project_fixture(
        "artifact_doctor_json_ready_nsld_smoke",
        r#"
name = "artifact_doctor_json_ready_nsld_smoke"
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
    let output_dir = temp_dir("artifact_doctor_json_ready_nsld_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);

    let json = render_artifact_doctor_json(&output_dir);

    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"artifact_closure_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"artifact_closure_status\":\"ready\""));
    assert!(json.contains("\"artifact_closure_evidence_status\":\"entrypoint-ready\""));
    assert!(
        json.contains("\"artifact_closure_command\":\"nuis-host-runner --manifest 'manifest.toml'")
    );
    assert!(json.contains("\"artifact_closure_runner_command_present\":true"));
    assert!(json.contains("\"artifact_closure_entrypoint_path\":\""));
    assert!(json.contains("nuis.host-entrypoint.sh"));
    assert!(json.contains("\"artifact_closure_entrypoint_present\":true"));
    assert!(
        json.contains("\"artifact_closure_entrypoint_protocol\":\"nuis-nsld-host-entrypoint-v1\"")
    );
    assert!(json.contains("\"artifact_closure_entrypoint_protocol_valid\":true"));
    assert!(json.contains(
        "\"artifact_closure_reason\":\"nsld final executable pipeline materialized a verified host entrypoint stub\""
    ));
    assert!(json.contains("\"nsld_final_executable_tail_ready\":true"));
}

#[test]
fn artifact_doctor_json_blocks_self_contained_route_until_nsld_handoff_exists() {
    let project_root = write_temp_project_fixture(
        "artifact_doctor_self_contained_without_handoff_smoke",
        r#"
name = "artifact_doctor_self_contained_without_handoff_smoke"
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
    let output_dir = temp_dir("artifact_doctor_self_contained_without_handoff_outputs");

    handle_build(
        project_root,
        output_dir.clone(),
        false,
        None,
        None,
        Some("nuis-self-contained-image".to_owned()),
    )
    .expect("build passes");
    let json = render_artifact_doctor_json(&output_dir);

    assert!(json.contains("\"ready_to_run\":false"));
    assert!(json.contains("\"recommended_next_step\":\"nsld_drive\""));
    assert!(json.contains("\"link_plan_final_stage\":\"nuis-self-contained-image\""));
    assert!(json.contains("\"link_plan_final_link_mode\":\"self-contained\""));
    assert!(json.contains("\"artifact_closure_kind\":\"none\""));
    assert!(json.contains("\"artifact_closure_status\":\"blocked\""));
    assert!(json.contains(
        "\"artifact_closure_evidence_status\":\"self-contained-image-awaiting-nsld-handoff\""
    ));
    assert!(json.contains(
        "\"artifact_closure_reason\":\"self-contained Nuis image route is selected, but no verified Nsld host entrypoint handoff is materialized yet\""
    ));
    assert!(!json.contains("\"artifact_closure_kind\":\"host-binary\""));
    assert!(json.contains("\"nsld_final_executable_output_recommended_next_action\":\"emit-final-executable-pipeline\""));
}

#[test]
fn artifact_doctor_json_reports_ready_self_contained_nsld_handoff() {
    let project_root = write_temp_project_fixture(
        "artifact_doctor_self_contained_ready_handoff_smoke",
        r#"
name = "artifact_doctor_self_contained_ready_handoff_smoke"
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
    let output_dir = temp_dir("artifact_doctor_self_contained_ready_handoff_outputs");

    handle_build(
        project_root,
        output_dir.clone(),
        false,
        None,
        None,
        Some("nuis-self-contained-image".to_owned()),
    )
    .expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);
    let json = render_artifact_doctor_json(&output_dir);

    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"recommended_next_step\":\"run_artifact\""));
    assert!(json.contains("\"link_plan_final_stage\":\"nuis-self-contained-image\""));
    assert!(json.contains("\"artifact_closure_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"artifact_closure_status\":\"ready\""));
    assert!(json.contains("\"artifact_closure_evidence_status\":\"entrypoint-ready\""));
    assert!(json.contains("nuis.host-entrypoint.sh"));
    assert!(!json.contains("\"artifact_closure_kind\":\"host-binary\""));
}

#[test]
fn build_report_json_exposes_real_heterogeneous_runtime_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_profile_demo");
    let output_dir = temp_dir("build_report_shader_profile_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"domain_units_count\":2"));
    assert!(json.contains("\"heterogeneous_domain_count\":1"));
    assert!(json.contains("\"domain_family\":\"shader\""));
    assert!(json.contains("\"packaging_role\":\"hetero-contract\""));
    assert!(json.contains("\"artifact_payload_format\":\"ndpb-v2\""));
    assert!(json.contains("\"bridge_registry_units\":1"));
    assert!(json.contains("\"bridge_registry_checked\":1"));
    assert!(json.contains("\"host_bridge_plan_units\":1"));
    assert!(json.contains("\"domain_payload_blobs_checked\":1"));
    assert!(json.contains("\"domain_payload_bridge_plans_checked\":1"));
    assert!(json.contains("\"domain_bridge_stubs_checked\":1"));
    assert!(json.contains("\"link_plan_domain_units\":2"));
    assert!(json.contains("\"runtime_execution_attempted\":true"));
    assert!(json.contains("\"runtime_execution_ok\":true"));
    assert!(json.contains("\"runtime_execution_domains\":1"));
    assert!(json.contains("\"runtime_execution_plan_phases\":"));
    assert!(json.contains("\"runtime_execution_trace_events\":"));
    assert!(json.contains("\"runtime_payload_backed_heterogeneous_units\":1"));
    assert!(json.contains("\"runtime_cpu_fallback_units\":"));
    assert!(json.contains("\"runtime_host_consumable_units\":"));
}

#[test]
fn build_report_json_exposes_host_cpu_fallback_runtime_events() {
    let project_root = write_temp_project_fixture(
        "shader_cpu_fallback_runtime_demo",
        r#"
name = "shader_cpu_fallback_runtime_demo"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns", "surface_shader.ns"]
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "shader=shader.render.cpu-fallback.v1",
]
"#
        .trim_start(),
        r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let vertex_budget: i64 = shader_profile_vertex_count("SurfaceShader");
    let instance_budget: i64 = shader_profile_instance_count("SurfaceShader");
    let packet_tag: i64 = shader_profile_packet_tag("SurfaceShader");
    let swapchain: Target = shader_profile_target("SurfaceShader");
    print(vertex_budget + instance_budget + packet_tag);
  }
}
"#,
    );
    fs::write(
        project_root.join("surface_shader.ns"),
        r#"
mod shader SurfaceShader {
  fn profile() {
    const vertex_count: i64 = 4;
    const instance_count: i64 = 1;
    const packet_tag: i64 = 17;

    let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
    let profile_view: Viewport = shader_viewport(160, 120);
    let profile_pipe: Pipeline = shader_pipeline("cpu_fallback_surface", "triangle_strip");
  }
}
"#,
    )
    .expect("write shader surface");
    let output_dir = temp_dir("build_report_shader_cpu_fallback_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"domain_family\":\"shader\""));
    assert!(json.contains("\"runtime_payload_backed_heterogeneous_units\":1"));
    assert!(json.contains("\"runtime_cpu_fallback_units\":1"));
    assert!(json.contains("\"runtime_host_consumable_units\":1"));
    assert!(json.contains("\"runtime_execution_host_fallback_events\":"));
    assert!(!json.contains("\"runtime_execution_host_fallback_events\":0"));
}

#[test]
fn build_report_json_executes_host_yir_kernel_values() {
    let project_root = write_temp_project_fixture(
        "kernel_host_yir_runtime_demo",
        r#"
name = "kernel_host_yir_runtime_demo"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() {
    let input = kernel_tensor(1, 3, "2,4,6");
    let weights = kernel_tensor(3, 2, "1,-2,3,0,2,1");
    let projected = kernel_matmul(input, weights);
    let summary: i64 = kernel_reduce_sum(projected);
    print(summary);
  }
}
"#,
    );
    let output_dir = temp_dir("build_report_kernel_host_yir_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"runtime_host_yir_attempted\":true"));
    assert!(json.contains("\"runtime_host_yir_ok\":true"));
    assert!(json.contains("\"runtime_host_yir_kernel_nodes\":4"));
    assert!(json.contains("\"runtime_host_yir_tensor_values\":3"));
    assert!(json.contains("\"runtime_host_yir_scalar_values\":2"));
    assert!(json.contains("\"runtime_host_yir_kernel_integer_checksum\":73"));
    assert!(json.contains("\"runtime_execution_kernel_host_reference_events\":4"));
}

#[test]
fn build_report_json_exposes_kernel_result_profile_bundle_summary() {
    let project_root =
        checked_in_path("../../examples/projects/domains/kernel_result_profile_demo");
    let output_dir = temp_dir("build_report_kernel_result_profile_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"binary_name\":\"kernel_result_profile_demo\""));
    assert!(json.contains("\"packaging_mode\":\"native-cpu-llvm\""));
    assert!(json.contains("\"domain_units_count\":2"));
    assert!(json.contains("\"heterogeneous_domain_count\":1"));
    assert!(json.contains("\"domain_family\":\"cpu\""));
    assert!(json.contains("\"domain_family\":\"kernel\""));
    assert!(json.contains("\"selected_lowering_target\":\"llvm\""));
    assert!(json.contains("\"backend_family\":\"coreml\""));
    assert!(json.contains("\"selected_lowering_target\":\"coreml.apple-ane\""));
    assert!(json.contains("\"bridge_registry_units\":1"));
    assert!(json.contains("\"host_bridge_plan_units\":1"));
    assert!(json.contains("\"runtime_payload_backed_heterogeneous_units\":1"));
    assert!(json.contains("\"runtime_execution_kernel_host_reference_events\":4"));
    assert!(json.contains("\"runtime_host_yir_attempted\":true"));
    assert!(json.contains("\"runtime_host_yir_ok\":true"));
    assert!(json.contains("\"runtime_host_yir_kernel_nodes\":4"));
    assert!(json.contains("\"runtime_host_yir_kernel_integer_checksum\":18"));
    assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
    assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
    assert!(json.contains("\"link_plan_domain_units\":2"));
}

#[test]
fn run_artifact_json_exposes_real_heterogeneous_runtime_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_profile_demo");
    let output_dir = temp_dir("run_artifact_shader_profile_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"binary_resolved\":true"));
    assert!(json.contains("\"heterogeneous_domain_count\":1"));
    assert!(json.contains("\"bridge_registry_units\":1"));
    assert!(json.contains("\"host_bridge_plan_units\":1"));
    assert!(json.contains("\"domain_payload_blobs_checked\":1"));
    assert!(json.contains("\"link_plan_domain_units\":2"));
    assert!(json.contains("\"domain_family\":\"shader\""));
}

#[test]
fn build_report_json_exposes_bridge_bearing_exchange_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_packet_bridge_demo");
    let output_dir = temp_dir("build_report_shader_packet_bridge_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"packaging_mode\":\"window-aot-bundle\""));
    assert!(json.contains("\"domain_units_count\":3"));
    assert!(json.contains("\"heterogeneous_domain_count\":2"));
    assert!(json.contains("\"domain_family\":\"data\""));
    assert!(json.contains("\"domain_family\":\"shader\""));
    assert!(json.contains("\"bridge_registry_units\":2"));
    assert!(json.contains("\"bridge_registry_entries_checked\":2"));
    assert!(json.contains("\"host_bridge_plan_units\":2"));
    assert!(json.contains("\"host_bridge_plan_entries_checked\":2"));
    assert!(json.contains("\"domain_payload_blobs_checked\":2"));
    assert!(json.contains("\"domain_payload_bridge_plans_checked\":2"));
    assert!(json.contains("\"domain_bridge_stubs_checked\":2"));
    assert!(json.contains("\"link_plan_final_stage\":\"heterogeneous-bundle-pack\""));
    assert!(json.contains("\"link_plan_final_driver\":\"yir-pack-aot\""));
    assert!(json.contains("\"link_plan_domain_units\":3"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_ready_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_readiness_ready\":true"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_families\":[\"data\",\"shader\"]"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_first_unready\":null"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_readiness\":[{"));
}

#[test]
fn build_report_json_exposes_shader_result_enum_bundle_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_result_enum_demo");
    let output_dir = temp_dir("build_report_shader_result_enum_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"binary_name\":\"shader_result_enum_demo\""));
    assert!(json.contains("\"packaging_mode\":\"window-aot-bundle\""));
    assert!(json.contains("\"domain_units_count\":3"));
    assert!(json.contains("\"heterogeneous_domain_count\":2"));
    assert!(json.contains("\"domain_family\":\"cpu\""));
    assert!(json.contains("\"domain_family\":\"data\""));
    assert!(json.contains("\"domain_family\":\"shader\""));
    assert!(json.contains("\"selected_lowering_target\":\"llvm\""));
    assert!(json.contains("\"selected_lowering_target\":\"metal.apple-silicon-gpu\""));
    assert!(json.contains("\"bridge_registry_units\":2"));
    assert!(json.contains("\"host_bridge_plan_units\":2"));
    assert!(json.contains("\"runtime_payload_backed_heterogeneous_units\":2"));
    assert!(json.contains("\"runtime_host_yir_attempted\":true"));
    assert!(json.contains("\"runtime_host_yir_ok\":true"));
    assert!(json.contains("\"link_plan_final_stage\":\"heterogeneous-bundle-pack\""));
    assert!(json.contains("\"link_plan_final_driver\":\"yir-pack-aot\""));
    assert!(json.contains("\"link_plan_domain_units\":3"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_ready_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_readiness_ready\":true"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_families\":[\"data\",\"shader\"]"));
}

#[test]
fn run_artifact_json_exposes_bridge_bearing_exchange_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_packet_bridge_demo");
    let output_dir = temp_dir("run_artifact_shader_packet_bridge_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"binary_resolved\":true"));
    assert!(json.contains("\"heterogeneous_domain_count\":2"));
    assert!(json.contains("\"bridge_registry_units\":2"));
    assert!(json.contains("\"host_bridge_plan_units\":2"));
    assert!(json.contains("\"domain_payload_blobs_checked\":2"));
    assert!(json.contains("\"domain_payload_bridge_plans_checked\":2"));
    assert!(json.contains("\"link_plan_final_stage\":\"heterogeneous-bundle-pack\""));
    assert!(json.contains("\"link_plan_final_driver\":\"yir-pack-aot\""));
    assert!(json.contains("\"link_plan_domain_units\":3"));
}

#[test]
fn unpack_artifact_support_materializes_embedded_sidecars_for_bridge_project() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_packet_bridge_demo");
    let output_dir = temp_dir("unpack_artifact_support_bridge_build_outputs");
    let unpack_dir = temp_dir("unpack_artifact_support_bridge_unpack_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    handle_unpack_artifact_support(
        output_dir.join("nuis.compiled.artifact"),
        unpack_dir.clone(),
        false,
    )
    .expect("unpack-artifact-support passes");

    for path in [
        unpack_dir.join("nuis.bridge.registry.toml"),
        unpack_dir.join("nuis.host-bridge.plan-index.toml"),
        unpack_dir.join("nuis.domain.data.artifact.toml"),
        unpack_dir.join("nuis.domain.data.payload.toml"),
        unpack_dir.join("nuis.domain.data.payload.bin"),
        unpack_dir.join("nuis.domain.data.bridge.stub.txt"),
        unpack_dir.join("nuis.domain.shader.artifact.toml"),
        unpack_dir.join("nuis.domain.shader.payload.toml"),
        unpack_dir.join("nuis.domain.shader.payload.bin"),
        unpack_dir.join("nuis.domain.shader.bridge.stub.txt"),
    ] {
        assert!(
            path.exists(),
            "expected unpacked support `{}`",
            path.display()
        );
    }
}

#[test]
fn materialize_artifact_rebuilds_frontdoor_bundle_and_support_sidecars() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_packet_bridge_demo");
    let build_output_dir = temp_dir("materialize_artifact_bridge_build_outputs");
    let materialize_dir = temp_dir("materialize_artifact_bridge_bundle_outputs");

    handle_build(
        project_root,
        build_output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    handle_materialize_artifact(
        build_output_dir.join("nuis.build.manifest.toml"),
        materialize_dir.clone(),
        false,
    )
    .expect("materialize-artifact passes");

    for path in [
        materialize_dir.join("nuis.executable.envelope.toml"),
        materialize_dir.join("nuis.build.manifest.toml"),
        materialize_dir.join("nuis.compiled.artifact"),
        materialize_dir.join("shader_packet_bridge_demo"),
        materialize_dir.join("nuis.bridge.registry.toml"),
        materialize_dir.join("nuis.host-bridge.plan-index.toml"),
        materialize_dir.join("nuis.domain.data.payload.bin"),
        materialize_dir.join("nuis.domain.shader.payload.bin"),
    ] {
        assert!(
            path.exists(),
            "expected materialized output `{}`",
            path.display()
        );
    }

    let report = nuisc::aot::verify_build_manifest(
        materialize_dir.join("nuis.build.manifest.toml").as_path(),
    )
    .expect("materialized manifest verifies");
    assert_eq!(report.artifact_binary_name, "shader_packet_bridge_demo");
    assert_eq!(report.packaging_mode, "window-aot-bundle");
}
