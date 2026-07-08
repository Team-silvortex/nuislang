use super::*;

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
fn compiles_reverse_network_project_via_data_bridge() {
    let root = write_temp_network_data_project(
        "reverse_network_via_data_bridge",
        reverse_network_data_bridge_entry(),
        {
            let mut modules = network_data_support_modules();
            modules.push((
                "network_data_bridge.ns",
                reverse_network_data_bridge_module(),
            ));
            modules
        },
        &["network.NetworkUnit -> cpu.Main via data.FabricPlane"],
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
        .any(|package| package == "official.data"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
}

#[test]
fn compiles_reverse_kernel_project_via_data_bridge() {
    let root = write_temp_kernel_data_project(
        "reverse_kernel_via_data_bridge",
        r#"
        use cpu KernelDataBridge;
        use kernel KernelUnit;
        use data FabricPlane;

        mod cpu Main {
          fn main() -> i64 {
            return KernelDataBridge.probe_roundtrip();
          }
        }
        "#,
        {
            let mut modules = kernel_data_support_modules();
            modules.push(("kernel_data_bridge.ns", reverse_kernel_data_bridge_module()));
            modules
        },
        &["kernel.KernelUnit -> cpu.Main via data.FabricPlane"],
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.kernel"));
    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.data"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "observe"));
}

#[test]
fn rejects_reverse_network_project_via_data_bridge_when_network_to_data_xfer_is_missing() {
    let root = write_temp_network_data_project(
        "reverse_network_via_data_bridge_missing_xfer",
        reverse_network_data_bridge_entry(),
        {
            let mut modules = network_data_support_modules();
            modules.push((
                "network_data_bridge.ns",
                reverse_network_data_bridge_module(),
            ));
            let fabric_plane = network_fabric_plane_module(false);
            modules.push(("fabric_plane.ns", Box::leak(fabric_plane.into_boxed_str())));
            modules
        },
        &["network.NetworkUnit -> cpu.Main via data.FabricPlane"],
    );
    let err = match crate::pipeline::compile_source_path(&root) {
        Ok(_) => panic!("expected reverse network/data bridge compile to fail"),
        Err(err) => err,
    };
    let _ = fs::remove_dir_all(&root);
    assert!(err.contains("requires a `network` -> `data` xfer segment"));
}

#[test]
fn rejects_reverse_kernel_project_via_data_bridge_when_kernel_to_data_xfer_is_missing() {
    let root = write_temp_kernel_data_project(
        "reverse_kernel_via_data_bridge_missing_xfer",
        r#"
        use cpu KernelDataBridge;
        use kernel KernelUnit;
        use data FabricPlane;

        mod cpu Main {
          fn main() -> i64 {
            return KernelDataBridge.probe_roundtrip();
          }
        }
        "#,
        {
            let mut modules = kernel_data_support_modules();
            modules.push(("kernel_data_bridge.ns", reverse_kernel_data_bridge_module()));
            let fabric_plane = kernel_fabric_plane_module(false);
            modules.push(("fabric_plane.ns", Box::leak(fabric_plane.into_boxed_str())));
            modules
        },
        &["kernel.KernelUnit -> cpu.Main via data.FabricPlane"],
    );
    let err = match crate::pipeline::compile_source_path(&root) {
        Ok(_) => panic!("expected reverse kernel/data bridge compile to fail"),
        Err(err) => err,
    };
    let _ = fs::remove_dir_all(&root);
    assert!(err.contains("requires a `kernel` -> `data` xfer segment"));
}

// Shared helper inference and bridge-link validation tests.
