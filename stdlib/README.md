# Nuis Standard Library And Official Galaxies

This tree contains Nuis library modules, source recipes and registered assets.
The [workspace index](index.toml) and each `module.toml` declare what a project
can resolve and import. A recipe, contract or backend declaration is not by
itself evidence of dispatched device execution.

## Layers And Import Policy

| Layer | Responsibility | Library Import Policy | Manifest |
| --- | --- | --- | --- |
| [core](core/README.md) | Minimal semantic base and primitive/source contracts. | `project-auto` | [module.toml](core/module.toml) |
| [std](std/README.md) | Practical task, I/O, filesystem, text, network and tooling contracts. | `project-auto` | [module.toml](std/module.toml) |
| [pixelmagic](pixelmagic/README.md) | Independent image/resource algorithms and shader-facing surfaces. | `project-auto` | [module.toml](pixelmagic/module.toml) |
| [witsage](witsage/README.md) | Independent classical ML, statistics and kernel/model surfaces. | `project-auto` | [module.toml](witsage/module.toml) |
| [ns-nova](ns-nova/README.md) | Engine lifecycle and application composition across independent owners. | `manual-only` | [module.toml](ns-nova/module.toml) |

The dependency direction is `core -> std`, with PixelMagic, WitSage and ns-nova
each depending on those lower layers. ns-nova may compose independent Galaxies
at project level; it does not absorb their algorithms or select their providers.

## What Automatic Import Means

Only declared `library_modules` participate in project library injection.
For a resolved `project-auto` package, those modules are injected by policy;
arbitrary `source_modules`, recipes and assets are not. This avoids injecting
several recipe entry bindings such as `mod cpu Main` into one project.

A project can request the workspace PixelMagic package:

```toml
galaxy = ["pixelmagic=workspace"]
```

The `manual-only` ns-nova libraries require explicit selection, for example:

```toml
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/app_runtime.ns"]
```

Duplicate `galaxy_imports` fail manifest loading. Stable `surfaces` identifiers
describe discovery vocabulary; they do not authorize importing undeclared files.
Use the manifests for the full inventory instead of a copied module count.

## Resolution And Cache Boundary

[Galaxy resolution locks](../docs/reference/galaxy-resolution-lock-contract.md)
provide canonical dependency identity, trust-aware selection, synchronized
content-addressed project caches and build-manifest verification. Registered
providers include workspace, offline-layout and locked-resolution-cache routes.
Local package pack/publish/install is a separate surface, not compiler resolution.

This is not a claim of arbitrary remote registry fetching or a live machine-wide
collector. The [Nuis-rc policy](../docs/reference/nuis-rc-cache-lifecycle.md) has
a tested read-only heap/stack planner; shared-store migration, compiler leases,
GC execution and the actual resident collector remain separate work. Current
compilation still uses the project-local synchronized cache.

## Source And Executable Evidence

| Goal | Source Or Contract | Observable Route And Boundary |
| --- | --- | --- |
| Semantic base | [Prelude](core/lib/prelude_contracts.ns), [basic scalars](core/basic_scalars.ns), [records](core/struct_patterns.ns), [math](core/math_runtime.ns), [references](core/ref_runtime.ns), [value blueprint](core/value_blueprint.ns). | Language and lowering regressions establish their individual admission scope. |
| Practical systems work | [Task](std/lib/task_contracts.ns), [text](std/lib/text_contracts.ns), [std guide](std/README.md). | [Observable CLI tests](../tools/nuis/tests/std_filesystem_smoke.rs) build bounded real I/O programs, not unlimited streaming/concurrency. |
| Image work | [Image contracts](pixelmagic/lib/image_contracts.ns), [PixelMagic guide](pixelmagic/README.md). | The [image showcase](../examples/projects/domains/ns_nova_image_showcase/README.md) checks actual Metal processing through the registered embedded-YIR host route. |
| Classical ML | [WitSage guide](witsage/README.md), [shader/kernel contract](../docs/reference/std-shader-kernel-project-contract.md). | Model/kernel recipes and report files are not physical provider acceptance by themselves. |
| Engine work | [Nova contracts](ns-nova/lib/nova_contracts.ns), [application runtime](ns-nova/lib/app_runtime.ns). | Persistent embedded-YIR windows and bounded native CPU sessions remain distinct; see the [proof index](../docs/reference/nuis-native-session-proof-index.md). |

Provider samples carry explicit adapter identity, input/output hashes and
capability status. Real dispatch/readback and host-simulated fallback must stay
distinguishable. PixelMagic owns image algorithms, WitSage owns classical ML,
and registered Nustars/providers own domain admission and execution.

## Current Development Direction

The current [application-led mainline](../docs/current-mainline-map.md) grows
ns-nova through bounded executable slices while strengthening general compiler,
library and provider contracts. Rendering, control, ML and audio form its engine
horizon; their combined maturity is not implied by the image/session milestone.

Keep core/std contracts usable for CLI applications, use PixelMagic and WitSage
to pressure independent shader/kernel execution, and retain registry-driven
placement rather than engine-specific compiler cases. Read package guides for
local detail and the [examples router](../examples/README.md) for runnable entrypoints.
