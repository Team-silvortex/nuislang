use super::*;

#[test]
fn checks_stdlib_source_modules() {
    std::thread::Builder::new()
        .name("nuis-stdlib-smoke".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let root = repo_root();
            for module_dir in ["core", "std", "ns-nova", "pixelmagic", "witsage"] {
                for relative in load_stdlib_source_modules(&root, module_dir) {
                    let input = root.join(relative);
                    handle_check(input.clone()).unwrap_or_else(|error| {
                        panic!("failed to check {}: {error}", input.display())
                    });
                }
            }
        })
        .expect("spawn stdlib smoke thread")
        .join()
        .expect("join stdlib smoke thread");
}

#[test]
fn single_source_frontdoor_surface_matches_compile_contract() {
    let frontdoor = build_workflow_frontdoor_surface(
        single_source_workflow_source_profile(),
        WorkflowRecommendation {
            label: "check",
            command: "nuis check <input.ns>",
            reason: "single-file inputs should re-check compile truth first",
        },
    );
    assert_eq!(frontdoor.source_kind, "single-file");
    assert_eq!(frontdoor.workflow_kind, "compile_workflow");
    assert_eq!(
        frontdoor.workflow_brief,
        "check -> test -> build -> artifact_doctor -> nsld_drive -> run_artifact -> release_check"
    );
    assert!(frontdoor
        .workflow_samples
        .contains("nuis artifact-doctor <output-dir>"));
    assert!(frontdoor.workflow_samples.contains("nsld drive"));
    assert_eq!(frontdoor.recommended_next_step, "check");
}

#[test]
fn project_frontdoor_surface_uses_project_compile_profile() {
    let frontdoor = build_workflow_frontdoor_surface(
        project_compile_workflow_source_profile(),
        WorkflowRecommendation {
            label: "project_lock_abi",
            command: "nuis project-lock-abi <project-dir|nuis.toml>",
            reason: "freeze ABI choice before broader compile work",
        },
    );
    assert_eq!(frontdoor.source_kind, "project");
    assert_eq!(frontdoor.workflow_kind, "project_compile_workflow");
    assert_eq!(
        frontdoor.workflow_brief,
        nuisc::project_compile_workflow_brief()
    );
    assert_eq!(
        frontdoor.workflow_samples,
        nuisc::project_compile_samples_brief()
    );
    assert_eq!(frontdoor.recommended_next_step, "project_lock_abi");
}

#[test]
fn project_compile_workflow_brief_includes_artifact_follow_up() {
    assert!(nuisc::project_compile_workflow_brief().contains("artifact_doctor"));
    assert!(nuisc::project_compile_workflow_brief().contains("nsld_drive"));
    assert!(nuisc::project_compile_workflow_brief().contains("run_artifact"));
    assert!(nuisc::project_compile_samples_brief().contains("nuis artifact-doctor"));
    assert!(nuisc::project_compile_samples_brief().contains("nsld drive"));
}

#[test]
fn single_source_workflow_helpers_emit_artifact_follow_up_commands() {
    let input = Path::new("examples/demo.ns");
    let output_dir = default_build_output_dir(input);
    assert!(artifact_workflow_brief().contains("artifact_doctor"));
    assert!(artifact_workflow_brief().contains("nsld_drive"));
    assert!(artifact_doctor_command_for_output_dir(&output_dir).contains("nuis artifact-doctor"));
    assert!(run_artifact_command_for_output_dir(&output_dir).contains("nuis run-artifact"));
    assert_eq!(
        release_check_nsld_drive_dry_run_command_for_output_dir(&output_dir),
        format!(
            "nsld drive {}/nuis.build.manifest.toml",
            output_dir.display()
        )
    );
    assert_eq!(
        release_check_nsld_drive_dry_run_json_command_for_output_dir(&output_dir),
        format!(
            "nsld drive {}/nuis.build.manifest.toml --json",
            output_dir.display()
        )
    );
    assert_eq!(
        release_check_nsld_drive_command_for_output_dir(&output_dir),
        format!(
            "nsld drive {}/nuis.build.manifest.toml --apply",
            output_dir.display()
        )
    );
    assert_eq!(
        release_check_nsld_drive_json_command_for_output_dir(&output_dir),
        format!(
            "nsld drive {}/nuis.build.manifest.toml --apply --json",
            output_dir.display()
        )
    );
    assert_eq!(
        release_check_nsld_drive_until_clean_command_for_output_dir(&output_dir),
        format!(
            "nsld drive {}/nuis.build.manifest.toml --apply --until-clean",
            output_dir.display()
        )
    );
    assert_eq!(
        release_check_nsld_drive_until_clean_json_command_for_output_dir(&output_dir),
        format!(
            "nsld drive {}/nuis.build.manifest.toml --apply --until-clean --json",
            output_dir.display()
        )
    );
    let command_set = nsld_drive_command_set_for_output_dir(&output_dir);
    assert_eq!(command_set.protocol, "nsld-drive-command-set-v1");
    assert_eq!(
        command_set.recommended_first_json_command,
        release_check_nsld_drive_dry_run_json_command_for_output_dir(&output_dir)
    );
    assert_eq!(
        command_set.dry_run_command,
        release_check_nsld_drive_dry_run_command_for_output_dir(&output_dir)
    );
    assert_eq!(
        command_set.apply_next_command,
        release_check_nsld_drive_command_for_output_dir(&output_dir)
    );
    assert_eq!(
        command_set.apply_until_clean_command,
        release_check_nsld_drive_until_clean_command_for_output_dir(&output_dir)
    );
    assert!(!command_set.dry_run_mutates_artifacts);
    assert!(command_set.apply_next_mutates_artifacts);
    assert!(command_set.apply_until_clean_mutates_artifacts);
    assert_eq!(
        run_artifact_command_for_output_dir(&output_dir),
        format!("nuis run-artifact {}", output_dir.display())
    );
}

#[test]
fn resolve_run_artifact_binary_path_accepts_output_dir() {
    let project_root = checked_in_path("../../examples/projects/tooling/cli_runtime_demo");
    let output_dir = temp_dir("resolve_run_artifact_binary_path_output_dir");
    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    let binary = resolve_run_artifact_binary_path(&output_dir).expect("resolve output-dir");
    assert_eq!(binary, output_dir.join("cli_runtime_demo"));
}

#[test]
fn test_command_checks_declared_project_tests() {
    let dir = temp_dir("project_tests");
    let manifest = dir.join("nuis.toml");
    let entry = dir.join("main.ns");
    let tests_dir = dir.join("tests");
    fs::create_dir_all(&tests_dir).expect("create tests dir");
    let smoke = tests_dir.join("smoke.ns");
    fs::write(
        &manifest,
        r#"
name = "smoke_project"
entry = "main.ns"
tests = ["tests/smoke.ns"]
"#,
    )
    .expect("write manifest");
    fs::write(
        &entry,
        r#"
mod cpu Main {
  fn main() {
    print(1);
  }
}
"#,
    )
    .expect("write entry");
    fs::write(
        &smoke,
        r#"
mod cpu Main {
  fn main() {
    print(2);
  }
}
"#,
    )
    .expect("write smoke");
    handle_test(manifest, false, false, false, false, None).expect("project tests pass");
}

#[test]
fn build_command_writes_project_compile_outputs() {
    let project_root = write_temp_project_fixture(
        "build_command_smoke",
        r#"
name = "build_command_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 7;
  }
}
"#,
    );
    let output_dir = temp_dir("build_command_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");

    for path in [
        output_dir.join("build_command_smoke.ast.txt"),
        output_dir.join("build_command_smoke.nir.txt"),
        output_dir.join("build_command_smoke.yir"),
        output_dir.join("build_command_smoke.ll"),
        output_dir.join("build_command_smoke"),
        output_dir.join("nuis.build.manifest.toml"),
        output_dir.join("nuis.executable.envelope.toml"),
        output_dir.join("nuis.compiled.artifact"),
    ] {
        assert!(path.exists(), "expected build output `{}`", path.display());
    }

    let manifest_report =
        nuisc::aot::verify_build_manifest(output_dir.join("nuis.build.manifest.toml").as_path())
            .expect("manifest verifies");
    assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
    assert_eq!(manifest_report.artifact_binary_name, "build_command_smoke");
}

#[test]
fn release_check_runs_project_compile_chain_end_to_end() {
    let project_root = write_temp_project_fixture(
        "release_check_smoke",
        r#"
name = "release_check_smoke"
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
    let output_dir = temp_dir("release_check_outputs");

    handle_release_check(project_root, output_dir.clone(), None, None)
        .expect("release-check passes");

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    assert!(manifest_path.exists(), "expected manifest output");
    let manifest_report =
        nuisc::aot::verify_build_manifest(manifest_path.as_path()).expect("manifest verifies");
    assert_eq!(manifest_report.packaging_mode, "native-cpu-llvm");
    assert_eq!(manifest_report.artifact_binary_name, "release_check_smoke");

    let artifact_report = nuisc::aot::verify_nuis_compiled_artifact(
        output_dir.join("nuis.compiled.artifact").as_path(),
    )
    .expect("artifact verifies");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);
}

#[test]
fn run_artifact_executes_binary_from_manifest_input() {
    let project_root = write_temp_project_fixture(
        "run_artifact_smoke",
        r#"
name = "run_artifact_smoke"
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
	    if text_handle_helper() <= 0 || handle <= 0 {
	      return 1;
	    }
	    return 0;
	  }
	}
"#,
    );
    let output_dir = temp_dir("run_artifact_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    handle_run_artifact(output_dir.join("nuis.build.manifest.toml"), false)
        .expect("run-artifact passes");
}

#[test]
fn run_artifact_executes_checked_in_cli_runtime_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_runtime_demo",
        "run_artifact_cli_runtime_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_session_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_session_demo",
        "run_artifact_cli_session_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_report_session_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_report_session_demo",
        "run_artifact_cli_report_session_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_workflow_runtime_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/workflow_runtime_demo",
        "run_artifact_workflow_runtime_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_command_runtime_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/command_runtime_demo",
        "run_artifact_command_runtime_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_subprocess_runtime_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/subprocess_runtime_demo",
        "run_artifact_subprocess_runtime_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_compile_workflow_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_compile_workflow_demo",
        "run_artifact_cli_compile_workflow_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_build_pipeline_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_build_pipeline_demo",
        "run_artifact_cli_build_pipeline_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_workflow_automation_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_workflow_automation_demo",
        "run_artifact_cli_workflow_automation_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_project_build_report_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_project_build_report_demo",
        "run_artifact_cli_project_build_report_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_pgm_info_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_pgm_info_demo",
        "run_artifact_cli_pgm_info_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_pgm_invert_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_pgm_invert_demo",
        "run_artifact_cli_pgm_invert_outputs",
    );
}

#[test]
fn run_artifact_executes_checked_in_cli_pgm_threshold_project() {
    assert_checked_in_tooling_project_runs(
        "../../examples/projects/tooling/cli_pgm_threshold_demo",
        "run_artifact_cli_pgm_threshold_outputs",
    );
}

#[test]
fn cli_pgm_info_binary_accepts_real_pgm_input_file() {
    let project_root = checked_in_path("../../examples/projects/tooling/cli_pgm_info_demo");
    let output_dir = temp_dir("cli_pgm_info_runtime_probe_outputs");
    let input_path = output_dir.join("probe.pgm");
    fs::write(&input_path, b"P2\n2 2\n15\n0 1 2 3\n").expect("write pgm fixture");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    let binary = resolve_run_artifact_binary_path(&output_dir.join("nuis.build.manifest.toml"))
        .expect("resolve built binary");
    let status = Command::new(&binary)
        .arg(&input_path)
        .status()
        .expect("launch cli pgm info binary");
    assert!(status.success(), "expected success status, got {status:?}");
}

#[test]
fn cli_pgm_invert_binary_writes_inverted_pgm_output_file() {
    let project_root = checked_in_path("../../examples/projects/tooling/cli_pgm_invert_demo");
    let output_dir = temp_dir("cli_pgm_invert_runtime_probe_outputs");
    let input_path = output_dir.join("probe_in.pgm");
    let output_path = output_dir.join("probe_out.pgm");
    fs::write(&input_path, b"P2\n2 2\n15\n0 1 2 3\n").expect("write pgm fixture");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    let binary = resolve_run_artifact_binary_path(&output_dir.join("nuis.build.manifest.toml"))
        .expect("resolve built binary");
    let status = Command::new(&binary)
        .arg(&input_path)
        .arg(&output_path)
        .status()
        .expect("launch cli pgm invert binary");
    assert!(status.success(), "expected success status, got {status:?}");

    let output = fs::read_to_string(&output_path).expect("read inverted pgm output");
    assert_eq!(output, "P2\n2 2\n15\n15 14 13 12\n");
}

#[test]
fn cli_pgm_threshold_binary_writes_mask_pgm_output_file() {
    let project_root = checked_in_path("../../examples/projects/tooling/cli_pgm_threshold_demo");
    let output_dir = temp_dir("cli_pgm_threshold_runtime_probe_outputs");
    let input_path = output_dir.join("probe_in.pgm");
    let output_path = output_dir.join("probe_out.pgm");
    fs::write(&input_path, b"P2\n2 2\n15\n0 1 2 3\n").expect("write pgm fixture");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    let binary = resolve_run_artifact_binary_path(&output_dir.join("nuis.build.manifest.toml"))
        .expect("resolve built binary");
    let status = Command::new(&binary)
        .arg(&input_path)
        .arg(&output_path)
        .status()
        .expect("launch cli pgm threshold binary");
    assert!(status.success(), "expected success status, got {status:?}");

    let output = fs::read_to_string(&output_path).expect("read threshold pgm output");
    assert_eq!(output, "P2\n2 2\n15\n0 0 15 15\n");
}

#[test]
fn artifact_doctor_json_reports_ready_to_run_for_built_output() {
    let project_root = write_temp_project_fixture(
        "artifact_doctor_smoke",
        r#"
name = "artifact_doctor_smoke"
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
    let output_dir = temp_dir("artifact_doctor_outputs");

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    let json = render_artifact_doctor_json(&output_dir);

    assert!(json.contains("\"kind\":\"artifact_doctor\""));
    assert!(json.contains("\"source_kind\":\"output_dir\""));
    assert!(json.contains("\"manifest_exists\":true"));
    assert!(json.contains("\"artifact_exists\":true"));
    assert!(json.contains("\"binary_exists\":true"));
    assert!(json.contains("\"manifest_verified\":true"));
    assert!(json.contains("\"artifact_verified\":true"));
    assert!(json.contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\""));
    assert!(json.contains("\"artifact_container_version\":2"));
    assert!(json.contains("\"artifact_section_count\":6"));
    assert!(json.contains("\"artifact_section_names\":[\"metadata_toml\""));
    assert!(json.contains("\"artifact_section_table_valid\":true"));
    assert!(json.contains("\"lowering_unit_count\":1"));
    assert!(json.contains("\"lowering_domain_families\":[\"cpu\"]"));
    assert!(json.contains("\"lowering_targets\":[\"llvm\"]"));
    assert!(json.contains("\"lowering_units\":[{"));
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
    assert!(json.contains("\"recommended_next_step\":\"run_artifact\""));
    assert!(json.contains("\"link_plan_available\":true"));
    assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
    assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
    assert!(json.contains("\"link_plan_final_link_mode\":\"host-toolchain-finalize\""));
    assert!(json.contains("\"link_plan_final_output\":\""));
    assert!(json.contains("\"link_plan_domain_units\":1"));
    assert!(json.contains("\"link_plan_domain_unit_records\":[{"));
    assert!(json.contains("\"domain_family\":\"cpu\""));
    assert!(json.contains("\"packaging_role\":\"host-binary\""));
}
