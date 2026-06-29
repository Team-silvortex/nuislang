use crate::registry::{capability_summary, NustarClockSummary, NustarPackageManifest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarSchedulerSummary {
    pub contract_stack: String,
    pub clock: NustarClockSummary,
    pub result_roles: String,
    pub sample_navigation: Option<String>,
    pub result_samples: Option<String>,
    pub transport_samples: Option<String>,
    pub summary_api: String,
    pub summary_samples: Option<String>,
    pub observer_classes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarStdNetSummary {
    pub sample_navigation: Option<String>,
    pub recipe_samples: Option<String>,
}

pub fn scheduler_summary(manifest: &NustarPackageManifest) -> NustarSchedulerSummary {
    let domain = manifest.domain_family.as_str();
    NustarSchedulerSummary {
        contract_stack:
            "placement -> timing -> result observation -> async summary observation -> observer classification"
                .to_owned(),
        clock: capability_summary(manifest).clock,
        result_roles: "entry=result-entry, probe=result-ready-probe, value=result-payload-value; variants=config_ready|send_ready|recv_ready|connect_ready|accept_ready|closed".to_owned(),
        sample_navigation: scheduler_sample_navigation(domain).map(str::to_owned),
        result_samples: scheduler_result_samples(domain).map(str::to_owned),
        transport_samples: scheduler_transport_samples(domain).map(str::to_owned),
        summary_api: "policy=async-policy-summary, batch=async-batch-summary, windowed=async-windowed-summary; classes=transport_split|transport_windowed_split|transport_session_bridge_split|control_split|control_windowed|control_session_bridge".to_owned(),
        summary_samples: scheduler_summary_samples(domain).map(str::to_owned),
        observer_classes: "source=profile-backed|result-backed|summary-backed; stage=entry|ready|payload|policy|batch|windowed; scope=local|cross-lane|cross-domain|bridge-visible; branch=primary|secondary|fallback|send|recv".to_owned(),
    }
}

pub fn std_net_summary(domain: &str) -> NustarStdNetSummary {
    NustarStdNetSummary {
        sample_navigation: std_net_sample_navigation(domain).map(str::to_owned),
        recipe_samples: std_net_recipe_samples(domain).map(str::to_owned),
    }
}

fn scheduler_sample_navigation(domain: &str) -> Option<&'static str> {
    match domain {
        "shader" => Some("policy -> windowed"),
        "kernel" => Some("policy -> windowed"),
        "network" => Some(
            "result_ladder -> transport_split_ladder -> transport_summary_ladder -> summary_classes",
        ),
        _ => None,
    }
}

fn scheduler_result_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "result_ladder=network_result_profile_demo -> network_connect_result_demo -> network_accept_result_demo -> network_result_task_policy_demo -> network_result_task_batch_demo -> network_result_task_windowed_batch_demo -> network_result_session_bridge_demo; control_ladder=network_connect_result_demo -> network_accept_result_demo -> network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo",
        ),
        _ => None,
    }
}

fn scheduler_transport_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "transport_runtime=network_host_handle_runtime_demo -> network_host_handle_transport_runtime_demo -> network_owned_transport_result_demo -> network_host_transport_runtime_demo -> network_transport_result_demo; transport_split_ladder=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo; transport_summary_ladder=network_owned_transport_result_task_policy_demo -> network_owned_transport_result_task_batch_demo -> network_owned_transport_result_task_windowed_batch_demo -> network_owned_transport_result_session_bridge_demo -> network_transport_result_task_policy_demo -> network_transport_result_task_batch_demo -> network_transport_result_task_windowed_batch_demo -> network_transport_result_session_bridge_demo",
        ),
        _ => None,
    }
}

fn scheduler_summary_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "shader" => Some(
            "policy=shader_async_policy_profile_demo -> shader_async_fallback_profile_demo; windowed=shader_async_batch_profile_demo -> shader_async_windowed_batch_profile_demo",
        ),
        "kernel" => Some(
            "policy=kernel_async_tensor_policy_profile_demo -> kernel_async_tensor_fallback_profile_demo; windowed=kernel_async_tensor_batch_profile_demo -> kernel_async_tensor_windowed_batch_profile_demo",
        ),
        "network" => Some(
            "transport_split=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo; control_split=network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo",
        ),
        _ => None,
    }
}

fn std_net_sample_navigation(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => {
            Some("profile_core -> transport_edge -> syscall_edge -> socket_edge -> control_edge -> protocol_edge -> http_edge -> result_spine -> task_spine -> session")
        }
        _ => None,
    }
}

fn std_net_recipe_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "profile_core=net_endpoint_recipe -> net_endpoint_recipe_demo; transport_edge=net_ip_packet_recipe -> net_tcp_stream_recipe -> net_udp_datagram_recipe -> net_ip_packet_recipe_demo -> net_tcp_stream_recipe_demo -> net_udp_datagram_recipe_demo; syscall_edge=net_tcp_open_recipe -> net_udp_open_recipe -> net_udp_bind_recipe -> net_udp_bound_socket_recipe -> net_udp_datagram_flow_recipe -> net_tcp_listener_recipe -> net_tcp_client_flow_recipe -> net_tcp_server_flow_recipe -> net_tcp_accepted_socket_recipe -> net_owned_send_recipe -> net_owned_recv_recipe -> net_owned_accept_recipe -> net_owned_close_recipe -> net_tcp_open_recipe_demo -> net_udp_open_recipe_demo -> net_udp_bind_recipe_demo -> net_udp_bound_socket_recipe_demo -> net_udp_datagram_flow_recipe_demo -> net_tcp_listener_recipe_demo -> net_tcp_client_flow_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_tcp_accepted_socket_recipe_demo -> net_owned_send_recipe_demo -> net_owned_recv_recipe_demo -> net_owned_accept_recipe_demo -> net_owned_close_recipe_demo; flow_group=tcp client flow -> tcp server flow -> udp datagram flow; flow=net_tcp_client_flow_recipe -> net_tcp_server_flow_recipe -> net_udp_datagram_flow_recipe -> net_tcp_client_flow_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_udp_datagram_flow_recipe_demo; socket_edge=net_tcp_connect_socket_recipe -> net_tcp_client_flow_recipe -> net_tcp_socket_recipe -> net_tcp_server_socket_recipe -> net_tcp_server_flow_recipe -> net_tcp_accepted_socket_recipe -> net_udp_bound_socket_recipe -> net_udp_datagram_flow_recipe -> net_udp_socket_recipe -> net_ip_socket_recipe -> net_tcp_connect_socket_recipe_demo -> net_tcp_client_flow_recipe_demo -> net_tcp_socket_recipe_demo -> net_tcp_server_socket_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_tcp_accepted_socket_recipe_demo -> net_udp_bound_socket_recipe_demo -> net_udp_datagram_flow_recipe_demo -> net_udp_socket_recipe_demo -> net_ip_socket_recipe_demo; control_edge=net_connect_recipe -> net_listen_recipe -> net_close_recipe -> net_connect_recipe_demo -> net_listen_recipe_demo -> net_close_recipe_demo; protocol_edge=net_protocol_experiment_recipe -> net_line_protocol_recipe -> net_datagram_protocol_recipe -> net_dnsish_protocol_recipe -> net_dnsish_query_recipe -> net_httpish_protocol_recipe -> net_httpish_request_recipe -> net_httpish_response_recipe -> net_httpish_roundtrip_recipe -> net_protocol_experiment_recipe_demo -> net_line_protocol_recipe_demo -> net_datagram_protocol_recipe_demo -> net_dnsish_protocol_recipe_demo -> net_dnsish_query_recipe_demo -> net_httpish_protocol_recipe_demo -> net_httpish_request_recipe_demo -> net_httpish_response_recipe_demo -> net_httpish_roundtrip_recipe_demo; http_edge=net_http_client_recipe -> net_http_request_builder_recipe -> net_http_client_headers_recipe -> net_http_client_url_recipe -> net_http_client_body_recipe -> net_http_client_status_recipe -> net_http_request_recipe -> net_http_response_recipe -> net_http_client_exchange_recipe -> net_http_client_session_recipe -> net_http_client_get_recipe -> net_http_client_post_recipe -> net_http_client_recipe_demo -> net_http_request_builder_recipe_demo -> net_http_client_headers_recipe_demo -> net_http_client_url_recipe_demo -> net_http_client_body_recipe_demo -> net_http_client_status_recipe_demo -> net_http_request_recipe_demo -> net_http_response_recipe_demo -> net_http_client_exchange_recipe_demo -> net_http_client_session_recipe_demo -> net_http_client_get_recipe_demo -> net_http_client_post_recipe_demo; result_spine=net_result_recipe -> net_result_bridge_recipe -> net_result_recipe_demo -> net_result_bridge_recipe_demo; task_spine=net_task_policy_recipe -> net_task_batch_recipe -> net_task_windowed_recipe -> net_task_windowed_bridge_recipe -> net_task_policy_recipe_demo -> net_task_batch_recipe_demo -> net_task_windowed_recipe_demo -> net_task_windowed_bridge_recipe_demo; compare_group=transport compare -> dnsish compare -> httpish compare; compare=net_transport_path_compare_recipe -> net_dnsish_path_compare_recipe -> net_httpish_path_compare_recipe -> net_transport_path_compare_recipe_demo -> net_dnsish_path_compare_recipe_demo -> net_httpish_path_compare_recipe_demo; owned_session=net_owned_transport_session_recipe -> net_owned_datagram_session_recipe -> net_owned_dnsish_exchange_session_recipe -> net_owned_dnsish_pipeline_recipe -> net_owned_transport_session_recipe_demo -> net_owned_datagram_session_recipe_demo -> net_owned_dnsish_exchange_session_recipe_demo -> net_owned_dnsish_pipeline_recipe_demo; session=net_control_session_recipe -> net_transport_session_recipe -> net_owned_transport_session_recipe -> net_tcp_listener_session_recipe -> net_transport_path_compare_recipe -> net_protocol_session_recipe -> net_datagram_session_recipe -> net_owned_datagram_session_recipe -> net_udp_bound_session_recipe -> net_datagram_exchange_session_recipe -> net_datagram_pipeline_recipe -> net_dnsish_exchange_session_recipe -> net_owned_dnsish_exchange_session_recipe -> net_dnsish_path_compare_recipe -> net_dnsish_pipeline_recipe -> net_owned_dnsish_pipeline_recipe -> net_http_client_session_recipe -> net_httpish_session_recipe -> net_httpish_exchange_session_recipe -> net_httpish_path_compare_recipe -> net_session_recipe -> net_control_session_recipe_demo -> net_transport_session_recipe_demo -> net_owned_transport_session_recipe_demo -> net_tcp_listener_session_recipe_demo -> net_transport_path_compare_recipe_demo -> net_protocol_session_recipe_demo -> net_datagram_session_recipe_demo -> net_owned_datagram_session_recipe_demo -> net_udp_bound_session_recipe_demo -> net_datagram_exchange_session_recipe_demo -> net_datagram_pipeline_recipe_demo -> net_dnsish_exchange_session_recipe_demo -> net_owned_dnsish_exchange_session_recipe_demo -> net_dnsish_path_compare_recipe_demo -> net_dnsish_pipeline_recipe_demo -> net_owned_dnsish_pipeline_recipe_demo -> net_http_client_session_recipe_demo -> net_httpish_session_recipe_demo -> net_httpish_exchange_session_recipe_demo -> net_httpish_path_compare_recipe_demo -> net_session_recipe_demo",
        ),
        _ => None,
    }
}
