use super::{
    fnv1a64_hex, main_test_support::empty_link_plan, nsld_artifact_chain_issues, nsld_check_report,
    nsld_closure_report, nsld_link_input_diagnostics, nsld_link_input_table_hash,
    nsld_prepare_report, nsld_sidecar_capability_diagnostics, toml,
};
use crate::container_verify::{self, TomlFieldKind};
use std::{env, fs, path::Path};

#[test]
fn closure_reports_container_metadata_fingerprint() {
    let dir = env::temp_dir().join(format!("nsld-closure-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let report = nsld_closure_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_closure_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.container_metadata_table_hash.starts_with("0x"));
    assert!(matches!(
        report.container_loader_readiness.as_str(),
        "blocked" | "host-assisted" | "self-contained"
    ));
    assert!(report_json.contains("\"container_metadata_table_hash\":\"0x"));
    assert!(report_json.contains("\"container_loader_readiness\":"));
}

#[test]
fn scoped_toml_helpers_read_the_first_matching_table_only() {
    let source = r#"
[[loader_symbol]]
symbol_id = "sym0000.loader-entry"
section_id = "sec0000.compiled-artifact"

[[external_import]]
import_id = "imp0000.final-stage-driver"
required = true

[[section]]
section_id = "sec9999.section-table"

[[external_import]]
import_id = "imp0001.clang-target"
required = false
"#;

    assert_eq!(
        toml::first_table_string_value(source, "loader_symbol", "section_id").as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(
        toml::first_table_string_value(source, "external_import", "import_id").as_deref(),
        Some("imp0000.final-stage-driver")
    );
    assert_eq!(
        toml::first_table_bool_value(source, "external_import", "required"),
        Some(true)
    );
    assert_eq!(
        toml::first_table_string_value(source, "missing", "section_id"),
        None
    );
}

#[test]
fn table_field_issues_report_missing_and_invalid_fields() {
    let source = r#"
[[relocation]]
relocation_id = "rel0000.lifecycle-entry"
source_offset = "not-a-number"

[[relocation]]
relocation_id = "rel0001.hetero-node"
source_offset = 12
"#;

    let issues = container_verify::table_field_issues(
        source,
        "relocation",
        "relocation",
        &[
            ("relocation_id", TomlFieldKind::String),
            ("relocation_kind", TomlFieldKind::String),
            ("source_offset", TomlFieldKind::Usize),
        ],
    );

    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[0].relocation_kind missing"));
    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[0].source_offset invalid"));
    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[1].relocation_kind missing"));
}

#[test]
fn artifact_chain_accepts_contiguous_prepared_prefix() {
    let issues = nsld_artifact_chain_issues(&[
        ("inputs", true),
        ("units", true),
        ("bundle", true),
        ("assemble", false),
        ("section", false),
        ("object", false),
    ]);
    assert!(issues.is_empty());
}

#[test]
fn artifact_chain_rejects_later_artifact_without_prerequisite() {
    let issues = nsld_artifact_chain_issues(&[
        ("inputs", true),
        ("units", false),
        ("bundle", true),
        ("assemble", true),
        ("section", true),
        ("object", true),
    ]);
    assert_eq!(
        issues,
        vec![
            "artifact `bundle` is present but prerequisite `units` is missing".to_owned(),
            "artifact `assemble` is present but prerequisite `units` is missing".to_owned(),
            "artifact `section` is present but prerequisite `units` is missing".to_owned(),
            "artifact `object` is present but prerequisite `units` is missing".to_owned(),
        ]
    );
}

#[test]
fn sidecar_capability_check_skips_hetero_domains_without_ir_sidecars() {
    let path = env::temp_dir().join(format!("nsld-sidecar-cap-{}.toml", std::process::id()));
    let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.data".to_owned(),
        domain_family: "data".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: None,
        vendor: None,
        device_class: None,
        selected_lowering_target: None,
        contract_family: "nustar.data".to_owned(),
        packaging_role: "domain-sidecar".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });
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
        artifact_ir_sidecar_path: Some(path.display().to_string()),
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });

    let diagnostics = nsld_sidecar_capability_diagnostics(&plan);
    fs::remove_file(path).unwrap();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].domain_family, "shader");
    assert_eq!(diagnostics[0].content_bytes, sidecar_source.len());
    assert_eq!(
        diagnostics[0].content_hash,
        fnv1a64_hex(sidecar_source.as_bytes())
    );
    assert!(diagnostics[0].valid);
    let link_inputs = nsld_link_input_diagnostics(&diagnostics);
    assert_eq!(link_inputs.len(), 1);
    assert_eq!(link_inputs[0].order_index, 0);
    assert_eq!(link_inputs[0].input_id, "li0000.shader.official.shader");
    assert_eq!(link_inputs[0].input_kind, "lowering-ir-sidecar");
    assert_eq!(link_inputs[0].native_ir, "msl2.4");
    assert_eq!(
        link_inputs[0].dispatch_lowering,
        "command-encoder-draw-dispatch"
    );
    assert_eq!(link_inputs[0].content_bytes, sidecar_source.len());
    assert_eq!(
        link_inputs[0].content_hash,
        fnv1a64_hex(sidecar_source.as_bytes())
    );
    let expected_material = format!(
        "0\tli0000.shader.official.shader\tlowering-ir-sidecar\tshader\tofficial.shader\tmsl2.4\tcommand-encoder-draw-dispatch\t1\t{}\t{}\n",
        sidecar_source.len(),
        fnv1a64_hex(sidecar_source.as_bytes())
    );
    assert_eq!(
        nsld_link_input_table_hash(&link_inputs),
        fnv1a64_hex(expected_material.as_bytes())
    );
    let table = toml::render_link_input_table(
        &link_inputs,
        link_inputs
            .iter()
            .map(|input| input.content_bytes)
            .sum::<usize>(),
        &nsld_link_input_table_hash(&link_inputs),
    );
    assert!(table.contains("schema = \"nuis-nsld-link-input-table-v1\""));
    assert!(table.contains("schema_version = 1"));
    assert!(table.contains("table_kind = \"lowering-sidecar-link-inputs\""));
    assert!(table.contains("producer = \"nsld\""));
    assert!(table.contains("producer_phase = \"alpha-0.6.0\""));
    assert!(table.contains("link_input_count = 1"));
    assert!(table.contains("input_id = \"li0000.shader.official.shader\""));
    assert!(table.contains("native_ir = \"msl2.4\""));
    assert!(table.contains("content_hash = \""));
}

#[test]
fn check_reports_container_loader_readiness_without_failing_host_assisted_state() {
    let dir = env::temp_dir().join(format!("nsld-check-loader-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.object_plan_present);
    assert_eq!(report.object_plan_valid, Some(true));
    assert!(report.object_plan_issues.is_empty());
    assert!(report.container_section_issues.is_empty());
    assert!(report.container_loader_symbol_issues.is_empty());
    assert!(report.container_relocation_issues.is_empty());
    assert!(report.container_external_import_issues.is_empty());
    assert!(report_json.contains("\"container_section_issues\":[]"));
    assert!(report_json.contains("\"container_loader_symbol_issues\":[]"));
    assert!(report_json.contains("\"container_relocation_issues\":[]"));
    assert!(report_json.contains("\"container_external_import_issues\":[]"));
    assert!(report_json.contains("\"object_plan_present\":true"));
    assert!(report_json.contains("\"object_plan_valid\":true"));
    assert!(report_json.contains("\"object_plan_issues\":[]"));
    assert_eq!(
        report.container_loader_readiness.as_deref(),
        Some("host-assisted")
    );
    assert!(report
        .container_metadata_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(report.container_external_import_count, Some(3));
    assert!(report
        .container_loader_blockers
        .iter()
        .any(|blocker| blocker == "external-import:final-stage-driver:clang"));
    assert!(report
        .issues
        .iter()
        .all(|issue| !issue.contains("container loader readiness is blocked")));
}
