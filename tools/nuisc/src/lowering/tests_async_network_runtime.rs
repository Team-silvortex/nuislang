use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

// Layer 1: observer-driven async steps that only sample network readiness/value.
#[test]
fn lowers_async_network_observer_step_into_async_loop_carry_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            let probe: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            if network_send_ready(probe) || network_recv_ready(probe) {
              return value + network_value(probe);
            }
            if network_config_ready(probe) {
              return value + network_value(probe);
            }
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_async_chain")
        .expect("expected loop_while_i64_async_chain node");
    assert_eq!(loop_node.op.args[2], "step");
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
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
}

// Layer 2: owned session steps that model open/send/recv/close within one async step.
#[test]
fn lowers_async_owned_network_session_step_into_async_post_flow_break_chain() {
    let mut module = parse_nuis_module(
        r#"
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
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
              if acc > 9 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_async_post_flow_chain"
        })
        .expect("expected loop_while_i64_async_post_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
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
}

// Layer 3: budgeted polling loops that add retry/timeout control on top of network steps.
#[test]
fn lowers_async_network_poll_step_with_retry_budget_into_async_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            let probe: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            if network_send_ready(probe) || network_recv_ready(probe) {
              return value + network_value(probe);
            }
            if network_config_ready(probe) {
              return value + network_value(probe);
            }
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let retries: i64 = 0;
            let bytes: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > network_profile_recv_window("NetworkUnit") {
                let retries: i64 = retries + value;
              } else {
                let retries: i64 = retries + 0;
              }
              let bytes: i64 = bytes + value;
              if bytes > network_profile_retry_budget("NetworkUnit") {
                break;
              }
            }
            return bytes;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_i64_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry1_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
    assert_eq!(loop_node.op.args[13], "always");
    assert_eq!(loop_node.op.args[15], "add_current");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
}

#[test]
fn lowers_async_owned_network_session_step_with_retry_budget_into_async_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
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
            let value: i64 = 0;
            let retries: i64 = 0;
            let bytes: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > network_profile_recv_window("NetworkUnit") {
                let retries: i64 = retries + value;
              } else {
                let retries: i64 = retries + 0;
              }
              let bytes: i64 = bytes + value;
              if bytes > network_profile_retry_budget("NetworkUnit") {
                break;
              }
            }
            return bytes;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_i64_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry1_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
    assert_eq!(loop_node.op.args[13], "always");
    assert_eq!(loop_node.op.args[15], "add_current");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
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
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
}

#[test]
fn lowers_async_owned_network_session_step_with_timeout_budget_into_async_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
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
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
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
              return value + network_value(send_result) + network_value(recv_result)
                + read_timeout_ms + write_timeout_ms + close_value;
            }
            return value + read_timeout_ms + close_value;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let retries: i64 = 0;
            let bytes: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > network_profile_write_timeout("NetworkUnit") {
                let retries: i64 = retries + value;
              } else {
                let retries: i64 = retries + 0;
              }
              let bytes: i64 = bytes + value;
              if bytes > network_profile_timeout_budget("NetworkUnit") {
                break;
              }
            }
            return bytes;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_i64_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry1_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
    assert_eq!(loop_node.op.args[13], "always");
    assert_eq!(loop_node.op.args[15], "add_current");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
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
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
}

// Layer 4: product-style request naming that turns the same lowering path into an HTTP-like flow.
#[test]
fn lowers_async_http_client_request_session_into_async_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
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

          async fn request_step(value: i64) -> i64 {
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let read_timeout_ms: i64 = network_profile_read_timeout("NetworkUnit");
            let write_timeout_ms: i64 = network_profile_write_timeout("NetworkUnit");
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
              return value + network_value(send_result) + network_value(recv_result)
                + read_timeout_ms + write_timeout_ms + close_value;
            }
            return value + read_timeout_ms + close_value;
          }

          async fn main() -> i64 {
            let request_progress: i64 = 0;
            let request_attempts: i64 = 0;
            let response_bytes: i64 = 0;
            while request_progress < 6 {
              let request_progress: i64 = await request_step(request_progress);
              if request_progress > network_profile_write_timeout("NetworkUnit") {
                let request_attempts: i64 = request_attempts + request_progress;
              } else {
                let request_attempts: i64 = request_attempts + 0;
              }
              let response_bytes: i64 = response_bytes + request_progress;
              if response_bytes > network_profile_timeout_budget("NetworkUnit") {
                break;
              }
            }
            return response_bytes;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_i64_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "request_step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry1_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
    assert_eq!(loop_node.op.args[13], "always");
    assert_eq!(loop_node.op.args[15], "add_current");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert!(yir
        .node_lanes
        .values()
        .any(|lane| lane == "fn:request_step"));
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
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
}
