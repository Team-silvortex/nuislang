use super::*;

#[test]
fn lowers_async_network_result_recursive_control_flow_observation_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_send_ready(result) || network_recv_ready(result) {
              return network_value(result) + 1;
            }
            if network_config_ready(result) {
              return network_value(result) + 7;
            }
            return 0;
          }

          fn main() -> i64 {
            let primary: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let fallback: NetworkResult<i64> =
              network_result(network_profile_recv_window("NetworkUnit"));
            let config_only: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));

            let primary_task: Task<i64> = spawn(consume_network_result(primary));
            let fallback_task: Task<i64> = spawn(consume_network_result(fallback));
            let config_task: Task<i64> = spawn(consume_network_result(config_only));

            let primary_result: TaskResult<i64> = join_result(primary_task);
            let fallback_result: TaskResult<i64> = join_result(fallback_task);
            let config_result: TaskResult<i64> = join_result(config_task);

            if task_completed(primary_result) {
              return task_value(primary_result);
            }
            if task_completed(fallback_result) {
              return task_value(fallback_result);
            }
            if task_completed(config_result) {
              return task_value(config_result);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_recv_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_lane_policy_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_result_capability_type"));
}

#[test]
fn lowers_async_network_result_task_policy_observation_path() {
    let module = parse_nuis_module(
        r#"
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
            let primary: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let fallback: NetworkResult<i64> =
              network_result(network_profile_recv_window("NetworkUnit"));
            let config_only: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));

            let completed_result: TaskResult<i64> =
              join_result(spawn(consume_network_result(primary)));
            let timed_result: TaskResult<i64> =
              join_result(timeout(spawn(consume_network_result(fallback)), 0));
            let cancelled_result: TaskResult<i64> =
              join_result(cancel(spawn(consume_network_result(config_only))));

            if task_completed(completed_result)
              && task_timed_out(timed_result)
              && task_cancelled(cancelled_result) {
              return task_value(completed_result);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "timeout"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cancel"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_timed_out"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_recv_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_lane_policy_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_result_capability_type"));
}

#[test]
fn lowers_async_kernel_result_recursive_control_flow_observation_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume_kernel_result(result: KernelResult<i64>) -> i64 {
            if kernel_config_ready(result) && kernel_value(result) > 3 {
              return kernel_value(result) + 10;
            }
            if kernel_config_ready(result) {
              return kernel_value(result) + 2;
            }
            return 0;
          }

          fn main() -> i64 {
            let primary: KernelResult<i64> =
              kernel_result(kernel_profile_queue_depth("KernelUnit"));
            let fallback: KernelResult<i64> =
              kernel_result(kernel_profile_batch_lanes("KernelUnit"));

            let primary_task: Task<i64> = spawn(consume_kernel_result(primary));
            let fallback_task: Task<i64> = spawn(consume_kernel_result(fallback));

            let primary_result: TaskResult<i64> = join_result(primary_task);
            let fallback_result: TaskResult<i64> = join_result(fallback_task);

            if task_completed(primary_result) && task_value(primary_result) > 0 {
              return task_value(primary_result);
            }
            if task_completed(fallback_result) {
              return task_value(fallback_result);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_kernel_lane_policy_type"));
}

#[test]
fn lowers_async_multidomain_result_orchestration_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_send_ready(result) || network_recv_ready(result) {
              return network_value(result) + 1;
            }
            if network_config_ready(result) {
              return network_value(result) + 3;
            }
            return 0;
          }

          async fn consume_kernel_result(result: KernelResult<i64>) -> i64 {
            if kernel_config_ready(result) && kernel_value(result) > 2 {
              return kernel_value(result) + 5;
            }
            if kernel_config_ready(result) {
              return kernel_value(result) + 1;
            }
            return 0;
          }

          async fn orchestrate(seed: i64) -> i64 {
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

            if task_completed(network_result_joined) && task_completed(kernel_result_joined) {
              return base + task_value(network_result_joined) + task_value(kernel_result_joined);
            }
            if task_completed(network_result_joined) {
              return base + task_value(network_result_joined);
            }
            if task_completed(kernel_result_joined) {
              return base + task_value(kernel_result_joined);
            }
            return base;
          }

          async fn main() {
            await orchestrate(7);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "async_call"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "await"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "is_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_lane_policy_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_kernel_lane_policy_type"));
}
