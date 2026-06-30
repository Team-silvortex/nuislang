# Repository Layout

This file is the short map of what each top-level directory is for.

Use it when you already know the project roughly, but need to re-orient
quickly.

## Current Mainline Directories

* [tools](/Users/Shared/chroot/dev/nuislang/tools)
  executable front doors and compiler/tool binaries such as `nuis`, `nuisc`,
  `yir-run`, and packaging/export helpers
* [crates](/Users/Shared/chroot/dev/nuislang/crates)
  reusable Rust implementation crates for semantics, verifier logic, runtime
  support, shader/domain helpers, and related internals
* [nustar-packages](/Users/Shared/chroot/dev/nuislang/nustar-packages)
  checked-in `nustar` manifests and package registration metadata, including
  ABI targets and lane defaults
* [examples](/Users/Shared/chroot/dev/nuislang/examples)
  canonical source examples, invalid/verifier examples, historical bridge
  examples, and current checked-in build bundles
* [stdlib](/Users/Shared/chroot/dev/nuislang/stdlib)
  standard-library layout plus staged `.ns` source assets, especially the
  `std`, `PixelMagic`, `WitSage`, and `ns-nova` official library/galaxy
  surfaces
* [subprojects](/Users/Shared/chroot/dev/nuislang/subprojects)
  sibling ecosystem projects hosted in-tree for now, currently `vulpoya` and
  `yalivia`, with boundaries kept explicit rather than merged into `nuisc`
* [docs](/Users/Shared/chroot/dev/nuislang/docs)
  current reference docs, grammar/front-end notes, design notes, and historical
  archive material

## Current Mainline Vs Experimental Surfaces

Use this as the shortest repo-level split:

* mainline today
  - [tools](/Users/Shared/chroot/dev/nuislang/tools)
  - [crates](/Users/Shared/chroot/dev/nuislang/crates)
  - [nustar-packages](/Users/Shared/chroot/dev/nuislang/nustar-packages)
  - [examples/projects](/Users/Shared/chroot/dev/nuislang/examples/projects)
  - [examples/ns](/Users/Shared/chroot/dev/nuislang/examples/ns)
  - [stdlib/std](/Users/Shared/chroot/dev/nuislang/stdlib/std)
  - [stdlib/pixelmagic](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic)
  - [stdlib/witsage](/Users/Shared/chroot/dev/nuislang/stdlib/witsage)
  - [docs/reference](/Users/Shared/chroot/dev/nuislang/docs/reference)
* experimental or softer-edged today
  - [examples/yir](/Users/Shared/chroot/dev/nuislang/examples/yir)
    when used as handwritten probes rather than current front-door behavior
  - [docs/fabric-spec](/Users/Shared/chroot/dev/nuislang/docs/fabric-spec)
  - [docs/glm-spec](/Users/Shared/chroot/dev/nuislang/docs/glm-spec)
  - [docs/yir-spec](/Users/Shared/chroot/dev/nuislang/docs/yir-spec)
  - future-sketch notes scattered under
    [examples/projects](/Users/Shared/chroot/dev/nuislang/examples/projects)
    and
    [examples/ns/ffi](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi)

This is not a value judgment. It is just the practical rule for deciding what
to trust first when implementation and future direction are both present.

## Support / Secondary Directories

* [nuis-logo](/Users/Shared/chroot/dev/nuislang/nuis-logo)
  branding assets
* [target](/Users/Shared/chroot/dev/nuislang/target)
  local build outputs and scratch artifacts; not part of the curated source
  layout
* [.github](/Users/Shared/chroot/dev/nuislang/.github)
  repository automation/workflow metadata

## Reading Order By Goal

If you want the current user-facing path:

* [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)

If you want the current practical systems/library path:

* [stdlib/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/README.md)
* [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* [examples/ns/ffi/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)
* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)

If you want implementation internals:

* [tools](/Users/Shared/chroot/dev/nuislang/tools)
* [crates](/Users/Shared/chroot/dev/nuislang/crates)
* [nustar-packages](/Users/Shared/chroot/dev/nuislang/nustar-packages)

If you want framework/library evolution:

* [stdlib/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/README.md)
* [stdlib/pixelmagic/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/README.md)
* [stdlib/witsage/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/README.md)
* [stdlib/ns-nova/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

If you want older design background:

* [docs/historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)
