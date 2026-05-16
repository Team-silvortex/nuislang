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

* the repository still does not have a crate-like automatic source import flow
  for stdlib modules yet
* the live implementation focus is still on `nuis / nuisc / YIR / nustar`
* but `stdlib` is no longer empty scaffolding; all three layers now carry real
  checked-in `.ns` assets

Asset view by layer:

* `core`
  - smallest checked-in source layer
  - currently reads best as `facade + blueprint` style source assets
  - start with:
    [basic_scalars.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/basic_scalars.ns),
    [struct_patterns.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/struct_patterns.ns),
    [math_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/math_runtime.ns),
    [ref_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/ref_runtime.ns),
    [value_blueprint.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/value_blueprint.ns)
* `std`
  - practical systems layer with many host-backed facade modules
  - now also carries project-shaped recipe modules
  - facade/recipe split is documented in
    [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* `ns-nova`
  - current framework/source-asset layer
  - currently the richest family-shaped stdlib surface
  - declared through
    [stdlib/ns-nova/module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/module.toml)
    with `10` checked-in source modules
  - see
    [stdlib/ns-nova/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

Current asset types:

* `core`
  facade modules plus a first small blueprint layer
* `std`
  host-backed facade modules plus recipe modules for CLI, reporting, automation,
  and early clock/test timing alignment
* `ns-nova`
  framework-oriented runtime/blueprint/recipe modules across `core`, `ui`, and `scene`

Current boundaries:

* none of these layers are yet an automatically imported library tree
* `core` is intentionally conservative
* `std` is broadening quickly, but most surfaces are still explicitly host-backed
* `ns-nova` is the most mature source-asset family, but it still relies on
  project/demo routes as the full end-to-end truth source

## Read In This Order

* [core](/Users/Shared/chroot/dev/nuislang/stdlib/core/README.md)
* [std](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* [ns-nova](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

See also:

* [index.toml](/Users/Shared/chroot/dev/nuislang/stdlib/index.toml)
