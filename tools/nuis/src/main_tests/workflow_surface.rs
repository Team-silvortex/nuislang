use super::*;

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
    assert!(json.contains("\"artifact_workflow\":\"build -> inspect_artifact -> verify_artifact -> artifact_doctor -> verify_build_manifest -> run_artifact\""));
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

    handle_build(project_root.clone(), output_dir.clone(), false, None, None)
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
    assert!(json.contains("\"nsld_prepare_command\":\"nsld prepare "));
    assert!(json.contains("\"nsld_prepared_artifact_chain_ready\":false"));
    assert!(json.contains("\"nsld_prepared_artifact_next_missing_stage\":\"link-inputs\""));
    assert!(json.contains(
        "\"nsld_final_executable_pipeline_command\":\"nsld emit-final-executable-pipeline "
    ));
    assert!(json.contains("\"nsld_final_executable_tail_ready\":false"));
    assert!(json.contains(
        "\"nsld_final_executable_tail_next_missing_stage\":\"final-executable-writer-input\""
    ));
    assert!(json.contains("\"compile_pipeline_available\":true"));
    assert!(json.contains("\"compile_pipeline_ready_for_aot\":true"));
    assert!(json.contains("\"compile_pipeline_summary\":\"source_kind=project"));
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
    handle_build(project_root.clone(), output_dir.clone(), false, None, None)
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
