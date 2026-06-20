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
* the current real-time demo path in `window_controls_demo_project` is the execution direction `ns-nova` should eventually absorb and abstract
* `nuis galaxy init --framework ns-nova` now emits an `ns-nova.toml` profile that carries framework-level assembly metadata, including the standard `ns-nova-selection-v1` selection contract for relational controls such as `list`, `table`, `tree`, `inspector`, and `outline`
* `ns-nova.toml` now also carries `ns-nova-family-v1` and `ns-nova-render-v1` scaffolding so projects can declare whether they currently lean toward `core`, `ui`, or future `scene` layers
* the `stdlib/ns-nova/*` tree now starts carrying real `.ns` source modules as canonical builder/state examples, even though full crate-style import wiring is not finished yet

Current source-asset status:

* this is currently the only `stdlib` layer that already declares a canonical
  checked-in source set through
  [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/module.toml)
* that manifest currently lists `11` source modules
* `nuis` smoke tests and `project-doctor` now both inspect that asset set

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/module.toml)
* [core/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/README.md)
* [ui/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/ui/README.md)
* [scene/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/README.md)

First source modules:

* [core/theme_surface.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/theme_surface.ns)
* [core/frame_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/frame_runtime.ns)
* [core/texture_resource_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/texture_resource_recipe.ns)
* [core/window_controls_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/window_controls_runtime_recipe.ns)
* [ui/panel_selection.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/ui/panel_selection.ns)
* [ui/panel_blueprint.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/ui/panel_blueprint.ns)
* [ui/window_controls_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/ui/window_controls_recipe.ns)
* [scene/scene_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/scene_runtime.ns)
* [scene/efficiency_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/efficiency_runtime.ns)
* [scene/scene_blueprint.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/scene_blueprint.ns)
* [scene/window_controls_scene_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/window_controls_scene_recipe.ns)

Current limitation:

* these files are the first canonical `ns-nova` source assets inside `stdlib`
* they are not yet imported automatically through a crate-like `use ns-nova ...` flow
* today they should be read as library-source anchors and compileable templates while project/dependency import management catches up

## Relationship To `window_controls_demo`

The current canonical project route is still:

* [examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)

That project is not “obsolete because recipes now exist”. It is still the main
truth source for the fully assembled end-to-end path.

The current migration split is:

* already extracted into stdlib recipes
  - render/runtime orchestration patterns:
    [core/texture_resource_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/texture_resource_recipe.ns),
    [core/window_controls_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/window_controls_runtime_recipe.ns)
  - UI/selection/control packing patterns:
    [ui/window_controls_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/ui/window_controls_recipe.ns)
  - scene/runtime efficiency and assembly patterns:
    [scene/window_controls_scene_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/window_controls_scene_recipe.ns)
* still intentionally left in the project demo
  - full multi-domain assembly in one realistic project
  - host/window integration details
  - exact demo-oriented packet mixes used to pressure-test current lowering and
    shader fallback behavior

So the rule of thumb is:

* read `examples/projects/window_controls_demo` for current complete workflow
* read `stdlib/ns-nova/*recipe.ns` for the pieces that have already become
  reusable source assets
