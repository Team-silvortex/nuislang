use super::*;

#[test]
fn project_contract_nodes_validate_data_shader_and_kernel_links() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "fabric0".to_owned(),
                kind: ResourceKind::parse("data.fabric"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
            Resource {
                name: "kernel0".to_owned(),
                kind: ResourceKind::parse("kernel.apple"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_data_FabricPlane_uplink_payload_class_type",
                "cpu0",
                "cpu.text",
                &["PayloadClassWindow"],
            ),
            node(
                "project_profile_data_FabricPlane_uplink_payload_class",
                "fabric0",
                "data.marker",
                &["uplink_payload_class"],
            ),
            node(
                "project_profile_data_FabricPlane_handle_table_schema_type",
                "cpu0",
                "cpu.text",
                &["FabricPlaneBindings"],
            ),
            node(
                "project_profile_data_FabricPlane_profile_handles",
                "fabric0",
                "data.handle_table",
                &["color=shader0"],
            ),
            node(
                "project_profile_shader_SurfaceShader_packet_type",
                "cpu0",
                "cpu.text",
                &["SurfaceShaderPacket"],
            ),
            node(
                "project_profile_shader_SurfaceShader_packet_class_type",
                "cpu0",
                "cpu.text",
                &["PayloadClassValue"],
            ),
            node(
                "project_profile_shader_SurfaceShader_packet_shape_type",
                "cpu0",
                "cpu.text",
                &["PayloadShapeSurfaceShaderPacket"],
            ),
            node(
                "project_profile_shader_SurfaceShader_packet_field_count",
                "cpu0",
                "cpu.const_i64",
                &["3"],
            ),
            node(
                "project_profile_kernel_KernelUnit_slot_contract_type",
                "cpu0",
                "cpu.text",
                &["bind_core=i64:2;queue_depth=i64:8;batch_lanes=i64:16"],
            ),
            node(
                "project_profile_kernel_KernelUnit_profile_entry",
                "kernel0",
                "kernel.target_config",
                &["apple_ane", "coreml", "16"],
            ),
        ],
        edges: vec![
            xfer(
                "project_profile_data_FabricPlane_uplink_payload_class_type",
                "project_profile_data_FabricPlane_uplink_payload_class",
            ),
            xfer(
                "project_profile_data_FabricPlane_handle_table_schema_type",
                "project_profile_data_FabricPlane_profile_handles",
            ),
            dep(
                "project_profile_shader_SurfaceShader_packet_type",
                "project_profile_shader_SurfaceShader_packet_field_count",
            ),
            dep(
                "project_profile_shader_SurfaceShader_packet_class_type",
                "project_profile_shader_SurfaceShader_packet_field_count",
            ),
            dep(
                "project_profile_shader_SurfaceShader_packet_shape_type",
                "project_profile_shader_SurfaceShader_packet_field_count",
            ),
            xfer(
                "project_profile_kernel_KernelUnit_slot_contract_type",
                "project_profile_kernel_KernelUnit_profile_entry",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn project_contract_nodes_require_contract_edge() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "fabric0".to_owned(),
                kind: ResourceKind::parse("data.fabric"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_data_FabricPlane_uplink_payload_shape_type",
                "cpu0",
                "cpu.text",
                &["PayloadShapeWindowFrame"],
            ),
            node(
                "project_profile_data_FabricPlane_uplink_payload_shape",
                "fabric0",
                "data.marker",
                &["uplink_payload_shape"],
            ),
        ],
        edges: vec![],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("requires dep/xfer edge"));
}

#[test]
fn project_contract_nodes_reject_kernel_slot_mismatch() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "kernel0".to_owned(),
                kind: ResourceKind::parse("kernel.apple"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_kernel_KernelUnit_slot_contract_type",
                "cpu0",
                "cpu.text",
                &["bind_core=i64:2;queue_depth=i64:8;batch_lanes=i64:12"],
            ),
            node(
                "project_profile_kernel_KernelUnit_profile_entry",
                "kernel0",
                "kernel.target_config",
                &["apple_ane", "coreml", "16"],
            ),
        ],
        edges: vec![xfer(
            "project_profile_kernel_KernelUnit_slot_contract_type",
            "project_profile_kernel_KernelUnit_profile_entry",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("encodes `batch_lanes=12`"));
}

#[test]
fn project_contract_nodes_validate_kernel_target_config() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "kernel0".to_owned(),
                kind: ResourceKind::parse("kernel.apple"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_kernel_KernelUnit_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:apple_ane;runtime=symbol:coreml;lane_width=i64:1"],
            ),
            node(
                "project_profile_kernel_KernelUnit_kernel_target_config_auto",
                "kernel0",
                "kernel.target_config",
                &["apple_ane", "coreml", "1"],
            ),
        ],
        edges: vec![xfer(
            "project_profile_kernel_KernelUnit_target_contract_type",
            "project_profile_kernel_KernelUnit_kernel_target_config_auto",
        )],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn project_contract_nodes_reject_kernel_target_runtime_mismatch() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "kernel0".to_owned(),
                kind: ResourceKind::parse("kernel.apple"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_kernel_KernelUnit_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:apple_ane;runtime=symbol:mlx;lane_width=i64:1"],
            ),
            node(
                "project_profile_kernel_KernelUnit_kernel_target_config_auto",
                "kernel0",
                "kernel.target_config",
                &["apple_ane", "coreml", "1"],
            ),
        ],
        edges: vec![xfer(
            "project_profile_kernel_KernelUnit_target_contract_type",
            "project_profile_kernel_KernelUnit_kernel_target_config_auto",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("encodes `runtime=mlx`"));
}

#[test]
fn project_contract_nodes_validate_shader_and_network_target_configs() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
            Resource {
                name: "network0".to_owned(),
                kind: ResourceKind::parse("network.urlsession"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_shader_SurfaceShader_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:arm64;runtime=symbol:metal;lane_width=i64:1"],
            ),
            node(
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
                "shader0",
                "shader.target_config",
                &["arm64", "metal", "1"],
            ),
            node(
                "project_profile_network_HttpLink_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:arm64;runtime=symbol:urlsession;lane_width=i64:1"],
            ),
            node(
                "project_profile_network_HttpLink_network_target_config_auto",
                "network0",
                "network.target_config",
                &["arm64", "urlsession", "1"],
            ),
        ],
        edges: vec![
            xfer(
                "project_profile_shader_SurfaceShader_target_contract_type",
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
            ),
            xfer(
                "project_profile_network_HttpLink_target_contract_type",
                "project_profile_network_HttpLink_network_target_config_auto",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn project_contract_nodes_validate_shader_abi_selection_contract() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_shader_SurfaceShader_abi_selection_contract_type",
                "cpu0",
                "cpu.text",
                &["mode=symbol:explicit;abi=symbol:shader.metal.msl2_4;arch=symbol:arm64;runtime=symbol:metal;lane_width=i64:1"],
            ),
            node(
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
                "shader0",
                "shader.target_config",
                &["arm64", "metal", "1"],
            ),
        ],
        edges: vec![xfer(
            "project_profile_shader_SurfaceShader_abi_selection_contract_type",
            "project_profile_shader_SurfaceShader_shader_target_config_auto",
        )],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn project_contract_nodes_validate_project_cpu_abi_summary() {
    let payload = "mode=symbol:explicit;abi=symbol:cpu.arm64.apple_aapcs64;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:none";
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node(
                "project_abi_cpu_selection_summary_type",
                "cpu0",
                "cpu.text",
                &[payload],
            ),
            node(
                "project_abi_cpu_selection_entry",
                "cpu0",
                "cpu.text",
                &[payload],
            ),
        ],
        edges: vec![dep(
            "project_abi_cpu_selection_summary_type",
            "project_abi_cpu_selection_entry",
        )],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn project_contract_nodes_validate_project_abi_graph_summary() {
    let payload = "mode=symbol:explicit;domains=symbol:cpu,data;cpu_summary=symbol:present;data_summary=symbol:present;kernel_target=symbol:absent;shader_target=symbol:absent;network_target=symbol:absent";
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node(
                "project_abi_graph_summary_type",
                "cpu0",
                "cpu.text",
                &[payload],
            ),
            node(
                "project_abi_graph_summary_entry",
                "cpu0",
                "cpu.text",
                &[payload],
            ),
        ],
        edges: vec![dep(
            "project_abi_graph_summary_type",
            "project_abi_graph_summary_entry",
        )],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn project_contract_nodes_reject_project_data_abi_summary_invalid_mode() {
    let bad = "mode=symbol:recommended;abi=symbol:data.fabric.host-match.v1;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:none";
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node(
                "project_abi_data_selection_summary_type",
                "cpu0",
                "cpu.text",
                &[bad],
            ),
            node(
                "project_abi_data_selection_entry",
                "cpu0",
                "cpu.text",
                &[bad],
            ),
        ],
        edges: vec![dep(
            "project_abi_data_selection_summary_type",
            "project_abi_data_selection_entry",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("requires `mode` to be `explicit` or `auto`"));
}

#[test]
fn project_contract_nodes_reject_shader_abi_selection_mode_mismatch() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_shader_SurfaceShader_abi_selection_contract_type",
                "cpu0",
                "cpu.text",
                &["mode=symbol:recommended;abi=symbol:shader.metal.msl2_4;arch=symbol:arm64;runtime=symbol:metal;lane_width=i64:1"],
            ),
            node(
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
                "shader0",
                "shader.target_config",
                &["arm64", "metal", "1"],
            ),
        ],
        edges: vec![xfer(
            "project_profile_shader_SurfaceShader_abi_selection_contract_type",
            "project_profile_shader_SurfaceShader_shader_target_config_auto",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("requires `mode` to be `explicit` or `auto`"));
}

#[test]
fn project_contract_nodes_reject_shader_target_runtime_mismatch() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_shader_SurfaceShader_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:arm64;runtime=symbol:vulkan;lane_width=i64:1"],
            ),
            node(
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
                "shader0",
                "shader.target_config",
                &["arm64", "metal", "1"],
            ),
        ],
        edges: vec![xfer(
            "project_profile_shader_SurfaceShader_target_contract_type",
            "project_profile_shader_SurfaceShader_shader_target_config_auto",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("encodes `runtime=vulkan`"));
}

#[test]
fn project_contract_nodes_reject_network_target_lane_width_mismatch() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "network0".to_owned(),
                kind: ResourceKind::parse("network.urlsession"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_network_HttpLink_target_contract_type",
                "cpu0",
                "cpu.text",
                &["arch=symbol:arm64;runtime=symbol:urlsession;lane_width=i64:4"],
            ),
            node(
                "project_profile_network_HttpLink_network_target_config_auto",
                "network0",
                "network.target_config",
                &["arm64", "urlsession", "1"],
            ),
        ],
        edges: vec![xfer(
            "project_profile_network_HttpLink_target_contract_type",
            "project_profile_network_HttpLink_network_target_config_auto",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("encodes `lane_width=4`"));
}

#[path = "project_lowering_contracts.rs"]
mod project_lowering_contracts;
