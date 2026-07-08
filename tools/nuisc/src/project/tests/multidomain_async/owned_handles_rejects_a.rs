use super::*;

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
