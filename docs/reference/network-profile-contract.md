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
-> timeout / retry budget
-> stream window shape
```

Current narrow companion:

* [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo)
  now reads:
  `bind_core -> endpoint_kind -> timeout_budget -> retry_budget -> stream_window`

## Scope

This document is only about the current manifest-first profile surface:

* `support_surface`
* `support_profile_slots`
* `default_lanes`
* the smallest reading order those names imply

It is not yet a promise that the repository already has:

* a complete `network` frontend
* a complete `NetworkResult<T>` language surface
* real socket lowering or runtime execution

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
3. `network_profile_timeout_budget(...)`
4. `network_profile_retry_budget(...)`
5. `network_profile_stream_window(...)`

That order keeps the first checked-in `network` route aligned with the current
manifest contract instead of jumping straight to a large socket or HTTP facade.

## Related References

* [network-domain-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-domain-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
