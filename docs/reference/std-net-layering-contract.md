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
* [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns)
* [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns)
* [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
* [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns)
* [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns)
* [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns)
* [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns)
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
* [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns)
* [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns)
* [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns)
* [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns)
* [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns)
* [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns)
* [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns)
* [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

Current companion validation routes:

* [net_endpoint_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_endpoint_recipe_demo)
* [net_ip_packet_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_ip_packet_recipe_demo)
* [net_tcp_stream_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_stream_recipe_demo)
* [net_udp_datagram_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_datagram_recipe_demo)
* [net_connect_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_connect_recipe_demo)
* [net_listen_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_listen_recipe_demo)
* [net_close_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_close_recipe_demo)
* [net_protocol_experiment_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_experiment_recipe_demo)
* [net_line_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_line_protocol_recipe_demo)
* [net_datagram_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_protocol_recipe_demo)
* [net_dnsish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_protocol_recipe_demo)
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
* [net_protocol_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_session_recipe_demo)
* [net_datagram_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_session_recipe_demo)
* [net_datagram_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_exchange_session_recipe_demo)
* [net_datagram_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_pipeline_recipe_demo)
* [net_dnsish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_exchange_session_recipe_demo)
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
* `net_connect`
  reads:
  `local_port -> remote_port -> connect_timeout -> connect summary`
* `net_listen`
  reads:
  `local_port -> read_timeout -> write_timeout -> listen summary`
* `net_close`
  reads:
  `local_port -> close result -> close summary`
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
* `net_httpish_session`
  reads:
  `httpish protocol summary + transport shape -> httpish session summary`
* `net_httpish_exchange_session`
  reads:
  `httpish roundtrip + timeout/retry -> exchange session summary`
* `net_session`
  reads:
  `control session + transport session + protocol session + datagram session + datagram exchange session + datagram pipeline + dns-ish exchange session + httpish session + exchange session + result bridge + task bridge -> session summary`

## Current Reading Rule

The shortest practical route today is easiest to read in three grouped steps:

* profile core
  [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
* transport edge
  [net_ip_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_packet_recipe.ns) ->
  [net_tcp_stream_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_stream_recipe.ns) ->
  [net_udp_datagram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_recipe.ns)
* control edge
  [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns) ->
  [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns) ->
  [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
* protocol edge
  [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns) ->
  [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns) ->
  [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns) ->
  [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns) ->
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
  [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns) ->
  [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns) ->
  [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns) ->
  [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns) ->
  [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns) ->
  [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns) ->
  [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns) ->
  [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

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
* httpish protocol:
  [net_httpish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_protocol_recipe.ns)
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
* protocol session:
  [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns)
* datagram session:
  [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns)
* datagram exchange session:
  [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns)
* datagram pipeline:
  [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns)
* dns-ish exchange session:
  [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns)
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
-> httpish protocol
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
-> httpish session
-> httpish exchange session
-> session
```

Grouped rule:

```text
profile core
-> transport edge
-> control edge
-> protocol edge
-> result spine
-> task spine
-> session
```

Project-facing CLI hint:

* `nuis project-status <network-project>`
* `nuis project-doctor <network-project>`

Those now surface:

* `std_net_navigation`
  `profile_core -> transport_edge -> control_edge -> protocol_edge -> result_spine -> task_spine -> session`
* `std_net_samples`
  the shortest checked-in recipe/demo companion route for each of those grouped lanes

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
