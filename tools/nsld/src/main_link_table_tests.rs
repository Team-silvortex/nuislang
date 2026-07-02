use super::{
    main_test_support::empty_link_plan, nsld_link_bundle_report, nsld_link_input_diagnostics,
    nsld_link_input_table_hash, nsld_link_unit_report, nsld_link_unit_table_hash,
    nsld_sidecar_capability_diagnostics, nsld_verify_link_bundle_report,
    nsld_verify_link_inputs_report, nsld_verify_link_units_report, toml,
};
use std::{env, fs, path::Path};

#[test]
fn verify_link_inputs_accepts_matching_emitted_table() {
    let dir = env::temp_dir().join(format!("nsld-link-input-verify-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let sidecar_path = dir.join("shader.sidecar.toml");
    let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&sidecar_path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: None,
        device_class: None,
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });
    let diagnostics = nsld_sidecar_capability_diagnostics(&plan);
    let inputs = nsld_link_input_diagnostics(&diagnostics);
    let total_bytes = inputs
        .iter()
        .map(|input| input.content_bytes)
        .sum::<usize>();
    let table_hash = nsld_link_input_table_hash(&inputs);
    fs::write(
        dir.join("nuis.nsld.link-inputs.toml"),
        toml::render_link_input_table(&inputs, total_bytes, &table_hash),
    )
    .unwrap();

    let report = nsld_verify_link_inputs_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.actual_link_input_count, Some(1));
    assert_eq!(
        report.actual_link_input_total_bytes,
        Some(sidecar_source.len())
    );
    assert_eq!(report.actual_link_input_table_hash, Some(table_hash));
}

#[test]
fn link_unit_report_attaches_registered_sidecar_inputs() {
    let dir = env::temp_dir().join(format!("nsld-link-unit-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let sidecar_path = dir.join("shader.sidecar.toml");
    let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&sidecar_path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: None,
        device_class: None,
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });

    let report = nsld_link_unit_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(report.unit_count, 1);
    assert_eq!(report.hetero_unit_count, 1);
    assert_eq!(report.link_input_count, 1);
    assert_eq!(report.units[0].unit_id, "lu0000.shader.official.shader");
    assert_eq!(report.units[0].unit_kind, "hetero-domain");
    assert_eq!(report.units[0].backend_family, "metal");
    assert_eq!(report.units[0].link_input_ids.len(), 1);
    assert_eq!(
        report.units[0].link_input_ids[0],
        "li0000.shader.official.shader"
    );
    assert_eq!(
        report.unit_table_hash,
        nsld_link_unit_table_hash(&report.units)
    );
}

#[test]
fn verify_link_units_accepts_matching_emitted_table() {
    let dir = env::temp_dir().join(format!("nsld-link-unit-verify-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let sidecar_path = dir.join("shader.sidecar.toml");
    let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&sidecar_path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: None,
        device_class: None,
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });
    let unit_report = nsld_link_unit_report(Path::new("manifest.toml"), &plan);
    fs::write(
        dir.join("nuis.nsld.link-units.toml"),
        toml::render_link_unit_table(&unit_report),
    )
    .unwrap();

    let report = nsld_verify_link_units_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.actual_unit_count, Some(1));
    assert_eq!(report.actual_hetero_unit_count, Some(1));
    assert_eq!(report.actual_link_input_count, Some(1));
    assert_eq!(
        report.actual_unit_table_hash,
        Some(unit_report.unit_table_hash)
    );
}

#[test]
fn verify_link_bundle_accepts_matching_emitted_bundle() {
    let dir = env::temp_dir().join(format!("nsld-link-bundle-verify-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let sidecar_path = dir.join("shader.sidecar.toml");
    let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&sidecar_path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: None,
        device_class: None,
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });
    let bundle_report = nsld_link_bundle_report(Path::new("manifest.toml"), &plan);
    fs::write(
        dir.join("nuis.nsld.link-bundle.toml"),
        toml::render_link_bundle(&bundle_report),
    )
    .unwrap();

    let report = nsld_verify_link_bundle_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.actual_bundle_id, Some(bundle_report.bundle_id));
    assert_eq!(report.actual_bundle_hash, Some(bundle_report.bundle_hash));
}
