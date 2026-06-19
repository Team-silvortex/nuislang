# `std` Net Layering Contract

This file captures the current contract for the first thin `std net` facade
over the `official.network` domain.

It sits below
[network-profile-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-profile-contract.md):
that file describes the domain-owned truth, while this file describes how the
checked-in `std` layer is expected to read and compose that truth.

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
* `std net` is the first thin readable facade over that truth
* the recipe surfaces are intentionally narrow and do not yet claim a finished
  socket API
* repository-stage validation still runs through companion project demos rather
  than a frozen standalone stdlib test harness

## Source And Demo Router

Use the dedicated router for the full grouped source list and the grouped
companion validation route:

* [stdlib/std/network/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/network/README.md)

That router now owns:

* grouped source modules
* grouped companion demos
* the shortest local reading route

This contract file should stay shorter and more stable than the source router.

## Current Role Split

At the current repository stage, the important split is the grouped one:

* profile core
  exposes the first typed summary surface over domain-owned profile truth
* transport + syscall + socket edges
  turn profile slots and owned handles into thin readable network facades
* control + protocol + HTTP edges
  stage progressively richer protocol summaries without claiming a final socket API
* result + task spines
  bridge `NetworkResult<T>` into reusable task-oriented shapes
* session
  composes control, transport, protocol, result, and task summaries into wider
  reusable routes

For the current HTTP/session frontdoor, the narrower split is:

* `net_http_client_session_recipe`
  owns the smallest host-transport lifecycle summary:
  open/send/status/body/close plus request/response byte estimates
* `net_httpish_header_session_recipe`
  owns packet-plus-session aggregation:
  request-header / response-header / body / retry staging on top of the
  smaller client-session shape
* `net_http_client_lane_recipe`
  owns the widest checked-in client lane summary:
  authority/path plus request/response header/body byte grouping on top of the
  lower transport shape
* `net_http_service_lane_recipe`
  is the current listener-side mirror:
  request/response header/body byte grouping on top of the lower service
  packet/session shape

The practical rule is:

* session is the transport lifecycle floor
* header-session is the current packet/session bridge
* lane is the current highest readable frontdoor layer, where client keeps the
  extra `authority/path` split and service keeps the listener-oriented mirror
* client/service lane summaries should prefer the shared names
  `request_header_bytes`, `request_body_bytes`, `request_bytes`,
  `response_header_bytes`, `response_body_bytes`, and `response_bytes`

## Current Reading Rule

If you want the shortest pass:

1. start with [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
2. follow the grouped lane in
   [stdlib/std/network/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/network/README.md)
3. validate with the matching `examples/projects/domains/*_demo`

## Current Non-Goals

The current `std net` layer does not yet claim:

* a final socket ownership API
* a frozen HTTP client abstraction
* a finished protocol builder taxonomy
* a stable file layout for every network recipe source

This is why the router and layering contract matter: they give the network
surface a clean reading front door now, while still leaving room for later
filesystem or API reshapes.
