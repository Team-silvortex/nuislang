# `nuis` Standard Library

This directory is the repository's standard-library layout and source-asset
staging area.

It is not yet a crate-like automatically imported library tree, but it is no
longer just empty scaffolding either.

The current top-level modules are:

* [core](/Users/Shared/chroot/dev/nuislang/stdlib/core/README.md)
  smallest semantics-first base surface and long-lived source contracts
* [std](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
  practical systems/helper layer built on `core`
* [ns-nova](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)
  rendering/application framework layer and the first place where real checked-in
  `.ns` source modules are already accumulating

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

* `core` and `std` are still mostly structure/contract layers
* `ns-nova` now already contains the first real checked-in `.ns` source modules and recipe-style helpers
* the repository still does not have a crate-like automatic source import flow for those modules yet
* the live implementation focus is still on `nuis / nuisc / YIR / nustar`
* the standard-library split is nevertheless important now, because it already informs how future APIs should be grouped and what should or should not leak across layers

Current asset reality by layer:

* `core`
  now has its first canonical checked-in `.ns` module set
* `std`
  now has its first canonical checked-in `.ns` module set
* `ns-nova`
  first real source-asset layer; currently declared through
  [stdlib/ns-nova/module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/module.toml)
  with `10` checked-in source modules

Current checked-in source modules by layer:

* `core`
  - [basic_scalars.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/basic_scalars.ns)
  - [struct_patterns.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/struct_patterns.ns)
  - [math_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/math_runtime.ns)
  - [ref_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/ref_runtime.ns)
* `std`
  - [window_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime.ns)
  - [pipe_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime.ns)
  - [fabric_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime.ns)
  - [handle_table_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime.ns)
* `ns-nova`
  - see [stdlib/ns-nova/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

## Read In This Order

* [core](/Users/Shared/chroot/dev/nuislang/stdlib/core/README.md)
* [std](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* [ns-nova](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

See also:

* [index.toml](/Users/Shared/chroot/dev/nuislang/stdlib/index.toml)
