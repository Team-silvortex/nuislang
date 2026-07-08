use super::*;

#[test]
fn rejects_invalid_project_bridge_stage_contract() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node(
                "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_bridge_stage_type",
                "cpu0",
                "cpu.text",
                &["uplink=ready;downlink=windowed"],
            ),
            node(
                "project_profile_data_FabricPlane_uplink_window_policy",
                "cpu0",
                "cpu.text",
                &["marker"],
            ),
        ],
        edges: vec![dep(
            "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_bridge_stage_type",
            "project_profile_data_FabricPlane_uplink_window_policy",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("uplink=windowed;downlink=windowed"));
}

#[test]
fn rejects_nested_data_window_values() {
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
            node("seed", "cpu0", "cpu.const", &["7"]),
            node("value", "fabric0", "data.move", &["seed", "cpu0"]),
            node(
                "window0",
                "fabric0",
                "data.immutable_window",
                &["value", "0", "1"],
            ),
            node(
                "window1",
                "fabric0",
                "data.copy_window",
                &["window0", "0", "1"],
            ),
        ],
        edges: vec![
            xfer("seed", "value"),
            dep("value", "window0"),
            dep("window0", "window1"),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("cannot create nested/illegal window"));
}

#[test]
fn rejects_mutable_window_payload_across_data_pipe() {
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
            node("seed", "cpu0", "cpu.const", &["7"]),
            node("value", "fabric0", "data.move", &["seed", "cpu0"]),
            node(
                "window0",
                "fabric0",
                "data.copy_window",
                &["value", "0", "1"],
            ),
            node("pipe", "fabric0", "data.output_pipe", &["window0"]),
        ],
        edges: vec![
            xfer("seed", "value"),
            dep("value", "window0"),
            dep("window0", "pipe"),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("cannot send mutable window payload"));
}

#[test]
fn accepts_frozen_window_payload_across_data_pipe() {
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
            node("seed", "cpu0", "cpu.const", &["7"]),
            node("value", "fabric0", "data.move", &["seed", "cpu0"]),
            node(
                "window0",
                "fabric0",
                "data.copy_window",
                &["value", "0", "1"],
            ),
            node("frozen", "fabric0", "data.freeze_window", &["window0"]),
            node("pipe", "fabric0", "data.output_pipe", &["frozen"]),
        ],
        edges: vec![
            xfer("seed", "value"),
            dep("value", "window0"),
            dep("window0", "frozen"),
            dep("frozen", "pipe"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn rejects_write_window_on_immutable_input() {
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
            node("seed", "cpu0", "cpu.const", &["7"]),
            node("value", "fabric0", "data.move", &["seed", "cpu0"]),
            node(
                "window0",
                "fabric0",
                "data.immutable_window",
                &["value", "0", "1"],
            ),
            node(
                "updated",
                "fabric0",
                "data.write_window",
                &["window0", "0", "value"],
            ),
        ],
        edges: vec![
            xfer("seed", "value"),
            dep("value", "window0"),
            dep("window0", "updated"),
            dep("value", "updated"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap_err();
}

#[test]
fn accepts_read_window_on_immutable_input() {
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
            node("seed", "cpu0", "cpu.const", &["7"]),
            node("value", "fabric0", "data.move", &["seed", "cpu0"]),
            node(
                "window0",
                "fabric0",
                "data.immutable_window",
                &["value", "0", "1"],
            ),
            node("read", "fabric0", "data.read_window", &["window0", "0"]),
        ],
        edges: vec![
            xfer("seed", "value"),
            dep("value", "window0"),
            dep("window0", "read"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn rejects_bridge_payload_shape_mismatch() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node(
                "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_uplink_bridge_payload_type",
                "cpu0",
                "cpu.text",
                &["Window<SurfaceShaderPacket>"],
            ),
            node(
                "project_profile_data_FabricPlane_uplink_payload_shape",
                "cpu0",
                "cpu.text",
                &["uplink_payload_shape"],
            ),
            node(
                "project_profile_data_FabricPlane_uplink_payload_shape_type",
                "cpu0",
                "cpu.text",
                &["PayloadShapeWindowFrame"],
            ),
        ],
        edges: vec![
            dep(
                "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_uplink_bridge_payload_type",
                "project_profile_data_FabricPlane_uplink_payload_shape",
            ),
            dep(
                "project_profile_data_FabricPlane_uplink_payload_shape_type",
                "project_profile_data_FabricPlane_uplink_payload_shape",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("PayloadShapeWindowSurfaceShaderPacket"));
    assert!(error.contains("PayloadShapeWindowFrame"));
}
