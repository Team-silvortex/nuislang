use super::*;

#[test]
fn project_imports_json_reports_explicit_manual_only_library_as_visible() {
    let project_root = write_temp_project_fixture(
        "imports_explicit_manual_only",
        r#"
name = "imports_explicit_manual_only"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]
"#
        .trim_start(),
        r#"
use cpu NovaContracts;

mod cpu Main {
  fn main() -> i64 {
    return NovaContracts.runtime_score(16, 4, 3, 2, 9, 1);
  }
}
"#,
    );

    let json = render_project_imports_json(&project_root).expect("render imports json");

    assert!(json.contains("\"explicit_galaxy_imports_count\":1"));
    assert!(json.contains("\"explicit_galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
    assert!(json.contains("\"hidden_manual_only_library_modules_count\":0"));
    assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
    assert!(json.contains("\"visible\":true"));
    assert!(json.contains("\"explicit\":true"));
    assert!(json.contains("\"source_kind\":\"galaxy-explicit-import\""));
}

#[test]
fn apply_suggested_project_imports_adds_manifest_field_when_missing() {
    let project_root = write_temp_project_fixture(
        "imports_apply_missing_field",
        r#"
name = "imports_apply_missing_field"
entry = "main.ns"
modules = ["main.ns"]
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

    let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
    assert_eq!(
        applied.applied,
        vec!["ns-nova:lib/nova_contracts.ns".to_owned()]
    );
    assert_eq!(applied.total_explicit_galaxy_imports, 1);
    assert!(applied.manifest_updated);

    let manifest = fs::read_to_string(project_root.join("nuis.toml")).expect("read manifest");
    assert!(manifest.contains("galaxy_imports = ["));
    assert!(manifest.contains("\"ns-nova:lib/nova_contracts.ns\""));

    let json = render_project_imports_json(&project_root).expect("render imports json");
    assert!(json.contains("\"explicit_galaxy_imports_count\":1"));
    assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
}

#[test]
fn apply_suggested_project_imports_preserves_existing_entries_and_appends_new_ones() {
    let project_root = write_temp_project_fixture(
        "imports_apply_append",
        r#"
name = "imports_apply_append"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["pixelmagic=workspace", "ns-nova=workspace"]
galaxy_imports = [
  "pixelmagic:lib/image_contracts.ns",
]
"#
        .trim_start(),
        r#"
use cpu PixelMagicContracts;

mod cpu Main {
  fn main() -> i64 {
    return PixelMagicContracts.blur_op_kind();
  }
}
"#,
    );

    let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
    assert_eq!(
        applied.applied,
        vec!["ns-nova:lib/nova_contracts.ns".to_owned()]
    );
    assert_eq!(applied.total_explicit_galaxy_imports, 2);
    assert!(applied.manifest_updated);

    let manifest = fs::read_to_string(project_root.join("nuis.toml")).expect("read manifest");
    assert!(manifest.contains("\"pixelmagic:lib/image_contracts.ns\""));
    assert!(manifest.contains("\"ns-nova:lib/nova_contracts.ns\""));
    assert!(manifest.contains("galaxy_imports = ["));

    let pixelmagic_pos = manifest
        .find("\"pixelmagic:lib/image_contracts.ns\"")
        .expect("pixelmagic import present");
    let ns_nova_pos = manifest
        .find("\"ns-nova:lib/nova_contracts.ns\"")
        .expect("ns-nova import present");
    assert!(pixelmagic_pos < ns_nova_pos);
}

#[test]
fn project_imports_apply_json_reports_mutation_result() {
    let project_root = write_temp_project_fixture(
        "imports_apply_json",
        r#"
name = "imports_apply_json"
entry = "main.ns"
modules = ["main.ns"]
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

    let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
    let json = render_project_imports_apply_json(&project_root, &applied)
        .expect("render imports apply json");

    assert!(json.contains("\"kind\":\"project_imports_apply\""));
    assert!(json.contains("\"action\":\"apply_suggested\""));
    assert!(json.contains("\"manifest_updated\":true"));
    assert!(json.contains("\"applied_galaxy_imports_count\":1"));
    assert!(json.contains("\"applied_galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
    assert!(json.contains("\"total_explicit_galaxy_imports\":1"));
    assert!(json.contains("\"explicit_galaxy_imports_count\":1"));
    assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
}

#[test]
fn project_imports_apply_json_reports_noop_when_manifest_already_complete() {
    let project_root = write_temp_project_fixture(
        "imports_apply_json_noop",
        r#"
name = "imports_apply_json_noop"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]
"#
        .trim_start(),
        r#"
use cpu NovaContracts;

mod cpu Main {
  fn main() -> i64 {
    return NovaContracts.runtime_score(16, 4, 3, 2, 9, 1);
  }
}
"#,
    );

    let applied = apply_suggested_project_imports(&project_root).expect("apply imports");
    assert!(!applied.manifest_updated);
    let json = render_project_imports_apply_json(&project_root, &applied)
        .expect("render imports apply json");

    assert!(json.contains("\"manifest_updated\":false"));
    assert!(json.contains("\"applied_galaxy_imports_count\":0"));
    assert!(json.contains("\"applied_galaxy_imports\":[]"));
    assert!(json.contains("\"total_explicit_galaxy_imports\":1"));
    assert!(json.contains("\"suggested_galaxy_imports_count\":0"));
}

#[test]
fn project_status_json_reports_link_plan_for_built_output() {
    let project_root = write_temp_project_fixture(
        "status_json_built_smoke",
        r#"
name = "status_json_built_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 5;
  }
}
"#,
    );
    let output_dir = default_build_output_dir(&project_root);

    handle_build(project_root.clone(), output_dir.clone(), false, None, None)
        .expect("build passes");

    let json = render_project_status_json(&project_root).expect("render status json");

    assert!(json.contains(&format!(
        "\"artifact_output_dir\":\"{}\"",
        output_dir.display()
    )));
    assert!(json.contains("\"artifact_ready_to_run\":true"));
    assert!(json.contains("\"link_plan_available\":true"));
    assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
    assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
    assert!(json.contains("\"link_plan_final_link_mode\":\"host-toolchain-finalize\""));
    assert!(json.contains("\"link_plan_domain_units\":"));
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
}

#[test]
fn scheduler_view_json_reports_project_domains_and_frontdoor() {
    let project_root = write_temp_project_fixture(
        "scheduler_project_smoke",
        r#"
name = "scheduler_project_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 5;
  }
}
"#,
    );

    let json = render_scheduler_view_json(&project_root).expect("render scheduler project json");

    assert!(json.contains("\"source_kind\":\"project\""));
    assert!(json.contains("\"project\":\"scheduler_project_smoke\""));
    assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
    assert!(json.contains("\"abi_mode\":\"explicit\""));
    assert!(json.contains("\"project_plan\":\""));
    assert!(json.contains("\"project_plan_output_count\":"));
    assert!(json.contains("\"domains\":["));
    assert!(json.contains("\"abi_selection\":{"));
    assert!(json.contains("\"domain\":\"cpu\""));
    assert!(json.contains("\"abi\":\"cpu.arm64.apple_aapcs64\""));
}

#[test]
fn scheduler_view_json_reports_single_file_domain_surface() {
    let input = repo_root().join("stdlib/core/basic_scalars.ns");
    let json = render_scheduler_view_json(&input).expect("render scheduler single-file json");

    assert!(json.contains("\"source_kind\":\"single-file\""));
    assert!(json.contains("\"ast_domain\":\"cpu\""));
    assert!(json.contains("\"ast_unit\":\"Main\""));
    assert!(json.contains("\"workflow_kind\":\"compile_workflow\""));
    assert!(json.contains("\"recommended_next_step\":\"check\""));
    assert!(json.contains("\"domains\":["));
    assert!(json.contains("\"registration\":{"));
    assert!(json.contains("\"abi_selection\":null"));
}
