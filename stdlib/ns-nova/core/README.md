# `ns-nova-core`

`ns-nova-core` is the shared rendering/runtime foundation of the `ns-nova` family.

It is not meant to be the final application-facing package on its own.
Its job is to hold the engine skeleton that both UI and scene packages depend on.

Current intended responsibility:

* render surface contracts
* viewport and layer contracts
* frame/pass lifecycle helpers
* shared packet/state schemas that should not belong only to UI widgets
* GPU-native inline-shader-facing render orchestration semantics

Concrete directions already visible in this repository:

* `NovaThemePacket`
* `NovaSurfacePacket`
* `NovaViewportPacket`
* `NovaLayerPacket`
* project-level render owner / bridge / surface roles inside `ns-nova.toml`

Current source anchor:

* [theme_surface.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/theme_surface.ns)
* [frame_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/frame_runtime.ns)
* [window_controls_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/window_controls_runtime_recipe.ns)

This file is intentionally small and compileable on its own. It is the first step toward
turning `ns-nova-core` from pure framework contract text into real `ns` source assets.

`frame_runtime.ns` adds the next layer up:

* pass / frame / target / frame-graph orchestration
* queue / semaphore / timeline / fence signaling
* dispatch / feedback / intent / reaction / commit / snapshot style runtime contracts

`window_controls_runtime_recipe.ns` is the first project-shaped extraction from the
runtime half of `window_controls_demo`:

* seed struct for stable per-frame knobs
* grouped render-chain helper and grouped feedback-chain helper
* a small proof that `ns-nova-core` can already hold reusable orchestration recipes

This layer should stay:

* renderer-oriented
* packageable as a shared dependency
* independent from any one widget demo or one 3D scene demo
