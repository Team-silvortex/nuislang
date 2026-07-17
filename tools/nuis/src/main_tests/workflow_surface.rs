use super::*;

#[test]
fn workflow_docs_define_closure_then_tensor_reading_order() {
    let workflow_doc = include_str!("../../../../docs/reference/nuis-native-artifact-workflow.md");
    assert!(workflow_doc.contains("`closure_summary_*` is the canonical human closure line"));
    assert!(workflow_doc.contains(
        "frontdoor_reading_order: closure_summary -> dev_tensor_weakest_task_card_handoff"
    ));
    assert!(workflow_doc.contains("frontdoor_sample_closure_summary"));
    assert!(workflow_doc.contains("frontdoor_sample_tensor_handoff"));
    assert!(workflow_doc.contains("dev_tensor_weakest_task_card_*"));
    assert!(workflow_doc.contains("dev_tensor_weakest_task_card_handoff_*"));
    assert!(workflow_doc.contains("artifact closure work and tensor-driven"));
}

#[test]
fn workflow_json_reports_frontdoor_and_artifact_fields_for_project() {
    let project_root = write_temp_project_fixture(
        "workflow_json_smoke",
        r#"
name = "workflow_json_smoke"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 6;
  }
}
"#,
    );
    let tests_dir = project_root.join("tests");
    fs::create_dir_all(&tests_dir).expect("create tests dir");
    fs::write(
        tests_dir.join("smoke.ns"),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
    )
    .expect("write smoke test");

    let json = render_workflow_json(&project_root).expect("render workflow json");
    let output_dir = default_build_output_dir(&project_root);

    assert!(json.contains("\"source_kind\":\"project\""));
    assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
    assert!(json.contains(&format!(
        "\"default_build_output_dir\":\"{}\"",
        output_dir.display()
    )));
    assert!(json.contains("\"artifact_workflow\":\"build -> inspect_artifact -> verify_artifact -> artifact_doctor -> nsld_drive -> verify_build_manifest -> run_artifact\""));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_dry_run_command\":\"nsld drive {}/nuis.build.manifest.toml\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_dry_run_json_command\":\"nsld drive {}/nuis.build.manifest.toml --json\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_next_command\":\"nsld drive {}/nuis.build.manifest.toml --apply\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_next_json_command\":\"nsld drive {}/nuis.build.manifest.toml --apply --json\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_until_clean_command\":\"nsld drive {}/nuis.build.manifest.toml --apply --until-clean\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_until_clean_json_command\":\"nsld drive {}/nuis.build.manifest.toml --apply --until-clean --json\"",
        output_dir.display()
    )));
    assert!(json.contains("\"artifact_nsld_drive_command_set\":{"));
    assert!(json.contains("\"protocol\":\"nsld-drive-command-set-v1\""));
    assert!(json.contains("\"safe_next_contract\":\"nsld-drive-safe-next-v1\""));
    assert!(json.contains("\"recommended_first_json_command\":\"nsld drive "));
    assert!(json.contains("\"safe_next_probe_json_command\":\"nsld drive "));
    assert!(json.contains("\"safe_next_action_field\":\"safe_next_action\""));
    assert!(json.contains("\"safe_next_command_field\":\"safe_next_command\""));
    assert!(json.contains("\"safe_next_gate_required_field\":\"safe_next_gate_required\""));
    assert!(json.contains("\"safe_next_gate_action_field\":\"safe_next_gate_action\""));
    assert!(json.contains("\"dry_run_mutates_artifacts\":false"));
    assert!(json.contains("\"apply_next_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_until_clean_mutates_artifacts\":true"));
    assert!(json.contains("\"dry_run_json_command\":\"nsld drive "));
    assert!(json.contains("\"apply_next_json_command\":\"nsld drive "));
    assert!(json.contains("\"apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains("\"artifact_ready_to_run\":false"));
    assert!(json.contains("\"artifact_diagnostic_code\":\"missing_outputs\""));
    assert!(json.contains("\"artifact_self_check_ready\":false"));
    assert!(json.contains("\"artifact_self_check_code\":\"missing_build_manifest\""));
    assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
    assert!(json.contains(&format!(
        "\"artifact_self_check_error\":\"`{}` does not contain `nuis.build.manifest.toml`\"",
        output_dir.display()
    )));
    assert!(json.contains("\"project_checks_available\":true"));
    assert!(json.contains("\"project_checks_code\":\"ok\""));
    assert!(json.contains("\"abi_checks_ok\":true"));
    assert!(json.contains("\"registry_checks_ok\":true"));
    assert!(json.contains("\"lowering_checks_ok\":true"));
    assert!(json.contains("\"link_plan_available\":false"));
    assert!(json.contains("\"link_plan_final_stage\":null"));
    assert!(json.contains("\"link_plan_lowering_plan_index_source\":null"));
    assert!(json.contains("\"compile_pipeline_available\":true"));
    assert!(json.contains("\"compile_pipeline_source_kind\":\"project\""));
    assert!(json.contains("\"compile_pipeline_ready_for_aot\":true"));
    assert!(json.contains("\"compile_pipeline_recommended_next_step\":\"build\""));
    assert!(json.contains("\"compile_pipeline_stage_count\":"));
    assert!(json.contains("\"compile_pipeline_ok_stage_count\":"));
    assert!(json.contains("\"id\":\"yir_lower\""));
    assert!(json.contains("\"id\":\"llvm_emit\""));
}

#[test]
fn workflow_json_reports_link_plan_for_built_project_output() {
    let project_root = write_temp_project_fixture(
        "workflow_json_built_smoke",
        r#"
name = "workflow_json_built_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 8;
  }
}
"#,
    );
    let output_dir = default_build_output_dir(&project_root);

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");

    let json = render_workflow_json(&project_root).expect("render workflow json");

    assert!(json.contains("\"artifact_ready_to_run\":true"));
    assert!(json.contains("\"artifact_diagnostic_code\":\"ready_to_run\""));
    assert!(json.contains("\"artifact_self_check_ready\":true"));
    assert!(json.contains("\"artifact_self_check_code\":\"ok\""));
    assert!(json.contains("\"artifact_recommended_next_step\":\"run_artifact\""));
    assert!(json.contains("\"artifact_self_check_error\":null"));
    assert!(json.contains("\"project_checks_available\":true"));
    assert!(json.contains("\"project_checks_code\":\"ok\""));
    assert!(json.contains("\"abi_checks_ok\":true"));
    assert!(json.contains("\"registry_checks_ok\":true"));
    assert!(json.contains("\"lowering_checks_ok\":true"));
    assert!(json.contains("\"link_plan_available\":true"));
    assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
    assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
    assert!(json.contains("\"link_plan_final_link_mode\":\"host-toolchain-finalize\""));
    assert!(json.contains("\"link_plan_lowering_plan_index_source\":\"compiled_artifact_section\""));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_units\":0"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_ready_units\":0"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_first_unready\":null"));
    assert!(json.contains("\"nsld_backend_artifact_payload_evidence_available\":"));
    assert!(json.contains("\"nsld_backend_artifact_payload_count\":"));
    assert!(json.contains("\"nsld_backend_artifact_payload_role_status\":"));
    assert!(json.contains("\"nsld_prepare_command\":\"nsld prepare "));
    assert!(json.contains("\"nsld_drive_dry_run_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_dry_run_json_command\":\"nsld drive "));
    assert!(json.contains(" --json\""));
    assert!(json.contains("\"nsld_drive_apply_next_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_apply_next_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --json\""));
    assert!(json.contains("\"nsld_drive_apply_until_clean_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --until-clean --json\""));
    assert!(json.contains("\"nsld_drive_command_set\":{"));
    assert!(json.contains("\"protocol\":\"nsld-drive-command-set-v1\""));
    assert!(json.contains("\"recommended_first_json_command\":\"nsld drive "));
    assert!(json.contains("\"dry_run_mutates_artifacts\":false"));
    assert!(json.contains("\"apply_next_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_until_clean_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_next_json_command\":\"nsld drive "));
    assert!(json.contains("\"apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains("\"nsld_drive_recommended_mode\":\"apply-next\""));
    assert!(json.contains("\"nsld_drive_recommended_mutates_artifacts\":true"));
    assert!(json.contains("\"closure_summary_source\":\"workflow-link-plan\""));
    assert!(json.contains("\"closure_summary_next_action\":\"nsld-drive-safe-next\""));
    assert!(json.contains("\"closure_summary_next_command\":\"nsld drive "));
    assert!(json.contains(" --apply --until-clean --json\""));
    assert!(
        json.contains("nsld-drive-safe-next-v1 gate required before mutating artifact-chain state")
    );
    assert!(json.contains("\"nsld_prepared_artifact_chain_ready\":false"));
    assert!(json.contains("\"nsld_prepared_artifact_next_missing_stage\":\"link-inputs\""));
    assert!(json.contains(
        "\"nsld_prepared_artifact_stage_records\":[{\"stage\":\"link-inputs\",\"file\":\"nuis.nsld.link-inputs.toml\",\"present\":false,\"required\":true,\"next_action_source\":\"required\",\"command_id\":\"emit-inputs\""
    ));
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
    assert!(json.contains(
        "\"nsld_artifact_chain_next_action_reason\":\"first missing required artifact stage `link-inputs`\""
    ));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_command\":\"nsld emit-final-executable-pipeline "
    ));
    assert!(json.contains("\"nsld_final_executable_tail_ready\":false"));
    assert!(json.contains(
        "\"nsld_final_executable_tail_next_missing_stage\":\"final-executable-writer-input\""
    ));
    assert!(json.contains("\"nsld_final_executable_pipeline_scheduler_metadata_payload_id\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_scheduler_metadata_present\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_scheduler_metadata_hash\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_final_executable_emitted\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_launcher_manifest_ready\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_launcher_dry_run_ready\":null"));
    assert!(json.contains("\"nsld_final_executable_pipeline_would_enter_lifecycle_hook\":null"));
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
    assert!(json.contains("\"nsld_final_executable_pipeline_required_stage_path_count\":null"));
    assert!(
        json.contains("\"nsld_final_executable_pipeline_required_stage_path_present_count\":null")
    );
    assert!(
        json.contains("\"nsld_final_executable_pipeline_first_missing_required_stage_path\":null")
    );
    assert!(json.contains("\"nsld_self_owned_image_status\":"));
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
    assert!(json.contains("\"nsld_final_executable_output_payload_execution_trace_protocol\":"));
    assert!(json.contains("\"nsld_final_executable_output_payload_execution_trace_available\":"));
    assert!(json.contains("\"nsld_final_executable_output_payload_execution_trace_record_count\":"));
    assert!(json
        .contains("\"nsld_final_executable_output_payload_execution_trace_ready_record_count\":"));
    assert!(json
        .contains("\"nsld_final_executable_output_device_provider_sample_manifest_available\":"));
    assert!(
        json.contains("\"nsld_final_executable_output_device_provider_sample_manifest_status\":")
    );
    assert!(json.contains(
        "\"nsld_final_executable_output_device_provider_sample_manifest_blocked_record_count\":"
    ));
    assert!(json.contains("\"nsld_final_executable_output_recommended_next_action\":"));
    assert!(json.contains("\"nsld_final_executable_output_nsld_owned\":null"));
    assert!(json.contains("\"nsld_final_executable_output_object_valid\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_path\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_family\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_magic_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_magic\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_expected_size_bytes\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_actual_size_bytes\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_expected_hash\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_actual_hash\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_issues\":["));
    assert!(json.contains("\"nsld_final_executable_output_blocker_count\":"));
    assert!(json.contains("\"nsld_final_executable_output_blockers\":["));
    assert!(json.contains("\"nsld_final_executable_output_first_blocker\":"));
    assert!(json.contains("\"compile_pipeline_available\":true"));
    assert!(json.contains("\"compile_pipeline_ready_for_aot\":true"));
    assert!(json.contains("\"compile_pipeline_summary\":\"source_kind=project"));
}

#[test]
fn workflow_json_reports_ready_nsld_next_action_for_complete_final_tail() {
    let project_root = write_temp_project_fixture(
        "workflow_json_ready_nsld_smoke",
        r#"
name = "workflow_json_ready_nsld_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 8;
  }
}
"#,
    );
    let output_dir = default_build_output_dir(&project_root);

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    write_prepared_nsld_chain_placeholders(&output_dir);
    write_ready_nsld_final_tail_placeholders(&output_dir);
    write_nsdb_payload_handoff_placeholder(&output_dir);

    let json = render_workflow_json(&project_root).expect("render workflow json");

    assert!(json.contains("\"nsld_prepared_artifact_chain_ready\":true"));
    assert!(json.contains("\"nsld_final_executable_tail_ready\":true"));
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
    assert!(json.contains("\"nsld_artifact_chain_next_action_command_id\":null"));
    assert!(json.contains("\"nsld_drive_recommended_mode\":\"dry-run\""));
    assert!(json.contains("\"nsld_drive_recommended_mutates_artifacts\":false"));
    assert!(json.contains(
        "\"nsld_drive_recommended_reason\":\"artifact-chain has no mutating next action; inspect the final executable output boundary blocked by `final-executable-output:ownership-unknown`\""
    ));
    assert!(json.contains("\"nsld_final_executable_pipeline_valid\":true"));
    assert!(
        json.contains("\"nsld_final_executable_pipeline_required_stage_path_present_count\":10")
    );
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
    assert!(json.contains("\"workflow_run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"workflow_run_artifact_prelaunch_status\":\"ready\""));
    assert!(
        json.contains("\"workflow_run_artifact_prelaunch_evidence_status\":\"entrypoint-ready\"")
    );
    assert!(json.contains(
        "\"workflow_run_artifact_prelaunch_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(json.contains(
        "\"workflow_run_artifact_prelaunch_reason\":\"nsld final executable pipeline materialized a verified host entrypoint stub\""
    ));
    assert!(json.contains("\"closure_summary_source\":\"workflow-link-plan\""));
    assert!(json.contains("\"closure_summary_status\":\"blocked\""));
    assert!(json.contains("\"closure_summary_ready\":false"));
    assert!(json.contains(
        "\"closure_summary_primary_blocker\":\"final executable output boundary is blocked by `final-executable-output:ownership-unknown`\""
    ));
    assert!(json.contains("\"closure_summary_next_action\":\"inspect-final-executable-output\""));
    assert!(json.contains("\"closure_summary_next_command\":\"nsld final-executable-output "));
    assert!(json.contains(
        "\"workflow_launch_evidence_protocol\":\"nuis-run-artifact-launch-evidence-v1\""
    ));
    assert!(json.contains("\"workflow_launch_evidence_status\":\"blocked\""));
    assert!(json.contains("\"workflow_launch_evidence_route\":\"nsld-host-entrypoint\""));
    assert!(json.contains("\"workflow_launch_evidence_status_code\":\"entrypoint-ready\""));
    assert!(json.contains(
        "\"workflow_launch_evidence_debugger_contract\":\"nsdb-yir-launch-evidence-v1\""
    ));
    assert!(
        json.contains("\"workflow_launch_evidence_host_runner_probe_status\":\"workflow-mirror\"")
    );
    assert!(json.contains(
        "\"workflow_launch_evidence_first_blocker\":\"host-runner-probe:workflow-mirror\""
    ));
    assert!(json.contains("\"workflow_nsdb_handoff_available\":true"));
    assert!(json
        .contains("\"workflow_nsdb_handoff_protocol\":\"nuis-nsdb-payload-execution-handoff-v1\""));
    assert!(json.contains(
        "\"workflow_nsdb_handoff_debugger_contract\":\"nsdb-yir-payload-execution-trace-v1\""
    ));
    assert!(json.contains("\"workflow_nsdb_handoff_record_count\":1"));
    assert!(json.contains("\"workflow_nsdb_handoff_ready_record_count\":1"));
    assert!(json.contains(
        "\"workflow_nsdb_handoff_first_trace_id\":\"payload-trace:container-loader:nuis.bootstrap.lifecycle.v1\""
    ));
    assert!(json.contains("\"workflow_nsdb_handoff_first_status\":\"ready\""));
    assert!(json
        .contains("\"workflow_nsdb_handoff_first_next_action\":\"handoff-payload-trace-to-nsdb\""));
    assert!(json.contains(
        "\"nsld_final_executable_output_nsdb_replay_contract\":\"nsdb-payload-execution-replay-plan-v1\""
    ));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_ready\":true"));
    assert!(json
        .contains("\"nsld_final_executable_output_nsdb_replay_status\":\"replay-evidence-ready\""));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_checkpoint_count\":1"));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replayable_checkpoint_count\":1"));
    assert!(
        json.contains("\"nsld_final_executable_output_nsdb_replay_command\":\"nsdb replay-plan ")
    );
    assert!(json.contains(
        "\"nsld_final_executable_output_nsdb_replay_next_action\":\"replay-nsdb-payload-execution\""
    ));
    assert!(json
        .contains("\"nsld_final_executable_output_nsdb_replay_next_command\":\"nsdb replay-plan "));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_first_blocker\":null"));
    for needle in [
        "\"nsld_final_executable_output_object_package_contract\":\"nsld-object-package-summary-v1\"",
        "\"nsld_final_executable_output_object_package_ready\":true",
        "\"nsld_final_executable_output_object_package_status\":\"replay-ready\"",
        "\"nsld_final_executable_output_debugger_transcript_contract\":\"nsdb-yir-replay-transcript-v1\"",
        "\"nsld_final_executable_output_debugger_transcript_ready\":true",
        "\"nsld_final_executable_output_debugger_transcript_status\":\"transcript-ready\"",
        "\"closure_summary_object_package_ready\":true",
        "\"closure_summary_debugger_transcript_contract\":\"nsdb-yir-replay-transcript-v1\"",
        "\"closure_summary_debugger_transcript_ready\":true",
        "\"closure_summary_debugger_transcript_status\":\"transcript-ready\"",
    ] {
        assert!(json.contains(needle));
    }
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
    assert!(json.contains("\"nsld_final_executable_output_payload_execution_trace_protocol\":"));
    assert!(json.contains("\"nsld_final_executable_output_payload_execution_trace_available\":"));
    assert!(json.contains("\"nsld_final_executable_output_payload_execution_trace_record_count\":"));
    assert!(json
        .contains("\"nsld_final_executable_output_payload_execution_trace_ready_record_count\":"));
    assert!(json.contains("\"nsld_final_executable_output_recommended_next_action\":"));
    assert!(json.contains("\"nsld_final_executable_output_path_present\":"));
    assert!(json.contains("\"nsld_final_executable_output_nsld_owned\":null"));
    assert!(json.contains("\"nsld_final_executable_output_object_valid\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_path\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_family\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_magic_status\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_magic\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_expected_size_bytes\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_actual_size_bytes\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_expected_hash\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_actual_hash\":"));
    assert!(json.contains("\"nsld_final_executable_output_object_issues\":["));
    assert!(json.contains("\"nsld_final_executable_output_blocker_count\":"));
    assert!(json.contains("\"nsld_final_executable_output_blockers\":["));
}

#[test]
fn workflow_json_blocks_replay_when_hetero_closure_is_pending() {
    let project_root = write_temp_project_fixture(
        "workflow_json_pending_hetero_closure",
        r#"
name = "workflow_json_pending_hetero_closure"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 9;
  }
}
"#,
    );
    let output_dir = default_build_output_dir(&project_root);
    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
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

    let json = render_workflow_json(&project_root).expect("render workflow json");

    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_ready\":false"));
    assert!(json.contains("\"nsld_final_executable_output_nsdb_replay_status\":\"blocked\""));
    assert!(json.contains(
        "\"nsld_final_executable_output_nsdb_replay_first_blocker\":\"hetero-execution-closure:host-runner-backend-artifact-payload:not-observed\""
    ));
}

#[test]
fn workflow_json_reports_frontdoor_and_artifact_fields_for_single_source() {
    let dir = temp_dir("workflow_json_single_source");
    let input = dir.join("hello.ns");
    fs::write(
        &input,
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
    )
    .expect("write source");

    let json = render_workflow_json(&input).expect("render workflow json");
    let output_dir = default_build_output_dir(&input);

    assert!(json.contains("\"source_kind\":\"single-file\""));
    assert!(json.contains("\"workflow_kind\":\"compile_workflow\""));
    assert!(json.contains(&format!(
        "\"default_build_output_dir\":\"{}\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_dry_run_command\":\"nsld drive {}/nuis.build.manifest.toml\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_dry_run_json_command\":\"nsld drive {}/nuis.build.manifest.toml --json\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_next_command\":\"nsld drive {}/nuis.build.manifest.toml --apply\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_next_json_command\":\"nsld drive {}/nuis.build.manifest.toml --apply --json\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_until_clean_command\":\"nsld drive {}/nuis.build.manifest.toml --apply --until-clean\"",
        output_dir.display()
    )));
    assert!(json.contains(&format!(
        "\"artifact_nsld_drive_apply_until_clean_json_command\":\"nsld drive {}/nuis.build.manifest.toml --apply --until-clean --json\"",
        output_dir.display()
    )));
    assert!(json.contains("\"artifact_nsld_drive_command_set\":{"));
    assert!(json.contains("\"protocol\":\"nsld-drive-command-set-v1\""));
    assert!(json.contains("\"safe_next_contract\":\"nsld-drive-safe-next-v1\""));
    assert!(json.contains("\"safe_next_probe_json_command\":\"nsld drive "));
    assert!(json.contains("\"safe_next_gate_required_field\":\"safe_next_gate_required\""));
    assert!(json.contains("\"artifact_ready_to_run\":false"));
    assert!(json.contains("\"artifact_diagnostic_code\":\"missing_outputs\""));
    assert!(json.contains("\"artifact_self_check_ready\":false"));
    assert!(json.contains("\"artifact_self_check_code\":\"missing_build_manifest\""));
    assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
    assert!(json.contains(&format!(
        "\"artifact_self_check_error\":\"`{}` does not contain `nuis.build.manifest.toml`\"",
        output_dir.display()
    )));
    assert!(json.contains("\"project_checks_available\":false"));
    assert!(json.contains("\"project_checks_code\":\"unavailable\""));
    assert!(json.contains("\"link_plan_available\":false"));
    assert!(json.contains("\"link_plan_final_stage\":null"));
    assert!(json.contains("\"link_plan_lowering_plan_index_source\":null"));
    assert!(json.contains("\"compile_pipeline_available\":true"));
    assert!(json.contains("\"compile_pipeline_source_kind\":\"single_source\""));
    assert!(json.contains("\"compile_pipeline_ready_for_aot\":true"));
    assert!(json.contains("\"compile_pipeline_recommended_next_step\":\"build\""));
    assert!(json.contains("\"compile_pipeline_stage_count\":"));
    assert!(json.contains("\"compile_pipeline_ok_stage_count\":"));
}

#[test]
fn workflow_json_reports_self_check_failure_for_damaged_output_dir() {
    let project_root = write_temp_project_fixture(
        "workflow_json_damaged_output",
        r#"
name = "workflow_json_damaged_output"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 9;
  }
}
"#,
    );
    let output_dir = default_build_output_dir(&project_root);
    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    fs::remove_file(output_dir.join("nuis.compiled.artifact")).expect("remove artifact");

    let json = render_workflow_json(&project_root).expect("render workflow json");

    assert!(json.contains("\"artifact_ready_to_run\":false"));
    assert!(json.contains("\"artifact_diagnostic_code\":\"manifest_invalid\""));
    assert!(json.contains("\"artifact_self_check_ready\":false"));
    assert!(json.contains("\"artifact_self_check_code\":\"manifest_verify_failed\""));
    assert!(json.contains("\"artifact_recommended_next_step\":\"verify_build_manifest\""));
    assert!(json.contains("\"project_checks_code\":\"ok\""));
    assert!(
        json.contains("\"artifact_self_check_error\":\"build self-check could not verify manifest")
    );
}

#[test]
fn project_workflow_recommendation_prefers_lock_abi_for_auto_projects() {
    let project_root = write_temp_project_fixture(
        "workflow_auto_abi",
        r#"
name = "workflow_auto_abi"
entry = "main.ns"
modules = ["main.ns"]
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
    let project = nuisc::project::load_project(&project_root).expect("load project");
    let plan =
        nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
    let doctor = empty_galaxy_doctor(&project.root);

    let recommendation = recommend_project_workflow_step(&plan, &[], &[], &doctor, false, false);

    assert_eq!(recommendation.label, "project_lock_abi");
    assert_eq!(
        recommendation.command,
        "nuis project-lock-abi <project-dir|nuis.toml>"
    );
}

#[test]
fn project_workflow_recommendation_prefers_project_status_for_missing_tests() {
    let project_root = write_temp_project_fixture(
        "workflow_missing_tests",
        r#"
name = "workflow_missing_tests"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
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
    let project = nuisc::project::load_project(&project_root).expect("load project");
    let plan =
        nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
    let doctor = empty_galaxy_doctor(&project.root);
    let missing_tests = vec![project.root.join("tests/smoke.ns")];

    let recommendation = recommend_project_workflow_step(
        &plan,
        &missing_tests,
        &missing_tests,
        &doctor,
        false,
        false,
    );

    assert_eq!(recommendation.label, "project_status");
    assert_eq!(
        recommendation.command,
        "nuis project-status <project-dir|nuis.toml>"
    );
}

#[test]
fn project_workflow_recommendation_defaults_to_check_once_shape_is_stable() {
    let project_root = write_temp_project_fixture(
        "workflow_ready",
        r#"
name = "workflow_ready"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
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
    let tests_dir = project_root.join("tests");
    fs::create_dir_all(&tests_dir).expect("create tests dir");
    fs::write(
        tests_dir.join("smoke.ns"),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
    )
    .expect("write smoke test");
    let project = nuisc::project::load_project(&project_root).expect("load project");
    let plan =
        nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
    let doctor = empty_galaxy_doctor(&project.root);
    let declared_tests = vec![project.root.join("tests/smoke.ns")];

    let recommendation =
        recommend_project_workflow_step(&plan, &declared_tests, &[], &doctor, false, false);

    assert_eq!(recommendation.label, "check");
    assert_eq!(recommendation.command, "nuis check <project-dir|nuis.toml>");
}

#[test]
fn project_workflow_recommendation_prefers_project_imports_apply_for_hidden_manual_only_modules() {
    let project_root = write_temp_project_fixture(
        "workflow_manual_only_imports",
        r#"
name = "workflow_manual_only_imports"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
galaxy = ["ns-nova=workspace"]
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
    let tests_dir = project_root.join("tests");
    fs::create_dir_all(&tests_dir).expect("create tests dir");
    fs::write(
        tests_dir.join("smoke.ns"),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 2;
  }
}
"#,
    )
    .expect("write smoke test");
    let project = nuisc::project::load_project(&project_root).expect("load project");
    let plan =
        nuisc::project::build_project_compilation_plan(&project).expect("build project plan");
    let mut doctor = empty_galaxy_doctor(&project.root);
    doctor.lock_status = "ok".to_owned();
    let declared_tests = vec![project.root.join("tests/smoke.ns")];

    let recommendation =
        recommend_project_workflow_step(&plan, &declared_tests, &[], &doctor, false, true);

    assert_eq!(recommendation.label, "project_imports_apply_suggested");
    assert_eq!(
        recommendation.command,
        "nuis project-imports --apply-suggested <project-dir|nuis.toml>"
    );
}

#[test]
fn project_workflow_json_fields_track_compile_and_galaxy_briefs() {
    let frontdoor = build_workflow_frontdoor_surface(
        project_compile_workflow_source_profile(),
        WorkflowRecommendation {
            label: "check",
            command: "nuis check <project-dir|nuis.toml>",
            reason: "compile truth should remain the default once the project shape is stable",
        },
    );

    let without_galaxy = project_workflow_json_fields(&frontdoor, false);
    assert!(without_galaxy.iter().any(|field| {
        field
            == &format!(
                "\"project_compile_workflow\":\"{}\"",
                nuisc::project_compile_workflow_brief()
            )
    }));
    assert!(without_galaxy.iter().any(|field| {
        field
            == &format!(
                "\"project_test_workflow\":\"{}\"",
                nuisc::project_test_workflow_brief()
            )
    }));
    assert!(!without_galaxy
        .iter()
        .any(|field| field.contains("\"project_galaxy_workflow\"")));

    let with_galaxy = project_workflow_json_fields(&frontdoor, true);
    assert!(with_galaxy.iter().any(|field| {
        field
            == &format!(
                "\"project_galaxy_workflow\":\"{}\"",
                nuisc::project_galaxy_workflow_brief()
            )
    }));
}

#[test]
fn workflow_contract_json_fields_expose_shared_frontdoor_keys() {
    let frontdoor = build_workflow_frontdoor_surface(
        project_compile_workflow_source_profile(),
        WorkflowRecommendation {
            label: "check",
            command: "nuis check <project-dir|nuis.toml>",
            reason: "shared workflow contract should always carry the frontdoor routing fields",
        },
    );

    let fields = workflow_contract_json_fields(&frontdoor, true, true, true, true);

    for key in [
            "\"frontdoor\":{",
            "\"workflow_kind\":\"project_compile_workflow\"",
            "\"workflow_brief\":\"",
            "\"workflow_samples\":\"",
            "\"recommended_next_step\":\"check\"",
            "\"recommended_command\":\"nuis check <project-dir|nuis.toml>\"",
            "\"recommended_reason\":\"shared workflow contract should always carry the frontdoor routing fields\"",
            "\"project_compile_workflow\":\"",
            "\"project_compile_samples\":\"",
            "\"project_test_workflow\":\"",
            "\"project_galaxy_workflow\":\"",
            "\"debug_workflow\":\"",
            "\"debug_samples\":\"",
        ] {
            assert!(
                fields.iter().any(|field| field.contains(key)),
                "missing shared workflow contract key {key}"
            );
        }
}

#[test]
fn galaxy_lock_json_fields_report_missing_lock_surface() {
    let dir = temp_dir("galaxy_lock_fields_missing");
    let lock_path = dir.join("nuis.galaxy.lock");

    let fields = galaxy_lock_json_fields(Err("missing".to_owned()), &lock_path, &[]);

    assert!(fields
        .iter()
        .any(|field| field == "\"galaxy_lock_status\":\"missing\""));
    assert!(fields
        .iter()
        .any(|field| field.contains("\"galaxy_lock_path\":\"")));
    assert!(!fields
        .iter()
        .any(|field| field.contains("\"galaxy_lock_error\"")));
}

#[test]
fn public_surface_summary_json_fields_count_public_members() {
    let records = vec![PublicSurfaceModuleRecord {
        module: "cpu::Main".to_owned(),
        externs: vec!["ffi_print".to_owned()],
        extern_interfaces: vec!["ClockBridge".to_owned()],
        consts: vec!["DEFAULT_PORT".to_owned()],
        type_aliases: vec!["ResultCode".to_owned()],
        functions: vec!["run".to_owned(), "tick".to_owned()],
        structs: vec!["State(fields=1)".to_owned()],
        traits: vec!["Runnable".to_owned()],
    }];

    let fields = public_surface_summary_json_fields(&records);

    assert!(fields
        .iter()
        .any(|field| field == "\"public_surface_modules\":1"));
    assert!(fields.iter().any(|field| field == "\"public_externs\":1"));
    assert!(fields
        .iter()
        .any(|field| field == "\"public_extern_interfaces\":1"));
    assert!(fields.iter().any(|field| field == "\"public_consts\":1"));
    assert!(fields
        .iter()
        .any(|field| field == "\"public_type_aliases\":1"));
    assert!(fields.iter().any(|field| field == "\"public_functions\":2"));
    assert!(fields.iter().any(|field| field == "\"public_structs\":1"));
    assert!(fields.iter().any(|field| field == "\"public_traits\":1"));
}
