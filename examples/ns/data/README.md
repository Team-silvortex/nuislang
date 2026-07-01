# Data `.ns` Examples

This folder contains front-end examples focused on `data` interaction and unit use:

* `hello_data.ns`
* `hello_data_window.ns`
* `hello_instantiate.ns`

Current role rule:

* this subtree is a narrow source-side `data` anchor set
* it is useful for compact frontend reading
* it should not compete with the richer project-form `data` routes already
  carried by the current showcase projects

## Current Frontdoor Ladder

Use this order first:

* `hello_data.ns`
  smallest data-oriented frontend route
* `hello_data_window.ns`
  immutable `Window<T>` plus local mutable `WindowMut<T>` framing
* `hello_instantiate.ns`
  CPU-side unit instantiation and cross-domain use

Practical rule:

* stop here if you only need the compact single-file `data` story
* jump to project-form routes once the question becomes domain composition,
  richer UI flow, or kernel/data interplay

Useful commands:

```bash
cargo run -p nuis -- dump-yir examples/ns/data/hello_data.ns
cargo run -p nuis -- build examples/ns/data/hello_instantiate.ns /tmp/nuis_hello_instantiate
```

Project-scale `data_profile_*` flows currently live in:

* [window_controls_demo/main.ns](../../projects/window_controls_demo/main.ns)
* [kernel_tensor_demo/main.ns](../../projects/kernel_tensor_demo/main.ns)

Reading rule:

* use this README for the shortest single-file `data` route
* use [examples/projects/README.md](../../../examples/projects/README.md)
  for the canonical multi-file project route
* treat project-scale `data` paths as the stronger current validation story

Current note:

* `data_immutable_window(...)` is the bridge-safe `Window<T>` route
* `data_copy_window(...)` is the local mutable `WindowMut<T>` route
