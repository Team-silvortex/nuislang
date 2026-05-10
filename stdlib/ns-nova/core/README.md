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

This layer should stay:

* renderer-oriented
* packageable as a shared dependency
* independent from any one widget demo or one 3D scene demo
