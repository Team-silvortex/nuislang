use super::*;

pub(super) fn network_fabric_plane_module(include_network_to_cpu: bool) -> String {
    let reverse_marker = if include_network_to_cpu {
        "            let network_to_cpu: Marker<NetworkToCpu> = data_marker(\"network_to_cpu\");\n"
    } else {
        ""
    };
    format!(
        r#"
        mod data FabricPlane {{
          fn profile() {{
            const window_offset: i64 = 0;
            const uplink_len: i64 = 1;
            const downlink_len: i64 = 1;

            data_bind_core(1);
            let profile_handles: HandleTable<FabricPlaneBindings> =
              data_handle_table("host=cpu0", "network=network0");
            let cpu_to_network: Marker<CpuToNetwork> = data_marker("cpu_to_network");
{reverse_marker}            let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
            let downlink_pipe: Marker<DownlinkPipe> = data_marker("downlink_pipe");
            let uplink_pipe_class: Marker<UplinkPipeClass> = data_marker("uplink_pipe_class");
            let downlink_pipe_class: Marker<DownlinkPipeClass> = data_marker("downlink_pipe_class");
            let uplink_payload_class: Marker<PayloadClassWindow> = data_marker("uplink_payload_class");
            let downlink_payload_class: Marker<PayloadClassWindow> =
              data_marker("downlink_payload_class");
            let uplink_payload_shape: Marker<PayloadShapeWindowi64> =
              data_marker("uplink_payload_shape");
            let downlink_payload_shape: Marker<PayloadShapeWindowWindowi64> =
              data_marker("downlink_payload_shape");
            let uplink_window_policy: Marker<UplinkWindowPolicy> =
              data_marker("uplink_window_policy");
            let downlink_window_policy: Marker<DownlinkWindowPolicy> =
              data_marker("downlink_window_policy");
            let uplink_window_mut: WindowMut<i64> =
              data_copy_window(window_offset, window_offset, uplink_len);
            let uplink_window: Window<i64> = data_freeze_window(uplink_window_mut);
            let downlink_window_mut: WindowMut<i64> =
              data_copy_window(window_offset, window_offset, downlink_len);
            let downlink_window: Window<i64> = data_freeze_window(downlink_window_mut);
          }}
        }}
        "#
    )
}

pub(super) fn kernel_fabric_plane_module(include_kernel_to_cpu: bool) -> String {
    let reverse_marker = if include_kernel_to_cpu {
        "            let kernel_to_cpu: Marker<KernelToCpu> = data_marker(\"kernel_to_cpu\");\n"
    } else {
        ""
    };
    format!(
        r#"
        mod data FabricPlane {{
          fn profile() {{
            const window_offset: i64 = 0;
            const uplink_len: i64 = 1;
            const downlink_len: i64 = 1;

            data_bind_core(1);
            let profile_handles: HandleTable<FabricPlaneBindings> =
              data_handle_table("host=cpu0", "compute=kernel0");
            let cpu_to_kernel: Marker<CpuToKernel> = data_marker("cpu_to_kernel");
{reverse_marker}            let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
            let downlink_pipe: Marker<DownlinkPipe> = data_marker("downlink_pipe");
            let uplink_pipe_class: Marker<UplinkPipeClass> = data_marker("uplink_pipe_class");
            let downlink_pipe_class: Marker<DownlinkPipeClass> = data_marker("downlink_pipe_class");
            let uplink_payload_class: Marker<PayloadClassWindow> = data_marker("uplink_payload_class");
            let downlink_payload_class: Marker<PayloadClassWindow> =
              data_marker("downlink_payload_class");
            let uplink_payload_shape: Marker<PayloadShapeWindowi64> =
              data_marker("uplink_payload_shape");
            let downlink_payload_shape: Marker<PayloadShapeWindowWindowi64> =
              data_marker("downlink_payload_shape");
            let uplink_window_policy: Marker<UplinkWindowPolicy> =
              data_marker("uplink_window_policy");
            let downlink_window_policy: Marker<DownlinkWindowPolicy> =
              data_marker("downlink_window_policy");
            let uplink_window_mut: WindowMut<i64> =
              data_copy_window(window_offset, window_offset, uplink_len);
            let uplink_window: Window<i64> = data_freeze_window(uplink_window_mut);
            let downlink_window_mut: WindowMut<i64> =
              data_copy_window(window_offset, window_offset, downlink_len);
            let downlink_window: Window<i64> = data_freeze_window(downlink_window_mut);
          }}
        }}
        "#
    )
}

// Higher-level project builders that bind common entry/module combinations to manifest links.
pub(super) fn reverse_network_data_bridge_project(bridge_source: &'static str) -> LoadedProject {
    network_data_project_with_entry(reverse_network_data_bridge_entry(), {
        let mut modules = network_data_support_modules();
        modules.push(("network_data_bridge.ns", bridge_source));
        modules
    })
}

pub(super) fn reverse_network_data_bridge_project_with_link(
    bridge_source: &'static str,
    from: &str,
    to: &str,
) -> LoadedProject {
    let mut project = reverse_network_data_bridge_project(bridge_source);
    project.manifest.links = vec![ProjectLink {
        from: from.to_owned(),
        to: to.to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];
    project
}

pub(super) fn forward_network_data_bridge_project(bridge_source: &'static str) -> LoadedProject {
    reverse_network_data_bridge_project(bridge_source)
}

pub(super) fn forward_network_data_bridge_project_with_link(
    bridge_source: &'static str,
) -> LoadedProject {
    let mut project = forward_network_data_bridge_project(bridge_source);
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];
    project
}

pub(super) fn kernel_task_async_shapes_project() -> LoadedProject {
    kernel_data_project_with_entry(kernel_task_async_shapes_entry(), {
        let mut modules = kernel_data_support_modules();
        modules.push((
            "kernel_task_async_shapes.ns",
            kernel_task_async_shapes_module(),
        ));
        modules
    })
}

pub(super) fn kernel_task_async_shapes_project_with_link() -> LoadedProject {
    let mut project = kernel_task_async_shapes_project();
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "kernel.KernelUnit".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];
    project
}

pub(super) fn network_task_async_project(
    entry_source: &str,
    module_source: &'static str,
) -> LoadedProject {
    multidomain_project_with_entry(entry_source, {
        let mut modules = multidomain_support_modules();
        modules.push(("network_task_async_shapes.ns", module_source));
        modules
    })
}

pub(super) fn network_task_async_project_with_link(
    entry_source: &str,
    module_source: &'static str,
) -> LoadedProject {
    let mut project = network_task_async_project(entry_source, module_source);
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];
    project
}

pub(super) fn direct_network_project_with_link(
    entry_source: &str,
    modules: Vec<(&'static str, &'static str)>,
) -> LoadedProject {
    let mut project = multidomain_project_with_entry(entry_source, modules);
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];
    project
}

pub(super) fn direct_network_default_project_with_link(entry_source: &str) -> LoadedProject {
    direct_network_project_with_link(entry_source, multidomain_support_modules())
}

pub(super) fn network_data_support_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "network_unit.ns",
            r#"
            use data FabricPlane;

            mod network NetworkUnit {
              fn profile() {
                const bind_core: i64 = 2;
                const endpoint_kind: i64 = 1;
                const transport_family: i64 = 6;
                const local_port: i64 = 9000;
                const remote_port: i64 = 443;
                const connect_timeout_ms: i64 = 250;
                const read_timeout_ms: i64 = 125;
                const write_timeout_ms: i64 = 150;
                const retry_budget: i64 = 3;
                const stream_window: i64 = 64;
                const recv_window: i64 = 32;
                const send_window: i64 = 32;
                const protocol_kind: i64 = 101;
                const protocol_version: i64 = 2;
                const protocol_header_bytes: i64 = 24;
              }
            }
            "#,
        ),
        (
            "fabric_plane.ns",
            r#"
            mod data FabricPlane {
              fn profile() {
                const window_offset: i64 = 0;
                const uplink_len: i64 = 1;
                const downlink_len: i64 = 1;

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
                let downlink_payload_class: Marker<PayloadClassWindow> =
                  data_marker("downlink_payload_class");
                let uplink_payload_shape: Marker<PayloadShapeWindowi64> =
                  data_marker("uplink_payload_shape");
                let downlink_payload_shape: Marker<PayloadShapeWindowWindowi64> =
                  data_marker("downlink_payload_shape");
                let uplink_window_policy: Marker<UplinkWindowPolicy> =
                  data_marker("uplink_window_policy");
                let downlink_window_policy: Marker<DownlinkWindowPolicy> =
                  data_marker("downlink_window_policy");
                let uplink_window_mut: WindowMut<i64> =
                  data_copy_window(window_offset, window_offset, uplink_len);
                let uplink_window: Window<i64> = data_freeze_window(uplink_window_mut);
                let downlink_window_mut: WindowMut<i64> =
                  data_copy_window(window_offset, window_offset, downlink_len);
                let downlink_window: Window<i64> = data_freeze_window(downlink_window_mut);
              }
            }
            "#,
        ),
    ]
}

// Temp project writers used by compile-path smoke tests.
pub(super) fn kernel_data_project_with_entry(
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> LoadedProject {
    test_support::loaded_project_fixture(
        "kernel_data_test",
        vec![
            ProjectAbiRequirement {
                domain: "cpu".to_owned(),
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "kernel".to_owned(),
                abi: "kernel.apple_ane.coreml.v1".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "data".to_owned(),
                abi: "data.fabric.macos.arm64.v1".to_owned(),
            },
        ],
        entry_source,
        extra_modules,
    )
}

pub(super) fn network_data_project_with_entry(
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> LoadedProject {
    test_support::loaded_project_fixture(
        "network_data_test",
        vec![
            ProjectAbiRequirement {
                domain: "cpu".to_owned(),
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "network".to_owned(),
                abi: "network.socket.macos.arm64.v1".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "data".to_owned(),
                abi: "data.fabric.macos.arm64.v1".to_owned(),
            },
        ],
        entry_source,
        extra_modules,
    )
}

pub(super) fn write_temp_project(
    name: &str,
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> PathBuf {
    test_support::write_temp_project_fixture(
        name,
        r#"
name = "multidomain_test"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns", "network_unit.ns", "kernel_unit.ns"]
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "network=network.socket.macos.arm64.v1",
  "kernel=kernel.apple_ane.coreml.v1",
]
"#
        .trim_start(),
        entry_source,
        extra_modules,
    )
}

pub(super) fn write_temp_network_data_project(
    name: &str,
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
    links: &[&str],
) -> PathBuf {
    let mut manifest = r#"
name = "network_data_test"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns", "network_unit.ns", "fabric_plane.ns", "network_data_bridge.ns"]
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "network=network.socket.macos.arm64.v1",
  "data=data.fabric.macos.arm64.v1",
]
"#
    .trim_start()
    .to_owned();
    test_support::append_manifest_links(&mut manifest, links);
    test_support::write_temp_project_fixture(name, &manifest, entry_source, extra_modules)
}

pub(super) fn write_temp_kernel_data_project(
    name: &str,
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
    links: &[&str],
) -> PathBuf {
    let mut manifest = r#"
name = "kernel_data_test"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns", "kernel_unit.ns", "fabric_plane.ns", "kernel_data_bridge.ns"]
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "kernel=kernel.apple_ane.coreml.v1",
  "data=data.fabric.macos.arm64.v1",
]
"#
    .trim_start()
    .to_owned();
    test_support::append_manifest_links(&mut manifest, links);
    test_support::write_temp_project_fixture(name, &manifest, entry_source, extra_modules)
}

// Compile-path smoke tests.
