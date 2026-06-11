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

## What `0.17.0` Should Not Overclaim

Even if `0.17.0` goes well, it should still avoid claiming more than is true:

* generic inference is not automatically a fully general HM-style system
* lowering is not “complete” just because more demos pass
* network compile truth is not the same thing as runtime portability truth
* a better `std net` shape is not the same thing as a final polished public API

## Best Current Reading Order

For the `0.17.0` line, the shortest route should be:

1. [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
2. [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
3. [nuis-0.17.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-mainline-goals.md)
4. [nuis-0.17.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-snapshot.md)
5. [nuis-0.17.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-release-checklist.md)
6. [nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md)
7. [nuis-0.16.0-generic-surface-audit.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-surface-audit.md)

## Rule Of Thumb

If `0.16.*` was about making the compile story easier to teach, `0.17.0`
should be about making more of that story line up across frontend, lowering,
projects, and runtime-oriented examples without hand-waving.
