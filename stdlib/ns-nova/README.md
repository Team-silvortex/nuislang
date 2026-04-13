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

Design principles:

* runtime-native and GPU-first, not a thin wrapper around software preview paths
* powered by `nuis` inline shader and heterogenous graph abilities rather than hiding them completely
* should remain mod-aware and ABI-aware so that packaged `nustar` capabilities stay visible

Current state:

* this repository now treats `ns-nova` as a standard-library/framework layer target, not as a separate future repository by default
* the current real-time demo path in `window_controls_demo_project` is the execution direction `ns-nova` should eventually absorb and abstract

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/module.toml)

