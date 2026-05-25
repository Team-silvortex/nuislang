# Network Profile Contract

This document captures the narrowest current profile-facing contract for the
bootstrap `official.network` domain.

It exists to answer a smaller question than
[network-domain-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-domain-contract.md):

* if `network` is the fifth `nustar`, what is the first readable surface we
  expect people to target?

The current answer is:

```text
bind_core
-> endpoint kind
-> endpoint ports
-> connect/read/write timeout
-> result observe
-> timeout / retry budget
-> stream window shape
```

Current short reading rule:

* `profile core`
  `bind_core -> endpoint_kind -> local/remote_port -> connect/read/write timeout -> timeout/retry -> stream/recv/send`
* `host control runtime`
  `endpoint/timing profile -> host_network_connect/accept/close`
* `host transport runtime`
  `stream/recv/send profile -> host_network_send/recv`
* `transport ladder`
  `transport result -> transport policy -> transport split -> transport batch split -> transport windowed split -> transport batch -> transport windowed -> transport/session bridge`
* `shared helper layer`
  `network_task_async_shapes -> async_session_summary / async_policy_summary / async_fallback_summary / async_batch_summary / async_windowed_summary`
* `branch class contract`
  `primary/secondary/fallback/send/recv -> stable observer branch labels`
* `result observe`
  `result -> config_ready -> value`
* `observer role variant contract`
  `config_ready/send_ready/recv_ready/connect_ready/accept_ready/closed -> stable observer role labels`
* `scheduler result samples`
  `CLI hint -> result ladder + connect/accept control ladder`
* `scheduler transport samples`
  `CLI hint -> transport runtime + transport split ladder + transport summary ladder`
* `scheduler sample navigation`
  `CLI ordering hint -> result ladder -> transport split ladder -> transport summary ladder -> summary classes`
* `scheduler-view block display`
  `project-frontdoor view -> multi-line sample hint blocks for network`
* `scheduler-view --json`
  `structured frontdoor view -> navigation/result/transport/summary sample hints as machine-readable fields`
* `summary class contract`
  `transport split/windowed/session-bridge + control split/windowed/session-bridge -> stable summary class labels`
* `scheduler summary samples`
  `CLI hint -> transport split ladder + control split ladder`
* `result ladder`
  `result observe -> result policy -> result batch -> result windowed -> result/session bridge`
* `connect/accept control ladder`
  `connect result -> accept result -> connect/accept policy -> connect/accept batch -> connect/accept windowed`
* `session`
  `summary -> session seed`
* `task control`
  `result policy -> profile policy -> fallback`
* `task summary`
  `result batch -> result windowed -> batch -> windowed`

Current narrow companion:

* [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo)
  now reads:
  `bind_core -> endpoint_kind -> timeout_budget -> retry_budget -> stream_window -> recv_window -> send_window`
* [network_endpoint_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_endpoint_profile_demo)
  now reads:
  `endpoint_kind -> local_port -> remote_port -> connect_timeout -> read_timeout -> write_timeout`
* [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo)
  now reads:
  `local_port -> remote_port -> connect_timeout -> read_timeout -> write_timeout -> host_network_connect/accept/close`
* [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo)
  now reads:
  `stream_window -> recv_window -> send_window -> local_port -> remote_port -> host_network_send/recv`
* [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo)
  now reads:
  `host_network_send/recv -> network_result -> network_send/recv_ready -> network_value`
* [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo)
  now reads:
  `transport result -> primary/secondary/fallback task policy`
* [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo)
  now reads:
  `send_ready/recv_ready split -> branch-local policy selection`
* [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo)
  now reads:
  `send/recv/fallback branch summaries -> merged batch value`
* [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo)
  now reads:
  `send/recv/fallback branch windows -> merged preview/final`
* [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo)
  now reads:
  `transport result -> send/recv/fallback task batch`
* [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo)
  now reads:
  `transport batch summary -> preview summary -> final summary`
* [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo)
  now reads:
  `send/recv/fallback branch windows -> branch session bridge`
* [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)
  now reads:
  `transport windowed summary -> session summary bridge`
* `nuis scheduler-view <network project>`
  now also prints:
  `scheduler_summary_samples`
  so the shortest CLI hint for the current split classes is:
  `transport_split=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo`
  and
  `control_split=network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo`
* [network_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/network_task_async_shapes.ns)
  now reads:
  `shared task-shaped session/result/batch/windowed helper layer`
* [network_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_profile_demo)
  now reads:
  `network_result -> network_config_ready -> network_value`
* [network_connect_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_result_demo)
  now reads:
  `local_port -> remote_port -> connect_timeout -> network_result -> network_config_ready -> network_value`
* [network_accept_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_accept_result_demo)
  now reads:
  `local_port -> read_timeout -> write_timeout -> network_result -> network_config_ready -> network_value`
* [network_connect_accept_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_policy_demo)
  now reads:
  `connect result -> accept result -> fallback timeout -> task policy`
* [network_connect_accept_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_batch_demo)
  now reads:
  `connect result -> accept result -> timeout result -> task batch`
* [network_connect_accept_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_windowed_batch_demo)
  now reads:
  `connect/accept batch summary -> preview summary -> final summary`
* [network_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_policy_demo)
  now reads:
  `network result -> async helper -> primary/secondary/fallback task policy`
* [network_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_batch_demo)
  now reads:
  `network result -> control/rx/tx task batch`
* [network_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_windowed_batch_demo)
  now reads:
  `result batch summary -> preview summary -> final summary`
* [network_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_session_bridge_demo)
  now reads:
  `result windowed summary -> session summary bridge`
* `nuis scheduler-view <network project>`
  now also prints:
  `scheduler_result_samples`
  so the shortest CLI hint for the current result-facing ladders is:
  `result_ladder=network_result_profile_demo -> network_connect_result_demo -> network_accept_result_demo -> network_result_task_policy_demo -> network_result_task_batch_demo -> network_result_task_windowed_batch_demo -> network_result_session_bridge_demo`
  and
  `control_ladder=network_connect_result_demo -> network_accept_result_demo -> network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo`
* `nuis scheduler-view <network project>`
  now also prints:
  `scheduler_transport_samples`
  so the shortest CLI hint for the current transport-facing ladders is:
  `transport_runtime=network_host_transport_runtime_demo -> network_transport_result_demo`
  and
  `transport_split_ladder=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo`
  and
  `transport_summary_ladder=network_transport_result_task_policy_demo -> network_transport_result_task_batch_demo -> network_transport_result_task_windowed_batch_demo -> network_transport_result_session_bridge_demo`
* [network_profile_summary_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_summary_demo)
  now reads:
  `profile slot observers -> summary struct capture`
* [network_profile_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_session_demo)
  now reads:
  `profile summary -> task seed -> timeout/join -> session summary`
* [network_profile_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_policy_demo)
  now reads:
  `profile summary -> primary/secondary/fallback task policy`
* [network_profile_task_fallback_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_fallback_demo)
  now reads:
  `primary timeout -> retry fallback -> shape fallback -> budget default`
* [network_profile_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_batch_demo)
  now reads:
  `profile summary -> control/tx/rx task batch`
* [network_profile_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_windowed_batch_demo)
  now reads:
  `batch summary -> preview summary -> final summary`

Suggested current reading ladder:

1. [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo)
2. [network_endpoint_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_endpoint_profile_demo)
3. [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo)
4. [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo)
5. [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo)
6. [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo)
7. [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo)
8. [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo)
9. [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo)
10. [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo)
11. [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo)
12. [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo)
13. [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)
14. [network_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_profile_demo)
15. [network_connect_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_result_demo)
16. [network_accept_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_accept_result_demo)
17. [network_connect_accept_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_policy_demo)
18. [network_connect_accept_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_batch_demo)
19. [network_connect_accept_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_windowed_batch_demo)
20. [network_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_policy_demo)
21. [network_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_batch_demo)
22. [network_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_windowed_batch_demo)
23. [network_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_session_bridge_demo)
24. [network_profile_summary_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_summary_demo)
25. [network_profile_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_session_demo)
26. [network_profile_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_policy_demo)
27. [network_profile_task_fallback_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_fallback_demo)
28. [network_profile_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_batch_demo)
29. [network_profile_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_windowed_batch_demo)

Compressed ladder:

* `shared helper`
  [network_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/network_task_async_shapes.ns)
* `result ladder`
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
* `session/task ladder`
  [network_profile_summary_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_summary_demo) ->
  [network_profile_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_session_demo) ->
  [network_profile_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_policy_demo) ->
  [network_profile_task_fallback_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_fallback_demo) ->
  [network_profile_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_batch_demo) ->
  [network_profile_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_windowed_batch_demo)

Shared helper reading rule:

* prefer `async_session_summary_*` when reading session seed / timeout / join
  shapes
* prefer `async_policy_summary_*` and `async_fallback_summary_*` when reading
  task control shapes
* prefer `async_batch_summary_*` and `async_windowed_*_summary_*` when reading
  both result and session/task fan-in ladders

Host control runtime reading rule:

* treat [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo)
  as the current narrow bridge from `network profile refs` into host-visible
  control symbols
* if you want the implementation-truth view, inspect its `YIR` and look for:
  `cpu.extern_call_i64 ... c host_network_connect_probe`,
  `cpu.extern_call_i64 ... c host_network_accept_probe`,
  `cpu.extern_call_i64 ... c host_network_close`
* read those as:
  `network endpoint/timing profile -> cpu host ffi bridge -> reserved network control symbol`

Host transport runtime reading rule:

* treat [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo)
  as the current narrow bridge from `network stream/recv/send profile refs`
  into host-visible transport symbols
* if you want the implementation-truth view, inspect its `YIR` and look for:
  `cpu.extern_call_i64 ... c host_network_send_probe`,
  `cpu.extern_call_i64 ... c host_network_recv_probe`
* read those as:
  `network transport shape profile -> cpu host ffi bridge -> reserved network transport symbol`

Transport result reading rule:

* start with:
  [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo)
* then read:
  [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo)
  ->
  [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo)
  ->
  [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo)
  ->
  [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo)
  ->
  [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo)
  ->
  [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo)
  ->
  [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo)
  ->
  [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo)
  ->
  [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)
* interpret that pair as:
  `transport profile -> host transport probe -> NetworkResult<T> observe/value -> task policy -> branch split -> branch batch split -> branch windowed split -> task batch -> task windowed -> branch session bridge -> session bridge`

Transport ladder reading rule:

* if you only want the narrow `tx/rx` path, read:
  [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo)
  ->
  [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo)
  ->
  [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo)
  ->
  [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo)
  ->
  [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo)
  ->
  [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo)
  ->
  [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo)
  ->
  [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo)
  ->
  [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)

Connect/accept control reading rule:

* start with endpoint-facing result observation:
  [network_connect_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_result_demo),
  [network_accept_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_accept_result_demo)
* then read control orchestration:
  [network_connect_accept_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_policy_demo) ->
  [network_connect_accept_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_batch_demo) ->
  [network_connect_accept_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_windowed_batch_demo)

## Scope

This document is only about the current manifest-first profile surface:

* `support_surface`
* `support_profile_slots`
* `default_lanes`
* the smallest reading order those names imply

It is not yet a promise that the repository already has:

* a complete `network` frontend
* a complete transport/runtime-facing `NetworkResult<T>` surface
* real socket lowering or runtime execution

What it does have now is a minimal control-result host skeleton in `YIR`:

* `network.connect`
* `network.accept`
* `network.close`
* matching probes:
  `network.is_connect_ready`, `network.is_accept_ready`, `network.is_closed`

So the safe current reading is:

* profile/demo lanes still explain intent at the source level
* `YIR` now has explicit control-result landing points for future syscall
  bridging

## Current Support Surface

The current bootstrap support surface registered by
[network.toml](/Users/Shared/chroot/dev/nuislang/nustar-packages/network.toml) is:

* `network.profile.bind-core.v1`
* `network.profile.connect.v1`
* `network.profile.accept.v1`
* `network.profile.send.v1`
* `network.profile.recv.v1`
* `network.profile.close.v1`
* `network.profile.timeout.v1`
* `network.profile.retry.v1`
* `network.profile.endpoint-kind.v1`
* `network.profile.stream-window.v1`

The important current reading rule is:

* control-facing lifecycle:
  `bind-core`, `connect`, `accept`, `close`, `timeout`, `retry`
* transport-facing movement:
  `send`, `recv`
* shape-facing metadata:
  `endpoint-kind`, `stream-window`

## Current Profile Slots

The current bootstrap profile slots are:

* `bind_core`
* `endpoint_kind`
* `local_port`
* `remote_port`
* `connect_timeout_ms`
* `read_timeout_ms`
* `write_timeout_ms`
* `retry_budget`
* `stream_window`
* `recv_window`
* `send_window`

These are intentionally narrow.

They are trying to freeze just enough contract to make transport semantics
readable without pretending we already know the final higher-level API.

## Current Slot Clusters

The current most useful way to read the slots is:

### Placement

* `bind_core`

This is the minimal scheduler-facing anchor.

It tells us `network` is expected to participate in the same
lane/clock/bridge world as the other checked-in domains.

### Endpoint identity

* `endpoint_kind`
* `local_port`
* `remote_port`

This is the narrowest endpoint-facing identity set that makes
`connect` / `accept` / `listen` readable.

### Timing and resilience

* `connect_timeout_ms`
* `read_timeout_ms`
* `write_timeout_ms`
* `retry_budget`

This is the first place where `network` clearly diverges from a thin `cpu`
primitive and starts behaving like its own system domain.

### Stream shape

* `stream_window`
* `recv_window`
* `send_window`

This is the narrowest shape-facing surface that hints at future:

* backpressure
* buffering
* windowed transport summaries

without freezing any specific final stream API.

## Current Lane Reading Rule

The current bootstrap lane contract implied by the manifest is:

* control lane:
  `bind_core`, `endpoint`, `listen`, `connect`, `accept`, `poll`, `close`,
  `retry`
* transport lanes:
  `send = tx`
  `recv = rx`

So the intended shortest mental model is:

```text
endpoint lifecycle lives on control
payload movement splits into tx / rx
timeouts and retry stay control-visible
```

## Current Growth Guidance

If you are adding the first actual `network` profile helper later, prefer this
growth order:

1. `network_profile_bind_core(...)`
2. `network_profile_endpoint_kind(...)`
3. `network_profile_local_port(...)`
4. `network_profile_remote_port(...)`
5. `network_profile_connect_timeout(...)`
6. `network_profile_read_timeout(...)`
7. `network_profile_write_timeout(...)`
8. `network_profile_timeout_budget(...)`
9. `network_profile_retry_budget(...)`
10. `network_profile_stream_window(...)`

That order keeps the first checked-in `network` route aligned with the current
manifest contract instead of jumping straight to a large socket or HTTP facade.

## Related References

* [network-domain-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-domain-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
