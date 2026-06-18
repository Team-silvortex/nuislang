# `nuis` `alpha-0.0.1` Preflight Report

This file is the final preflight summary for the commit line immediately before
`alpha-0.0.1`.

It is not a broad roadmap.

It answers one narrower question:

`what is already coherent enough for alpha, what can still safely enter this commit, and what should wait until after alpha?`

Read this together with:

* [nuis-alpha-0.0.1-closeout-board.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.0.1-closeout-board.md)
* [nuis-alpha-0.0.1-closeout-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.0.1-closeout-checklist.md)
* [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)

## Short Verdict

`alpha-0.0.1` is closeable if this next commit behaves like a closeout commit,
not another expansion commit.

The repository already has:

* one readable toolchain center
* one much cleaner example-tree hierarchy
* one defendable semantic spine for the currently claimed subset

The remaining pressure is mostly:

* wording drift
* over-promotion risk
* a few still-dense companion subtrees

That is alpha-preflight pressure, not pre-alpha architectural confusion.

## Ready Enough For `alpha`

These look coherent enough to stand on for `alpha-0.0.1` if no new drift is
introduced.

### 1. Toolchain Center

Current read:

* [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
* [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [nuis-alpha-0.0.1-closeout-board.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.0.1-closeout-board.md)

Why it is ready enough:

* the repo now has one explicit frontdoor workflow
* the mainline router and alpha closeout board are linked from the top
* frontend truth versus compile-closure truth is described as a boundary rather
  than hidden as a surprise

### 2. Example Tree Hierarchy

Current read:

* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* [examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)

Why it is ready enough:

* frontdoor routes are much narrower
* companion-only, probe-only, validation-only, mirror-only, and
  exploration-only language is now explicit in the most drift-prone subtrees
* the example tree now reads like a guided system instead of a flat warehouse

### 3. Semantic Spine

Current read:

* [nuis-0.20.0-generic-validation-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-generic-validation-regression-matrix.md)
* [nuis-0.20.0-branch-runtime-lowering-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-branch-runtime-lowering-matrix.md)
* [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)
* [tests_lambda_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_lambda_higher_order.rs)

Why it is ready enough:

* generics, control flow, higher-order routes, `?`, `await`, and task-thread
  staging now read as one defended subset
* the repo no longer depends only on narrative confidence for those routes
* the currently claimed subset is much easier to point at concretely

## Safe To Still Include In The Next Commit

These are still alpha-safe if kept small and closeout-oriented.

### 1. Wording And Router Cleanup

Safe examples:

* remove small remaining wording drift where a file still sounds more current
  than the mainline map
* tighten repeated phrases like "current route" when the local README can
  simply defer to the router
* align a few remaining subtree READMEs with the already-established
  frontdoor/companion split

Why safe:

* this reduces ambiguity without changing capability claims

### 2. Explicit Boundary Marking

Safe examples:

* mark one or two remaining dense routes as companion-only or probe-only
* sharpen any still-soft local wording around mirror or validation material

Why safe:

* this narrows the claimed surface instead of inflating it

### 3. Tiny Historical Demotion

Safe examples:

* demote an obviously older "current-sounding" doc to historical wording
* add one missing link from a local README back to the mainline router or
  preflight/closeout board

Why safe:

* this strengthens alpha coherence without widening the supported subset

## Should Not Enter This Commit Unless Absolutely Necessary

These are the most likely ways to accidentally turn the alpha commit back into
an expansion commit.

### 1. New Semantic Surface

Do not add:

* new generic corners
* new async/runtime contracts
* new pointer/ABI behavior
* new domain capability semantics

Why:

* even if individually small, they change the truth we are trying to freeze

### 2. Big Example-Tree Moves

Do not add:

* broad file moves
* large-scale archive shuffles
* path renaming cascades across many example references

Why:

* the semantic split is now documented; the physical moves can wait until after
  alpha

### 3. New Ambition Docs

Do not add:

* broad new future sketches
* new architecture manifestos
* new claims about self-hosting readiness

Why:

* alpha should freeze the current route, not reopen the horizon

## Best Remaining Gaps To Defer

These should now be treated as post-alpha pressure, not alpha blockers.

* dense `state` long-tail example compression
* stronger local grouping for `examples/ns/demos`
* physical split or relocation of network validation probes
* physical split or relocation of filesystem `path_*` micro-probes
* deeper `std` frontdoor normalization beyond the already current ladders
* fully exhaustive trait/generic or async/runtime edge coverage

## Recommended Shape Of The Next Commit

Best form:

* closeout-only
* no new capability claims
* no new major semantic surface
* only routing, wording, boundary marking, and maybe one or two tiny historical
  demotions

Short rule:

`the alpha commit should freeze the route, not widen the map`

## Release Gate

`alpha-0.0.1` should proceed when these statements remain true together:

* the mainline router still reads as one coherent route
* the example tree still has obvious frontdoors and explicit non-frontdoor
  categories
* no subtree README is pretending a probe, mirror, or experiment is a default
  route
* no last-minute feature addition changed the semantic subset being claimed

If those stay true, the next commit can honestly become `alpha-0.0.1`.
