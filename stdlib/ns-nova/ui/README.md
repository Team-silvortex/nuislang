# `ns-nova-ui`

`ns-nova-ui` is the UI-focused package in the `ns-nova` family.

Its purpose is not just to expose widgets.
It should become the GPU-native application UI framework built on top of `ns-nova-core`.

Current intended responsibility:

* control and panel packets
* theme and selection contracts
* widget state views
* relational tooling controls such as list/table/tree/inspector/outline
* future layout, input, focus, and animation contracts

The current `window_controls_demo` is the main execution path pushing this direction.

This layer should absorb and formalize:

* `NovaPanelPacket`
* widget packet/state families
* shared selection contracts
* UI-oriented render assembly on top of surface / viewport / layer

Current source anchor:

* [panel_selection.ns](panel_selection.ns)
* [panel_blueprint.ns](panel_blueprint.ns)
* [window_controls_recipe.ns](window_controls_recipe.ns)

This module focuses on the more stable first slice of `ns-nova-ui`: controls, shared
selection, and relational state helpers.

`panel_blueprint.ns` is the first intentionally library-shaped experiment in this tree:

* small helper functions returning `Nova*Packet`
* packet-to-state summarizers
* a way to probe how close current `ns` is to supporting real reusable `ns-nova` source modules

`window_controls_recipe.ns` is the first explicit extraction from the `window_controls_demo`
style of assembly:

* packs scalar control seeds through `WindowMut -> freeze -> Window`
* builds shared relational controls from those packed values
* keeps the recipe project-shaped while still staying compileable as a standalone source module
