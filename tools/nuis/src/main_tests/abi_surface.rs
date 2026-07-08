use super::*;

#[test]
fn scheduler_view_domain_record_json_exposes_registration_section() {
    let record = scheduler_view_domain_record("network", None, None)
        .expect("expected network scheduler registration record");
    let json = scheduler_view_domain_record_json(&record);

    assert!(json.contains("\"abi_selection\":null"));
    assert!(json.contains("\"registration\":{"));
    assert!(json.contains("\"manifest_path\":\""));
    assert!(json.contains("network.toml"));
    assert!(json.contains("\"entry_crate\":"));
    assert!(json.contains("\"ast_entry\":"));
    assert!(json.contains("\"nir_entry\":"));
    assert!(json.contains("\"yir_lowering_entry\":"));
    assert!(json.contains("\"part_verify_entry\":"));
    assert!(json.contains("\"resource_families\":["));
    assert!(json.contains("\"unit_types\":["));
    assert!(json.contains("\"lowering_targets\":["));
    assert!(json.contains("\"ops\":["));
}

#[test]
fn scheduler_view_domain_record_json_exposes_shared_abi_selection_section() {
    let record = scheduler_view_domain_record(
        "network",
        None,
        Some("network.socket.macos.arm64.v1".to_owned()),
    )
    .expect("expected network scheduler registration record");
    let json = scheduler_view_domain_record_json(&record);

    assert!(json.contains("\"abi_selection\":{"));
    assert!(json.contains("\"domain\":\"network\""));
    assert!(json.contains("\"abi\":\"network.socket.macos.arm64.v1\""));
    assert!(json.contains("\"abi_target_machine\":\"arm64-darwin\""));
    assert!(json.contains("\"abi_target_host_adaptive\":false"));
}

#[test]
fn project_domain_registry_checks_report_registered_abis() {
    let project = nuisc::project::load_project(
        &repo_root().join("examples/projects/domains/net_session_recipe_demo"),
    )
    .expect("load project");
    let plan = nuisc::project::build_project_compilation_plan(&project).expect("build plan");
    let checks = nuisc::registry::validate_project_domain_registry(&plan);
    assert!(!checks.is_empty());
    assert!(checks.iter().all(|check| check.ok));
    assert!(checks.iter().any(|check| check.domain == "network"));
    let network = checks
        .iter()
        .find(|check| check.domain == "network")
        .unwrap();
    assert!(network.abi_registered);
    assert!(network.issues.is_empty());
    assert_eq!(
        network.contract_schema.as_deref(),
        Some(nuisc::registry::NUSTAR_DOMAIN_CONTRACT_SCHEMA)
    );
    let json = project_domain_registry_checks_json(&checks).join(",");
    assert!(json.contains("\"issues\":[]"));
    assert!(json.contains("\"abi_registered\":true"));
}

#[test]
fn project_abi_checks_report_recommended_abis() {
    let project = nuisc::project::load_project(
        &repo_root().join("examples/projects/domains/net_session_recipe_demo"),
    )
    .expect("load project");
    let plan = nuisc::project::build_project_compilation_plan(&project).expect("build plan");
    let checks = nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)
        .expect("abi checks");
    assert!(!checks.is_empty());
    assert!(checks.iter().all(|check| check.ok));
    assert!(checks.iter().any(|check| check.source == "explicit"));
    let json = project_abi_checks_json(&checks).join(",");
    assert!(json.contains("\"source\":\"explicit\""));
    assert!(json.contains("\"abi_registered\":true"));
    assert!(json.contains("\"issues\":[]"));
}

#[test]
fn upsert_abi_block_appends_sorted_block_when_missing() {
    let source = "[package]\nname = \"demo\"\n";
    let requirements = vec![
        nuisc::project::ProjectAbiRequirement {
            domain: "shader".to_owned(),
            abi: "shader.metal.msl2_4".to_owned(),
        },
        nuisc::project::ProjectAbiRequirement {
            domain: "cpu".to_owned(),
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
        },
    ];

    let updated = upsert_abi_block(source, &requirements);

    assert_eq!(
            updated,
            "[package]\nname = \"demo\"\n\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n  \"shader=shader.metal.msl2_4\",\n]\n"
        );
}

#[test]
fn upsert_abi_block_replaces_existing_block_with_normalized_sorted_entries() {
    let source = "[package]\nname = \"demo\"\nabi = [\n  \"shader=shader.cpu-fallback.v1\",\n]\nversion = \"0.1.0\"\n";
    let requirements = vec![
        nuisc::project::ProjectAbiRequirement {
            domain: "network".to_owned(),
            abi: "network.socket.macos.arm64.v1".to_owned(),
        },
        nuisc::project::ProjectAbiRequirement {
            domain: "cpu".to_owned(),
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
        },
    ];

    let updated = upsert_abi_block(source, &requirements);

    assert_eq!(
            updated,
            "[package]\nname = \"demo\"\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n  \"network=network.socket.macos.arm64.v1\",\n]\nversion = \"0.1.0\"\n"
        );
}

#[test]
fn upsert_abi_block_is_idempotent_for_matching_normalized_block() {
    let source = "[package]\nname = \"demo\"\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n  \"network=network.socket.macos.arm64.v1\",\n]\n";
    let requirements = vec![
        nuisc::project::ProjectAbiRequirement {
            domain: "network".to_owned(),
            abi: "network.socket.macos.arm64.v1".to_owned(),
        },
        nuisc::project::ProjectAbiRequirement {
            domain: "cpu".to_owned(),
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
        },
    ];

    let updated = upsert_abi_block(source, &requirements);

    assert_eq!(updated, source);
}

#[test]
fn find_abi_block_span_stops_at_closing_bracket_before_following_fields() {
    let source = "[package]\nname = \"demo\"\nabi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n]\nsummary = \"kept\"\n";

    let (start, end) = find_abi_block_span(source).expect("abi block span");

    assert_eq!(
        &source[start..end],
        "abi = [\n  \"cpu=cpu.arm64.apple_aapcs64\",\n]\n"
    );
    assert_eq!(&source[end..], "summary = \"kept\"\n");
}

#[test]
fn language_test_runner_prints_clock_policy_metadata() {
    let dir = temp_dir("language_test_clock_policy");
    let input = dir.join("clock_policy.ns");
    fs::write(
            &input,
            r#"
mod cpu Main {
  extern "c" fn usleep(usec: i64) -> i32;

  test("slow_global", should_fail=true, reason="bridge policy demo", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_global() -> i64 {
    let _slept: i32 = usleep(100000);
    return 1;
  }
}
"#,
        )
        .expect("write clock policy test file");

    let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
        .expect("clock policy language test should run");
    assert_eq!(report.collected, 1);
    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.skipped, 0);
}
