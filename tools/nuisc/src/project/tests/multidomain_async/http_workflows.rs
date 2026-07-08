use super::*;

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
