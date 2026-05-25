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
-> connect recipe
-> listen recipe
-> close recipe
-> result recipe
-> result-bridge recipe
-> task-policy recipe
-> task-batch recipe
-> task-windowed recipe
-> task-windowed-bridge recipe
-> control-session recipe
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
* [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns)
* [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns)
* [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
* [net_result_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_recipe.ns)
* [net_result_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_bridge_recipe.ns)
* [net_task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_policy_recipe.ns)
* [net_task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_batch_recipe.ns)
* [net_task_windowed_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_recipe.ns)
* [net_task_windowed_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_bridge_recipe.ns)
* [net_control_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_control_session_recipe.ns)
* [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

Current companion validation routes:

* [net_endpoint_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_endpoint_recipe_demo)
* [net_connect_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_connect_recipe_demo)
* [net_listen_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_listen_recipe_demo)
* [net_close_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_close_recipe_demo)
* [net_result_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_recipe_demo)
* [net_result_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_bridge_recipe_demo)
* [net_task_policy_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_policy_recipe_demo)
* [net_task_batch_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_batch_recipe_demo)
* [net_task_windowed_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_recipe_demo)
* [net_task_windowed_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_bridge_recipe_demo)
* [net_control_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_control_session_recipe_demo)
* [net_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_recipe_demo)

Current role split:

* `net_endpoint`
  reads:
  `bind_core -> endpoint_kind -> local/remote_port -> connect/read/write timeout`
* `net_connect`
  reads:
  `local_port -> remote_port -> connect_timeout -> connect summary`
* `net_listen`
  reads:
  `local_port -> read_timeout -> write_timeout -> listen summary`
* `net_close`
  reads:
  `local_port -> close result -> close summary`
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
* `net_session`
  reads:
  `result bridge + task bridge -> session summary`

## Current Reading Rule

The shortest practical route today is easiest to read in three grouped steps:

* profile core
  [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
* control edge
  [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns) ->
  [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns) ->
  [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
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
  [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

Expanded route:

* endpoint:
  [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
* connect:
  [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns)
* listen:
  [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns)
* close:
  [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
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
* session:
  [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)

That means the current `std net` front door should be read as:

```text
endpoint/timing
-> connect/listen/close
-> result observe
-> result bridge
-> task policy
-> task batch
-> task windowed
-> task bridge
-> control session
-> session
```

Grouped rule:

```text
profile core
-> control edge
-> result spine
-> task spine
-> session
```

Project-facing CLI hint:

* `nuis project-status <network-project>`
* `nuis project-doctor <network-project>`

Those now surface:

* `std_net_navigation`
  `profile_core -> control_edge -> result_spine -> task_spine -> session`
* `std_net_samples`
  the shortest checked-in recipe/demo companion route for each of those four groups

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
