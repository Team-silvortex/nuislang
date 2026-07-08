use super::*;

#[test]
fn rejects_network_host_transport_calls_without_profile_routing() {
    let project =
        direct_network_default_project_with_link(network_host_transport_missing_routing_entry());

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_send_probe"));
    assert!(err.contains("network_profile_stream_window(\"NetworkUnit\")"));
}

#[test]
fn validates_network_project_links_for_owned_udp_open_calls() {
    let project = direct_network_default_project_with_link(network_owned_udp_open_entry());

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_owned_udp_open_calls_without_profile_routing() {
    let project =
        direct_network_default_project_with_link(network_owned_udp_open_missing_routing_entry());

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_open_udp_datagram"));
    assert!(err.contains("network_profile_local_port(\"NetworkUnit\")"));
}

#[test]
fn rejects_accept_owned_without_listener_source() {
    let project = direct_network_default_project_with_link(
        network_accept_owned_without_listener_source_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("host_network_open_tcp_listener"));
    assert!(err.contains("host_network_accept_owned"));
}

#[test]
fn rejects_close_owned_without_owned_handle_source() {
    let project = direct_network_default_project_with_link(
        network_close_owned_without_owned_handle_source_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("owned network handle"));
    assert!(err.contains("host_network_close_owned"));
}

#[test]
fn rejects_close_owned_after_shadowing_handle_with_plain_value() {
    let project = direct_network_default_project_with_link(
        network_close_owned_after_shadowing_handle_entry(),
    );

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
    let project = direct_network_default_project_with_link(
        network_close_owned_after_while_shadowing_handle_entry(),
    );

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
    let project = direct_network_default_project_with_link(
        network_close_owned_through_helper_parameter_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_spawned_helper_parameter() {
    let project = direct_network_default_project_with_link(
        network_close_owned_through_spawned_helper_parameter_entry(),
    );

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
    let project = direct_network_default_project_with_link(
        network_close_owned_through_helper_returned_handle_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_timed_and_cancelled_spawned_helper_parameter() {
    let project = direct_network_default_project_with_link(
        network_close_owned_through_timed_and_cancelled_spawned_helper_parameter_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_nested_helper_returned_handle() {
    let project = direct_network_default_project_with_link(
        network_close_owned_through_nested_helper_returned_handle_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_network_result_helper_returned_handle() {
    let project = direct_network_default_project_with_link(
        network_close_owned_through_network_result_helper_returned_handle_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_recursive_helper_returned_handle() {
    let project = direct_network_default_project_with_link(
        network_close_owned_through_recursive_helper_returned_handle_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_close_owned_through_datagram_helper_returned_handle() {
    let project = direct_network_default_project_with_link(
        network_close_owned_through_datagram_helper_returned_handle_entry(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

// Network ownership misuse regressions and HTTP/session workflow coverage.
