# Domain Project Companions

This folder contains narrow project-form companions for current non-CPU helper
lanes such as `shader` and `kernel`.

These are currently the canonical validation route for those two lanes, because
standalone single-file `shader` and `kernel` sources still depend on loaded
`nustar` lowering implementations.

Current shared async helper modules used by the shader/kernel/network async
policy, fallback, batch, and bridge demos live here:

* [shared](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared)
* [shader_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/shader_task_async_shapes.ns)
* [kernel_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/kernel_task_async_shapes.ns)
* [network_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/network_task_async_shapes.ns)

If you want the shortest source-level sample that now lines up with the current
`YIR` scheduler reading order, start with:

* [shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo)
* [kernel_async_tensor_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_policy_profile_demo)

If you want the next wider pair where `windowed batch` reads directly in source
and now also reuses shared task-shaped batch/windowed helpers,
continue with:

* [shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo)
* [kernel_async_tensor_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_windowed_batch_profile_demo)

Current async sample ladder:

* shader:
  [shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo) ->
  [shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo)
* kernel:
  [kernel_async_tensor_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_policy_profile_demo) ->
  [kernel_async_tensor_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_windowed_batch_profile_demo)

Start here:

* shared helper layer:
  [shared](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared),
  [shader_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/shader_task_async_shapes.ns),
  [kernel_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/kernel_task_async_shapes.ns),
  [network_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/network_task_async_shapes.ns)
* shader spine:
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo) ->
  surface branch ->
  packet branch ->
  bridge branch
* kernel spine:
  [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo) ->
  async base ->
  async tensor ->
  tensor lane
* network edge:
  std-net grouped rule:
  `profile core -> transport edge -> socket edge -> control edge -> protocol edge -> http edge -> result spine -> task spine -> session`
  std-front-door validation:
  [net_endpoint_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_endpoint_recipe_demo) ->
  [net_ip_packet_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_ip_packet_recipe_demo) ->
  [net_tcp_stream_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_stream_recipe_demo) ->
  [net_udp_datagram_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_datagram_recipe_demo) ->
  [net_tcp_connect_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_connect_socket_recipe_demo) ->
  [net_tcp_client_flow_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_client_flow_recipe_demo) ->
  [net_tcp_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_socket_recipe_demo) ->
  [net_tcp_server_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_server_socket_recipe_demo) ->
  [net_tcp_server_flow_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_server_flow_recipe_demo) ->
  [net_tcp_accepted_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_accepted_socket_recipe_demo) ->
  [net_udp_bound_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_bound_socket_recipe_demo) ->
  [net_udp_datagram_flow_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_datagram_flow_recipe_demo) ->
  [net_udp_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_socket_recipe_demo) ->
  [net_ip_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_ip_socket_recipe_demo) ->
  [net_connect_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_connect_recipe_demo) ->
  [net_listen_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_listen_recipe_demo) ->
  [net_close_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_close_recipe_demo) ->
  [net_protocol_experiment_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_experiment_recipe_demo) ->
  [net_line_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_line_protocol_recipe_demo) ->
  [net_datagram_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_protocol_recipe_demo) ->
  [net_dnsish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_protocol_recipe_demo) ->
  [net_dnsish_query_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_query_recipe_demo) ->
  [net_httpish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_protocol_recipe_demo) ->
  [net_http_client_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_recipe_demo) ->
  [net_http_request_builder_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_request_builder_recipe_demo) ->
  [net_http_client_headers_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_headers_recipe_demo) ->
  [net_http_client_url_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_url_recipe_demo) ->
  [net_http_client_body_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_body_recipe_demo) ->
  [net_http_client_status_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_status_recipe_demo) ->
  [net_http_request_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_request_recipe_demo) ->
  [net_http_response_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_response_recipe_demo) ->
  [net_http_client_exchange_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_exchange_recipe_demo) ->
  [net_http_client_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_session_recipe_demo) ->
  [net_http_client_get_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_get_recipe_demo) ->
  [net_http_client_post_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_post_recipe_demo) ->
  [net_httpish_request_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_request_recipe_demo) ->
  [net_httpish_response_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_response_recipe_demo) ->
  [net_httpish_roundtrip_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_roundtrip_recipe_demo) ->
  [net_result_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_recipe_demo) ->
  [net_result_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_bridge_recipe_demo) ->
  [net_task_policy_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_policy_recipe_demo) ->
  [net_task_batch_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_batch_recipe_demo) ->
  [net_task_windowed_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_recipe_demo) ->
  [net_task_windowed_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_bridge_recipe_demo) ->
  [net_control_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_control_session_recipe_demo) ->
  [net_transport_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_transport_session_recipe_demo) ->
  [net_owned_transport_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_transport_session_recipe_demo) ->
  [net_transport_path_compare_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_transport_path_compare_recipe_demo) ->
  [net_protocol_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_session_recipe_demo) ->
  [net_datagram_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_session_recipe_demo) ->
  [net_owned_datagram_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_datagram_session_recipe_demo) ->
  [net_udp_bound_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_bound_session_recipe_demo) ->
  [net_tcp_listener_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_listener_session_recipe_demo) ->
  [net_datagram_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_exchange_session_recipe_demo) ->
  [net_datagram_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_pipeline_recipe_demo) ->
  [net_dnsish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_exchange_session_recipe_demo) ->
  [net_owned_dnsish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_dnsish_exchange_session_recipe_demo) ->
  [net_dnsish_path_compare_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_path_compare_recipe_demo) ->
  [net_dnsish_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_pipeline_recipe_demo) ->
  [net_owned_dnsish_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_dnsish_pipeline_recipe_demo) ->
  [net_httpish_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_session_recipe_demo) ->
  [net_httpish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_session_recipe_demo) ->
  [net_httpish_path_compare_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_path_compare_recipe_demo) ->
  [net_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_recipe_demo) ->
  [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo) ->
  [network_endpoint_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_endpoint_profile_demo) ->
  [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo) ->
  [network_host_handle_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_demo) ->
  [network_host_handle_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_transport_runtime_demo) ->
  [network_owned_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_demo) ->
  [network_owned_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_policy_demo) ->
  [network_owned_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_batch_demo) ->
  [network_owned_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_windowed_batch_demo) ->
  [network_owned_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_session_bridge_demo) ->
  [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo) ->
  [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo) ->
  [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo) ->
  [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo) ->
  [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo) ->
  [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo) ->
  [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo) ->
  [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo) ->
  [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo) ->
  [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo) ->
  [network_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/network_task_async_shapes.ns) ->
  result ladder ->
  session/task ladder
* short network rule:
  `profile core -> endpoint/timing -> host control/runtime transport -> shared helper -> result observe -> session -> result-policy/result-batch/result-windowed/policy/fallback -> batch/windowed`
* owned transport rule:
  `owned transport result -> owned policy -> owned batch -> owned windowed -> owned session bridge`
* syscall edge rule:
  `tcp open -> udp open -> udp bind -> tcp listener -> owned send -> owned recv -> owned accept -> owned close`
* owned std net rule:
  `owned transport session -> owned datagram session -> owned dns-ish exchange -> owned dns-ish pipeline`
* compare rule:
  `probe transport session -> owned transport session -> transport path compare`
* dns-ish compare rule:
  `dns-ish exchange session -> owned dns-ish exchange session -> dns-ish path compare`
* httpish compare rule:
  `httpish exchange session -> httpish path compare`
* flow rule:
  `tcp client flow -> tcp server flow -> udp datagram flow`
* grouped compare rule:
  `transport compare -> dns-ish compare -> httpish compare`
* transport ladder rule:
  `transport result -> transport policy -> transport split -> transport batch split -> transport windowed split -> transport batch -> transport windowed -> transport/session bridge`
* network shared helper rule:
  `async_session_summary -> async_policy/fallback_summary -> async_batch/windowed_summary`
* connect/accept control rule:
  `connect result -> accept result -> connect/accept policy -> connect/accept batch -> connect/accept windowed`
* network result ladder:
  [network_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_profile_demo) ->
  [network_connect_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_result_demo) ->
  [network_accept_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_accept_result_demo) ->
  [network_connect_accept_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_policy_demo) ->
  [network_connect_accept_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_batch_demo) ->
  [network_connect_accept_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_windowed_batch_demo) ->
  [network_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_policy_demo) ->
  [network_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_batch_demo) ->
  [network_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_windowed_batch_demo) ->
  [network_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_session_bridge_demo)
* network session/task ladder:
  [network_profile_summary_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_summary_demo) ->
  [network_profile_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_session_demo) ->
  [network_profile_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_policy_demo) ->
  [network_profile_task_fallback_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_fallback_demo) ->
  [network_profile_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_batch_demo) ->
  [network_profile_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_windowed_batch_demo)
* network host control runtime:
  [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo)
* network host handle runtime:
  [network_host_handle_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_demo)
* network host handle transport runtime:
  [network_host_handle_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_transport_runtime_demo)
* network owned transport result:
  [network_owned_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_demo)
* network owned transport result policy:
  [network_owned_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_policy_demo)
* network owned transport result batch:
  [network_owned_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_batch_demo)
* network owned transport result windowed:
  [network_owned_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_windowed_batch_demo)
* network owned transport result session bridge:
  [network_owned_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_session_bridge_demo)
* network host transport runtime:
  [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo)
* network transport result:
  [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo)
* network transport result policy:
  [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo)
* network transport result split:
  [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo)
* network transport result batch split:
  [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo)
* network transport result windowed split:
  [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo)
* network transport result batch:
  [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo)
* network transport result windowed:
  [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo)
* network transport result session bridge split:
  [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo)
* network transport result session bridge:
  [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)

Shader branch index:

* surface branch:
  [shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo),
  [shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo),
  [shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo),
  [shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo),
  [shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo),
  [shader_surface_state_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_profile_demo),
  [shader_surface_state_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_packet_profile_demo),
  [shader_surface_state_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_pass_profile_demo),
  [shader_surface_state_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_flow_profile_demo),
  [shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo),
  [shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo),
  [shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo)
* packet branch:
  [shader_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_profile_demo),
  [shader_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo)
* bridge branch:
  [shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo),
  [shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo),
  [shader_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_result_profile_demo),
  [shader_async_fanin_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fanin_profile_demo),
  [shader_async_schedule_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_schedule_profile_demo),
  [shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo),
  [shader_async_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fallback_profile_demo),
  [shader_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_batch_profile_demo),
  [shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo),
  [shader_result_family_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_family_profile_demo),
  [shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo),
  [shader_draw_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_render_profile_demo),
  [shader_draw_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_profile_demo),
  [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)

Kernel branch index:

* base:
  [kernel_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_result_profile_demo),
  [kernel_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_result_profile_demo),
  [kernel_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_batch_profile_demo),
  [kernel_async_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_roundtrip_profile_demo)
* async tensor:
  [kernel_async_tensor_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_batch_profile_demo),
  [kernel_async_tensor_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_policy_profile_demo),
  [kernel_async_tensor_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_fallback_profile_demo),
  [kernel_async_tensor_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_windowed_batch_profile_demo),
  [kernel_async_tensor_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_roundtrip_profile_demo)
* tensor lane:
  [kernel_tensor_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_profile_demo),
  [kernel_tensor_inspect_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_inspect_demo),
  [kernel_tensor_slice_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_slice_demo),
  [kernel_tensor_reshape_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reshape_demo),
  [kernel_tensor_broadcast_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_broadcast_demo),
  [kernel_tensor_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reduce_demo),
  [kernel_tensor_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_select_demo),
  [kernel_tensor_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_order_demo),
  [kernel_tensor_axis_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_reduce_demo),
  [kernel_tensor_axis_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_family_demo),
  [kernel_tensor_axis_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_select_demo),
  [kernel_tensor_axis_sort_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_sort_demo),
  [kernel_tensor_axis_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_order_demo),
  [kernel_tensor_axis_map_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_map_demo),
  [kernel_tensor_axis_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_pipeline_demo),
  [kernel_tensor_axis_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_roundtrip_demo),
  [kernel_tensor_map_zip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_map_zip_demo),
  [kernel_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_roundtrip_profile_demo)

Axis-aware kernel subgroup:

* [kernel_tensor_axis_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_reduce_demo)
* [kernel_tensor_axis_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_family_demo)
* [kernel_tensor_axis_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_select_demo)
* [kernel_tensor_axis_sort_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_sort_demo)
* [kernel_tensor_axis_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_order_demo)
* [kernel_tensor_axis_map_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_map_demo)
* [kernel_tensor_axis_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_pipeline_demo)

Suggested reading order inside this subgroup:

* reduction:
  [kernel_tensor_axis_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_reduce_demo),
  [kernel_tensor_axis_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_family_demo)
* selection:
  [kernel_tensor_axis_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_select_demo)
* ordered selection:
  [kernel_tensor_axis_sort_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_sort_demo),
  [kernel_tensor_axis_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_order_demo)
* transform:
  [kernel_tensor_axis_map_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_map_demo)
* composed mini-flow:
  [kernel_tensor_axis_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_pipeline_demo)
* bridge:
  [kernel_tensor_axis_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_roundtrip_demo)

Shader subgroup:

* surface branch:
  [shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo),
  [shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo),
  [shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo),
  [shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo),
  [shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo),
  [shader_surface_state_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_profile_demo),
  [shader_surface_state_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_packet_profile_demo),
  [shader_surface_state_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_pass_profile_demo),
  [shader_surface_state_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_flow_profile_demo),
  [shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo),
  [shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo),
  [shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo)
* packet branch:
  [shader_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_profile_demo),
  [shader_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo)
* bridge branch:
  [shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo),
  [shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo),
  [shader_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_result_profile_demo),
  [shader_async_fanin_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fanin_profile_demo),
  [shader_async_schedule_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_schedule_profile_demo),
  [shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo),
  [shader_async_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fallback_profile_demo),
  [shader_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_batch_profile_demo),
  [shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo),
  [shader_result_family_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_family_profile_demo),
  [shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo),
  [shader_draw_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_render_profile_demo),
  [shader_draw_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_profile_demo),
  [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)

Reading rule:

* use [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  for the project-wide route
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* read shader lanes in this order:
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo) ->
  surface branch ->
  packet branch ->
  bridge branch ->
  [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* inside shader, use this shorter local rule:
  surface = metadata -> material seeds -> state set -> state+packet / state+pass -> state mini-flow
  packet = packet contract -> packet bridge
  bridge = pass -> frame -> async result consume -> async fan-in -> async scheduling -> async policy -> async fallback -> async batch -> async windowed batch -> result family -> draw/render split -> wider draw/render
* when comparing shader/kernel async lanes back to `std` task:
  task async control = fallback -> policy -> batch -> windowed batch
  task async result = result family -> result policy -> result batch -> result windowed batch
  shader reads closest to task async control first
  kernel reads closest to task async result first
  shader policy/fallback now also use explicit local task-shaped helpers in
  source: `ShaderTaskPolicySummary` / `capture_task_policy(...)` and
  `ShaderTaskFallbackSummary` / `capture_task_fallback(...)`
  kernel tensor policy/fallback now also use explicit local task-shaped
  helpers in source: `KernelTaskPolicySummary` / `capture_task_policy(...)`
  and `KernelTaskFallbackSummary` / `capture_task_fallback(...)`
