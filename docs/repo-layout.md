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
  first real `ns-nova` modules
* [docs](/Users/Shared/chroot/dev/nuislang/docs)
  current reference docs, grammar/front-end notes, design notes, and historical
  archive material

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

If you want implementation internals:

* [tools](/Users/Shared/chroot/dev/nuislang/tools)
* [crates](/Users/Shared/chroot/dev/nuislang/crates)
* [nustar-packages](/Users/Shared/chroot/dev/nuislang/nustar-packages)

If you want framework/library evolution:

* [stdlib/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/README.md)
* [stdlib/ns-nova/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

If you want older design background:

* [docs/historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)
