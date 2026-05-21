# Demo `.ns` Examples

This folder contains higher-signal end-to-end demos and source-shaped domain
stubs:

* `window_controls_demo.ns`
* `shader_profile_demo.ns`
* `kernel_profile_demo.ns`
* `kernel_result_profile_demo.ns`
* `kernel_tensor_profile_demo.ns`
* `kernel_tensor_inspect_demo.ns`
* `kernel_tensor_slice_demo.ns`
* `kernel_tensor_reshape_demo.ns`
* `kernel_tensor_broadcast_demo.ns`
* `kernel_tensor_reduce_demo.ns`
* `kernel_tensor_select_demo.ns`
* `kernel_tensor_order_demo.ns`
* `kernel_tensor_axis_reduce_demo.ns`
* `kernel_tensor_map_zip_demo.ns`

Current guidance:

* read this file when you want the single-file end-to-end story
* prefer [examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo) when you want the canonical multi-file route with explicit project links, ABI state, and support-module profiles
* treat `shader_profile_demo.ns`, `kernel_profile_demo.ns`,
  `kernel_result_profile_demo.ns`, `kernel_tensor_profile_demo.ns`, and
  `kernel_tensor_inspect_demo.ns`, `kernel_tensor_slice_demo.ns`,
  `kernel_tensor_reshape_demo.ns`, `kernel_tensor_broadcast_demo.ns`,
  `kernel_tensor_reduce_demo.ns`, `kernel_tensor_select_demo.ns`,
  `kernel_tensor_order_demo.ns`, `kernel_tensor_axis_reduce_demo.ns`,
  `kernel_tensor_map_zip_demo.ns` as current source-shaped mirrors of
  project-first lanes
* today those two domain stubs do not pass standalone `nuis check` because
  `shader` and `kernel` lowering still rely on loaded `nustar` implementations
  rather than a bootstrap compatibility shim

Useful commands:

```bash
cargo run -p nuis -- check examples/ns/demos/window_controls_demo.ns
cargo run -p nuis -- build examples/ns/demos/window_controls_demo.ns /tmp/window_controls_demo_ns
cargo run -p nuis -- check examples/projects/domains/shader_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_result_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_inspect_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_slice_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_reshape_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_broadcast_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_reduce_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_select_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_order_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_reduce_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_map_zip_demo
```
