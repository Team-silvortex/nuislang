use super::*;

#[test]
fn lowering_contract_nodes_validate_cpu_target_config() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.x86_64"),
        }],
        nodes: vec![
            node(
                "lowering_cpu_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:x86_64;abi=symbol:cpu.x86_64.sysv64;vector_bits=i64:128"],
            ),
            node(
                "lowering_cpu_target_config",
                "cpu0",
                "cpu.target_config",
                &["x86_64", "cpu.x86_64.sysv64", "128"],
            ),
        ],
        edges: vec![dep(
            "lowering_cpu_target_contract_type",
            "lowering_cpu_target_config",
        )],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn lowering_contract_nodes_reject_cpu_target_vector_mismatch() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.x86_64"),
        }],
        nodes: vec![
            node(
                "lowering_cpu_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:x86_64;abi=symbol:cpu.x86_64.sysv64;vector_bits=i64:256"],
            ),
            node(
                "lowering_cpu_target_config",
                "cpu0",
                "cpu.target_config",
                &["x86_64", "cpu.x86_64.sysv64", "128"],
            ),
        ],
        edges: vec![dep(
            "lowering_cpu_target_contract_type",
            "lowering_cpu_target_config",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("encodes `vector_bits=256`"));
}
