# Repository Layout

This file is the short map of what each top-level directory is for.

Use it when you already know the project roughly, but need to re-orient
quickly.

## Current Mainline Directories

* [tools](../tools)
  executable front doors and compiler/tool binaries such as `nuis`, `nuisc`,
  `yir-run`, and packaging/export helpers
* [crates](../crates)
  reusable Rust implementation crates for semantics, verifier logic, runtime
  support, shader/domain helpers, and related internals
* [nustar-packages](../nustar-packages)
  checked-in `nustar` manifests and package registration metadata, including
  ABI targets and lane defaults
* [examples](../examples)
  canonical source examples, invalid/verifier examples, historical bridge
  examples, and current checked-in build bundles
* [stdlib](../stdlib)
  standard-library layout plus staged `.ns` source assets, especially the
  `std`, `PixelMagic`, `WitSage`, and `ns-nova` official library/galaxy
  surfaces
* [subprojects](../subprojects)
  sibling ecosystem projects hosted in-tree for now, currently `vulpoya` and
  `yalivia`, with boundaries kept explicit rather than merged into `nuisc`
* [docs](./)
  current reference docs, grammar/front-end notes, design notes, and historical
  archive material

## Current Mainline Vs Experimental Surfaces

Use this as the shortest repo-level split:

* mainline today
  - [tools](../tools)
  - [crates](../crates)
  - [nustar-packages](../nustar-packages)
  - [examples/projects](../examples/projects)
  - [examples/ns](../examples/ns)
  - [stdlib/std](../stdlib/std)
  - [stdlib/pixelmagic](../stdlib/pixelmagic)
  - [stdlib/witsage](../stdlib/witsage)
  - [docs/reference](reference)
* experimental or softer-edged today
  - [examples/yir](../examples/yir)
    when used as handwritten probes rather than current front-door behavior
  - [docs/fabric-spec](fabric-spec)
  - [docs/glm-spec](glm-spec)
  - [docs/yir-spec](yir-spec)
  - future-sketch notes scattered under
    [examples/projects](../examples/projects)
    and
    [examples/ns/ffi](../examples/ns/ffi)

This is not a value judgment. It is just the practical rule for deciding what
to trust first when implementation and future direction are both present.

## Support / Secondary Directories

* [nuis-logo](../nuis-logo)
  branding assets
* [target](../target)
  local build outputs and scratch artifacts; not part of the curated source
  layout
* [.github](../.github)
  repository automation/workflow metadata

## Reading Order By Goal

If you want the current user-facing path:

* [README.md](../README.md)
* [examples/projects/README.md](../examples/projects/README.md)
* [docs/reference/README.md](reference/README.md)

If you want the current practical systems/library path:

* [stdlib/README.md](../stdlib/README.md)
* [stdlib/std/README.md](../stdlib/std/README.md)
* [examples/ns/ffi/README.md](../examples/ns/ffi/README.md)
* [examples/projects/README.md](../examples/projects/README.md)

If you want implementation internals:

* [tools](../tools)
* [crates](../crates)
* [nustar-packages](../nustar-packages)

If you want framework/library evolution:

* [stdlib/README.md](../stdlib/README.md)
* [stdlib/pixelmagic/README.md](../stdlib/pixelmagic/README.md)
* [stdlib/witsage/README.md](../stdlib/witsage/README.md)
* [stdlib/ns-nova/README.md](../stdlib/ns-nova/README.md)

If you want older design background:

* [docs/historical/README.md](historical/README.md)
