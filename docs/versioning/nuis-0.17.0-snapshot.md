# `nuis` 0.17.0 Snapshot

This file is the history anchor for the `0.17.*` line.

It follows the `0.16.*` maturity-line cleanup and marks the point where the
next question becomes more ambitious:

how much of the already validated compile surface can now be turned into a
clearer end-to-end compiler/runtime bridge?

It is not yet a “finished language” milestone.

It is a completion-and-integration milestone.

## What `0.17.0` Means Here

`0.17.0` is the point where the repository should push harder on three linked
goals:

* generic completion
* lowering completion
* network/runtime bridge completion

The mainline intent is:

* take `0.16.*` compile truths that are already test-backed
* reduce the remaining “works here but not there” gaps
* make the front half and back half of the compiler feel more like one system
* give `std net` / syscall / async-task / memory routes a stronger path toward
  real runtime-oriented examples

## High-Signal Target Surface

The highest-signal targets for `0.17.0` are:

* generic surfaces should keep becoming less exception-shaped:
  helper chains, explicit args, inference, control flow, lambda lifting, and
  project compile routes should align more often and fail more predictably when
  they do not align
* lowering should keep catching up with validated frontend surfaces:
  if a route is already believable at frontend/NIR level, the next goal is to
  make it much less likely to stall later in lowering or verifier stages
* async/task and memory/session routes should become stronger integration
  ground:
  not just isolated recipes, but clearer building blocks for richer project
  examples
* `std net` should keep moving from compile ladders toward more coherent
  end-to-end runtime-facing structure:
  syscall edges, transport/session bridges, and http-oriented examples should
  stand on the same async/task/memory story instead of feeling like separate
  islands

## Current Generic/Project Shape Win

One concrete `0.17.0` mainline gain is that higher-order generic truth now has
a clearer project-facing forwarding route instead of living only in
frontend-local probes.

The current rule of thumb is:

* explicit generic helper calls should keep their specialization truth even
  when wrapped by higher-order templates
* callable parameters should be able to forward through nested helper layers
  instead of only surviving one direct call site
* project anchors should prove this with ordinary `relay -> chain -> apply`
  structure, not only toy one-function snippets

This is now visible in the current checked-in state project anchors:

* [generic_payload_alias_method_hof_demo](../../examples/projects/state/generic_payload_alias_method_hof_demo)
* [generic_callable_forwarding_hof_demo](../../examples/projects/state/generic_callable_forwarding_hof_demo)
* [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)

That is still not a claim that callable values are a fully general first-class
surface.

It is a real integration step: `Fn1` / `Fn2` / `Fn3` forwarding through nested
higher-order helper chains is now part of the project-backed `0.17.0` generic
story.

## Current Lowering/Project Shape Win

One concrete `0.17.0` mainline gain is that several composed control-flow
lowering shapes now have state-project anchors instead of living only in
lowering-local snippet tests.

The current rule of thumb is:

* `flow` control should not only lower correctly in isolated loop tests
* post-body `break` / `continue` control should not only lower correctly in
  isolated post-flow probes
* a few ordinary checked-in state projects should prove the same loop-family
  truth through the real project pipeline

This is now visible in the current checked-in state project anchors:

* [flow_branching_while_demo](../../examples/projects/state/flow_branching_while_demo)
* [post_flow_branching_while_demo](../../examples/projects/state/post_flow_branching_while_demo)
* [post_flow_branching_continuing_while_demo](../../examples/projects/state/post_flow_branching_continuing_while_demo)
* [tail_recursive_branching_cross_carry_demo](../../examples/projects/state/tail_recursive_branching_cross_carry_demo)
* [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)

That is still not a claim that every loop/control form has equal project-level
coverage.

It is a real integration step: `flow_cond_chain`, `post_flow_cond_chain`, and
branching carry loop shapes now belong to the project-backed `0.17.0`
lowering story.

## Current `std net` Shape Win

One concrete `0.17.0` mainline gain is that the network-facing `std` recipes
have started converging on a repeatable workflow shape instead of drifting as
isolated demos.

The current rule of thumb is:

* explicit workflow helpers first:
  `open_*`, `accept_*`, `send_*`, `recv_*`, `close_*`
* then a plan layer:
  `build_*_plan(...)`
* then a packet layer:
  `stage_*_packet(...)`, `compute_packet_value(...)`
* then a wider summary layer:
  `compute_session_value(...)` when the recipe truly spans a session lifecycle
* then the normal capture/summarize pair:
  `capture_*_summary(...)`, `summarize_*_recipe(...)`

This is already visible in the current `std` surface:

* [net_http_client_session_recipe.ns](../../stdlib/std/net_http_client_session_recipe.ns)
* [net_http_client_session_async_loop_recipe.ns](../../stdlib/std/net_http_client_session_async_loop_recipe.ns)
* [net_http_service_lane_recipe.ns](../../stdlib/std/net_http_service_lane_recipe.ns)
* [net_httpish_client_session_packet_recipe.ns](../../stdlib/std/net_httpish_client_session_packet_recipe.ns)
* [net_httpish_service_session_packet_recipe.ns](../../stdlib/std/net_httpish_service_session_packet_recipe.ns)
* [net_httpish_header_session_recipe.ns](../../stdlib/std/net_httpish_header_session_recipe.ns)

That is still not a final public API story, but it is a real mainline
stabilization step: new network examples now have a clearer default shape to
copy, and the async/task/memory-aware recipe layer is starting to read like one
system.

## What `0.17.0` Should Not Overclaim

Even if `0.17.0` goes well, it should still avoid claiming more than is true:

* generic inference is not automatically a fully general HM-style system
* lowering is not “complete” just because more demos pass
* network compile truth is not the same thing as runtime portability truth
* a better `std net` shape is not the same thing as a final polished public API

## Best Current Reading Order

For the `0.17.0` line, the shortest route should be:

1. [README.md](../../README.md)
2. [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
3. [nuis-0.17.0-mainline-goals.md](nuis-0.17.0-mainline-goals.md)
4. [nuis-0.17.0-snapshot.md](nuis-0.17.0-snapshot.md)
5. [nuis-0.17.0-release-checklist.md](nuis-0.17.0-release-checklist.md)
6. [nuis-0.16.0-compile-workflow.md](nuis-0.16.0-compile-workflow.md)
7. [nuis-0.16.0-generic-surface-audit.md](nuis-0.16.0-generic-surface-audit.md)

## Rule Of Thumb

If `0.16.*` was about making the compile story easier to teach, `0.17.0`
should be about making more of that story line up across frontend, lowering,
projects, and runtime-oriented examples without hand-waving.
