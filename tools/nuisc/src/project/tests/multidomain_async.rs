use super::*;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn multidomain_project_with_entry(
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> LoadedProject {
    let mut modules = vec![("main.ns", entry_source)];
    modules.extend(extra_modules);

    LoadedProject {
        root: PathBuf::from("."),
        manifest_path: PathBuf::from("nuis.toml"),
        manifest: NuisProjectManifest {
            name: "multidomain_test".to_owned(),
            entry: "main.ns".to_owned(),
            modules: modules.iter().map(|(path, _)| (*path).to_owned()).collect(),
            tests: vec![],
            links: vec![],
            abi_requirements: vec![
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
            galaxy_dependencies: vec![],
        },
        entry_path: PathBuf::from("main.ns"),
        entry_source: entry_source.to_owned(),
        modules: modules
            .into_iter()
            .map(|(path, source)| ProjectModule {
                path: PathBuf::from(path),
                ast: crate::frontend::parse_nuis_ast(source).unwrap(),
            })
            .collect(),
    }
}

fn multidomain_support_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "network_unit.ns",
            r#"
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

fn kernel_data_support_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "kernel_unit.ns",
            r#"
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

fn kernel_data_project_with_entry(
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> LoadedProject {
    let mut modules = vec![("main.ns", entry_source)];
    modules.extend(extra_modules);

    LoadedProject {
        root: PathBuf::from("."),
        manifest_path: PathBuf::from("nuis.toml"),
        manifest: NuisProjectManifest {
            name: "kernel_data_test".to_owned(),
            entry: "main.ns".to_owned(),
            modules: modules.iter().map(|(path, _)| (*path).to_owned()).collect(),
            tests: vec![],
            links: vec![],
            abi_requirements: vec![
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
            galaxy_dependencies: vec![],
        },
        entry_path: PathBuf::from("main.ns"),
        entry_source: entry_source.to_owned(),
        modules: modules
            .into_iter()
            .map(|(path, source)| ProjectModule {
                path: PathBuf::from(path),
                ast: crate::frontend::parse_nuis_ast(source).unwrap(),
            })
            .collect(),
    }
}

fn write_temp_project(name: &str, entry_source: &str, extra_modules: Vec<(&str, &str)>) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuisc_{name}_{nonce}"));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("nuis.toml"),
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
    )
    .unwrap();
    fs::write(root.join("main.ns"), entry_source).unwrap();
    for (path, source) in extra_modules {
        fs::write(root.join(path), source).unwrap();
    }
    root
}

#[test]
fn compiles_multidomain_async_probe_project() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;
        use kernel KernelUnit;

        mod cpu Main {
          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_config_ready(result) {
              return network_value(result) + 3;
            }
            return 0;
          }

          async fn consume_kernel_result(result: KernelResult<i64>) -> i64 {
            if kernel_config_ready(result) {
              return kernel_value(result) + 5;
            }
            return 0;
          }

          fn main() -> i64 {
            let network_probe: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let kernel_probe: KernelResult<i64> =
              kernel_result(kernel_profile_batch_lanes("KernelUnit"));

            let network_task: Task<i64> = spawn(consume_network_result(network_probe));
            let kernel_task: Task<i64> = spawn(consume_kernel_result(kernel_probe));

            let network_joined: TaskResult<i64> = join_result(network_task);
            let kernel_joined: TaskResult<i64> = join_result(kernel_task);

            if task_completed(network_joined) {
              return task_value(network_joined);
            }
            if task_completed(kernel_joined) {
              return task_value(kernel_joined);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let plan = build_project_compilation_plan(&project).unwrap();
    let artifacts = crate::pipeline::compile_project_plan(&project, &plan).unwrap();

    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.network"));
    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.kernel"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
}

#[test]
fn compiles_multidomain_data_orchestration_project_after_cycle_fix() {
    let root = write_temp_project(
        "multidomain_data_orchestration",
        r#"
        use network NetworkUnit;
        use kernel KernelUnit;

        mod cpu Main {
          struct MultiDomainAsyncSummary {
            payload_ready: i64,
            orchestrated_value: i64
          }

          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_send_ready(result) {
              return network_value(result) + 1;
            }
            if network_recv_ready(result) {
              return network_value(result) + 1;
            }
            if network_config_ready(result) {
              return network_value(result) + 3;
            }
            return 0;
          }

          async fn consume_kernel_result(result: KernelResult<i64>) -> i64 {
            if kernel_config_ready(result) {
              return kernel_value(result) + 5;
            }
            return 0;
          }

          fn orchestrate(seed: i64) -> i64 {
            let payload: DataResult<i64> =
              data_result(data_input_pipe(data_output_pipe(seed)));
            let base: i64 = if data_ready(payload) {
              data_value(payload)
            } else {
              0
            };

            let net_probe: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let kernel_probe: KernelResult<i64> =
              kernel_result(kernel_profile_batch_lanes("KernelUnit"));

            let network_task: Task<i64> = spawn(consume_network_result(net_probe));
            let kernel_task: Task<i64> = spawn(consume_kernel_result(kernel_probe));

            let network_result_joined: TaskResult<i64> = join_result(network_task);
            let kernel_result_joined: TaskResult<i64> = join_result(kernel_task);

            if task_completed(network_result_joined) {
              return base + task_value(network_result_joined);
            }
            if task_completed(kernel_result_joined) {
              return base + task_value(kernel_result_joined);
            }
            return base;
          }

          fn encode_data_ready(result: DataResult<i64>) -> i64 {
            if data_ready(result) {
              return 1;
            }
            return 0;
          }

          fn capture_multidomain_async_summary(seed: i64) -> MultiDomainAsyncSummary {
            let payload: DataResult<i64> =
              data_result(data_input_pipe(data_output_pipe(seed + 4)));

            return MultiDomainAsyncSummary {
              payload_ready: encode_data_ready(payload),
              orchestrated_value: orchestrate(seed)
            };
          }

          fn main() {
            let summary: MultiDomainAsyncSummary =
              capture_multidomain_async_summary(7);
            print(
              summary.payload_ready
                + summary.orchestrated_value
            );
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.network"));
    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.kernel"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "observe"));
}

#[test]
fn infers_kernel_data_route_payload_types_through_shared_cpu_helper() {
    let project = kernel_data_project_with_entry(
        r#"
        use cpu KernelTaskAsyncShapes;
        use data FabricPlane;
        use kernel KernelUnit;

        mod cpu Main {
          fn main() {
            let roundtrip_seed: i64 = KernelTaskAsyncShapes.roundtrip_seed();
            let uplink: Window<i64> = KernelTaskAsyncShapes.send_roundtrip(roundtrip_seed);
            let downlink: Window<Window<i64>> =
              KernelTaskAsyncShapes.receive_roundtrip(uplink);
            print(downlink);
          }
        }
        "#,
        {
            let mut modules = kernel_data_support_modules();
            modules.push((
                "kernel_task_async_shapes.ns",
                r#"
                use data FabricPlane;
                use kernel KernelUnit;

                mod cpu KernelTaskAsyncShapes {
                  pub fn roundtrip_seed() -> i64 {
                    let bind_core: KernelResult<i64> =
                      kernel_result(kernel_profile_bind_core("KernelUnit"));
                    let queue_depth: KernelResult<i64> =
                      kernel_result(kernel_profile_queue_depth("KernelUnit"));
                    let batch_lanes: KernelResult<i64> =
                      kernel_result(kernel_profile_batch_lanes("KernelUnit"));
                    return kernel_value(bind_core)
                      + kernel_value(queue_depth)
                      + kernel_value(batch_lanes);
                  }

                  pub fn send_roundtrip(value: i64) -> Window<i64> {
                    data_profile_bind_core("FabricPlane");
                    let handles: HandleTable<FabricPlaneBindings> =
                      data_profile_handle_table("FabricPlane");
                    return data_profile_send_uplink("FabricPlane", value);
                  }

                  pub fn receive_roundtrip(uplink: Window<i64>) -> Window<Window<i64>> {
                    return data_profile_send_downlink("FabricPlane", uplink);
                  }
                }
                "#,
            ));
            modules
        },
    );

    let uplink = infer_project_route_payload_type(&project, "cpu.Main", "FabricPlane", true)
        .unwrap()
        .expect("expected uplink payload");
    assert_eq!(uplink.render(), "Window<i64>");

    let downlink = infer_project_route_payload_type(&project, "cpu.Main", "FabricPlane", false)
        .unwrap()
        .expect("expected downlink payload");
    assert_eq!(downlink.render(), "Window<Window<i64>>");
}

#[test]
fn validates_kernel_project_links_against_nir_with_shared_cpu_helper_indirection() {
    let project = kernel_data_project_with_entry(
        r#"
        use cpu KernelTaskAsyncShapes;
        use data FabricPlane;
        use kernel KernelUnit;

        mod cpu Main {
          fn main() {
            let roundtrip_seed: i64 = KernelTaskAsyncShapes.roundtrip_seed();
            let uplink: Window<i64> = KernelTaskAsyncShapes.send_roundtrip(roundtrip_seed);
            let downlink: Window<Window<i64>> =
              KernelTaskAsyncShapes.receive_roundtrip(uplink);
            print(downlink);
          }
        }
        "#,
        {
            let mut modules = kernel_data_support_modules();
            modules.push((
                "kernel_task_async_shapes.ns",
                r#"
                use data FabricPlane;
                use kernel KernelUnit;

                mod cpu KernelTaskAsyncShapes {
                  pub fn roundtrip_seed() -> i64 {
                    let bind_core: KernelResult<i64> =
                      kernel_result(kernel_profile_bind_core("KernelUnit"));
                    let queue_depth: KernelResult<i64> =
                      kernel_result(kernel_profile_queue_depth("KernelUnit"));
                    let batch_lanes: KernelResult<i64> =
                      kernel_result(kernel_profile_batch_lanes("KernelUnit"));
                    return kernel_value(bind_core)
                      + kernel_value(queue_depth)
                      + kernel_value(batch_lanes);
                  }

                  pub fn send_roundtrip(value: i64) -> Window<i64> {
                    data_profile_bind_core("FabricPlane");
                    let handles: HandleTable<FabricPlaneBindings> =
                      data_profile_handle_table("FabricPlane");
                    return data_profile_send_uplink("FabricPlane", value);
                  }

                  pub fn receive_roundtrip(uplink: Window<i64>) -> Window<Window<i64>> {
                    return data_profile_send_downlink("FabricPlane", uplink);
                  }
                }
                "#,
            ));
            modules
        },
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "kernel.KernelUnit".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_network_project_links_against_nir_with_shared_cpu_helper_indirection() {
    let project = multidomain_project_with_entry(
        r#"
        use cpu NetworkTaskAsyncShapes;
        use network NetworkUnit;

        mod cpu Main {
          fn main() -> i64 {
            return NetworkTaskAsyncShapes.probe();
          }
        }
        "#,
        {
            let mut modules = multidomain_support_modules();
            modules.push((
                "network_task_async_shapes.ns",
                r#"
                use network NetworkUnit;

                mod cpu NetworkTaskAsyncShapes {
                  pub fn probe() -> i64 {
                    let bind_core: NetworkResult<i64> =
                      network_result(network_profile_bind_core("NetworkUnit"));
                    let endpoint_kind: NetworkResult<i64> =
                      network_result(network_profile_endpoint_kind("NetworkUnit"));
                    let send_window: NetworkResult<i64> =
                      network_result(network_profile_send_window("NetworkUnit"));
                    if network_config_ready(bind_core) {
                      return network_value(bind_core)
                        + network_value(endpoint_kind)
                        + network_value(send_window);
                    }
                    return 0;
                  }
                }
                "#,
            ));
            modules
        },
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_network_project_links_missing_endpoint_kind_usage() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            if network_config_ready(bind_core) {
              return network_value(bind_core);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("network_profile_endpoint_kind(\"NetworkUnit\")"));
}

#[test]
fn validates_network_project_links_for_transport_and_protocol_profile_usage() {
    let project = multidomain_project_with_entry(
        r#"
        use cpu NetworkTaskAsyncShapes;
        use network NetworkUnit;

        mod cpu Main {
          fn main() -> i64 {
            return NetworkTaskAsyncShapes.transport_probe();
          }
        }
        "#,
        {
            let mut modules = multidomain_support_modules();
            modules.push((
                "network_task_async_shapes.ns",
                r#"
                use network NetworkUnit;

                mod cpu NetworkTaskAsyncShapes {
                  pub fn transport_probe() -> i64 {
                    let bind_core: NetworkResult<i64> =
                      network_result(network_profile_bind_core("NetworkUnit"));
                    let endpoint_kind: NetworkResult<i64> =
                      network_result(network_profile_endpoint_kind("NetworkUnit"));
                    let transport_family: NetworkResult<i64> =
                      network_result(network_profile_transport_family("NetworkUnit"));
                    let protocol_kind: NetworkResult<i64> =
                      network_result(network_profile_protocol_kind("NetworkUnit"));
                    let protocol_version: NetworkResult<i64> =
                      network_result(network_profile_protocol_version("NetworkUnit"));
                    if network_config_ready(bind_core) {
                      return network_value(bind_core)
                        + network_value(endpoint_kind)
                        + network_value(transport_family)
                        + network_value(protocol_kind)
                        + network_value(protocol_version);
                    }
                    return 0;
                  }
                }
                "#,
            ));
            modules
        },
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_network_project_links_when_protocol_profile_const_is_missing() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let protocol_kind: NetworkResult<i64> =
              network_result(network_profile_protocol_kind("NetworkUnit"));
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + network_value(protocol_kind);
            }
            return 0;
          }
        }
        "#,
        vec![(
            "network_unit.ns",
            r#"
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
              }
            }
            "#,
        )],
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("requires `protocol_kind` profile const"));
}

#[test]
fn validates_network_project_links_for_host_transport_calls() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_send_probe(
            stream_window: i64,
            send_window: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_recv_probe(
            stream_window: i64,
            recv_window: i64,
            local_port: i64
          ) -> i64;
          extern "c" fn host_network_close(handle: i64) -> i64;

          fn main() -> i64 {
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let send_result: NetworkResult<i64> = network_result(
              host_network_send_probe(stream_window, send_window, remote_port)
            );
            let recv_result: NetworkResult<i64> = network_result(
              host_network_recv_probe(stream_window, recv_window, local_port)
            );
            let close_result: NetworkResult<i64> =
              network_result(host_network_close(local_port));
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + network_value(send_result)
                + network_value(recv_result)
                + network_value(close_result);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_network_host_transport_calls_without_profile_routing() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_send_probe(
            stream_window: i64,
            send_window: i64,
            remote_port: i64
          ) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let send_result: NetworkResult<i64> =
              network_result(host_network_send_probe(64, 32, 443));
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + network_value(send_result);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_send_probe"));
    assert!(err.contains("network_profile_stream_window(\"NetworkUnit\")"));
}

#[test]
fn validates_network_project_links_for_owned_udp_open_calls() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn main() -> i64 {
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let handle: i64 = host_network_open_udp_datagram(local_port, remote_port);
            let send_result: NetworkResult<i64> =
              network_result(host_network_send_owned(handle, stream_window, send_window));
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + network_value(send_result)
                + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_owned_udp_open_calls_without_profile_routing() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let handle: i64 = host_network_open_udp_datagram(9000, 443);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + handle;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_open_udp_datagram"));
    assert!(err.contains("network_profile_local_port(\"NetworkUnit\")"));
}

#[test]
fn rejects_accept_owned_without_listener_source() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_accept_owned(
            listener_handle: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let accept_result: NetworkResult<i64> = network_result(
              host_network_accept_owned(7, read_timeout_ms, write_timeout_ms)
            );
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + network_value(accept_result);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_open_tcp_listener"));
    assert!(err.contains("host_network_accept_owned"));
}

#[test]
fn rejects_close_owned_without_owned_handle_source() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let close_value: i64 = host_network_close_owned(9);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("owned network handle"));
    assert!(err.contains("host_network_close_owned"));
}

#[test]
fn rejects_close_owned_after_shadowing_handle_with_plain_value() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let handle: i64 = 7;
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_close_owned"), "{err}");
    assert!(
        err.contains("does not come from an owned network open/accept path"),
        "{err}"
    );
}

#[test]
fn rejects_close_owned_after_while_shadowing_handle_with_plain_value() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let keep_running: bool = false;
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            while keep_running {
              let handle: i64 = 7;
            }
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_close_owned"), "{err}");
    assert!(
        err.contains("does not come from an owned network open/accept path"),
        "{err}"
    );
}

#[test]
fn validates_close_owned_through_helper_parameter() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn close_handle(handle: i64) -> i64 {
            return host_network_close_owned(handle);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let close_value: i64 = close_handle(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_spawned_helper_parameter() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          async fn close_handle(handle: i64) -> i64 {
            return host_network_close_owned(handle);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let close_task: Task<i64> = spawn(close_handle(handle));
            let close_joined: TaskResult<i64> = join_result(close_task);
            if network_config_ready(bind_core) {
              if task_completed(close_joined) {
                return network_value(bind_core)
                  + network_value(endpoint_kind)
                  + task_value(close_joined);
              }
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_network_result_timeout_and_cancel_async_policy_workflow() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_send_ready(result) {
              return network_value(result) + 1;
            }
            if network_recv_ready(result) {
              return network_value(result) + 2;
            }
            if network_config_ready(result) {
              return network_value(result) + 3;
            }
            return 0;
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let send_probe: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let recv_probe: NetworkResult<i64> =
              network_result(network_profile_recv_window("NetworkUnit"));
            let timeout_budget: i64 = network_profile_connect_timeout("NetworkUnit");

            let completed_result: TaskResult<i64> =
              join_result(spawn(consume_network_result(send_probe)));
            let timed_result: TaskResult<i64> =
              join_result(timeout(spawn(consume_network_result(recv_probe)), timeout_budget));
            let cancelled_result: TaskResult<i64> =
              join_result(cancel(spawn(consume_network_result(bind_core))));

            if task_completed(completed_result)
              && task_timed_out(timed_result)
              && task_cancelled(cancelled_result) {
              return task_value(completed_result)
                + network_value(bind_core)
                + network_value(endpoint_kind);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn open_handle(remote_port: i64, connect_timeout_ms: i64) -> i64 {
            return host_network_open_tcp_stream(remote_port, connect_timeout_ms);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let handle: i64 = open_handle(remote_port, connect_timeout_ms);
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_timed_and_cancelled_spawned_helper_parameter() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          async fn close_handle(handle: i64) -> i64 {
            return host_network_close_owned(handle);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let timed_close: TaskResult<i64> =
              join_result(timeout(spawn(close_handle(handle)), connect_timeout_ms));
            let cancelled_close: TaskResult<i64> =
              join_result(cancel(spawn(close_handle(handle))));
            if network_config_ready(bind_core)
              && task_timed_out(timed_close)
              && task_cancelled(cancelled_close) {
              return network_value(bind_core) + network_value(endpoint_kind);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_nested_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn open_handle(remote_port: i64, connect_timeout_ms: i64) -> i64 {
            return host_network_open_tcp_stream(remote_port, connect_timeout_ms);
          }

          fn forward_open(remote_port: i64, connect_timeout_ms: i64) -> i64 {
            return open_handle(remote_port, connect_timeout_ms);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let handle: i64 = forward_open(remote_port, connect_timeout_ms);
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_network_result_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn open_handle_result(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> NetworkResult<i64> {
            return network_result(host_network_open_tcp_stream(remote_port, connect_timeout_ms));
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let opened: NetworkResult<i64> = open_handle_result(remote_port, connect_timeout_ms);
            let handle: i64 = network_value(opened);
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_recursive_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn open_handle_recursive(step: i64, remote_port: i64, connect_timeout_ms: i64) -> i64 {
            if step < 1 {
              return host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            }
            return open_handle_recursive(0, remote_port, connect_timeout_ms);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let handle: i64 = open_handle_recursive(1, remote_port, connect_timeout_ms);
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_datagram_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn open_handle(local_port: i64, remote_port: i64) -> i64 {
            return host_network_open_udp_datagram(local_port, remote_port);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let handle: i64 = open_handle(local_port, remote_port);
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_http_status_recv_through_datagram_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_recv_http_status_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;

          fn open_handle(local_port: i64, remote_port: i64) -> i64 {
            return host_network_open_udp_datagram(local_port, remote_port);
          }

          fn forward_handle(local_port: i64, remote_port: i64) -> i64 {
            return open_handle(local_port, remote_port);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let protocol_kind: i64 = network_profile_protocol_kind("NetworkUnit");
            let protocol_version: i64 = network_profile_protocol_version("NetworkUnit");
            let protocol_header_bytes: i64 =
              network_profile_protocol_header_bytes("NetworkUnit");
            let handle: i64 = forward_handle(local_port, remote_port);
            let recv_result: NetworkResult<i64> = network_result(
              host_network_recv_http_status_owned(handle, stream_window, recv_window)
            );
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + protocol_kind
                + protocol_version
                + protocol_header_bytes
                + network_value(recv_result);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_recv_http_status_owned"), "{err}");
    assert!(err.contains("datagram-owned source"), "{err}");
}

#[test]
fn rejects_send_owned_through_helper_parameter_with_listener_argument() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;

          fn send_handle(handle: i64, stream_window: i64, send_window: i64) -> i64 {
            return host_network_send_owned(handle, stream_window, send_window);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let handle: i64 = host_network_open_tcp_listener(
              local_port,
              read_timeout_ms,
              write_timeout_ms
            );
            let datagram_handle: i64 =
              host_network_open_udp_datagram(local_port, remote_port);
            let send_value: i64 = send_handle(handle, stream_window, send_window);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + datagram_handle
                + send_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("send_handle"), "{err}");
    assert!(err.contains("listener-owned source"), "{err}");
}

#[test]
fn rejects_send_owned_through_mutual_recursive_helper_parameter_with_listener_argument() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;

          fn send_chain_a(
            step: i64,
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64 {
            if step < 1 {
              return host_network_send_owned(handle, stream_window, send_window);
            }
            return send_chain_b(0, handle, stream_window, send_window);
          }

          fn send_chain_b(
            step: i64,
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64 {
            return send_chain_a(step, handle, stream_window, send_window);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let datagram_handle: i64 =
              host_network_open_udp_datagram(local_port, remote_port);
            let handle: i64 = host_network_open_tcp_listener(
              local_port,
              read_timeout_ms,
              write_timeout_ms
            );
            let send_value: i64 = send_chain_a(1, handle, stream_window, send_window);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + datagram_handle
                + send_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("send_chain_a"), "{err}");
    assert!(err.contains("listener-owned source"), "{err}");
}

#[test]
fn rejects_send_owned_with_listener_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;

          fn open_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64 {
            return host_network_open_tcp_listener(local_port, read_timeout_ms, write_timeout_ms);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let datagram_handle: i64 =
              host_network_open_udp_datagram(local_port, remote_port);
            let handle: i64 = open_listener(local_port, read_timeout_ms, write_timeout_ms);
            let send_value: i64 = host_network_send_owned(handle, stream_window, send_window);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + datagram_handle
                + send_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_send_owned"), "{err}");
    assert!(err.contains("listener-owned source"), "{err}");
}

#[test]
fn rejects_send_owned_with_listener_network_result_helper_returned_handle() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;

          fn open_listener_result(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> NetworkResult<i64> {
            return network_result(
              host_network_open_tcp_listener(local_port, read_timeout_ms, write_timeout_ms)
            );
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let datagram_handle: i64 =
              host_network_open_udp_datagram(local_port, remote_port);
            let opened: NetworkResult<i64> =
              open_listener_result(local_port, read_timeout_ms, write_timeout_ms);
            let handle: i64 = network_value(opened);
            let send_value: i64 = host_network_send_owned(handle, stream_window, send_window);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + datagram_handle
                + send_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_send_owned"), "{err}");
    assert!(err.contains("listener-owned source"), "{err}");
}

#[test]
fn rejects_send_owned_through_nested_helper_parameter_with_listener_argument() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;

          fn send_handle(handle: i64, stream_window: i64, send_window: i64) -> i64 {
            return host_network_send_owned(handle, stream_window, send_window);
          }

          fn forward_send(handle: i64, stream_window: i64, send_window: i64) -> i64 {
            return send_handle(handle, stream_window, send_window);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let handle: i64 = host_network_open_tcp_listener(
              local_port,
              read_timeout_ms,
              write_timeout_ms
            );
            let datagram_handle: i64 =
              host_network_open_udp_datagram(local_port, remote_port);
            let send_value: i64 = forward_send(handle, stream_window, send_window);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + datagram_handle
                + send_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("forward_send"), "{err}");
    assert!(err.contains("listener-owned source"), "{err}");
}

#[test]
fn rejects_send_owned_with_listener_handle_variable() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let listener_handle: i64 = host_network_open_tcp_listener(
              local_port,
              read_timeout_ms,
              write_timeout_ms
            );
            let transport_handle: i64 =
              host_network_open_udp_datagram(local_port, remote_port);
            let send_result: NetworkResult<i64> =
              network_result(host_network_send_owned(listener_handle, stream_window, send_window));
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + transport_handle
                + network_value(send_result);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_send_owned"), "{err}");
    assert!(err.contains("listener-owned source"), "{err}");
}

#[test]
fn rejects_accept_owned_with_transport_handle_variable() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_accept_owned(
            listener_handle: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let listener_handle: i64 = host_network_open_tcp_listener(
              local_port,
              read_timeout_ms,
              write_timeout_ms
            );
            let handle: i64 = host_network_open_udp_datagram(local_port, remote_port);
            let accept_result: NetworkResult<i64> =
              network_result(host_network_accept_owned(handle, read_timeout_ms, write_timeout_ms));
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + listener_handle
                + network_value(accept_result);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_accept_owned"), "{err}");
    assert!(err.contains("datagram-owned source"), "{err}");
}

#[test]
fn rejects_http_status_recv_with_datagram_handle_variable() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_udp_datagram(
            local_port: i64,
            remote_port: i64
          ) -> i64;
          extern "c" fn host_network_recv_http_status_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let protocol_kind: i64 = network_profile_protocol_kind("NetworkUnit");
            let protocol_version: i64 = network_profile_protocol_version("NetworkUnit");
            let protocol_header_bytes: i64 =
              network_profile_protocol_header_bytes("NetworkUnit");
            let handle: i64 = host_network_open_udp_datagram(local_port, remote_port);
            let recv_result: NetworkResult<i64> = network_result(
              host_network_recv_http_status_owned(handle, stream_window, recv_window)
            );
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + protocol_kind
                + protocol_version
                + protocol_header_bytes
                + network_value(recv_result);
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_recv_http_status_owned"), "{err}");
    assert!(err.contains("datagram-owned source"), "{err}");
}

#[test]
fn validates_http_status_recv_with_stream_handle_variable() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_recv_http_status_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let protocol_kind: i64 = network_profile_protocol_kind("NetworkUnit");
            let protocol_version: i64 = network_profile_protocol_version("NetworkUnit");
            let protocol_header_bytes: i64 =
              network_profile_protocol_header_bytes("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let recv_result: NetworkResult<i64> = network_result(
              host_network_recv_http_status_owned(handle, stream_window, recv_window)
            );
            let close_value: i64 = host_network_close_owned(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + protocol_kind
                + protocol_version
                + protocol_header_bytes
                + network_value(recv_result)
                + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_http_status_recv_through_stream_helper_workflow() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_http_status_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn open_handle(remote_port: i64, connect_timeout_ms: i64) -> i64 {
            return host_network_open_tcp_stream(remote_port, connect_timeout_ms);
          }

          fn send_request(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> NetworkResult<i64> {
            return network_result(host_network_send_owned(handle, stream_window, send_window));
          }

          fn recv_status(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> NetworkResult<i64> {
            return network_result(
              host_network_recv_http_status_owned(handle, stream_window, recv_window)
            );
          }

          fn recv_body(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> NetworkResult<i64> {
            return network_result(host_network_recv_owned(handle, stream_window, recv_window));
          }

          fn close_handle(handle: i64) -> i64 {
            return host_network_close_owned(handle);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let protocol_kind: i64 = network_profile_protocol_kind("NetworkUnit");
            let protocol_version: i64 = network_profile_protocol_version("NetworkUnit");
            let protocol_header_bytes: i64 =
              network_profile_protocol_header_bytes("NetworkUnit");
            let handle: i64 = open_handle(remote_port, connect_timeout_ms);
            let send_result: NetworkResult<i64> =
              send_request(handle, stream_window, send_window);
            let status_result: NetworkResult<i64> =
              recv_status(handle, stream_window, recv_window);
            let recv_result: NetworkResult<i64> =
              recv_body(handle, stream_window, recv_window);
            let close_value: i64 = close_handle(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + protocol_kind
                + protocol_version
                + protocol_header_bytes
                + network_value(send_result)
                + network_value(status_result)
                + network_value(recv_result)
                + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_service_lane_helper_workflow() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_accept_owned(
            listener_handle: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_recv_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          fn open_listener(
            local_port: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> i64 {
            return host_network_open_tcp_listener(
              local_port,
              read_timeout_ms,
              write_timeout_ms
            );
          }

          fn accept_session(
            listener_handle: i64,
            read_timeout_ms: i64,
            write_timeout_ms: i64
          ) -> NetworkResult<i64> {
            return network_result(
              host_network_accept_owned(listener_handle, read_timeout_ms, write_timeout_ms)
            );
          }

          fn recv_request(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> NetworkResult<i64> {
            return network_result(host_network_recv_owned(handle, stream_window, recv_window));
          }

          fn send_response(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> NetworkResult<i64> {
            return network_result(host_network_send_owned(handle, stream_window, send_window));
          }

          fn close_transport(handle: i64) -> i64 {
            return host_network_close_owned(handle);
          }

          fn close_listener(listener_handle: i64) -> i64 {
            return host_network_close_owned(listener_handle);
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let local_port: i64 = network_profile_local_port("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let protocol_kind: i64 = network_profile_protocol_kind("NetworkUnit");
            let protocol_version: i64 = network_profile_protocol_version("NetworkUnit");
            let protocol_header_bytes: i64 =
              network_profile_protocol_header_bytes("NetworkUnit");
            let listener_handle: i64 =
              open_listener(local_port, read_timeout_ms, write_timeout_ms);
            let accepted: NetworkResult<i64> =
              accept_session(listener_handle, read_timeout_ms, write_timeout_ms);
            let handle: i64 = network_value(accepted);
            let recv_result: NetworkResult<i64> =
              recv_request(handle, stream_window, recv_window);
            let send_result: NetworkResult<i64> =
              send_response(handle, stream_window, send_window);
            let transport_close_value: i64 = close_transport(handle);
            let listener_close_value: i64 = close_listener(listener_handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + protocol_kind
                + protocol_version
                + protocol_header_bytes
                + network_value(recv_result)
                + network_value(send_result)
                + transport_close_value
                + listener_close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_httpish_header_session_helper_workflow() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_http_status_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          struct RequestHeaders {
            auth_code: i64,
            trace_code: i64
          }

          fn open_session_handle(remote_port: i64, connect_timeout_ms: i64) -> i64 {
            return host_network_open_tcp_stream(remote_port, connect_timeout_ms);
          }

          fn send_session_headers(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> NetworkResult<i64> {
            return network_result(host_network_send_owned(handle, stream_window, send_window));
          }

          fn recv_session_status(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> NetworkResult<i64> {
            return network_result(
              host_network_recv_http_status_owned(handle, stream_window, recv_window)
            );
          }

          fn recv_session_body(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> NetworkResult<i64> {
            return network_result(host_network_recv_owned(handle, stream_window, recv_window));
          }

          fn close_session_handle(handle: i64) -> i64 {
            return host_network_close_owned(handle);
          }

          fn build_headers(
            protocol_kind: i64,
            protocol_version: i64,
            protocol_header_bytes: i64
          ) -> RequestHeaders {
            return RequestHeaders {
              auth_code: 64 + protocol_kind + protocol_header_bytes,
              trace_code: 1000 + protocol_version + protocol_header_bytes
            };
          }

          fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let protocol_kind: i64 = network_profile_protocol_kind("NetworkUnit");
            let protocol_version: i64 = network_profile_protocol_version("NetworkUnit");
            let protocol_header_bytes: i64 =
              network_profile_protocol_header_bytes("NetworkUnit");
            let headers: RequestHeaders =
              build_headers(protocol_kind, protocol_version, protocol_header_bytes);
            let handle: i64 = open_session_handle(remote_port, connect_timeout_ms);
            let send_result: NetworkResult<i64> =
              send_session_headers(handle, stream_window, send_window);
            let status_result: NetworkResult<i64> =
              recv_session_status(handle, stream_window, recv_window);
            let recv_result: NetworkResult<i64> =
              recv_session_body(handle, stream_window, recv_window);
            let close_value: i64 = close_session_handle(handle);
            if network_config_ready(bind_core) {
              return network_value(bind_core)
                + network_value(endpoint_kind)
                + headers.auth_code
                + headers.trace_code
                + network_value(send_result)
                + network_value(status_result)
                + network_value(recv_result)
                + close_value;
            }
            return 0;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_async_loop_owned_network_session_step_workflow() {
    let project = multidomain_project_with_entry(
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          async fn step(value: i64) -> i64 {
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let send_result: NetworkResult<i64> =
              network_result(host_network_send_owned(handle, stream_window, send_window));
            let recv_result: NetworkResult<i64> =
              network_result(host_network_recv_owned(handle, stream_window, recv_window));
            let close_value: i64 = host_network_close_owned(handle);
            if network_send_ready(send_result) || network_recv_ready(recv_result) {
              return value + network_value(send_result) + network_value(recv_result) + close_value;
            }
            return value + close_value;
          }

          async fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
              if acc > 9 {
                break;
              }
            }
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + acc;
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: None,
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn compiles_async_loop_owned_network_http_session_project() {
    let root = write_temp_project(
        "async_loop_owned_network_http_session",
        r#"
        use network NetworkUnit;

        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_http_status_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          async fn step(value: i64) -> i64 {
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let send_result: NetworkResult<i64> =
              network_result(host_network_send_owned(handle, stream_window, send_window));
            let status_result: NetworkResult<i64> =
              network_result(host_network_recv_http_status_owned(handle, stream_window, recv_window));
            let recv_result: NetworkResult<i64> =
              network_result(host_network_recv_owned(handle, stream_window, recv_window));
            let close_value: i64 = host_network_close_owned(handle);
            if network_send_ready(send_result) || network_recv_ready(recv_result) {
              return value
                + network_value(send_result)
                + network_value(status_result)
                + network_value(recv_result)
                + close_value;
            }
            if network_config_ready(status_result) {
              return value + network_value(status_result) + close_value;
            }
            return value + close_value;
          }

          async fn main() -> i64 {
            let bind_core: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));
            let endpoint_kind: NetworkResult<i64> =
              network_result(network_profile_endpoint_kind("NetworkUnit"));
            let timeout_budget: i64 = network_profile_timeout_budget("NetworkUnit");
            let retry_budget: i64 = network_profile_retry_budget("NetworkUnit");
            let protocol_kind: i64 = network_profile_protocol_kind("NetworkUnit");
            let protocol_version: i64 = network_profile_protocol_version("NetworkUnit");
            let value: i64 = 0;
            let acc: i64 = 0;
            let break_budget: i64 =
              timeout_budget + retry_budget + protocol_kind + protocol_version;
            while value < retry_budget {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
              if acc > break_budget {
                break;
              }
            }
            if network_config_ready(bind_core) {
              return network_value(bind_core) + network_value(endpoint_kind) + acc + break_budget;
            }
            return acc + break_budget;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.network"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_post_flow_chain"));
    assert!(artifacts
        .llvm_ir
        .contains("host_network_recv_http_status_owned"));
}

#[test]
fn compiles_async_loop_chain_project() {
    let root = write_temp_project(
        "async_loop_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 4 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_flow_chain_project() {
    let root = write_temp_project(
        "async_loop_flow_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 2 {
                break;
              }
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_cond_chain_project() {
    let root = write_temp_project(
        "async_loop_cond_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_flow_cond_chain_project() {
    let root = write_temp_project(
        "async_loop_flow_cond_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 3 {
                continue;
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_flow_cond_chain_compound_control_project() {
    let root = write_temp_project(
        "async_loop_flow_cond_chain_compound_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > 1 {
                if value > 4 {
                  break;
                } else {
                }
              } else {
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_flow_cond_chain_recursive_control_project() {
    let root = write_temp_project(
        "async_loop_flow_cond_chain_recursive_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 1 && value > 3 && value < 6 {
                break;
              } else {
              }
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_post_flow_cond_chain_project() {
    let root = write_temp_project(
        "async_loop_post_flow_cond_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 5 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_post_flow_cond_chain_compound_control_project() {
    let root = write_temp_project(
        "async_loop_post_flow_cond_chain_compound_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              match acc {
                5 => { continue; },
                _ => {
                  if acc < 6 {
                    continue;
                  } else {
                  }
                }
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_post_flow_cond_chain_recursive_control_project() {
    let root = write_temp_project(
        "async_loop_post_flow_cond_chain_recursive_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 1 && acc > 3 && acc < 10 {
                continue;
              } else {
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}
