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
