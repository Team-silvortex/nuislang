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

See also:

* [index.toml](/Users/Shared/chroot/dev/nuislang/stdlib/index.toml)

