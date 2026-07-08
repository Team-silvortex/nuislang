pub(super) fn network_support_modules_without_protocol_kind() -> Vec<(&'static str, &'static str)> {
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
    )]
}

pub(super) fn network_host_transport_entry() -> &'static str {
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
    "#
}

pub(super) fn network_host_transport_missing_routing_entry() -> &'static str {
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
    "#
}

pub(super) fn network_owned_udp_open_entry() -> &'static str {
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
    "#
}

pub(super) fn network_owned_udp_open_missing_routing_entry() -> &'static str {
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
    "#
}

pub(super) fn network_accept_owned_without_listener_source_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_without_owned_handle_source_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_after_shadowing_handle_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_after_while_shadowing_handle_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_helper_parameter_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_spawned_helper_parameter_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_helper_returned_handle_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_nested_helper_returned_handle_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_network_result_helper_returned_handle_entry(
) -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_recursive_helper_returned_handle_entry() -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_timed_and_cancelled_spawned_helper_parameter_entry(
) -> &'static str {
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
    "#
}

pub(super) fn network_close_owned_through_datagram_helper_returned_handle_entry() -> &'static str {
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
    "#
}

// Data-plane profile variants used by forward/reverse bridge contract tests.
