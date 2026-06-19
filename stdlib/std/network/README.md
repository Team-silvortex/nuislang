# `std/network`

This directory is the reading router for the `std net` facade.

Keep the actual recipe sources in
[`stdlib/std`](/Users/Shared/chroot/dev/nuislang/stdlib/std) for now; this file
exists to give the network surface a module-shaped front door before we do any
higher-risk filesystem reshuffle.

Canonical companions:

* domain-owned truth:
  [network-profile-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-profile-contract.md)
* `std net` layering rule:
  [std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
* shortest repo-wide route:
  [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)

## Current Lane Shape

Read the current network surface in this order:

```text
profile core
-> transport edge
-> syscall edge
-> socket edge
-> control edge
-> protocol edge
-> http edge
-> result spine
-> task spine
-> session
```

## Current Workflow Convention

Within the current `http` / `httpish` session recipes, the preferred internal
shape is now:

```text
workflow helpers
-> build_*_plan
-> run / send / recv step helpers
-> stage_*_packet
-> compute_packet_value
-> compute_session_value (only when a wider session layer exists)
-> capture_*_summary
-> summarize_*_recipe
```

Value naming rule:

* use `packet_value` for packet-shaped summaries
* use `session_value` for summaries that include transport/session lifecycle
* keep helper names explicit about the transport phase:
  `open_*`, `accept_*`, `send_*`, `recv_*`, `close_*`

Current anchor files for this convention:

* packet-first async/httpish
  - [net_httpish_client_session_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_client_session_packet_recipe.ns)
  - [net_httpish_service_session_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_service_session_packet_recipe.ns)
* packet plus session aggregation
  - [net_httpish_header_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_header_session_recipe.ns)
  - [net_httpish_header_service_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_header_service_session_recipe.ns)
* host-owned transport workflow helpers
  - [net_http_client_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_session_recipe.ns)
  - [net_http_client_session_async_loop_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_session_async_loop_recipe.ns)
  - [net_http_service_lane_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_service_lane_recipe.ns)

Current narrow frontdoor for the HTTP/session cluster:

```text
net_http_client_session_recipe
-> net_httpish_header_session_recipe
-> net_http_client_lane_recipe
```

Service-side mirror:

```text
net_httpish_service_session_packet_recipe
-> net_httpish_header_service_session_recipe
-> net_http_service_lane_recipe
```

Current frontdoor cheat sheet:

* client chain
  `net_http_client_session_recipe -> net_httpish_header_session_recipe -> net_http_client_lane_recipe`
* service chain
  `net_httpish_service_session_packet_recipe -> net_httpish_header_service_session_recipe -> net_http_service_lane_recipe`
* shared lane summary names
  `request_header_bytes`, `request_body_bytes`, `request_bytes`,
  `response_header_bytes`, `response_body_bytes`, `response_bytes`
* client-only extra split
  `authority_bytes`, `path_bytes`

## Source Router

### Profile Core

* [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)

### Transport Edge

* [net_ip_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_packet_recipe.ns)
* [net_tcp_stream_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_stream_recipe.ns)
* [net_udp_datagram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_recipe.ns)

### Syscall Edge

* [net_tcp_open_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_open_recipe.ns)
* [net_udp_open_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_open_recipe.ns)
* [net_udp_bind_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bind_recipe.ns)
* [net_tcp_listener_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_listener_recipe.ns)
* [net_owned_send_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_send_recipe.ns)
* [net_owned_recv_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_recv_recipe.ns)
* [net_owned_accept_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_accept_recipe.ns)
* [net_owned_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_close_recipe.ns)

### Socket Edge

* [net_tcp_connect_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_connect_socket_recipe.ns)
* [net_tcp_client_flow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_client_flow_recipe.ns)
* [net_tcp_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_socket_recipe.ns)
* [net_tcp_server_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_server_socket_recipe.ns)
* [net_tcp_server_flow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_server_flow_recipe.ns)
* [net_tcp_accepted_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_accepted_socket_recipe.ns)
* [net_udp_bound_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bound_socket_recipe.ns)
* [net_udp_datagram_flow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_flow_recipe.ns)
* [net_udp_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_socket_recipe.ns)
* [net_ip_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_socket_recipe.ns)

### Control Edge

* [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns)
* [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns)
* [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)

### Protocol Edge

* [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns)
* [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns)
* [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns)
* [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns)
* [net_dnsish_query_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_query_recipe.ns)
* [net_httpish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_protocol_recipe.ns)
* [net_httpish_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_request_recipe.ns)
* [net_httpish_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_response_recipe.ns)
* [net_httpish_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_roundtrip_recipe.ns)

### HTTP Edge

* [net_http_client_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_recipe.ns)
* [net_http_request_builder_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_request_builder_recipe.ns)
* [net_http_client_headers_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_headers_recipe.ns)
* [net_http_client_url_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_url_recipe.ns)
* [net_http_client_body_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_body_recipe.ns)
* [net_http_client_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_status_recipe.ns)
* [net_http_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_request_recipe.ns)
* [net_http_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_response_recipe.ns)
* [net_http_client_exchange_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_exchange_recipe.ns)
* [net_http_client_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_session_recipe.ns)
* [net_http_client_lane_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_lane_recipe.ns)
* [net_http_client_get_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_get_recipe.ns)
* [net_http_client_post_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_post_recipe.ns)
* [net_http_service_lane_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_service_lane_recipe.ns)

### Result Spine

* [net_result_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_recipe.ns)
* [net_result_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_bridge_recipe.ns)

### Task Spine

* [net_task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_policy_recipe.ns)
* [net_task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_batch_recipe.ns)
* [net_task_windowed_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_recipe.ns)
* [net_task_windowed_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_bridge_recipe.ns)

### Session

* [net_control_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_control_session_recipe.ns)
* [net_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_session_recipe.ns)
* [net_owned_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_transport_session_recipe.ns)
* [net_transport_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_path_compare_recipe.ns)
* [net_dnsish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_path_compare_recipe.ns)
* [net_httpish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_path_compare_recipe.ns)
* [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns)
* [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns)
* [net_owned_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_datagram_session_recipe.ns)
* [net_udp_bound_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bound_session_recipe.ns)
* [net_tcp_listener_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_listener_session_recipe.ns)
* [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns)
* [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns)
* [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns)
* [net_owned_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_exchange_session_recipe.ns)
* [net_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_pipeline_recipe.ns)
* [net_owned_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_pipeline_recipe.ns)
* [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns)
* [net_httpish_header_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_header_session_recipe.ns)
* [net_httpish_header_service_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_header_service_session_recipe.ns)
* [net_httpish_client_session_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_client_session_packet_recipe.ns)
* [net_httpish_service_session_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_service_session_packet_recipe.ns)
* [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns)
* [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

## Companion Validation Router

Use [examples/projects/domains](/Users/Shared/chroot/dev/nuislang/examples/projects/domains) as the executable companion set.

Shortest grouped route:

* profile / transport
  - [net_endpoint_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_endpoint_recipe_demo)
  - [net_ip_packet_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_ip_packet_recipe_demo)
  - [net_tcp_stream_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_stream_recipe_demo)
  - [net_udp_datagram_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_datagram_recipe_demo)
* sockets / control
  - [net_tcp_connect_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_connect_socket_recipe_demo)
  - [net_tcp_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_socket_recipe_demo)
  - [net_tcp_server_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_server_socket_recipe_demo)
  - [net_connect_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_connect_recipe_demo)
  - [net_listen_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_listen_recipe_demo)
  - [net_close_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_close_recipe_demo)
* protocol / http
  - [net_protocol_experiment_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_experiment_recipe_demo)
  - [net_datagram_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_protocol_recipe_demo)
  - [net_dnsish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_protocol_recipe_demo)
  - [net_httpish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_protocol_recipe_demo)
  - [net_httpish_client_session_packet_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_client_session_packet_recipe_demo)
  - [net_httpish_service_session_packet_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_service_session_packet_recipe_demo)
  - [net_httpish_header_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_header_session_recipe_demo)
  - [net_httpish_header_service_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_header_service_session_recipe_demo)
  - [net_http_client_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_recipe_demo)
  - [net_http_client_exchange_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_exchange_recipe_demo)
  - [net_http_client_lane_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_lane_recipe_demo)
  - [net_http_service_lane_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_service_lane_recipe_demo)
* result / task / session
  - [net_result_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_recipe_demo)
  - [net_task_policy_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_policy_recipe_demo)
  - [net_transport_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_transport_session_recipe_demo)
  - [net_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_recipe_demo)

## Current Reading Rule

If you only want one pass:

1. start with [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
2. follow the grouped lane above until `session`
3. if your target is the current HTTP/session frontdoor, prefer
   `net_http_client_session_recipe -> net_httpish_header_session_recipe -> net_http_client_lane_recipe`
4. use the service mirror when validating listener-side shape:
   `net_httpish_service_session_packet_recipe -> net_httpish_header_service_session_recipe -> net_http_service_lane_recipe`
5. jump into the matching `examples/projects/domains/*_demo`
6. return to
   [std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
   when you want the contract language instead of the raw source list
