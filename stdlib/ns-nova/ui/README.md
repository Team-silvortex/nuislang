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
