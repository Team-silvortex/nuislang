# Network Runtime Host Validation

This file is the shortest trustworthy route for validating real `network`
syscall behavior on a host that is expected to allow loopback sockets.

Use it when the question is not "does it compile?" but "does the generated
binary actually open, bind, connect, send, recv, and close?".

## Why This Exists

The repository now has a clear split between:

* compile truth:
  `frontend -> lowering -> YIR -> LLVM/AOT -> native binary`
* runtime truth:
  whether the produced binary can actually use the host socket surface

The current checked-in probes intentionally separate those layers.

## Important Cache Rule

`nuis build` compile cache keys follow source/project inputs, not compiler
source files under `tools/nuisc` or `crates/yir-lower-llvm`.

That means:

* after changing runtime/lowering/compiler code, do **not** trust an old output
  directory
* either use a fresh output directory every run
* or make a tiny source change in the probe project before rebuilding

For host validation, prefer a fresh `target/nuis-host-validation/..._out*`
directory every time.

## Probe Order

Run the probes in this order.

### 1. Control Probe

Project:

* [network_host_control_runtime_demo](../../examples/projects/domains/network_host_control_runtime_demo)

Build:

```bash
cargo run -q -p nuis -- build \
  examples/projects/domains/network_host_control_runtime_demo \
  target/nuis-host-validation/network_host_control_runtime_demo_out
```

Run:

```bash
target/nuis-host-validation/network_host_control_runtime_demo_out/network_host_control_runtime_demo
```

Printed fields:

1. `local_port`
2. `remote_port`
3. `connect_timeout_ms`
4. `read_timeout_ms`
5. `write_timeout_ms`
6. `connect_probe`
7. `accept_probe`
8. `close_code`

Read it like this:

* `connect_probe = 1` means the internal loopback connect handshake succeeded
* `accept_probe = 1` means the internal loopback accept handshake succeeded
* `0` means the host runtime could not complete that syscall path

### 2. Open Surface Matrix

Project:

* [network_host_open_surface_runtime_demo](../../examples/projects/domains/network_host_open_surface_runtime_demo)

Build:

```bash
cargo run -q -p nuis -- build \
  examples/projects/domains/network_host_open_surface_runtime_demo \
  target/nuis-host-validation/network_host_open_surface_runtime_demo_out
```

Run:

```bash
target/nuis-host-validation/network_host_open_surface_runtime_demo_out/network_host_open_surface_runtime_demo
```

Printed fields:

1. `base_local_port`
2. `remote_port`
3. `read_timeout_ms`
4. `write_timeout_ms`
5. `udp_local_only_port`
6. `udp_connected_only_port`
7. `udp_full_port`
8. `tcp_listener_port`
9. `udp_local_only_handle`
10. `udp_connected_only_handle`
11. `udp_full_handle`
12. `tcp_listener_handle`
13. `udp_local_only_close`
14. `udp_connected_only_close`
15. `udp_full_close`
16. `tcp_listener_close`

Read it like this:

* non-zero handle means the public owned-handle surface acquired a real socket
* non-zero close means that owned handle could be released successfully
* all-zero handles means the public open surface is still blocked even if the
  project compiled

### 3. Handle Runtime Probes

Projects:

* [network_host_handle_runtime_demo](../../examples/projects/domains/network_host_handle_runtime_demo)
* [network_host_handle_transport_runtime_demo](../../examples/projects/domains/network_host_handle_transport_runtime_demo)
* [network_loopback_runtime_demo](../../examples/projects/domains/network_loopback_runtime_demo)

Use these only after the first two probes.

They answer:

* whether public TCP/UDP handle acquisition works
* whether owned `send/recv/close` works on those handles
* whether a same-process listener/client loopback can complete

## Current Known Result In This Environment

In the current desktop/container host used for repository development, the
latest truthful probe results are:

* control probe:
  `connect_probe = 0`, `accept_probe = 0`
* open surface matrix:
  every public handle field is `0`
* loopback runtime demo:
  listener/client handles remain `0`

That means the repo currently has:

* healthy compile truth
* healthy probe instrumentation
* but no confirmed positive socket runtime truth in this host

## What Counts As Success On A Socket-Enabled Host

We can claim meaningful runtime progress when the next host produces:

* `network_host_control_runtime_demo`
  - `connect_probe = 1`
  - `accept_probe = 1`
* `network_host_open_surface_runtime_demo`
  - at least one non-zero UDP handle
  - a non-zero TCP listener handle
* `network_loopback_runtime_demo`
  - non-zero `listener_handle`
  - non-zero `client_handle`
  - non-zero `server_task_completed`

After that, the next layer to verify is:

* [net_tcp_send_runtime_probe_demo](../../examples/projects/domains/net_tcp_send_runtime_probe_demo)
* [net_tcp_socket_runtime_probe_demo](../../examples/projects/domains/net_tcp_socket_runtime_probe_demo)
* [net_http_status_runtime_probe_demo](../../examples/projects/domains/net_http_status_runtime_probe_demo)
* [net_http_client_runtime_probe_demo](../../examples/projects/domains/net_http_client_runtime_probe_demo)

## Minimal Command Set

If you only want the shortest host check:

```bash
cargo run -q -p nuis -- build \
  examples/projects/domains/network_host_control_runtime_demo \
  target/nuis-host-validation/network_host_control_runtime_demo_out

target/nuis-host-validation/network_host_control_runtime_demo_out/network_host_control_runtime_demo

cargo run -q -p nuis -- build \
  examples/projects/domains/network_host_open_surface_runtime_demo \
  target/nuis-host-validation/network_host_open_surface_runtime_demo_out

target/nuis-host-validation/network_host_open_surface_runtime_demo_out/network_host_open_surface_runtime_demo
```

If both stay zero on the target host, do not spend time debugging higher-level
HTTP-ish demos yet. The runtime socket surface itself is still blocked.
