use super::*;

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

// End-to-end async control-flow compile coverage.
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
fn compiles_loop_chain_project_with_dynamic_buffer_index_carry() {
    let root = write_temp_project(
        "loop_chain_dynamic_buffer_index",
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let buffer: ref Buffer = alloc_buffer(8, 9);
            while value < 4 {
              let value: i64 = value + 1;
              let acc: i64 = acc + load_at(buffer, value);
            }
            free(buffer);
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_read_at_dynamic_current"));
    assert!(artifacts.llvm_ir.contains("load_at"));
}

#[test]
fn compiles_tail_recursive_project_with_dynamic_buffer_index_carry() {
    let root = write_temp_project(
        "tail_recursive_dynamic_buffer_index",
        r#"
        mod cpu Main {
          fn accumulate(current: i64, buffer: ref Buffer, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            return accumulate(current - 1, buffer, acc + load_at(buffer, current));
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = accumulate(4, buffer, 0);
            free(buffer);
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected tail-recursive loop_while_scalar_chain node");
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_read_at_dynamic_prev_current"));
    assert!(artifacts.llvm_ir.contains("load_at"));
}

#[test]
fn compiles_tail_recursive_project_with_fixed_buffer_index_carry() {
    let root = write_temp_project(
        "tail_recursive_fixed_buffer_index",
        r#"
        mod cpu Main {
          fn accumulate(current: i64, buffer: ref Buffer, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            return accumulate(current - 1, buffer, acc + load_at(buffer, 0));
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = accumulate(4, buffer, 0);
            free(buffer);
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected tail-recursive loop_while_scalar_chain node");
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_read_at_fixed"));
    assert!(artifacts.llvm_ir.contains("getelementptr inbounds i64"));
}

#[test]
fn compiles_loop_chain_project_with_fixed_node_value_carry() {
    let root = write_temp_project(
        "loop_chain_fixed_node_value",
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let current: i64 = 0;
            let acc: i64 = 0;
            let head: ref Node = alloc_node(7, null());
            while current < 3 {
              let current: i64 = current + 1;
              let acc: i64 = acc + load_value(head);
            }
            free(head);
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_read_value_fixed"));
    assert!(artifacts.llvm_ir.contains("load i64, ptr"));
}
