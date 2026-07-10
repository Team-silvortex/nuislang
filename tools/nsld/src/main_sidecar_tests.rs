use super::{
    fnv1a64_hex, main_test_support::empty_link_plan, nsld_link_input_diagnostics,
    nsld_link_input_table_hash, nsld_sidecar_capability_diagnostics, toml,
};
use std::{env, fs};

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
        target_device: None,
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: None,
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
        target_device: None,
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: None,
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
