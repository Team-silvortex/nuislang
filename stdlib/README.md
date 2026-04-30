# `nuis` Standard Library

The `nuis` standard library is now organized into three top-level modules:

* [core](/Users/Shared/chroot/dev/nuislang/stdlib/core/README.md)
  the smallest semantics-first base surface; future home of primitive value types,
  platform-neutral traits, ownership-oriented helpers, and the lowest stable source-level contracts
* [std](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
  the practical systems layer built on `core`; future home of collections,
  I/O-facing facades, data-plane helpers, host integration helpers, and common workflow utilities
* [ns-nova](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)
  the rendering/application framework layer; future home of the engine-style GPU runtime surface
  that turns `nuis` heterogeneous execution and inline shader capability into a native cross-platform
  2D/3D rendering framework

This directory currently defines structure and intent, not a fully implemented importable source library yet.
That is deliberate: the current frontend still centers on executable entry modules, while the standard-library
level is being stabilized first as a repository/module contract.

## Relationship

The intended dependency direction is:

```text
core -> std -> ns-nova
```

Read that as:

* `core` should carry the smallest source-level semantic contracts
* `std` should add practical systems helpers without hiding execution semantics
* `ns-nova` should build a GPU-first application/rendering framework on top of those lower layers

## Current Reality

At the current repo stage:

* this is still mostly a structure/contract layer, not a fully populated importable source tree
* the live implementation focus is still on `nuis / nuisc / YIR / nustar`
* the standard-library split is nevertheless important now, because it already informs how future APIs should be grouped and what should or should not leak across layers

## Read In This Order

* [core](/Users/Shared/chroot/dev/nuislang/stdlib/core/README.md)
* [std](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* [ns-nova](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

See also:

* [index.toml](/Users/Shared/chroot/dev/nuislang/stdlib/index.toml)
