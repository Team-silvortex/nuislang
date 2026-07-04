use super::{
    artifact_chain::{
        nsld_artifact_chain_issues, nsld_artifact_stage_file_name, nsld_artifact_stage_kind_path,
        NsldArtifactStage, NsldArtifactStageKind,
    },
    fnv1a64_hex,
    main_test_support::empty_link_plan,
    nsld_check_report, nsld_closure_report, nsld_link_input_diagnostics,
    nsld_link_input_table_hash, nsld_prepare_report, nsld_sidecar_capability_diagnostics, toml,
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
    assert_eq!(report.compatibility_domain_count, 1);
    assert!(report.compatibility_domain_table_hash.starts_with("0x"));
    assert_eq!(
        report.compatibility_domain_id.as_deref(),
        Some("compat0000.cffi-von-neumann")
    );
    assert_eq!(
        report.compatibility_domain_kind.as_deref(),
        Some("cffi-host-compat")
    );
    assert_eq!(
        report.compatibility_domain_paradigm.as_deref(),
        Some("classic-von-neumann-host")
    );
    assert_eq!(
        report.compatibility_domain_lifecycle_hook.as_deref(),
        Some("on_cffi_native_object")
    );
    assert_eq!(
        report.compatibility_domain_abi_family.as_deref(),
        Some("mach-o")
    );
    assert_eq!(
        report.compatibility_domain_wrapper_policy.as_deref(),
        Some("wrapped")
    );
    assert_eq!(report.compatibility_domain_required, Some(true));
    assert!(report_json.contains("\"container_metadata_table_hash\":\"0x"));
    assert!(report_json.contains("\"container_loader_readiness\":"));
    assert!(report_json.contains("\"compatibility_domain_count\":1"));
    assert!(report_json.contains("\"compatibility_domain_table_hash\":\"0x"));
    assert!(report_json.contains("\"compatibility_domain_id\":\"compat0000.cffi-von-neumann\""));
    assert!(report_json.contains("\"compatibility_domain_kind\":\"cffi-host-compat\""));
    assert!(report_json.contains("\"compatibility_domain_paradigm\":\"classic-von-neumann-host\""));
    assert!(
        report_json.contains("\"compatibility_domain_lifecycle_hook\":\"on_cffi_native_object\"")
    );
    assert!(report_json.contains("\"compatibility_domain_abi_family\":\"mach-o\""));
    assert!(report_json.contains("\"compatibility_domain_wrapper_policy\":\"wrapped\""));
    assert!(report_json.contains("\"compatibility_domain_required\":true"));
    assert!(
        report_json.contains("\"compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x")
    );
}

#[test]
fn closure_reports_verified_prepared_artifact_chain_after_prepare() {
    let dir = env::temp_dir().join(format!("nsld-closure-prepared-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_closure_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_closure_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.prepared_artifact_chain_valid);
    assert!(report.prepared_artifact_chain_issues.is_empty());
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-prepared-artifact-chain"));
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-object-writer-input"));
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-object-output"));
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-object-writer-dry-run"));
    assert!(report_json.contains("\"prepared_artifact_chain_valid\":true"));
    assert!(report_json.contains("\"prepared_artifact_chain_issues\":[]"));
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
        test_artifact_stage("inputs", true),
        test_artifact_stage("units", true),
        test_artifact_stage("bundle", true),
        test_artifact_stage("assemble", false),
        test_artifact_stage("section", false),
        test_artifact_stage("object", false),
    ]);
    assert!(issues.is_empty());
}

#[test]
fn artifact_chain_rejects_later_artifact_without_prerequisite() {
    let issues = nsld_artifact_chain_issues(&[
        test_artifact_stage("inputs", true),
        test_artifact_stage("units", false),
        test_artifact_stage("bundle", true),
        test_artifact_stage("assemble", true),
        test_artifact_stage("section", true),
        test_artifact_stage("object", true),
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
fn artifact_chain_allows_missing_optional_object_output_before_later_artifacts() {
    let issues = nsld_artifact_chain_issues(&[
        test_artifact_stage("object-emit", true),
        test_optional_artifact_stage("nuis.nsld.mach-o", false),
        test_artifact_stage("object-writer-dry-run", true),
        test_artifact_stage("container-plan", true),
    ]);
    assert!(issues.is_empty());
}

#[test]
fn artifact_stage_kind_paths_are_canonical() {
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::ObjectWriterInput),
        "nuis.nsld.object-writer-input.toml"
    );
    assert_eq!(
        nsld_artifact_stage_kind_path("out", NsldArtifactStageKind::ContainerPayload)
            .display()
            .to_string(),
        "out/nuis.nsld.container.payload"
    );
}

fn test_artifact_stage(file_name: &'static str, present: bool) -> NsldArtifactStage {
    NsldArtifactStage {
        kind: NsldArtifactStageKind::LinkInputs,
        file_name,
        present,
        required: true,
    }
}

fn test_optional_artifact_stage(file_name: &'static str, present: bool) -> NsldArtifactStage {
    NsldArtifactStage {
        kind: NsldArtifactStageKind::ObjectOutput,
        file_name,
        present,
        required: false,
    }
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
    assert!(report.object_writer_input_present);
    assert_eq!(report.object_writer_input_valid, Some(true));
    assert!(report.object_writer_input_issues.is_empty());
    assert!(report.object_byte_layout_present);
    assert_eq!(report.object_byte_layout_valid, Some(true));
    assert!(report.object_byte_layout_issues.is_empty());
    assert!(report.object_file_layout_present);
    assert_eq!(report.object_file_layout_valid, Some(true));
    assert!(report.object_file_layout_issues.is_empty());
    assert!(report.object_image_dry_run_present);
    assert_eq!(report.object_image_dry_run_valid, Some(true));
    assert!(report.object_image_dry_run_issues.is_empty());
    assert_eq!(report.object_image_relocation_lowering_valid, Some(true));
    assert_eq!(report.object_image_relocation_lowering_rule_count, Some(4));
    assert_eq!(report.object_image_relocation_lowering_rules.len(), 4);
    assert_eq!(
        report.object_image_relocation_lowering_rules[0].source_seed_kind,
        "bootstrap-entry-seed"
    );
    assert!(report.object_image_relocation_lowering_issues.is_empty());
    assert!(report.object_image_dry_run_bytes_present);
    assert!(report.object_emit_blocked_present);
    assert_eq!(report.object_emit_blocked_valid, Some(true));
    assert!(report.object_emit_blocked_issues.is_empty());
    assert!(report.object_output_present);
    assert_eq!(report.object_output_valid, Some(true));
    assert!(report.object_output_expected_size_bytes.is_some());
    assert_eq!(
        report.object_output_expected_size_bytes,
        report.object_output_actual_size_bytes
    );
    assert!(report
        .object_output_expected_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(
        report.object_output_expected_hash,
        report.object_output_actual_hash
    );
    assert!(report.object_output_issues.is_empty());
    assert!(report.object_writer_dry_run_present);
    assert_eq!(report.object_writer_dry_run_valid, Some(true));
    assert!(report.object_writer_dry_run_issues.is_empty());
    assert!(report.container_section_issues.is_empty());
    assert!(report.container_loader_symbol_issues.is_empty());
    assert!(report.container_relocation_issues.is_empty());
    assert!(report.container_compatibility_domain_issues.is_empty());
    assert!(report.container_external_import_issues.is_empty());
    assert!(report_json.contains("\"container_section_issues\":[]"));
    assert!(report_json.contains("\"container_loader_symbol_issues\":[]"));
    assert!(report_json.contains("\"container_relocation_issues\":[]"));
    assert!(report_json.contains("\"container_compatibility_domain_issues\":[]"));
    assert!(report_json.contains("\"container_external_import_issues\":[]"));
    assert!(report_json.contains("\"object_plan_present\":true"));
    assert!(report_json.contains("\"object_plan_valid\":true"));
    assert!(report_json.contains("\"object_plan_issues\":[]"));
    assert!(report_json.contains("\"object_writer_input_present\":true"));
    assert!(report_json.contains("\"object_writer_input_valid\":true"));
    assert!(report_json.contains("\"object_byte_layout_present\":true"));
    assert!(report_json.contains("\"object_byte_layout_valid\":true"));
    assert!(report_json.contains("\"object_file_layout_present\":true"));
    assert!(report_json.contains("\"object_file_layout_valid\":true"));
    assert!(report_json.contains("\"object_image_dry_run_present\":true"));
    assert!(report_json.contains("\"object_image_dry_run_valid\":true"));
    assert!(report_json.contains("\"object_image_relocation_lowering_valid\":true"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rule_count\":4"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rules\":[{"));
    assert!(report_json.contains("\"source_seed_kind\":\"bootstrap-entry-seed\""));
    assert!(report_json.contains("\"object_image_relocation_lowering_issues\":[]"));
    assert!(report_json.contains("\"object_image_dry_run_bytes_present\":true"));
    assert!(report_json.contains("\"object_emit_blocked_present\":true"));
    assert!(report_json.contains("\"object_emit_blocked_valid\":true"));
    assert!(report_json.contains("\"object_output_present\":true"));
    assert!(report_json.contains("\"object_output_valid\":true"));
    assert!(report_json.contains("\"object_output_expected_size_bytes\":"));
    assert!(report_json.contains("\"object_output_actual_size_bytes\":"));
    assert!(report_json.contains("\"object_output_expected_hash\":\"0x"));
    assert!(report_json.contains("\"object_output_actual_hash\":\"0x"));
    assert!(report_json.contains("\"object_writer_dry_run_present\":true"));
    assert!(report_json.contains("\"object_writer_dry_run_valid\":true"));
    assert_eq!(
        report.container_loader_readiness.as_deref(),
        Some("host-assisted")
    );
    assert!(report
        .container_metadata_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(report.container_compatibility_domain_count, Some(1));
    assert!(report
        .container_compatibility_domain_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(
        report.container_compatibility_domain_id.as_deref(),
        Some("compat0000.cffi-von-neumann")
    );
    assert_eq!(
        report.container_compatibility_domain_kind.as_deref(),
        Some("cffi-host-compat")
    );
    assert_eq!(
        report.container_compatibility_domain_paradigm.as_deref(),
        Some("classic-von-neumann-host")
    );
    assert_eq!(
        report
            .container_compatibility_domain_lifecycle_hook
            .as_deref(),
        Some("on_cffi_native_object")
    );
    assert_eq!(
        report.container_compatibility_domain_abi_family.as_deref(),
        Some("mach-o")
    );
    assert_eq!(
        report
            .container_compatibility_domain_wrapper_policy
            .as_deref(),
        Some("wrapped")
    );
    assert_eq!(report.container_compatibility_domain_required, Some(true));
    assert_eq!(report.container_external_import_count, Some(3));
    assert!(report.container_native_object_section_present);
    assert_eq!(
        report.container_native_object_section_id.as_deref(),
        Some("sec0004.native-object-output")
    );
    assert!(report.container_native_object_loader_symbol_present);
    assert_eq!(
        report.container_native_object_loader_symbol_id.as_deref(),
        Some("sym0001.native-object-output")
    );
    assert!(report.container_native_object_relocation_present);
    assert_eq!(
        report.container_native_object_relocation_id.as_deref(),
        Some("rel0001.native-object")
    );
    assert!(report_json.contains("\"container_native_object_section_present\":true"));
    assert!(report_json.contains("\"container_compatibility_domain_count\":1"));
    assert!(report_json.contains("\"container_compatibility_domain_table_hash\":\"0x"));
    assert!(report_json
        .contains("\"container_compatibility_domain_id\":\"compat0000.cffi-von-neumann\""));
    assert!(report_json.contains("\"container_compatibility_domain_kind\":\"cffi-host-compat\""));
    assert!(report_json
        .contains("\"container_compatibility_domain_paradigm\":\"classic-von-neumann-host\""));
    assert!(report_json
        .contains("\"container_compatibility_domain_lifecycle_hook\":\"on_cffi_native_object\""));
    assert!(report_json.contains("\"container_compatibility_domain_abi_family\":\"mach-o\""));
    assert!(report_json.contains("\"container_compatibility_domain_wrapper_policy\":\"wrapped\""));
    assert!(report_json.contains("\"container_compatibility_domain_required\":true"));
    assert!(report_json
        .contains("\"container_compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x"));
    assert!(report_json
        .contains("\"container_native_object_section_id\":\"sec0004.native-object-output\""));
    assert!(report_json
        .contains("\"container_native_object_loader_symbol_id\":\"sym0001.native-object-output\""));
    assert!(
        report_json.contains("\"container_native_object_relocation_id\":\"rel0001.native-object\"")
    );
    assert!(report
        .container_loader_blockers
        .iter()
        .any(|blocker| blocker == "external-import:final-stage-driver:clang"));
    assert!(report
        .issues
        .iter()
        .all(|issue| !issue.contains("container loader readiness is blocked")));
}

#[test]
fn check_reports_tampered_object_output() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-object-output-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::write(dir.join("nuis.nsld.mach-o"), b"damaged-object").unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.object_output_present);
    assert_eq!(report.object_output_valid, Some(false));
    assert_ne!(
        report.object_output_expected_size_bytes,
        report.object_output_actual_size_bytes
    );
    assert_ne!(
        report.object_output_expected_hash,
        report.object_output_actual_hash
    );
    assert!(report
        .object_output_issues
        .iter()
        .any(|issue| issue.contains("object_output_hash mismatch")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "object output verification failed"));
}
