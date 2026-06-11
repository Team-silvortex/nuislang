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
    let mut modules = vec![(
        "main.ns",
        entry_source,
    )];
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
                const local_port: i64 = 9000;
                const remote_port: i64 = 443;
                const connect_timeout_ms: i64 = 250;
                const retry_budget: i64 = 3;
                const stream_window: i64 = 64;
                const recv_window: i64 = 32;
                const send_window: i64 = 32;
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
                    let send_window: NetworkResult<i64> =
                      network_result(network_profile_send_window("NetworkUnit"));
                    if network_config_ready(bind_core) {
                      return network_value(bind_core) + network_value(send_window);
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
