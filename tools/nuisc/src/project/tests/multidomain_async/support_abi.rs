use super::*;

pub(super) fn darwin_x86_64_cpu_abi() -> ProjectAbiRequirement {
    ProjectAbiRequirement {
        domain: "cpu".to_owned(),
        abi: "cpu.x86_64.apple_sysv64".to_owned(),
    }
}

pub(super) fn darwin_x86_64_network_abi() -> ProjectAbiRequirement {
    ProjectAbiRequirement {
        domain: "network".to_owned(),
        abi: "network.socket.macos.x86_64.v1".to_owned(),
    }
}

pub(super) fn darwin_x86_64_data_abi() -> ProjectAbiRequirement {
    ProjectAbiRequirement {
        domain: "data".to_owned(),
        abi: "data.fabric.macos.x86_64.v1".to_owned(),
    }
}

// Base project builders and shared support-module fixtures for multidomain tests.
pub(super) fn multidomain_project_with_entry(
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> LoadedProject {
    test_support::loaded_project_fixture(
        "multidomain_test",
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
                domain: "kernel".to_owned(),
                abi: "kernel.apple_ane.coreml.v1".to_owned(),
            },
        ],
        entry_source,
        extra_modules,
    )
}

pub(super) fn multidomain_support_modules() -> Vec<(&'static str, &'static str)> {
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
            "kernel_unit.ns",
            r#"
            use data FabricPlane;

            mod kernel KernelUnit {
              fn profile() {
                const bind_core: i64 = 2;
                const queue_depth: i64 = 8;
                const batch_lanes: i64 = 16;
                let profile_entry: Unit = kernel_target_config("apple_ane", "coreml", batch_lanes);
              }
            }
            "#,
        ),
    ]
}

pub(super) fn kernel_data_support_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "kernel_unit.ns",
            r#"
            use data FabricPlane;

            mod kernel KernelUnit {
              fn profile() {
                const bind_core: i64 = 2;
                const queue_depth: i64 = 8;
                const batch_lanes: i64 = 16;
                let profile_entry: Unit = kernel_target_config("apple_ane", "coreml", batch_lanes);
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
                  data_handle_table("host=cpu0", "compute=kernel0");
                let cpu_to_kernel: Marker<CpuToKernel> = data_marker("cpu_to_kernel");
                let kernel_to_cpu: Marker<KernelToCpu> = data_marker("kernel_to_cpu");
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

// Reusable source snippets for bridge/task-helper entrypoints and route-validation variants.
