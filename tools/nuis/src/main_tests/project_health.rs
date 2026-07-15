use super::*;

#[test]
fn project_check_summary_json_fields_report_all_green() {
    let project = nuisc::project::load_project(
        &repo_root().join("examples/projects/domains/net_session_recipe_demo"),
    )
    .expect("load project");
    let plan = nuisc::project::build_project_compilation_plan(&project).expect("build plan");
    let abi_checks =
        nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)
            .expect("abi checks");
    let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
    let lowering_checks =
        nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);

    let fields = project_check_summary_json_fields(&abi_checks, &registry_checks, &lowering_checks);

    assert!(fields.iter().any(|field| field == "\"abi_checks_ok\":true"));
    assert!(fields
        .iter()
        .any(|field| field == "\"registry_checks_ok\":true"));
    assert!(fields
        .iter()
        .any(|field| field == "\"lowering_checks_ok\":true"));
    assert!(fields
        .iter()
        .any(|field| field.starts_with("\"abi_checks_count\":")));
    assert!(fields
        .iter()
        .any(|field| field.starts_with("\"registry_checks_count\":")));
    assert!(fields
        .iter()
        .any(|field| field.starts_with("\"lowering_checks_count\":")));
}

#[test]
fn project_status_json_reports_frontdoor_and_surface_fields() {
    let project_root = write_temp_project_fixture(
        "status_json_smoke",
        r#"
name = "status_json_smoke"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/smoke.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
galaxy = ["ns-nova=workspace"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn text_handle_helper() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("demo", buffer, 0);
    return deserialize_text_from(buffer, 0, len);
  }

  pub fn exported() -> i64 {
    let buffer: ref Buffer = alloc_buffer(128, 0);
    let len: i64 = serialize_text_into("hello", buffer, 0);
    let handle: i64 = deserialize_text_from(buffer, 0, len);
    return handle;
  }

  fn main() -> i64 {
    return text_handle_helper() + exported();
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

    let json = render_project_status_json(&project_root).expect("render status json");

    assert!(json.contains("\"source_kind\":\"project\""));
    assert!(json.contains("\"project\":\"status_json_smoke\""));
    assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
    assert!(json.contains(&format!(
        "\"project_compile_workflow\":\"{}\"",
        nuisc::project_compile_workflow_brief()
    )));
    assert!(json.contains("\"recommended_next_step\":\"galaxy_lock_deps\""));
    assert!(
        json.contains("\"recommended_command\":\"nuis galaxy lock-deps <project-dir|nuis.toml>\"")
    );
    assert!(json.contains(
            "\"recommended_reason\":\"the project already declares galaxy dependencies but does not yet have a lockfile\""
        ));
    assert!(json.contains("\"artifact_output_dir\":\""));
    assert!(json.contains("\"artifact_ready_to_run\":false"));
    assert!(json.contains("\"artifact_recommended_next_step\":\"build\""));
    assert!(json.contains("\"artifact_nsld_drive_dry_run_command\":\"nsld drive "));
    assert!(json.contains("\"artifact_nsld_drive_dry_run_json_command\":\"nsld drive "));
    assert!(json.contains(" --json\""));
    assert!(json.contains("\"artifact_nsld_drive_apply_next_command\":\"nsld drive "));
    assert!(json.contains("\"artifact_nsld_drive_apply_next_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --json\""));
    assert!(json.contains("\"artifact_nsld_drive_apply_until_clean_command\":\"nsld drive "));
    assert!(json.contains("\"artifact_nsld_drive_apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --until-clean --json\""));
    assert!(json.contains("\"artifact_nsld_drive_command_set\":{"));
    assert!(json.contains("\"protocol\":\"nsld-drive-command-set-v1\""));
    assert!(json.contains("\"recommended_first_json_command\":\"nsld drive "));
    assert!(json.contains("\"dry_run_mutates_artifacts\":false"));
    assert!(json.contains("\"apply_next_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_until_clean_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_next_json_command\":\"nsld drive "));
    assert!(json.contains("\"apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains("\"link_plan_available\":false"));
    assert!(json.contains("\"link_plan_final_stage\":null"));
    assert!(json.contains("\"tests_declared\":1"));
    assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
    assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
    assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
    assert!(json.contains("\"public_surface_modules\":11"));
    assert!(json.contains("\"functions\":[\"exported\"]"));
    assert!(json.contains("\"public_functions\":"));
    assert!(json.contains("\"galaxy_lock_status\":\"missing\""));
    assert!(json.contains("\"galaxy_surface_ids_count\":19"));
    assert!(json.contains("\"surface.ns-nova.renderer.v1\""));
    assert!(json.contains("\"contract.core.prelude.primitive-values.v1\""));
    assert!(json.contains("\"surface.std.collections.v1\""));
    assert!(json.contains("\"surface.std.cli-report-file-contracts.v1\""));
    assert!(json.contains("\"galaxy_records\":[{"));
    assert!(json.contains("\"galaxy_imports_count\":0"));
    assert!(json.contains("\"galaxy_hidden_manual_only_library_modules_count\":1"));
    assert!(json.contains(
        "\"galaxy_hidden_manual_only_library_modules\":[\"ns-nova:lib/nova_contracts.ns\"]"
    ));
    assert!(json.contains("\"tests\":[{"));
    assert!(json.contains("\"exists\":true"));
    assert!(json.contains("\"domains\":["));
}

#[test]
fn project_status_text_summary_reports_text_handle_rewrite_hits() {
    let project_root = write_temp_project_fixture(
        "status_text_handle_summary",
        r#"
name = "status_text_handle_summary"
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

    let lines = surface_render::render_project_status_text_summary(&project_root)
        .expect("render status text summary");

    assert!(lines
        .iter()
        .any(|line| line == "  text_handle_rewrite_helper_hits: 1"));
    assert!(lines
        .iter()
        .any(|line| line == "  text_handle_rewrite_local_hits: 1"));
    assert!(lines
        .iter()
        .any(|line| line == "  text_handle_rewrite_total_hits: 2"));
    assert!(lines
        .iter()
        .any(|line| line.starts_with("  artifact_nsld_drive_dry_run_command: nsld drive ")));
    assert!(lines
        .iter()
        .any(|line| line.starts_with("  artifact_nsld_drive_dry_run_json_command: nsld drive ")));
    assert!(lines.iter().any(|line| line.ends_with(" --json")));
    assert!(
        lines
            .iter()
            .any(|line| line
                .starts_with("  artifact_nsld_drive_apply_next_json_command: nsld drive "))
    );
    assert!(lines.iter().any(|line| line.ends_with(" --apply --json")));
    assert!(lines.iter().any(|line| line
        .starts_with("  artifact_nsld_drive_apply_until_clean_json_command: nsld drive ")));
    assert!(lines
        .iter()
        .any(|line| line.ends_with(" --apply --until-clean --json")));

    let mut written = String::new();
    surface_render::write_project_status_text_summary(&mut written, &project_root)
        .expect("write status text summary");
    assert_eq!(written.lines().collect::<Vec<_>>(), lines);
}

#[test]
fn project_doctor_json_reports_missing_test_and_health_checks() {
    let project_root = write_temp_project_fixture(
        "doctor_json_smoke",
        r#"
name = "doctor_json_smoke"
entry = "main.ns"
modules = ["main.ns"]
tests = ["tests/missing.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]
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

    let json = render_project_doctor_json(&project_root).expect("render doctor json");

    assert!(json.contains("\"source_kind\":\"project\""));
    assert!(json.contains("\"project\":\"doctor_json_smoke\""));
    assert!(json.contains("\"workflow_kind\":\"project_compile_workflow\""));
    assert!(json.contains("\"tests_declared\":1"));
    assert!(json.contains("\"tests_missing\":1"));
    assert!(json.contains("\"text_handle_rewrite_helper_hits\":1"));
    assert!(json.contains("\"text_handle_rewrite_local_hits\":1"));
    assert!(json.contains("\"text_handle_rewrite_total_hits\":2"));
    assert!(json.contains("\"abi_checks_ok\":true"));
    assert!(json.contains("\"registry_checks_ok\":true"));
    assert!(json.contains("\"lowering_checks_ok\":true"));
    assert!(json.contains("\"artifact_output_dir\":\""));
    assert!(json.contains("\"artifact_ready_to_run\":false"));
    assert!(json.contains("\"artifact_nsld_drive_dry_run_command\":\"nsld drive "));
    assert!(json.contains("\"artifact_nsld_drive_dry_run_json_command\":\"nsld drive "));
    assert!(json.contains(" --json\""));
    assert!(json.contains("\"artifact_nsld_drive_apply_next_command\":\"nsld drive "));
    assert!(json.contains("\"artifact_nsld_drive_apply_next_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --json\""));
    assert!(json.contains("\"artifact_nsld_drive_apply_until_clean_command\":\"nsld drive "));
    assert!(json.contains("\"artifact_nsld_drive_apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains(" --apply --until-clean --json\""));
    assert!(json.contains("\"artifact_nsld_drive_command_set\":{"));
    assert!(json.contains("\"protocol\":\"nsld-drive-command-set-v1\""));
    assert!(json.contains("\"recommended_first_json_command\":\"nsld drive "));
    assert!(json.contains("\"dry_run_mutates_artifacts\":false"));
    assert!(json.contains("\"apply_next_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_until_clean_mutates_artifacts\":true"));
    assert!(json.contains("\"apply_next_json_command\":\"nsld drive "));
    assert!(json.contains("\"apply_until_clean_json_command\":\"nsld drive "));
    assert!(json.contains("\"link_plan_available\":false"));
    assert!(json.contains("\"galaxy_check_status\":\"skipped\""));
    assert!(json.contains("\"galaxy_lock_status\":\"missing\""));
    assert!(json.contains("\"galaxy_imports_count\":1"));
    assert!(json.contains("\"galaxy_surface_ids_count\":19"));
    assert!(json.contains("\"surface.std.cli-report-file-contracts.v1\""));
    assert!(json.contains("\"surface.ns-nova.renderer.v1\""));
    assert!(json.contains("\"contract.core.prelude.primitive-values.v1\""));
    assert!(json.contains("\"surface.std.collections.v1\""));
    assert!(json.contains("\"galaxy_records\":[{"));
    assert!(json.contains("\"galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
    assert!(json.contains("\"galaxy_hidden_manual_only_library_modules_count\":0"));
    assert!(json.contains("\"galaxy_hidden_manual_only_library_modules\":[]"));
    assert!(json.contains("\"next_steps\":["));
    assert!(json.contains("some declared project tests are missing on disk"));
    assert!(json.contains("\"tests\":[{"));
    assert!(json.contains("\"exists\":false"));
    assert!(json.contains("\"domains\":["));
}

#[test]
fn project_doctor_text_summary_reports_text_handle_rewrite_hits() {
    let project_root = write_temp_project_fixture(
        "doctor_text_handle_summary",
        r#"
name = "doctor_text_handle_summary"
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

    let lines = surface_render::render_project_doctor_text_summary(&project_root)
        .expect("render doctor text summary");

    assert!(lines
        .iter()
        .any(|line| line == "  text_handle_rewrite_helper_hits: 1"));
    assert!(lines
        .iter()
        .any(|line| line == "  text_handle_rewrite_local_hits: 1"));
    assert!(lines
        .iter()
        .any(|line| line == "  text_handle_rewrite_total_hits: 2"));

    let mut written = String::new();
    surface_render::write_project_doctor_text_summary(&mut written, &project_root)
        .expect("write doctor text summary");
    assert_eq!(written.lines().collect::<Vec<_>>(), lines);
}

#[test]
fn project_doctor_json_suggests_galaxy_imports_for_hidden_manual_only_modules() {
    let project_root = write_temp_project_fixture(
        "doctor_manual_only_import_hint",
        r#"
name = "doctor_manual_only_import_hint"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["ns-nova=workspace"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 4;
  }
}
"#,
    );

    let json = render_project_doctor_json(&project_root).expect("render doctor json");

    assert!(json.contains("\"galaxy_surface_ids_count\":19"));
    assert!(json.contains("\"surface.std.cli-report-file-contracts.v1\""));
    assert!(json.contains("\"surface.ns-nova.renderer.v1\""));
    assert!(json.contains("\"contract.core.prelude.primitive-values.v1\""));
    assert!(json.contains("\"surface.std.collections.v1\""));
    assert!(json.contains("\"galaxy_records\":[{"));
    assert!(json.contains("\"galaxy_hidden_manual_only_library_modules_count\":1"));
    assert!(json.contains(
        "\"galaxy_hidden_manual_only_library_modules\":[\"ns-nova:lib/nova_contracts.ns\"]"
    ));
    assert!(json.contains("manual-only galaxy library modules"));
    assert!(json.contains("nuis project-imports --apply-suggested <project-dir>"));
    assert!(json.contains("galaxy_imports = [...]"));
    assert!(json.contains("ns-nova:lib/nova_contracts.ns"));
}

#[test]
fn project_imports_json_reports_hidden_manual_only_library_modules() {
    let project_root = write_temp_project_fixture(
        "imports_manual_only_hint",
        r#"
name = "imports_manual_only_hint"
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

    let json = render_project_imports_json(&project_root).expect("render imports json");

    assert!(json.contains("\"source_kind\":\"project\""));
    assert!(json.contains("\"project\":\"imports_manual_only_hint\""));
    assert!(json.contains("\"explicit_galaxy_imports_count\":0"));
    assert!(json.contains("\"visible_library_modules_count\":10"));
    assert!(json.contains("\"std:lib/report_contracts.ns\""));
    assert!(json.contains("\"hidden_manual_only_library_modules_count\":1"));
    assert!(
        json.contains("\"hidden_manual_only_library_modules\":[\"ns-nova:lib/nova_contracts.ns\"]")
    );
    assert!(json.contains("\"suggested_galaxy_imports_count\":1"));
    assert!(json.contains("\"suggested_galaxy_imports\":[\"ns-nova:lib/nova_contracts.ns\"]"));
    assert!(json.contains(
            "\"suggested_manifest_snippet\":\"galaxy_imports = [\\\"ns-nova:lib/nova_contracts.ns\\\"]\""
        ));
    assert!(json.contains("\"library_records\":[{"));
    assert!(json.contains("\"import_policy\":\"manual-only\""));
    assert!(json.contains("\"visible\":false"));
    assert!(json.contains("\"explicit\":false"));
}
