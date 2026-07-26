use super::*;

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
