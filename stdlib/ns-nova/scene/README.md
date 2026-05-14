# `ns-nova-scene`

`ns-nova-scene` is the scene/render-world package in the `ns-nova` family.

Its job is to push `ns-nova` from a strong GPU-native UI framework into a true engine-style
2D/3D rendering framework.

Current intended responsibility:

* scene graph contracts
* camera contracts
* material / mesh / light / sprite / effect contracts
* render-world composition on top of `ns-nova-core`
* future compute-assisted scene workflows via `kernel` domain collaboration

This layer is still mostly a direction rather than a finished subsystem in the repository,
but it is the natural next step after the current UI + render-skeleton work.

Current source anchor:

* [scene_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/scene_runtime.ns)
* [efficiency_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/efficiency_runtime.ns)
* [scene_blueprint.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/scene_blueprint.ns)
* [window_controls_scene_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/window_controls_scene_recipe.ns)

This first scene-side source module keeps to the more stable packet/state surface:
scene, camera, material, light, mesh, transform, node, visibility, cull, lod, streaming,
and budget.

`efficiency_runtime.ns` keeps pushing the resource-efficiency side of the scene contract:

* visibility / cull / lod
* streaming / residency / eviction / prefetch
* budget / pressure / thermal / power
* latency / frame pacing / frame variance / jank

`scene_blueprint.ns` is the first scene-side helper module with library-shaped functions:

* small builders returning `NovaSceneLinkPacket` and `NovaSceneClusterPacket`
* scene link summarizers
* cluster visibility/cull/lod summaries

`window_controls_scene_recipe.ns` pushes the same idea one step closer to the real demo path:

* seed structs that look more like project-side runtime inputs
* grouped helpers for scene / camera / material / node / cluster assembly
* a compileable extraction of the stable scene-side wiring pattern from `window_controls_demo`
