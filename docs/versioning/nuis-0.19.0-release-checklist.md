# `nuis` 0.19.0 Release Checklist

This file is the lightweight checklist for the `0.19.0` line.

It is intentionally operational.

The goal is to confirm that the repository really has one maintained current
mainline, not only one inherited `0.18` gate plus newer prose.

## Documentation

* [ ] confirm [nuis-0.19.0-snapshot.md](nuis-0.19.0-snapshot.md)
  still matches checked-in repository truth
* [ ] confirm [nuis-0.19.0-mainline-goals.md](nuis-0.19.0-mainline-goals.md)
  still describes the actual current push
* [ ] confirm [nuis-0.19.0-compile-workflow.md](nuis-0.19.0-compile-workflow.md)
  still matches the real CLI/compiler frontdoor story
* [ ] confirm [nuis-0.19.0-workflow-capability-matrix.md](nuis-0.19.0-workflow-capability-matrix.md)
  still matches the current workflow/frontdoor/mainline route combinations
* [ ] confirm [nuis-0.19.0-mainline-regression-matrix.md](nuis-0.19.0-mainline-regression-matrix.md)
  still matches the practical current gate
* [ ] confirm [nuis-0.19.0-frontend-capability-matrix.md](nuis-0.19.0-frontend-capability-matrix.md)
  still matches the current frontend combination truth
* [ ] confirm [nuis-0.19.0-project-capability-matrix.md](nuis-0.19.0-project-capability-matrix.md)
  still matches the current project/example anchor story
* [ ] confirm [nuis-0.19.0-address-pointer-mainline.md](nuis-0.19.0-address-pointer-mainline.md)
  still matches the honest current address/source-style story
* [ ] confirm [README.md](../../README.md),
  [docs/current-mainline-map.md](../../docs/current-mainline-map.md),
  and [docs/versioning/README.md](README.md)
  all point at the correct current anchors

## Toolchain And Validation

* [ ] `cargo fmt --all`
* [ ] wider repo-level package test pass still holds:
  `cargo test -q -p nuisc -p nuis`
* [ ] current fast mainline gate still passes:
  `bash scripts/check-0.19-mainline.sh`
* [ ] current heavier compiler gate still passes:
  `bash scripts/check-0.19-release.sh`
* [ ] project-backed anchors still pass:
  `cargo test -q -p nuisc --test state_compile`
  `cargo test -q -p nuisc --test task_compile`
  `cargo test -q -p nuisc --test memory_compile`
  `cargo test -q -p nuisc shader_nova_contracts`
  `cargo test -q -p nuisc --test network_compile`
* [ ] staged thread/lock frontdoor sample still passes:
  `nuis project-doctor examples/projects/task/task_thread_mutex_demo`
  `nuis check examples/projects/task/task_thread_mutex_demo`
  `nuis test examples/projects/task/task_thread_mutex_demo`
* [ ] wider integration complements still pass:
  `cargo test -q -p nuisc multidomain_async`
  `cargo test -q -p nuisc tests_async_runtime`
  `cargo test -q -p nuisc tests_async_network_runtime`

## `0.19.0` Version-Facing Surfaces To Reconfirm

* [ ] current mainline version is obvious from the entry docs
* [ ] current compile frontdoor still reads like one grouped route
* [ ] source-facing address syntax remains consistent across `.ns` examples and
  `std`
* [ ] implementation-facing docs still explain lowered builtin truth without
  pretending those names are the preferred source spelling
* [ ] project-backed anchors remain the honest proof set for the current line
* [ ] compatibility wrappers for old `0.18` gate names still behave correctly

## Version Number Decision

Before any real release cut, decide explicitly:

* [ ] this is a repository/toolchain-line snapshot only
* [ ] or this is a coordinated manifest/version bump line

## Rule Of Thumb

If `0.19.0` claims the current mainline is more internalized, a teammate
should be able to find the right doc, command, and gate without reconstructing
history from scattered files.
