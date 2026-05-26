# Network Domain Contract

This document captures the intended current contract for `official.network` as
the fifth `nustar` family.

It is intentionally a bootstrap contract, not a claim that the repository
already has a finished network runtime.

## Current Role

`network` should be treated as a distinct system domain rather than a long-term
extension of `cpu`.

The split is:

* `cpu`
  owns host execution, task lifecycle, host FFI, and the thinnest bootstrap
  primitives
* `network`
  owns connection, stream, timeout, retry, and transport-facing observation
  semantics

That means the safe current mental model is:

```text
cpu = host primitive
network = system domain
```

## Why It Is Not Just `cpu`

The repository already treats domains as separate once they grow their own:

* lane defaults
* bridge defaults
* clock domain
* result observation surface
* async summary surface

`network` is expected to need all of those.

So the current contract direction is:

* do not let `cpu` become the long-term owner of connect/send/recv/retry logic
* let `cpu` continue to expose the thinnest host bootstrap edges
* let `network` become the domain that owns transport semantics

## Current Registered Skeleton

The registered bootstrap skeleton for `official.network` is:

* package:
  [network.toml](/Users/Shared/chroot/dev/nuislang/nustar-packages/network.toml)
* index entry:
  [index.toml](/Users/Shared/chroot/dev/nuislang/nustar-packages/index.toml)

Current domain identity:

* `package_id = "official.network"`
* `domain_family = "network"`
* `frontend = "nustar-network"`
* `entry_crate = "crates/yir-domain-network"`

This is currently a manifest-first contract skeleton.
It is not yet a promise that the crate or lowering implementation already
exists.

## Current Surface Shape

The current intended bootstrap surfaces are:

* AST:
  `network.mod-ast.v1`, `network.endpoint-ast.v1`
* NIR:
  `nir.network.surface.v1`, `nir.stream.surface.v1`
* YIR lowering:
  `yir.network.lowering.v1`, `yir.socket-lowering.v1`
* partial verify:
  `verify.network.endpoint-shape.v1`, `verify.network.flow-control.v1`

These names intentionally bias toward:

* endpoint shape
* stream transport
* flow control

rather than trying to freeze HTTP, RPC, or application protocol layers too
early.

## Lane / Clock Direction

Current lane direction:

* control:
  `network.bind_core`, `network.endpoint`, `network.listen`, `network.connect`,
  `network.accept`, `network.poll`, `network.close`, `network.retry`
* transport:
  `network.send = tx`
  `network.recv = rx`

Current clock direction:

* `clock_domain_id = "network.clock.io.v1"`
* `clock_kind = "io-monotonic"`
* `clock_epoch_kind = "io-epoch"`
* `clock_resolution = "network-poll-step"`
* `clock_bridge_default = "global->io:bridge"`

So the intended short reading rule is:

```text
connect / accept / retry / close = control
send = tx
recv = rx
timeouts and polling = io clock
```

## Result / Summary Direction

The current manifest does not yet define a dedicated language-level
`NetworkResult<T>` or `NetworkSummary<T>` surface.

But it is being registered with the expectation that `network` will eventually
want its own:

* result-entry / ready-probe / payload-value observation
* async policy summary
* async batch summary
* async windowed summary

In other words, it is expected to fit the same scheduler registration stack as
the current `cpu / data / shader / kernel` families:

```text
placement
-> timing
-> result observation
-> async summary observation
-> observer classification
```

There is now also a minimal `YIR`-level control-result skeleton for the host
bridge side:

* `network.observe ... config_ready`
* `network.connect ... -> network.is_connect_ready / network.value`
* `network.accept ... -> network.is_accept_ready / network.value`
* `network.close ... -> network.is_closed / network.value`

That means the repository still does not claim a finished socket runtime, but it
does now have a stable place in `YIR` where `connect / accept / close` syscalls
can land.

The current loader-facing symbol reservation is also explicit now. The minimal
host bridge names are:

* `host_network_connect_probe`
* `host_network_open_tcp_stream`
* `host_network_open_udp_datagram`
* `host_network_accept_probe`
* `host_network_close`
* `host_network_close_owned`
* `host_network_send_owned`
* `host_network_recv_owned`
* `host_network_send_probe`
* `host_network_recv_probe`

`nuis loader-contract official.network` now surfaces those names so the bridge
contract is visible before a full runtime implementation exists.

## Current CPU Host-Bridge Interpretation

The current `connect / accept / close` bridge is still routed through the
existing CPU host-FFI path rather than a dedicated `network.syscall_*` opcode.

The current honest reading is:

```text
source extern "c" fn host_network_* ...
-> YIR cpu.extern_call_i64 ... c host_network_* ...
-> loader-contract reserved host_symbol=network.*:host_network_* ...
-> runtime/AOT stub symbol
```

The narrow checked-in sample for this is:

* [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo)

If you inspect its `YIR`, you will currently see `cpu.extern_call_i64` nodes
such as:

* `cpu.extern_call_i64 ... c host_network_connect_probe ...`
* `cpu.extern_call_i64 ... c host_network_accept_probe ...`
* `cpu.extern_call_i64 ... c host_network_close ...`

That means the repository now has a stable syscall-facing interpretation path,
even though the `network` family does not yet claim a full socket runtime or a
dedicated `network`-native host-call instruction surface.

At the current binary/AOT layer, that bridge has now moved one step closer to
real runtime behavior:

* `host_network_connect_probe`
  now first attempts a loopback `socket/bind/listen/connect/accept/close`
  handshake inside the generated host shim
* `host_network_open_tcp_stream`
  now reserves a binary-owned TCP socket-handle acquisition path
* `host_network_open_udp_datagram`
  now reserves a binary-owned UDP datagram socket-handle acquisition path
* `host_network_accept_probe`
  now first attempts a local listener/client handshake before falling back
* `host_network_close_owned`
  now closes only binary-owned network handles recorded by the generated host shim
* `host_network_send_owned`
  now attempts `send(..., MSG_DONTWAIT)` against a binary-owned network handle
* `host_network_recv_owned`
  now attempts `recv(..., MSG_DONTWAIT)` against a binary-owned network handle
* `host_network_send_probe`
  now first attempts a local `socketpair + send`
* `host_network_recv_probe`
  now first attempts a local `socketpair + recv`

The current contract is still intentionally conservative:

* those probes keep the same `i64`-shaped host symbol surface
* successful syscall attempts do not yet expose stable socket handles upward
* `host_network_close` remains conservative and now delegates only to owned-handle
  close when the incoming value is known to the binary-owned network handle table

The first narrow checked-in transport sample for that bridge is:

* [network_host_handle_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_demo)
* [network_host_handle_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_transport_runtime_demo)
* [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo)

The handle-facing sample currently reads:

* `remote_port / connect_timeout`
  -> `host_network_open_tcp_stream`
* `local_port / remote_port`
  -> `host_network_open_udp_datagram`
* `tcp_handle / udp_handle`
  -> `host_network_close_owned`

The handle-transport sample currently reads:

* `local_port / remote_port`
  -> `host_network_open_udp_datagram`
* `handle / stream_window / send_window`
  -> `host_network_send_owned`
* `handle / stream_window / recv_window`
  -> `host_network_recv_owned`
* `handle`
  -> `host_network_close_owned`

The transport probe sample currently reads:

* `stream_window / send_window / remote_port`
  -> `host_network_send_probe`
* `stream_window / recv_window / local_port`
  -> `host_network_recv_probe`

The first narrow result-facing companion for those probes is:

* [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo)

That sample proves the current bridge can already be read as:

* `host_network_send_probe(...) -> network_result(...) -> network_send_ready / network_value`
* `host_network_recv_probe(...) -> network_result(...) -> network_recv_ready / network_value`

At the scheduler-contract layer, those transport-facing probes now also line up
with stable observer-role variants:

* `config_ready=config-ready-observer`
* `send_ready=send-ready-observer`
* `recv_ready=recv-ready-observer`

The next narrow orchestration companion is:

* [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo)

It shows that those same transport-facing probes can already flow into:

* `spawn(...)`
* `timeout(...)`
* `join_result(...)`
* shared task-shaped policy selection

The next narrow split companion is:

* [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo)

It keeps the same transport-facing probes, but makes the split explicit:

* `send_ready branch`
* `recv_ready branch`
* `fallback config_ready branch`
* shared task-shaped final policy selection

At the scheduler-contract layer, this split now also lines up with stable
branch labels:

* `send=send-branch`
* `recv=recv-branch`
* `fallback=fallback-branch`

And the corresponding transport-side split summaries now line up with:

* `transport_split=transport-split-summary`
* `transport_windowed_split=transport-windowed-split-summary`
* `transport_session_bridge_split=transport-session-bridge-split-summary`

`nuis scheduler-view <network project>` now also surfaces the shortest checked-in
sample hint for those classes through `scheduler_summary_samples`.

The next narrow batch-split companion is:

* [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo)

It keeps the same three branches, but makes the fan-in shape more explicit as:

* `send branch summary`
* `recv branch summary`
* `fallback branch summary`
* merged batch value

The next narrow windowed-split companion is:

* [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo)

It keeps the same three branches, but lets preview/final preserve branch
identity as:

* `send branch preview/final`
* `recv branch preview/final`
* `fallback branch preview/final`
* merged preview/final values

The next narrow fan-in companion is:

* [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo)

It keeps the same narrow transport-facing inputs, but now collects them as:

* `send result`
* `recv result`
* `fallback result`
* shared task-shaped batch summary

The next narrow windowed companion is:

* [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo)

It keeps the same transport-facing inputs and shared helper layer, but now
pushes them one step wider as:

* `transport batch summary`
* `preview summary`
* `final summary`

The next narrow bridge companion is:

* [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)

It keeps the same transport-facing probes and shared helper layer, but now
lets the transport windowed summary meet the existing session/task side as:

* `transport windowed summary`
* `session seed`
* `session summary bridge`

The next narrow bridge-split companion is:

* [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo)

It keeps the same transport-facing probes, but now lets branch identity reach
the session side as:

* `send branch session bridge`
* `recv branch session bridge`
* `fallback branch session bridge`
* merged session bridge value

That session-facing split is the current narrowest checked-in sample for the
`transport_session_bridge_split` summary class.

## Bootstrap Scope

The current bootstrap scope is intentionally narrow.

It is meant to justify these first-class transport-facing concepts:

* endpoint
* listen
* connect
* accept
* send
* recv
* close
* poll
* retry

It is not yet trying to freeze:

* HTTP client/server
* DNS abstraction
* TLS policy
* RPC framing
* websocket semantics

Those can grow later on top of the domain once the transport contract is real.

## Current Guidance

If you are deciding whether a new capability belongs in `cpu` or `network`,
use this rule:

* if it is a thin host primitive that could reasonably stay as a bootstrap FFI
  hook, it can remain in `cpu`
* if it owns transport lifecycle, timeout, retry, stream windowing, or
  cross-lane async observation, it should grow under `network`

## Related References

* [network-profile-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-profile-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* [std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
