use super::*;

#[test]
fn project_plan_accepts_darwin_x86_64_network_and_data_abis() {
    let project = test_support::loaded_project_fixture(
        "multidomain_darwin_x86_64",
        vec![
            darwin_x86_64_cpu_abi(),
            darwin_x86_64_network_abi(),
            darwin_x86_64_data_abi(),
        ],
        reverse_network_data_bridge_entry(),
        vec![
            (
                "network_data_bridge.ns",
                reverse_network_data_bridge_module(),
            ),
            ("network_unit.ns", multidomain_support_modules()[0].1),
            (
                "fabric_plane.ns",
                shader_fabric_plane_like_network_module().as_str(),
            ),
        ],
    );

    let plan = build_project_compilation_plan(&project).unwrap();
    let checks = validate_project_abi_selections(&project, &plan.abi_resolution).unwrap();

    assert!(checks.iter().all(|check| check.ok));
    assert!(checks.iter().any(|check| {
        check.domain == "network" && check.abi.as_deref() == Some("network.socket.macos.x86_64.v1")
    }));
    assert!(checks.iter().any(|check| {
        check.domain == "data" && check.abi.as_deref() == Some("data.fabric.macos.x86_64.v1")
    }));
}

fn shader_fabric_plane_like_network_module() -> String {
    r#"
    mod data FabricPlane {
      fn profile() {
        data_bind_core(1);
        let profile_handles: HandleTable<FabricPlaneBindings> =
          data_handle_table("host=cpu0", "network=network0");
        let cpu_to_network: Marker<CpuToNetwork> = data_marker("cpu_to_network");
        let network_to_cpu: Marker<NetworkToCpu> = data_marker("network_to_cpu");
        let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
        let downlink_pipe: Marker<DownlinkPipe> = data_marker("downlink_pipe");
        let uplink_pipe_class: Marker<UplinkPipeClass> = data_marker("uplink_pipe_class");
        let downlink_pipe_class: Marker<DownlinkPipeClass> = data_marker("downlink_pipe_class");
        let uplink_payload_class: Marker<PayloadClassWindow> = data_marker("uplink_payload_class");
        let downlink_payload_class: Marker<PayloadClassWindow> = data_marker("downlink_payload_class");
        let uplink_payload_shape: Marker<PayloadShapeWindowi64> = data_marker("uplink_payload_shape");
        let downlink_payload_shape: Marker<PayloadShapeWindowWindowi64> = data_marker("downlink_payload_shape");
        let uplink_window_policy: Marker<UplinkWindowPolicy> = data_marker("uplink_window_policy");
        let downlink_window_policy: Marker<DownlinkWindowPolicy> = data_marker("downlink_window_policy");
      }
    }
    "#
    .to_owned()
}
