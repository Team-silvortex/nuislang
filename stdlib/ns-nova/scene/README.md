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
