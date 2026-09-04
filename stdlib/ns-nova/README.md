# `ns-nova`

`ns-nova` is the third major standard-library module of `nuis`.

Its role is similar to what `Flutter` means to `Dart`, but for the `nuis` execution model:
it should turn heterogeneous execution, data-plane orchestration, and inline shader capability
into a native GPU-first application and rendering framework.

Target character:

* native GPU cross-platform 2D/3D rendering framework
* engine-style driver/runtime surface rather than only a widget kit
* built on `nuis` domain composition:
  `cpu` for orchestration,
  `data` for exchange,
  `shader` for rendering,
  `kernel` for future compute-heavy scene or simulation workflows

Intended scope:

* renderer and scene/frame orchestration
* material/pipeline/shader packaging helpers
* window/input/frame lifecycle abstractions
* 2D UI, 3D scene, and game-style application driving built on the same GPU-native core

Family structure:

* `ns-nova-core`
  shared render/runtime skeleton such as theme, surface, viewport, layer, and frame-facing contracts
* `ns-nova-ui`
  UI/widget/control framework built on top of the core render skeleton
* `ns-nova-scene`
  future 2D/3D scene/render-world framework built on the same core

Design principles:

* runtime-native and GPU-first, not a thin wrapper around software preview paths
* powered by `nuis` inline shader and heterogenous graph abilities rather than hiding them completely
* should remain mod-aware and ABI-aware so that packaged `nustar` capabilities stay visible

Current state:

* this repository now treats `ns-nova` as a standard-library/framework layer target, not as a separate future repository by default
* `lib/app_runtime.ns` now owns the first reusable application and frame lifecycle in Nuis source
* `examples/projects/domains/ns_nova_showcase` composes that lifecycle with PixelMagic, Data, and Shader as separate Galaxy/Nustar owners
* the showcase now carries checked `galaxy.toml` and `ns-nova.toml` manifests whose package inputs are project-owned relative paths; Galaxy dependencies remain registry-resolved rather than embedded as host paths
* `nuis galaxy init --framework ns-nova` now emits an `ns-nova.toml` profile that carries framework-level assembly metadata, including the standard `ns-nova-selection-v1` selection contract for relational controls such as `list`, `table`, `tree`, `inspector`, and `outline`
* `ns-nova.toml` now also carries `ns-nova-family-v1` and `ns-nova-render-v1` scaffolding so projects can declare whether they currently lean toward `core`, `ui`, or future `scene` layers
* host ABI selection remains project-driven and automatic; the framework source does not select Metal, Vulkan, or a host OS

Current source-asset status:

* this is currently the only `stdlib` layer that already declares a canonical
  checked-in source set through
  [module.toml](module.toml)
* the initial score-oriented project library module is
  [lib/nova_contracts.ns](lib/nova_contracts.ns)
* the first executable lifecycle module is
  [lib/app_runtime.ns](lib/app_runtime.ns), which exposes owned
  `NovaAppState` and `NovaFrameTransaction` transitions
* both library modules currently use `library_import_policy = "manual-only"`
  so it is declared and discoverable through project metadata, but it is not
  auto-injected into project scope by default
* projects may still opt into it explicitly through
  `galaxy_imports = ["ns-nova:lib/app_runtime.ns"]`
* duplicate `galaxy_imports` entries are rejected during manifest loading, so
  this opt-in should be listed at most once
* that manifest currently lists `11` source modules
* `nuis` smoke tests and `project-doctor` now both inspect that asset set

See metadata:

* [module.toml](module.toml)
* [core/README.md](core/README.md)
* [ui/README.md](ui/README.md)
* [scene/README.md](scene/README.md)

First source modules:

* [core/theme_surface.ns](core/theme_surface.ns)
* [core/frame_runtime.ns](core/frame_runtime.ns)
* [core/texture_resource_recipe.ns](core/texture_resource_recipe.ns)
* [core/window_controls_runtime_recipe.ns](core/window_controls_runtime_recipe.ns)
* [ui/panel_selection.ns](ui/panel_selection.ns)
* [ui/panel_blueprint.ns](ui/panel_blueprint.ns)
* [ui/window_controls_recipe.ns](ui/window_controls_recipe.ns)
* [scene/scene_runtime.ns](scene/scene_runtime.ns)
* [scene/efficiency_runtime.ns](scene/efficiency_runtime.ns)
* [scene/scene_blueprint.ns](scene/scene_blueprint.ns)
* [scene/window_controls_scene_recipe.ns](scene/window_controls_scene_recipe.ns)

Current limitation:

* the first lifecycle is a one-frame prototype, not a stable interactive event loop
* conditional `cpu_present_frame` branches are not yet lowered, so the reference
  slice records readiness and uses the existing unconditional host presentation path
* the framework does not yet own a renderer; PixelMagic remains an independent
  project-level composition dependency in the showcase
* the current checked AOT window shell is the replaceable Apple bootstrap adapter;
  non-Apple window adapters and backend execution closure remain open

The machine-readable honesty boundary is
[nuis-ns-nova-application-lifecycle-v1.toml](../../docs/reference/nuis-ns-nova-application-lifecycle-v1.toml).

## First Executable Slice

The current shortest route is:

* [examples/projects/domains/ns_nova_showcase](../../examples/projects/domains/ns_nova_showcase)

It proves:

* explicit import of the reusable Nova lifecycle
* independent PixelMagic shader ownership
* registered Data transfer through `FabricPlane`
* automatic CPU, Data, and Shader ABI recommendation
* project compilation and Apple arm64 window AOT packaging

## Relationship To `window_controls_demo`

The older broad stress route remains:

* [examples/projects/window_controls_demo](../../examples/projects/window_controls_demo)

That project is not obsolete. It remains the detailed control/scene stress
fixture, while `ns_nova_showcase` is now the smaller framework front door.

The current migration split is:

* already extracted into stdlib recipes
  - render/runtime orchestration patterns:
    [core/texture_resource_recipe.ns](core/texture_resource_recipe.ns),
    [core/window_controls_runtime_recipe.ns](core/window_controls_runtime_recipe.ns)
  - UI/selection/control packing patterns:
    [ui/window_controls_recipe.ns](ui/window_controls_recipe.ns)
  - scene/runtime efficiency and assembly patterns:
    [scene/window_controls_scene_recipe.ns](scene/window_controls_scene_recipe.ns)
* still intentionally left in the project demo
  - full multi-domain assembly in one realistic project
  - host/window integration details
  - exact demo-oriented packet mixes used to pressure-test current lowering and
    shader fallback behavior

So the rule of thumb is:

* read `examples/projects/domains/ns_nova_showcase` for the current shortest workflow
* read `examples/projects/window_controls_demo` for the broad stress workflow
* read `stdlib/ns-nova/*recipe.ns` for the pieces that have already become
  reusable source assets
