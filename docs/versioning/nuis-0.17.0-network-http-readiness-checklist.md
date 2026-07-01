# `nuis` 0.17.0 Network/HTTP Readiness Checklist

This file is the release-facing checklist for the `network/http` story inside
the `0.17.0` line.

It is intentionally narrower than the broad release checklist.

Use it when the question is:

`what can the current repository honestly claim about network/http-oriented compile and lowering readiness, and what should still be described more carefully?`

Read it together with:

* [nuis-0.17.0-lowering-capability-map.md](nuis-0.17.0-lowering-capability-map.md)
* [nuis-0.17.0-mainline-regression-matrix.md](nuis-0.17.0-mainline-regression-matrix.md)
* [../reference/std-net-layering-contract.md](../../docs/reference/std-net-layering-contract.md)
* [../reference/network-domain-contract.md](../../docs/reference/network-domain-contract.md)

## Scope

This checklist is about the current repository/toolchain story for:

* async network observer lowering
* owned session-style network steps
* budgeted polling loops
* HTTP-like request-session naming
* project-form compile anchors for those routes

It is not a claim that `nuis` already ships a final high-level HTTP client
product surface.

## Documentation Truth

* [ ] confirm
  [nuis-0.17.0-lowering-capability-map.md](nuis-0.17.0-lowering-capability-map.md)
  still matches the checked-in lowering/test reality
* [ ] confirm
  [std-net-layering-contract.md](../../docs/reference/std-net-layering-contract.md)
  still describes the current `std net` reading order honestly
* [ ] confirm
  [examples/projects/domains/README.md](../../examples/projects/domains/README.md)
  still points at the right HTTP/session bridge examples
* [ ] confirm
  [current-mainline-map.md](../../docs/current-mainline-map.md)
  still exposes the right `0.17.0` network/http anchors

## Test-Backed Compiler Truth

* [ ] focused lowering family still passes:
  `cargo test -q -p nuisc tests_async_network_runtime`
* [ ] broader async lowering family still passes:
  `cargo test -q -p nuisc lowering::tests_async_runtime`
* [ ] project-form network compile proof still passes:
  `cargo test -q -p nuisc --test network_compile`
* [ ] helper-aware project integration still passes:
  `cargo test -q -p nuisc multidomain_async`

## What We Can Honestly Say

These should all be true before `0.17.0` says the network/http compile story
is ready enough to teach:

* [ ] async network observer/value probes survive into executable async loop
  lowering
* [ ] host-owned `open/send/recv/close` session steps survive inside async step
  helpers
* [ ] retry-budget and timeout-budget polling loops survive as checked-in async
  lowering shapes
* [ ] request-shaped naming such as
  `request_step / request_progress / request_attempts / response_bytes`
  still maps onto that same lowering-backed story
* [ ] the same story is visible both in lowering tests and in at least one
  project-form example/compile proof pair

## What We Should Still Say Carefully

These are the statements that should remain careful, narrow, or explicitly
qualified:

* [ ] do not describe the current repository as having a final first-class HTTP
  client API
* [ ] do not imply that every boolean observer source shape is equally mature
  in every conditional-carry/control route
* [ ] do not blur “HTTP-like request-session lowering story” into
  “fully general runtime protocol stack”
* [ ] do not claim that example naming stability already means public API
  stability

## Release-Facing Rule

Before saying the `0.17.0` line has a real network/http story, the shortest
honest summary should still be:

`nuis can now compile a test-backed async network/session/request-shaped lowering story through one coherent path, but it is still better described as a session-and-budget compile spine than as a finished HTTP client product surface`
