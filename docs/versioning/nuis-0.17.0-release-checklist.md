# `nuis` 0.17.0 Release Checklist

This file is the lightweight checklist for the `0.17.0` line.

It is intentionally operational. The goal is to confirm that the repository is
actually becoming more integrated, not just acquiring more surface area.

## Scope

Use this checklist when you want to sanity-check whether `0.17.0` still looks
like the integration/completion line it claims to be.

## Documentation

* [ ] confirm [nuis-0.17.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-snapshot.md)
  still matches the checked-in mainline
* [ ] confirm [nuis-0.17.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-mainline-goals.md)
  still describes the actual current push
* [ ] confirm [nuis-0.17.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-release-checklist.md)
  still reflects real release gates instead of wishful thinking
* [ ] confirm [README.md](/Users/Shared/chroot/dev/nuislang/README.md),
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md),
  and [docs/versioning/README.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/README.md)
  still point at the right current anchors

## Toolchain And Validation

* [ ] `cargo fmt --all`
* [ ] `cargo test -q -p nuisc -p nuis`
* [ ] focused frontend generic probes still pass
* [ ] focused lowering/control-flow probes still pass
* [ ] real project compile harnesses still pass
* [ ] async/task + memory/session integration probes still pass
* [ ] network-oriented compile harnesses still pass
* [ ] spot-check `nuis help`
* [ ] spot-check `nuis project-doctor <project-dir>`
* [ ] spot-check `nuis check <project-dir|nuis.toml>`
* [ ] spot-check `nuis build <project-dir|nuis.toml> <output-dir>`
* [ ] spot-check `nuis release-check <project-dir|nuis.toml> <output-dir>`

## `0.17.0` Version-Facing Surfaces To Reconfirm

* [ ] generic completion:
  explicit args, inferred routes, helper chains, control-flow-local
  specialization, and higher-order/lambda routes still feel like one coherent
  system
* [ ] generic diagnostics:
  when generic routes fail, they fail specifically and honestly instead of
  collapsing into opaque internal mismatches
* [ ] lowering completion:
  frontend-validated routes increasingly survive into checked-in lowering and
  verifier-backed compile outputs
* [ ] control-flow lowering:
  loop-family, branch-local flow, and generic-heavy branch assembly continue to
  compose instead of regressing in combination
* [ ] async/task/memory bridge:
  session/policy/batch/windowed/task-value routes continue to compose as shared
  groundwork
* [ ] `std net` bridge:
  network/profile/transport/session/http-facing routes increasingly stand on
  the same async/task/memory/lowering spine
* [ ] compile truth vs runtime truth:
  docs and examples still distinguish them honestly
* [ ] real project anchors:
  the checked-in project demos used as mainline proofs still compile through
  the actual project pipeline

## Version Number Decision

Before any real release cut, decide explicitly:

* [ ] this is a repository/toolchain-line snapshot only
* [ ] or this is a coordinated manifest/version bump line

Do not leave that distinction implicit.

## Rule Of Thumb

If `0.17.0` claims integration, the checked-in repository should show routes
that work together, not only more isolated wins.
