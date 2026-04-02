# `YIR` Examples

This folder contains the current handwritten `YIR` examples.

The main families represented here are:

* `cpu`
* `shader`
* `kernel`
* `data`

Recommended starting points:

* [hello_yir.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/hello_yir.yir)
  smallest cross-domain example
* [window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/window_controls_demo.yir)
  current main `cpu + data + shader` control/render demo
* [shader_bindings_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/shader_bindings_demo.yir)
  current shader resource/binding geometry path
* [kernel_auto_broadcast_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel_auto_broadcast_demo.yir)
  current kernel tensor/broadcast path
* [data_fabric_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/data_fabric_demo.yir)
  current typed Fabric/data surface

Use:

```bash
cargo run -p yir-run -- examples/yir/hello_yir.yir
cargo run -p yir-run -- examples/yir/window_controls_demo.yir
cargo run -p yir-pack-aot -- examples/yir/window_controls_demo.yir examples/bins/window_controls_demo 4
```
