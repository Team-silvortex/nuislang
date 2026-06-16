use super::assign_default_lanes;
use yir_core::{Node, Operation, Resource, ResourceKind, YirModule};

fn push_node(module: &mut YirModule, name: &str, resource: &str, op: Operation) {
    module.nodes.push(Node {
        name: name.to_owned(),
        resource: resource.to_owned(),
        op,
    });
}

#[test]
fn classifies_contract_metadata_and_project_profile_nodes_into_stable_lanes() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.arm64"),
    });
    module.resources.push(Resource {
        name: "data0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    });
    module.resources.push(Resource {
        name: "shader0".to_owned(),
        kind: ResourceKind::parse("shader.metal"),
    });
    module.resources.push(Resource {
        name: "kernel0".to_owned(),
        kind: ResourceKind::parse("kernel.metal"),
    });

    push_node(
        &mut module,
        "scheduler_contract_cpu_lane_policy_type",
        "cpu0",
        Operation::parse("cpu.text", vec!["family=cpu".to_owned()]).unwrap(),
    );
    push_node(
        &mut module,
        "lowering_cpu_target_config",
        "cpu0",
        Operation::parse(
            "cpu.target_config",
            vec![
                "arm64".to_owned(),
                "cpu.arm64.apple_aapcs64".to_owned(),
                "128".to_owned(),
            ],
        )
        .unwrap(),
    );
    push_node(
        &mut module,
        "project_link_cpu_Main_shader_Surface_bridge_stage_type",
        "cpu0",
        Operation::parse("cpu.text", vec!["stage=symbol:windowed".to_owned()]).unwrap(),
    );
    push_node(
        &mut module,
        "project_abi_cpu_selection_summary_type",
        "cpu0",
        Operation::parse(
            "cpu.text",
            vec!["mode=symbol:auto;abi=symbol:cpu.arm64.apple_aapcs64".to_owned()],
        )
        .unwrap(),
    );
    push_node(
        &mut module,
        "project_profile_data_Fabric_uplink_payload_shape_type",
        "cpu0",
        Operation::parse("cpu.text", vec!["PayloadShapeI64".to_owned()]).unwrap(),
    );
    push_node(
        &mut module,
        "project_profile_cpu_Main_target_contract_type",
        "cpu0",
        Operation::parse("cpu.text", vec!["arch=symbol:arm64".to_owned()]).unwrap(),
    );
    push_node(
        &mut module,
        "project_profile_cpu_Main_profile_entry",
        "cpu0",
        Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    );
    push_node(
        &mut module,
        "project_profile_data_Fabric_handle_table",
        "data0",
        Operation::parse("data.handle_table", vec!["host=cpu0".to_owned()]).unwrap(),
    );
    push_node(
        &mut module,
        "project_profile_shader_Surface_shader_target_config_auto",
        "shader0",
        Operation::parse(
            "shader.target_config",
            vec!["arm64".to_owned(), "metal".to_owned(), "1".to_owned()],
        )
        .unwrap(),
    );
    push_node(
        &mut module,
        "project_profile_kernel_Worker_kernel_target_config_auto",
        "kernel0",
        Operation::parse(
            "kernel.target_config",
            vec!["apple_ane".to_owned(), "coreml".to_owned(), "32".to_owned()],
        )
        .unwrap(),
    );
    push_node(
        &mut module,
        "project_link_instantiate_shader_SurfaceShader",
        "cpu0",
        Operation::parse(
            "cpu.instantiate_unit",
            vec!["shader".to_owned(), "SurfaceShader".to_owned()],
        )
        .unwrap(),
    );

    assign_default_lanes(&mut module);

    assert_eq!(
        module
            .node_lanes
            .get("scheduler_contract_cpu_lane_policy_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        module
            .node_lanes
            .get("lowering_cpu_target_config")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_link_cpu_Main_shader_Surface_bridge_stage_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_abi_cpu_selection_summary_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_profile_data_Fabric_uplink_payload_shape_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_profile_cpu_Main_target_contract_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_profile_cpu_Main_profile_entry")
            .map(String::as_str),
        Some("profile")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_profile_data_Fabric_handle_table")
            .map(String::as_str),
        Some("profile_control")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_profile_shader_Surface_shader_target_config_auto")
            .map(String::as_str),
        Some("profile_setup")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_profile_kernel_Worker_kernel_target_config_auto")
            .map(String::as_str),
        Some("profile_compute")
    );
    assert_eq!(
        module
            .node_lanes
            .get("project_link_instantiate_shader_SurfaceShader")
            .map(String::as_str),
        Some("main")
    );
}
