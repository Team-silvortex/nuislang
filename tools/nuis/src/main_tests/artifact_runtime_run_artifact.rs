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

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);

    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"nsld_prepared_artifact_chain_ready\":true"));
    assert!(json.contains("\"nsld_final_executable_tail_ready\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"ready\""));
    assert!(json.contains("\"run_artifact_prelaunch_evidence_status\":\"entrypoint-ready\""));
    assert!(json.contains(
        "\"run_artifact_prelaunch_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(json.contains("\"run_artifact_prelaunch_runner_command_present\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_path\":\""));
    assert!(json.contains("nuis.host-entrypoint.sh"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_present\":true"));
    assert!(json.contains(
        "\"run_artifact_prelaunch_entrypoint_protocol\":\"nuis-nsld-host-entrypoint-v1\""
    ));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_protocol_valid\":true"));
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
fn run_artifact_json_blocks_self_contained_route_until_nsld_handoff_exists() {
    let project_root = write_temp_project_fixture(
        "run_artifact_self_contained_without_handoff_smoke",
        r#"
name = "run_artifact_self_contained_without_handoff_smoke"
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
    let output_dir = temp_dir("run_artifact_self_contained_without_handoff_outputs");

    handle_build(
        project_root,
        output_dir.clone(),
        false,
        None,
        None,
        Some("nuis-self-contained-image".to_owned()),
    )
    .expect("build passes");
    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"ready_to_run\":false"));
    assert!(json.contains("\"recommended_next_step\":\"nsld_drive\""));
    assert!(json.contains("\"binary_resolved\":false"));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"none\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"blocked\""));
    assert!(json.contains(
        "\"run_artifact_prelaunch_evidence_status\":\"self-contained-image-awaiting-nsld-handoff\""
    ));
    assert!(json.contains("\"run_artifact_prelaunch_command\":null"));
    assert!(!json.contains("\"run_artifact_prelaunch_kind\":\"host-binary\""));
    assert!(json.contains("\"nsld_final_executable_output_recommended_next_action\":\"emit-final-executable-pipeline\""));
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

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    let legacy_binary =
        resolve_run_artifact_binary_path(&manifest_path).expect("legacy binary path resolves");
    fs::remove_file(&legacy_binary).expect("remove legacy binary");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);

    handle_run_artifact(manifest_path, false).expect("run-artifact accepts nsld handoff");
}

#[test]
fn run_artifact_accepts_self_contained_nsld_handoff_without_host_binary_fallback() {
    let project_root = write_temp_project_fixture(
        "run_artifact_self_contained_ready_handoff",
        r#"
name = "run_artifact_self_contained_ready_handoff"
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
    let output_dir = temp_dir("run_artifact_self_contained_ready_handoff_outputs");
    let manifest_path = output_dir.join("nuis.build.manifest.toml");

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

    let json = render_run_artifact_json(&manifest_path);

    assert!(json.contains("\"binary_resolved\":false"));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"ready\""));
    assert!(json.contains("\"run_artifact_prelaunch_evidence_status\":\"entrypoint-ready\""));
    assert!(!json.contains("\"run_artifact_prelaunch_kind\":\"host-binary\""));
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

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);
    fs::remove_file(output_dir.join("nuis.host-entrypoint.sh")).expect("remove entrypoint stub");

    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"nsld_final_executable_tail_ready\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"run_artifact_prelaunch_status\":\"blocked\""));
    assert!(json.contains("\"run_artifact_prelaunch_evidence_status\":\"entrypoint-missing\""));
    assert!(json.contains(
        "\"run_artifact_prelaunch_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(json.contains("\"run_artifact_prelaunch_runner_command_present\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_path\":\""));
    assert!(json.contains("nuis.host-entrypoint.sh"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_present\":false"));
    assert!(json.contains(
        "\"run_artifact_prelaunch_entrypoint_protocol\":\"nuis-nsld-host-entrypoint-v1\""
    ));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_protocol_valid\":null"));
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

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
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
    assert!(
        json.contains("\"run_artifact_prelaunch_evidence_status\":\"entrypoint-protocol-invalid\"")
    );
    assert!(json.contains("\"run_artifact_prelaunch_runner_command_present\":true"));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_present\":true"));
    assert!(json.contains(
        "\"run_artifact_prelaunch_entrypoint_protocol\":\"nuis-nsld-host-entrypoint-v1\""
    ));
    assert!(json.contains("\"run_artifact_prelaunch_entrypoint_protocol_valid\":false"));
    assert!(json.contains(
        "\"run_artifact_prelaunch_reason\":\"nsld final executable pipeline reports an entrypoint, but the host entrypoint stub does not declare `nuis-nsld-host-entrypoint-v1`\""
    ));
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
    assert!(json.contains("\"link_plan_heterogeneous_domain_registry_dispatch_ready_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_units\":1"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_ready_units\":1"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_readiness_ready\":true"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_families\":[\"data\",\"shader\"]"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_first_unready\":null"));
    assert!(
        json.contains("\"link_plan_heterogeneous_domain_registry_dispatch_first_blocked\":null")
    );
    assert!(json.contains("\"backend_artifact_candidate\":true"));
    assert!(json.contains("\"backend_artifact_ready\":true"));
    assert!(json.contains("\"registry_dispatch_readiness_status\":\"ready\""));
    assert!(json.contains("\"registry_dispatch_readiness_ready\":true"));
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
    assert!(json.contains("\"link_plan_heterogeneous_domain_registry_dispatch_ready_units\":2"));
    assert!(
        json.contains("\"link_plan_heterogeneous_domain_registry_dispatch_first_blocked\":null")
    );
    assert!(json.contains("\"registry_dispatch_readiness_status\":\"ready\""));
    assert!(json.contains("\"hetero_runtime_trace_available\":true"));
    assert!(json.contains("\"hetero_runtime_trace_status\":\"execution-pending\""));
    assert!(json.contains(
        "\"hetero_runtime_trace_debugger_contract\":\"nsdb-yir-hetero-runtime-trace-v1\""
    ));
    assert!(json.contains("\"hetero_runtime_trace_domain_count\":2"));
    assert!(json.contains("\"hetero_runtime_trace_backend_artifact_count\":1"));
    assert!(json.contains("\"hetero_runtime_trace_backend_artifact_ready_count\":1"));
    assert!(json.contains("\"hetero_runtime_trace_record_count\":2"));
    assert!(json.contains("\"hetero_runtime_trace_ready_record_count\":0"));
    assert!(json.contains("\"hetero_runtime_trace_backend_execution_record_count\":1"));
    assert!(json.contains("\"hetero_runtime_trace_device_sample_descriptor_count\":1"));
    assert!(json.contains("\"hetero_runtime_trace_device_sample_ready_count\":0"));
    assert!(json.contains("\"hetero_runtime_trace_device_sample_pending_count\":1"));
    assert!(json.contains("\"hetero_runtime_trace_device_sample_pending_validation_count\":1"));
    assert!(json.contains("\"hetero_runtime_trace_device_sample_handoff_record_count\":1"));
    assert!(json.contains(
        "\"hetero_runtime_trace_device_sample_handoff_protocol\":\"nuis-device-sample-provider-handoff-v1\""
    ));
    assert!(json.contains("\"hetero_runtime_trace_domain_families\":[\"data\",\"shader\"]"));
    assert!(json.contains("\"hetero_runtime_trace_target_devices\":[\"apple-silicon-gpu\"]"));
    assert!(json.contains(
        "\"hetero_runtime_trace_device_sample_providers\":[\"nustar-deferred-device-sample-v1\"]"
    ));
    assert!(json.contains(
        "\"hetero_runtime_trace_device_sample_provider_families\":[\"metal:apple-silicon-gpu\"]"
    ));
    assert!(json.contains(
        "\"hetero_runtime_trace_device_sample_validation_statuses\":[\"pending-provider-execution\"]"
    ));
    assert!(json.contains(
        "\"hetero_runtime_trace_device_sample_handoff_status\":\"provider-handoff-pending\""
    ));
    assert!(json.contains(
        "\"hetero_runtime_trace_device_sample_first_pending_provider_family\":\"metal:apple-silicon-gpu\""
    ));
    assert!(json.contains("\"hetero_runtime_trace_records\":[{"));
    assert!(json.contains("\"hetero_runtime_trace_persisted\":true"));
    assert!(json.contains(
        "\"hetero_runtime_trace_persistence_protocol\":\"nuis-nsdb-hetero-runtime-trace-v1\""
    ));
    assert!(json.contains("\"hetero_runtime_trace_path\":\""));
    assert!(json.contains("\"hetero_runtime_trace_persisted_record_count\":2"));
    assert!(json.contains("\"hetero_runtime_trace_persist_error\":null"));
    assert!(json.contains("\"payload_decoder_manifest_persisted\":true"));
    assert!(json.contains("\"payload_decoder_manifest_path\":\""));
    assert!(json.contains("\"payload_decoder_manifest_persisted_record_count\":1"));
    assert!(json.contains("\"payload_decoder_manifest_persist_error\":null"));
    assert!(json.contains("\"device_provider_sample_manifest_persisted\":true"));
    assert!(json.contains("\"device_provider_sample_manifest_path\":\""));
    assert!(json.contains("\"device_provider_sample_manifest_persisted_record_count\":1"));
    assert!(json.contains("\"device_provider_sample_manifest_persist_error\":null"));
    assert!(json.contains("\"trace_id\":\"hetero-trace:data:none:none\""));
    assert!(json.contains("\"trace_role\":\"domain-metadata\""));
    assert!(json.contains("\"status\":\"metadata-only\""));
    assert!(json.contains("\"trace_id\":\"hetero-trace:shader:metal:apple-silicon-gpu\""));
    assert!(json.contains("\"trace_role\":\"backend-artifact\""));
    assert!(json.contains("\"status\":\"execution-pending\""));
    assert!(json.contains("\"device_sample_provider\":\"nustar-deferred-device-sample-v1\""));
    assert!(json.contains("\"device_sample_provider_family\":\"metal:apple-silicon-gpu\""));
    assert!(json.contains("\"device_sample_kind\":\"deferred-provider-sample-descriptor\""));
    assert!(json.contains("\"device_sample_status\":\"device-execution-pending\""));
    assert!(json.contains("\"device_sample_schema\":\"nsdb-yir-device-execution-sample-v1\""));
    assert!(json.contains("\"device_sample_input_evidence\":\"ndpb-v2:"));
    assert!(json.contains("\"device_sample_output_evidence\":\"not-materialized\""));
    assert!(json.contains("\"device_sample_validation_status\":\"pending-provider-execution\""));
    assert!(json.contains("\"device_sample_handoff_target\":\"metal:apple-silicon-gpu\""));
    assert!(json.contains("\"device_sample_handoff_status\":\"awaiting-provider-handoff\""));
    assert!(json.contains("\"device_sample_next_action\":\"materialize-device-execution-sample\""));
    assert!(json.contains("\"next_action\":\"materialize-device-execution-trace\""));
    assert!(json.contains("\"hetero_runtime_trace_first_blocker\":null"));
    assert!(json
        .contains("\"hetero_runtime_trace_next_action\":\"materialize-device-execution-trace\""));
    let persisted_trace =
        fs::read_to_string(output_dir.join("nuis.nsdb.hetero-runtime-trace.toml"))
            .expect("hetero runtime trace metadata is persisted");
    assert!(persisted_trace.contains("protocol = \"nuis-nsdb-hetero-runtime-trace-v1\""));
    assert!(persisted_trace.contains("debugger_contract = \"nsdb-yir-hetero-runtime-trace-v1\""));
    assert!(persisted_trace.contains("device_sample_descriptor_count = 1"));
    assert!(persisted_trace.contains("device_sample_pending_count = 1"));
    assert!(persisted_trace.contains("device_sample_pending_validation_count = 1"));
    assert!(persisted_trace.contains("device_sample_handoff_record_count = 1"));
    assert!(persisted_trace
        .contains("device_sample_handoff_protocol = \"nuis-device-sample-provider-handoff-v1\""));
    assert!(persisted_trace
        .contains("device_sample_providers = [\"nustar-deferred-device-sample-v1\"]"));
    assert!(
        persisted_trace.contains("device_sample_provider_families = [\"metal:apple-silicon-gpu\"]")
    );
    assert!(persisted_trace
        .contains("device_sample_validation_statuses = [\"pending-provider-execution\"]"));
    assert!(persisted_trace.contains("device_sample_handoff_status = \"provider-handoff-pending\""));
    assert!(persisted_trace
        .contains("device_sample_first_pending_provider_family = \"metal:apple-silicon-gpu\""));
    assert!(persisted_trace.contains("[[device_sample_handoffs]]"));
    assert!(persisted_trace.contains("protocol = \"nuis-device-sample-provider-handoff-v1\""));
    assert!(persisted_trace.contains("provider_family = \"metal:apple-silicon-gpu\""));
    assert!(persisted_trace.contains("handoff_status = \"awaiting-provider-handoff\""));
    assert!(persisted_trace.contains("[[records]]"));
    assert!(persisted_trace.contains("trace_id = \"hetero-trace:shader:metal:apple-silicon-gpu\""));
    assert!(
        persisted_trace.contains("device_sample_provider = \"nustar-deferred-device-sample-v1\"")
    );
    assert!(persisted_trace.contains("device_sample_provider_family = \"metal:apple-silicon-gpu\""));
    assert!(
        persisted_trace.contains("device_sample_kind = \"deferred-provider-sample-descriptor\"")
    );
    assert!(
        persisted_trace.contains("device_sample_schema = \"nsdb-yir-device-execution-sample-v1\"")
    );
    assert!(persisted_trace.contains("device_sample_output_evidence = \"not-materialized\""));
    assert!(persisted_trace
        .contains("device_sample_validation_status = \"pending-provider-execution\""));
    assert!(persisted_trace.contains("device_sample_handoff_target = \"metal:apple-silicon-gpu\""));
    assert!(
        persisted_trace.contains("device_sample_handoff_status = \"awaiting-provider-handoff\"")
    );
    assert!(persisted_trace.contains("next_action = \"materialize-device-execution-trace\""));
    let decoder_manifest = fs::read_to_string(output_dir.join("nuis.nsdb.payload-decoders.toml"))
        .expect("payload decoder manifest is persisted");
    assert!(decoder_manifest.contains("protocol = \"nuis-nsdb-payload-decoders-v1\""));
    assert!(decoder_manifest.contains("schema = \"nsdb-payload-decoder-manifest-v1\""));
    assert!(decoder_manifest.contains("[[decoders]]"));
    assert!(decoder_manifest.contains("payload_format = \"ndpb-v2\""));
    assert!(decoder_manifest.contains("decoder_capability = \"opaque-file-summary\""));
    let provider_samples =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml"))
            .expect("device provider sample manifest is persisted");
    assert!(provider_samples.contains("protocol = \"nuis-device-provider-samples-v1\""));
    assert!(provider_samples.contains("schema = \"nsdb-yir-device-provider-sample-v1\""));
    assert!(provider_samples.contains("status = \"awaiting-provider-materialization\""));
    assert!(provider_samples.contains("pending_record_count = 1"));
    assert!(provider_samples.contains("[[device_provider_samples]]"));
    assert!(provider_samples.contains("trace_id = \"hetero-trace:shader:metal:apple-silicon-gpu\""));
    assert!(provider_samples.contains("provider_family = \"metal:apple-silicon-gpu\""));
    assert!(provider_samples.contains("materialization_status = \"provider-sample-pending\""));

    let doctor_json = render_artifact_doctor_json(&output_dir);
    assert!(doctor_json.contains("\"artifact_payload_decoder_manifest_available\":true"));
    assert!(doctor_json.contains("\"artifact_payload_decoder_manifest_path\":\""));
    assert!(doctor_json.contains(
        "\"artifact_payload_decoder_manifest_protocol\":\"nuis-nsdb-payload-decoders-v1\""
    ));
    assert!(doctor_json.contains(
        "\"artifact_payload_decoder_manifest_schema\":\"nsdb-payload-decoder-manifest-v1\""
    ));
    assert!(doctor_json.contains("\"artifact_payload_decoder_manifest_status\":\"ready\""));
    assert!(doctor_json.contains("\"artifact_payload_decoder_manifest_record_count\":1"));
    assert!(doctor_json.contains("\"artifact_payload_decoder_manifest_invalid_record_count\":0"));
    assert!(doctor_json.contains(
        "\"artifact_payload_decoder_manifest_first_diagnostic\":\"manifest-external-decoder-loaded\""
    ));
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
