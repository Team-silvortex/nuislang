# `nuis` 0.13.0 Phase Checklist

This file is a lightweight phase checklist for the `0.13.0` line.

It is intentionally short. The goal is to keep release preparation visible
without turning the repository into process-heavy bureaucracy.

## Scope

Use this checklist when you want to sanity-check whether the repository still
looks like the `0.13.0` phase you meant to describe.

## Documentation

* [ ] confirm [nuis-0.13.0-snapshot.md](nuis-0.13.0-snapshot.md)
  still matches the checked-in mainline
* [ ] confirm [README.md](../../README.md),
  [docs/current-mainline-map.md](../../docs/current-mainline-map.md),
  and [docs/reference/README.md](../../docs/reference/README.md)
  still point at the right current anchors
* [ ] confirm `0.13.0` wording does not overstate sketch/future-edge material as
  already promised repository behavior

## Toolchain And Validation

* [ ] `cargo fmt --all`
* [ ] `cargo test -q -p nuisc -p nuis`
* [ ] spot-check `nuis project-status <project-dir>`
* [ ] spot-check `nuis project-doctor <project-dir>`
* [ ] spot-check one project `check` and one project `build`

## Current Phase-Facing Surfaces To Reconfirm

* [ ] visibility:
  `pub/private` source boundary plus `public_surface` reporting
* [ ] annotations:
  `@test`, `@export`, `@inline`, `@noinline`, `@host_symbol`
* [ ] packet schema:
  `@packet`, `@packet_field`, `@packet_control_field`, packet metadata index
* [ ] trait/generic MVP:
  parse + constrained monomorphization path
* [ ] executable loop MVP:
  counted/carry/flow subset
* [ ] `std net` low-level ladder:
  syscall edge -> socket edge -> flow/session-facing recipes

## Version Number Decision

`0.13.0` phase documentation does not require immediately bumping every
workspace crate version string.

Before any actual release cut, decide one of these explicitly:

* [ ] keep crate/package versions on `0.1.0` for now and treat `0.13.0` as a
  repository/language-line snapshot only
* [ ] perform a coordinated manifest/version bump as part of the real release
  cut

Do not leave that distinction implicit at cut time.

## Rule Of Thumb

If an item is green in docs but fuzzy in implementation:

* fix the implementation,
* or narrow the docs,
* but do not ship the mismatch on purpose.
