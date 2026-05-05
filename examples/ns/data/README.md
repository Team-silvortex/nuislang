# Data `.ns` Examples

This folder contains front-end examples focused on `data` interaction and unit use:

* `hello_data.ns`
* `hello_data_window.ns`
* `hello_instantiate.ns`

Recommended reading order:

* `hello_data.ns`
  smallest data-oriented frontend route
* `hello_data_window.ns`
  immutable `Window<T>` plus local mutable `WindowMut<T>` framing
* `hello_instantiate.ns`
  CPU-side unit instantiation and cross-domain use

Useful commands:

```bash
cargo run -p nuis -- dump-yir examples/ns/data/hello_data.ns
cargo run -p nuis -- build examples/ns/data/hello_instantiate.ns /tmp/nuis_hello_instantiate
```

Project-scale `data_profile_*` flows currently live in:

* `/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo/main.ns`
* `/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo/main.ns`

Current note:

* `data_immutable_window(...)` is the bridge-safe `Window<T>` route
* `data_copy_window(...)` is the local mutable `WindowMut<T>` route
