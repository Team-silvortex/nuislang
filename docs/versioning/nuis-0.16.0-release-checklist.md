# `nuis` 0.16.0 Release Checklist

This file is the lightweight checklist for the `0.16.0` line.

It is intentionally operational. The goal is to confirm that the stronger
compile workflow and maturity claims still match the checked-in repository.

## Scope

Use this checklist when you want to sanity-check whether the repository still
looks like the `0.16.0` line it claims to describe.

## Documentation

* [ ] confirm [nuis-0.16.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-snapshot.md)
  still matches the checked-in mainline
* [ ] confirm [nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md)
  still describes the canonical route
* [ ] confirm [nuis-0.16.0-binary-compile-maturity.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-binary-compile-maturity.md)
  still matches actual compiler/runtime truth
* [ ] confirm [nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md)
  still matches actual frontend validation coverage
* [ ] confirm [nuis-0.16.0-generic-constraint-gaps.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-gaps.md)
  still reflects the remaining follow-up list honestly
* [ ] confirm [nuis-0.16.0-generic-surface-audit.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-surface-audit.md)
  still matches the test-backed generic crossover matrix
* [ ] confirm [README.md](/Users/Shared/chroot/dev/nuislang/README.md),
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md),
  and [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
  still point at the right current anchors

## Toolchain And Validation

* [ ] `cargo fmt --all`
* [ ] `cargo test -q -p nuisc -p nuis`
* [ ] focused frontend crossover probes still pass:
  `tests_generics`, `tests_higher_order`, and the closest generic diagnostic probes
* [ ] real project compile harnesses still pass:
  especially `network_compile` and `task_compile`
* [ ] spot-check `nuis help`
* [ ] spot-check `nuis project-status <project-dir>`
* [ ] spot-check `nuis project-doctor <project-dir>`
* [ ] spot-check one project `check`
* [ ] spot-check one project `build`
* [ ] spot-check one `release-check`

## `0.16.0` Version-Facing Surfaces To Reconfirm

* [ ] canonical route:
  `project-doctor -> check -> test -> build -> release-check`
* [ ] validation spine:
  frontend probes -> real project compile harnesses -> diagnostic guardrails -> CLI workflow
* [ ] manifest verification:
  `verify-build-manifest` and build output contract still line up
* [ ] generic struct route:
  explicit literals, payload constructors, shorthand destructuring, shorthand
  `match`, alias-aware patterns
* [ ] payload-constructor matrix:
  direct explicit / expected / inferred routes and transparent alias explicit /
  inferred routes still match the checked-in tests and docs
* [ ] alias constructor diagnostics:
  non-transparent or insufficiently inferred alias constructor routes still
  fail directly and honestly when they cannot be resolved
* [ ] generic constraint / method-bound diagnostics:
  alias-chain and control-flow-local binding routes still behave clearly
* [ ] generic constraint coverage map:
  lambda/higher-order/call-inferred/destructure routes still match the checked-in tests
* [ ] generic surface audit matrix:
  constructor / literal / specialization / pattern / destructure / method-bound crossover rows
  still match the checked-in tests
* [ ] std-net-shaped generic async/task/session routes:
  facade-shaped and demo-shaped summary/session flows still match the checked-in tests
* [ ] real network project compile entrypoints:
  `net_http_client_exchange_recipe_demo` and `net_session_recipe_demo`
  still compile through `compile_project(...)`
* [ ] loop-family lowering matrix:
  counted / carry / flow / post-flow subsets still match the checked-in docs
* [ ] nested loop control lowering:
  `if -> break` and `match` / branch-local `continue` still lower into checked-in
  `and` / `or` loop predicates
* [ ] async ownership:
  `join`, `join_result`, `cancel`, and `timeout` stay aligned with verifier
  truth
* [ ] task-result lifecycle facts:
  completed-path `task_value(...)` rule still matches implementation truth
* [ ] task + memory packet/session groundwork:
  task compile examples still hold
* [ ] network compile ladders:
  network compile examples still hold
* [ ] network runtime notes:
  compile truth vs runtime truth language is still honest
* [ ] diagnostic guardrails:
  network-style sync/async misuse routes still fail with stable, specific messages

## Version Number Decision

`0.16.0` documentation does not require every crate/package manifest to
immediately move in lockstep.

Before any real release cut, decide explicitly:

* [ ] this is a repository/toolchain-line snapshot only
* [ ] or this is a coordinated manifest/version bump line

Do not leave that distinction implicit.

## Rule Of Thumb

If the implementation is greener than the docs:

* refresh the docs.

If the docs are stronger than the implementation:

* narrow the docs,
* or fix the implementation,
* but do not ship the mismatch on purpose.
