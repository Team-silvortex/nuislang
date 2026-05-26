# `std` Net Layering Contract

This file captures the first thin `std net` facade over the current
`official.network` domain.

It sits below
[network-profile-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-profile-contract.md):
that file describes the domain-owned truth, while this file describes the first
checked-in `std` reading front door.

## Current Lane Shape

The current `std net` lane prefers this order:

```text
network profile truth
-> endpoint recipe
-> ip-packet recipe
-> tcp-open recipe
-> udp-open recipe
-> udp-bind recipe
-> tcp-listener recipe
-> owned-send recipe
-> owned-recv recipe
-> owned-accept recipe
-> owned-close recipe
-> connect recipe
-> listen recipe
-> close recipe
-> protocol-experiment recipe
-> line-protocol recipe
-> datagram-protocol recipe
-> httpish-protocol recipe
-> result recipe
-> result-bridge recipe
-> task-policy recipe
-> task-batch recipe
-> task-windowed recipe
-> task-windowed-bridge recipe
-> control-session recipe
-> transport-session recipe
-> protocol-session recipe
-> dnsish-exchange-session recipe
-> session recipe
```

The practical current rule is:

* `official.network` still owns ABI, scheduler contract, host bridge, and
  result semantics
* `std net` is now the first thin readable facade layer over that truth
* these recipes are intentionally narrow and do not yet claim a finished socket
  API
* these recipe sources are single-module front doors; current repository-stage
  validation goes through project companions

## Current Thin Recipes

* [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
* [net_ip_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_packet_recipe.ns)
* [net_tcp_stream_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_stream_recipe.ns)
* [net_udp_datagram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_recipe.ns)
* [net_tcp_open_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_open_recipe.ns)
* [net_udp_open_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_open_recipe.ns)
* [net_udp_bind_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bind_recipe.ns)
* [net_udp_bound_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bound_socket_recipe.ns)
* [net_tcp_listener_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_listener_recipe.ns)
* [net_tcp_accepted_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_accepted_socket_recipe.ns)
* [net_owned_send_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_send_recipe.ns)
* [net_owned_recv_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_recv_recipe.ns)
* [net_owned_accept_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_accept_recipe.ns)
* [net_owned_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_close_recipe.ns)
* [net_tcp_connect_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_connect_socket_recipe.ns)
* [net_tcp_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_socket_recipe.ns)
* [net_tcp_server_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_server_socket_recipe.ns)
* [net_udp_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_socket_recipe.ns)
* [net_ip_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_socket_recipe.ns)
* [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns)
* [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns)
* [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
* [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns)
* [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns)
* [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns)
* [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns)
* [net_dnsish_query_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_query_recipe.ns)
* [net_httpish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_protocol_recipe.ns)
* [net_httpish_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_request_recipe.ns)
* [net_httpish_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_response_recipe.ns)
* [net_httpish_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_roundtrip_recipe.ns)
* [net_result_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_recipe.ns)
* [net_result_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_bridge_recipe.ns)
* [net_task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_policy_recipe.ns)
* [net_task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_batch_recipe.ns)
* [net_task_windowed_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_recipe.ns)
* [net_task_windowed_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_bridge_recipe.ns)
* [net_control_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_control_session_recipe.ns)
* [net_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_session_recipe.ns)
* [net_tcp_listener_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_listener_session_recipe.ns)
* [net_owned_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_transport_session_recipe.ns)
* [net_transport_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_path_compare_recipe.ns)
* [net_dnsish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_path_compare_recipe.ns)
* [net_httpish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_path_compare_recipe.ns)
* [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns)
* [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns)
* [net_owned_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_datagram_session_recipe.ns)
* [net_udp_bound_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bound_session_recipe.ns)
* [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns)
* [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns)
* [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns)
* [net_owned_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_exchange_session_recipe.ns)
* [net_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_pipeline_recipe.ns)
* [net_owned_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_pipeline_recipe.ns)
* [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns)
* [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns)
* [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

Current companion validation routes:

* [net_endpoint_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_endpoint_recipe_demo)
* [net_ip_packet_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_ip_packet_recipe_demo)
* [net_tcp_stream_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_stream_recipe_demo)
* [net_udp_datagram_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_datagram_recipe_demo)
* [net_tcp_open_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_open_recipe_demo)
* [net_udp_open_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_open_recipe_demo)
* [net_udp_bind_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_bind_recipe_demo)
* [net_udp_bound_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_bound_socket_recipe_demo)
* [net_tcp_listener_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_listener_recipe_demo)
* [net_tcp_accepted_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_accepted_socket_recipe_demo)
* [net_owned_send_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_send_recipe_demo)
* [net_owned_recv_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_recv_recipe_demo)
* [net_owned_accept_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_accept_recipe_demo)
* [net_owned_close_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_close_recipe_demo)
* [net_tcp_connect_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_connect_socket_recipe_demo)
* [net_tcp_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_socket_recipe_demo)
* [net_tcp_server_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_server_socket_recipe_demo)
* [net_udp_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_socket_recipe_demo)
* [net_ip_socket_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_ip_socket_recipe_demo)
* [net_connect_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_connect_recipe_demo)
* [net_listen_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_listen_recipe_demo)
* [net_close_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_close_recipe_demo)
* [net_protocol_experiment_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_experiment_recipe_demo)
* [net_line_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_line_protocol_recipe_demo)
* [net_datagram_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_protocol_recipe_demo)
* [net_dnsish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_protocol_recipe_demo)
* [net_dnsish_query_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_query_recipe_demo)
* [net_httpish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_protocol_recipe_demo)
* [net_httpish_request_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_request_recipe_demo)
* [net_httpish_response_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_response_recipe_demo)
* [net_httpish_roundtrip_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_roundtrip_recipe_demo)
* [net_result_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_recipe_demo)
* [net_result_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_bridge_recipe_demo)
* [net_task_policy_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_policy_recipe_demo)
* [net_task_batch_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_batch_recipe_demo)
* [net_task_windowed_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_recipe_demo)
* [net_task_windowed_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_bridge_recipe_demo)
* [net_control_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_control_session_recipe_demo)
* [net_transport_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_transport_session_recipe_demo)
* [net_tcp_listener_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_listener_session_recipe_demo)
* [net_owned_transport_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_transport_session_recipe_demo)
* [net_transport_path_compare_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_transport_path_compare_recipe_demo)
* [net_dnsish_path_compare_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_path_compare_recipe_demo)
* [net_httpish_path_compare_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_path_compare_recipe_demo)
* [net_protocol_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_session_recipe_demo)
* [net_datagram_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_session_recipe_demo)
* [net_owned_datagram_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_datagram_session_recipe_demo)
* [net_udp_bound_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_bound_session_recipe_demo)
* [net_datagram_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_exchange_session_recipe_demo)
* [net_datagram_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_pipeline_recipe_demo)
* [net_dnsish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_exchange_session_recipe_demo)
* [net_owned_dnsish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_dnsish_exchange_session_recipe_demo)
* [net_dnsish_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_pipeline_recipe_demo)
* [net_owned_dnsish_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_owned_dnsish_pipeline_recipe_demo)
* [net_httpish_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_session_recipe_demo)
* [net_httpish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_session_recipe_demo)
* [net_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_recipe_demo)

Current role split:

* `net_endpoint`
  reads:
  `bind_core -> endpoint_kind -> transport_family -> local/remote_port -> connect/read/write timeout`
* `net_ip_packet`
  reads:
  `transport_family + protocol_header_bytes + send/recv probes -> ip packet summary`
* `net_tcp_stream`
  reads:
  `transport_family + stream windows + send/recv probes -> tcp stream summary`
* `net_udp_datagram`
  reads:
  `transport_family + datagram windows + send/recv probes -> udp datagram summary`
* `net_tcp_open`
  reads:
  `remote_port + connect_timeout -> owned tcp handle summary`
* `net_udp_open`
  reads:
  `local_port + remote_port + timeout -> owned udp handle summary`
* `net_udp_bind`
  reads:
  `local_port + read/write timeout -> owned udp bind summary`
* `net_tcp_listener`
  reads:
  `local_port + read/write timeout -> owned tcp listener summary`
* `net_owned_send`
  reads:
  `owned handle + send window -> owned send summary`
* `net_owned_recv`
  reads:
  `owned handle + recv window -> owned recv summary`
* `net_owned_accept`
  reads:
  `owned listener handle + timeout -> owned accept summary`
* `net_owned_close`
  reads:
  `owned handle -> owned close summary`
* `net_connect`
  reads:
  `local_port -> remote_port -> connect_timeout -> connect summary`
* `net_listen`
  reads:
  `local_port -> read_timeout -> write_timeout -> listen summary`
* `net_close`
  reads:
  `local_port -> close result -> close summary`
* `net_tcp_server_socket`
  reads:
  `listener handle + accept result + owned closes -> tcp server socket summary`
* `net_tcp_server_flow`
  reads:
  `listener + accept + owned send/recv + owned closes -> tcp server flow summary`
* `net_tcp_connect_socket`
  reads:
  `remote_port + connect_timeout + owned send/recv/close -> tcp connect socket summary`
* `net_tcp_client_flow`
  reads:
  `connect + owned send/recv/close -> tcp client flow summary`
* `net_tcp_accepted_socket`
  reads:
  `listener + accept + owned send/recv/close -> tcp accepted socket summary`
* `net_udp_bound_socket`
  reads:
  `local_port + read/write timeout + owned send/recv/close -> udp bound socket summary`
* `net_udp_datagram_flow`
  reads:
  `bind + owned send/recv/close -> udp datagram flow summary`
* `net_protocol_experiment`
  reads:
  `transport_family + protocol slots + send/recv probes -> protocol experiment summary`
* `net_line_protocol`
  reads:
  `tcp-stream-backed protocol slots + line framing + send/recv probes -> line protocol summary`
* `net_datagram_protocol`
  reads:
  `udp-datagram-backed protocol slots + send/recv probes -> datagram protocol summary`
* `net_dnsish_protocol`
  reads:
  `udp-datagram-backed dns-ish framing + send/recv probes -> dns-ish protocol summary`
* `net_dnsish_query`
  reads:
  `dns-ish framing + send probe -> query summary`
* `net_httpish_protocol`
  reads:
  `tcp-stream-backed protocol slots + request/header/body framing + send/recv probes -> httpish protocol summary`
* `net_httpish_request`
  reads:
  `httpish framing + send probe -> request summary`
* `net_httpish_response`
  reads:
  `httpish framing + recv probe -> response summary`
* `net_httpish_roundtrip`
  reads:
  `httpish request + response shape -> roundtrip summary`
* `net_http_client`
  reads:
  `tcp-socket-backed request/response exchange + close -> http client summary`
* `net_http_request_builder`
  reads:
  `tcp-socket-backed method/path/header/body builder + close -> http request builder summary`
* `net_http_client_headers`
  reads:
  `tcp-socket-backed header shaping + close -> http client headers summary`
* `net_http_client_url`
  reads:
  `tcp-socket-backed url authority/path/query shaping + close -> http client url summary`
* `net_http_client_body`
  reads:
  `tcp-socket-backed body shaping + close -> http client body summary`
* `net_http_client_status`
  reads:
  `tcp-socket-backed response status shaping + close -> http client status summary`
* `net_http_request`
  reads:
  `tcp-socket-backed request send + close -> http request summary`
* `net_http_response`
  reads:
  `tcp-socket-backed response recv + close -> http response summary`
* `net_http_client_exchange`
  reads:
  `tcp-socket-backed request/response exchange + timeout/retry + close -> http client exchange summary`
* `net_http_client_session`
  reads:
  `http client/request/response/exchange summaries -> http client session summary`
* `net_http_client_get`
  reads:
  `tcp-socket-backed GET exchange + close -> http client GET summary`
* `net_http_client_post`
  reads:
  `tcp-socket-backed POST exchange + close -> http client POST summary`
* `net_result`
  reads:
  `config_ready + send_ready + recv_ready -> network_value`
* `net_result_bridge`
  reads:
  `result batch/windowed -> session bridge summary`
* `net_task_policy`
  reads:
  `send/recv/config result -> primary/secondary/fallback task policy`
* `net_task_batch`
  reads:
  `control/tx/rx task batch`
* `net_task_windowed`
  reads:
  `batch summary -> preview/final`
* `net_task_windowed_bridge`
  reads:
  `windowed summary -> session bridge summary`
* `net_control_session`
  reads:
  `connect/listen/close summaries -> control session summary`
* `net_transport_session`
  reads:
  `send/recv transport results -> transport session summary`
* `net_protocol_session`
  reads:
  `protocol experiment + transport shape -> protocol session summary`
* `net_datagram_session`
  reads:
  `udp-datagram-backed protocol summary + timeout/retry -> datagram session summary`
* `net_datagram_exchange_session`
  reads:
  `udp-datagram-backed exchange summary + timeout/retry -> datagram exchange session summary`
* `net_datagram_pipeline`
  reads:
  `multiple udp-datagram-backed exchanges -> datagram pipeline summary`
* `net_dnsish_exchange_session`
  reads:
  `dns-ish query/answer framing + timeout/retry -> dns-ish exchange session summary`
* `net_dnsish_pipeline`
  reads:
  `multiple dns-ish exchanges -> dns-ish pipeline summary`
* `net_httpish_session`
  reads:
  `httpish protocol summary + transport shape -> httpish session summary`
* `net_httpish_exchange_session`
  reads:
  `httpish roundtrip + timeout/retry -> exchange session summary`
* `net_session`
  reads:
  `control session + transport session + protocol session + datagram session + datagram exchange session + datagram pipeline + dns-ish exchange session + dns-ish pipeline + httpish session + exchange session + result bridge + task bridge -> session summary`

## Current Reading Rule

The shortest practical route today is easiest to read in three grouped steps:

* profile core
  [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
* transport edge
  [net_ip_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_packet_recipe.ns) ->
  [net_tcp_stream_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_stream_recipe.ns) ->
  [net_udp_datagram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_recipe.ns)
* socket edge
  [net_tcp_connect_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_connect_socket_recipe.ns) ->
  [net_tcp_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_socket_recipe.ns) ->
  [net_tcp_server_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_server_socket_recipe.ns) ->
  [net_tcp_accepted_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_accepted_socket_recipe.ns) ->
  [net_udp_bound_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bound_socket_recipe.ns) ->
  [net_udp_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_socket_recipe.ns) ->
  [net_ip_socket_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_socket_recipe.ns)
* control edge
  [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns) ->
  [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns) ->
  [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
* protocol edge
  [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns) ->
  [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns) ->
  [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns) ->
  [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns) ->
  [net_dnsish_query_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_query_recipe.ns) ->
  [net_httpish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_protocol_recipe.ns) ->
  [net_httpish_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_request_recipe.ns) ->
  [net_httpish_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_response_recipe.ns) ->
  [net_httpish_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_roundtrip_recipe.ns)
* result spine
  [net_result_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_recipe.ns) ->
  [net_result_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_bridge_recipe.ns)
* task spine
  [net_task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_policy_recipe.ns) ->
  [net_task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_batch_recipe.ns) ->
  [net_task_windowed_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_recipe.ns) ->
  [net_task_windowed_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_bridge_recipe.ns)
* session
  [net_control_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_control_session_recipe.ns) ->
  [net_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_session_recipe.ns) ->
  [net_tcp_listener_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_listener_session_recipe.ns) ->
  [net_owned_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_transport_session_recipe.ns) ->
  [net_transport_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_path_compare_recipe.ns) ->
  [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns) ->
  [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns) ->
  [net_owned_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_datagram_session_recipe.ns) ->
  [net_udp_bound_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bound_session_recipe.ns) ->
  [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns) ->
  [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns) ->
  [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns) ->
  [net_owned_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_exchange_session_recipe.ns) ->
  [net_dnsish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_path_compare_recipe.ns) ->
  [net_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_pipeline_recipe.ns) ->
  [net_owned_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_pipeline_recipe.ns) ->
  [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns) ->
  [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns) ->
  [net_httpish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_path_compare_recipe.ns) ->
  [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)
* owned session rule
  `owned transport session -> owned datagram session -> owned dns-ish exchange -> owned dns-ish pipeline`
* compare rule
  `probe transport session -> owned transport session -> transport path compare`
* dns-ish compare rule
  `dns-ish exchange session -> owned dns-ish exchange session -> dns-ish path compare`
* httpish compare rule
  `httpish exchange session -> httpish path compare`
* flow rule
  `tcp client flow -> tcp server flow -> udp datagram flow`
* grouped compare rule
  `transport compare -> dns-ish compare -> httpish compare`

Expanded route:

* endpoint:
  [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
* tcp stream:
  [net_tcp_stream_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_stream_recipe.ns)
* ip packet:
  [net_ip_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_packet_recipe.ns)
* udp datagram:
  [net_udp_datagram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_recipe.ns)
* connect:
  [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns)
* listen:
  [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns)
* close:
  [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
* protocol experiment:
  [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns)
* line protocol:
  [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns)
* datagram protocol:
  [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns)
* dns-ish protocol:
  [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns)
* dns-ish query:
  [net_dnsish_query_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_query_recipe.ns)
* httpish protocol:
  [net_httpish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_protocol_recipe.ns)
* http client:
  [net_http_client_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_recipe.ns)
* http request builder:
  [net_http_request_builder_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_request_builder_recipe.ns)
* http client headers:
  [net_http_client_headers_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_headers_recipe.ns)
* http client url:
  [net_http_client_url_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_url_recipe.ns)
* http client body:
  [net_http_client_body_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_body_recipe.ns)
* http client status:
  [net_http_client_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_status_recipe.ns)
* http request:
  [net_http_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_request_recipe.ns)
* http response:
  [net_http_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_response_recipe.ns)
* http client exchange:
  [net_http_client_exchange_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_exchange_recipe.ns)
* http client session:
  [net_http_client_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_session_recipe.ns)
* http client GET:
  [net_http_client_get_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_get_recipe.ns)
* http client POST:
  [net_http_client_post_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_post_recipe.ns)
* httpish request:
  [net_httpish_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_request_recipe.ns)
* httpish response:
  [net_httpish_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_response_recipe.ns)
* httpish roundtrip:
  [net_httpish_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_roundtrip_recipe.ns)
* result:
  [net_result_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_recipe.ns)
* result bridge:
  [net_result_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_bridge_recipe.ns)
* async control:
  [net_task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_policy_recipe.ns)
* async fan-in:
  [net_task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_batch_recipe.ns)
* async windowed:
  [net_task_windowed_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_recipe.ns)
* async bridge:
  [net_task_windowed_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_bridge_recipe.ns)
* control session:
  [net_control_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_control_session_recipe.ns)
* transport session:
  [net_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_session_recipe.ns)
* owned transport session:
  [net_owned_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_transport_session_recipe.ns)
* transport path compare:
  [net_transport_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_path_compare_recipe.ns)
* dns-ish path compare:
  [net_dnsish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_path_compare_recipe.ns)
* httpish path compare:
  [net_httpish_path_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_path_compare_recipe.ns)
* protocol session:
  [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns)
* datagram session:
  [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns)
* owned datagram session:
  [net_owned_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_datagram_session_recipe.ns)
* udp bound session:
  [net_udp_bound_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_bound_session_recipe.ns)
* datagram exchange session:
  [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns)
* datagram pipeline:
  [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns)
* dns-ish exchange session:
  [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns)
* owned dns-ish exchange session:
  [net_owned_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_exchange_session_recipe.ns)
* dns-ish pipeline:
  [net_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_pipeline_recipe.ns)
* owned dns-ish pipeline:
  [net_owned_dnsish_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_owned_dnsish_pipeline_recipe.ns)
* httpish session:
  [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns)
* httpish exchange session:
  [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns)
* session:
  [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

That means the current `std net` front door should be read as:

```text
endpoint/timing
-> tcp stream
-> udp datagram
-> connect/listen/close
-> protocol experiment
-> line protocol
-> dns-ish query
-> httpish protocol
-> http client
-> http request builder
-> http client headers
-> http client url
-> http client body
-> http client status
-> http request
-> http response
-> http client exchange
-> http client session
-> http client GET
-> http client POST
-> httpish request
-> httpish response
-> httpish roundtrip
-> result observe
-> result bridge
-> task policy
-> task batch
-> task windowed
-> task bridge
-> control session
-> transport session
-> protocol session
-> dns-ish exchange session
-> dns-ish pipeline
-> httpish session
-> httpish exchange session
-> session
```

Grouped rule:

```text
profile core
-> transport edge
-> socket edge
-> control edge
-> protocol edge
-> http edge
-> result spine
-> task spine
-> session
```

Project-facing CLI hint:

* `nuis project-status <network-project>`
* `nuis project-doctor <network-project>`

Those now surface:

* `std_net_navigation`
  `profile_core -> transport_edge -> syscall_edge -> socket_edge -> control_edge -> protocol_edge -> http_edge -> result_spine -> task_spine -> session`
* `std_net_samples`
  the shortest checked-in recipe/demo companion route for each of those grouped lanes

Low-level syscall rule:

* `tcp open -> udp open -> owned send -> owned recv -> owned close`

Current validation rule:

```text
std recipe source
-> project companion check/build
```

Local naming rule:

* `capture_*_summary()` returns the current struct-shaped summary layer
* `summarize_*_recipe()` returns the final scalar recipe reading

Local field-order rule:

* profile-facing summaries keep identity before timing
  `bind_core -> endpoint_kind -> local/remote_port -> timeout fields`
* result/task observation summaries keep status before aggregate value
  `*_ready|*_completed|*_timed_out -> *value`
* bridge/session summaries keep context first, stage values next, terminal sinks last
  `argv/bind/endpoint/timeout/retry -> preview/final/result/task values -> session_value/tick_ns`

## Boundary

This lane does not yet try to freeze:

* `std net socket`
* listener/session object models
* full send/recv stream buffering
* higher-level protocol APIs

Those should grow later once the thin facade surface is stable.
