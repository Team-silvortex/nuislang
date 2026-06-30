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
* [pixelmagic](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/README.md)
  official image/resource Galaxy built on `core + std`
* [witsage](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/README.md)
  official classical ML Galaxy built on `core + std`
* [ns-nova](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)
  rendering/application framework layer and the first place where real checked-in
  `.ns` source modules are already accumulating

## Relationship

The intended dependency direction is:

```text
core -> std -> pixelmagic
core -> std -> witsage
core -> std -> ns-nova
```

Read that as:

* `core` should carry the smallest source-level semantic contracts
* `std` should add practical systems helpers without hiding execution semantics
* `pixelmagic` should hold image/resource contracts and shader-facing image prep on top of those lower layers
* `witsage` should hold classical ML contracts and kernel-facing model plans on top of those lower layers
* `ns-nova` should build a GPU-first application/rendering framework on top of those lower layers

## Current Reality

At the current repo stage:

* the repository still does not have a crate-like automatic source import flow
  for stdlib modules yet
* project manifests can now declare local stdlib galaxy dependencies such as
  `galaxy = ["pixelmagic=workspace"]`; the compiler resolves them through
  [index.toml](/Users/Shared/chroot/dev/nuislang/stdlib/index.toml) and emits
  `nuis.project.galaxy.txt` metadata during project compilation
* galaxy dependencies may also declare dedicated `library_modules` for safe
  automatic project injection; `pixelmagic` now exposes its first one through
  [pixelmagic/lib/image_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/lib/image_contracts.ns)
* stdlib package manifests now also use registry-style stable `surfaces` ids,
  so discovery vocabulary can stay stable even if concrete module filenames
  continue to evolve
* `core` and `std` now also expose their first library-module surfaces through
  [core/lib/prelude_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/lib/prelude_contracts.ns)
  and
  [std/lib/task_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/lib/task_contracts.ns),
  with additional std contract companions such as
  [std/lib/text_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/lib/text_contracts.ns)
* `ns-nova` now also exposes its first library-module surface through
  [ns-nova/lib/nova_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/lib/nova_contracts.ns)
  but unlike `core`, `std`, and `pixelmagic`, it is currently marked
  `manual-only` rather than auto-injected
* explicit manifest imports such as
  `galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]` are now validated as a
  unique set, so duplicate entries fail manifest loading instead of being
  silently collapsed during path resolution
* those resolved galaxy dependencies are not auto-injected into the source
  module set yet, because many current stdlib recipe assets still collide on
  repeated bindings such as `mod cpu Main`
* the live implementation focus is still on `nuis / nuisc / YIR / nustar`
* but `stdlib` is no longer empty scaffolding; all four layers now carry real
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
* `pixelmagic`
  - official image/resource Galaxy
  - current earliest checked-in package skeleton for future GPU-side image work
  - declared through
    [stdlib/pixelmagic/module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/module.toml)
  - see
    [stdlib/pixelmagic/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/README.md)
* `witsage`
  - official classical ML Galaxy
  - first checked-in package skeleton for feature statistics and kernel-backed model plans
  - declared through
    [stdlib/witsage/module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/module.toml)
  - see
    [stdlib/witsage/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/README.md)
* `ns-nova`
  - current future framework/source-asset layer
  - currently important as an official GUI/rendering galaxy, but intentionally
    behind AOT, `std`, PixelMagic, and WitSage hardening
  - declared through
    [stdlib/ns-nova/module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/module.toml)
    with `11` checked-in source modules
  - see
    [stdlib/ns-nova/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

Current asset types:

* `core`
  facade modules plus a first small blueprint layer
* `std`
  host-backed facade modules plus auto-injectable task/IO/filesystem/CLI/network
  contracts and recipe modules for reporting, automation, and early
  clock/test timing alignment
* `pixelmagic`
  image/resource handoff, render-plan, and future shader-facing image prep
  contracts
* `witsage`
  dataset/statistics/model-plan/pipeline modules and future kernel-facing
  classical ML contracts
* `ns-nova`
  framework-oriented runtime/blueprint/recipe modules across `core`, `ui`, and
  `scene`, still mostly contract/source-asset oriented

Current boundaries:

* none of these layers are yet an automatically imported library tree
* `core` is intentionally conservative
* `std` is broadening quickly, but most surfaces are still explicitly host-backed
* `pixelmagic` and `witsage` are the current official pressure tests for
  shader/kernel cooperation
* `ns-nova` remains deliberately later-stage because it depends on the lower
  AOT, library, shader, kernel, and future runtime layers

## Read In This Order

* [core](/Users/Shared/chroot/dev/nuislang/stdlib/core/README.md)
* [std](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* [pixelmagic](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/README.md)
* [witsage](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/README.md)
* [ns-nova](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)

See also:

* [index.toml](/Users/Shared/chroot/dev/nuislang/stdlib/index.toml)
