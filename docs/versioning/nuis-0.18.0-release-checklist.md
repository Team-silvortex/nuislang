# `nuis` 0.18.0 Release Checklist

This file is the lightweight checklist for the `0.18.0` line.

It is intentionally operational.

The goal is to confirm that the repository really has one clearer mainline,
not just more green tests in isolated areas.

## Scope

Use this checklist when you want to sanity-check whether `0.18.0` still looks
like the mainline-closure line it claims to be.

## Documentation

* [ ] confirm [nuis-0.18.0-snapshot.md](nuis-0.18.0-snapshot.md)
  still matches the checked-in repository truth
* [ ] confirm [nuis-0.18.0-mainline-goals.md](nuis-0.18.0-mainline-goals.md)
  still describes the actual current push
* [ ] confirm [nuis-0.18.0-compile-workflow.md](nuis-0.18.0-compile-workflow.md)
  still matches the real frontend/project/lowering story and the current CLI
  frontdoor family
* [ ] confirm [nuis-0.18.0-mainline-regression-matrix.md](nuis-0.18.0-mainline-regression-matrix.md)
  still matches the practical release gate
* [ ] confirm [nuis-0.18.0-address-pointer-mainline.md](nuis-0.18.0-address-pointer-mainline.md)
  still matches the honest current pointer/address story
* [ ] confirm [README.md](../../README.md),
  [docs/current-mainline-map.md](../../docs/current-mainline-map.md),
  and [docs/versioning/README.md](README.md)
  all point at the correct current anchors

## Toolchain And Validation

* [ ] `cargo fmt --all`
* [ ] wider repo-level package test pass still holds:
  `cargo test -q -p nuisc -p nuis`
* [ ] fast `0.18.0` mainline gate still passes:
  `scripts/check-0.18-mainline.sh`
* [ ] heavier `0.18.0` compiler release gate still passes:
  `scripts/check-0.18-release.sh`
* [ ] project-backed anchors still pass:
  `cargo test -q -p nuisc --test state_compile`
  `cargo test -q -p nuisc --test task_compile`
  `cargo test -q -p nuisc --test memory_compile`
  `cargo test -q -p nuisc shader_nova_contracts`
  `cargo test -q -p nuisc --test network_compile`
* [ ] helper-aware multidomain probes still pass:
  `cargo test -q -p nuisc multidomain_async`
* [ ] async lowering probes still pass:
  `cargo test -q -p nuisc tests_async_runtime`
  `cargo test -q -p nuisc tests_async_network_runtime`
* [ ] spot-check `nuis help`
* [ ] spot-check `nuis status`
* [ ] spot-check `nuis workflow <input.ns|project-dir|nuis.toml>`
* [ ] spot-check `nuis project-doctor <project-dir>`
* [ ] spot-check `nuis project-status <project-dir>`
* [ ] spot-check `nuis scheduler-view <input.ns|project-dir|nuis.toml>`
* [ ] spot-check `nuis check <project-dir|nuis.toml>`
* [ ] spot-check `nuis build <project-dir|nuis.toml> <output-dir>`
* [ ] spot-check `nuis release-check <project-dir|nuis.toml> <output-dir>`

## `0.18.0` Version-Facing Surfaces To Reconfirm

* [ ] control-flow mainline:
  `if`, `match`, and `while` still read like one compiler story instead of
  separate exception piles
* [ ] lowering honesty:
  docs still name the supported loop/control families directly instead of
  implying general loop completeness
* [ ] generic diagnostics:
  alias-aware, lambda-aware, destructure-aware, and control-flow-local bound
  errors still fail specifically and honestly
* [ ] async/task composition:
  result/policy/fallback/batch/windowed routes still compose through the real
  project pipeline
* [ ] address/pointer closure:
  internal `ref` truth remains stable while host-boundary pointer ABI remains
  intentionally narrow
* [ ] shader/helper-mediated closure:
  project-backed shader routes still stand as real compile truth
* [ ] network/http/session closure:
  compile routes still survive helper-heavy project lowering without being
  overstated as full runtime portability
* [ ] compile truth vs runtime truth:
  docs still distinguish them clearly
* [ ] CLI frontdoor family:
  `status/help/workflow/project-doctor/project-status/scheduler-view` still
  expose one grouped frontdoor summary instead of drifting into separate
  command-local narratives
* [ ] repo-wide stdlib smoke:
  `cargo test -q -p nuis stdlib_source_modules -- --nocapture` still passes,
  especially the helper-heavy network/session recipes
* [ ] wider package sweep:
  `cargo test -q -p nuisc -p nuis` still agrees with the narrower
  compiler-facing gate instead of silently drifting away

## Version Number Decision

Before any real release cut, decide explicitly:

* [ ] this is a repository/toolchain-line snapshot only
* [ ] or this is a coordinated manifest/version bump line

Do not leave that distinction implicit.

## Rule Of Thumb

If `0.18.0` claims one clearer mainline, a teammate should be able to point to
small, named project and test anchors for control flow, generics, async/task,
memory/address, shader, and network without hand-waving.
