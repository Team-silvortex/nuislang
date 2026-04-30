# `YIR` Examples

This folder contains the current handwritten `YIR` examples.

The main families represented here are:

* `cpu`
* `shader`
* `kernel`
* `data`

These handwritten examples are still important even though project-route `.ns`
examples are becoming the main user-facing path, because they remain the
clearest way to inspect raw execution-graph shape and verifier behavior.

Subdirectories:

* [cpu](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu/README.md)
* [shader](/Users/Shared/chroot/dev/nuislang/examples/yir/shader/README.md)
* [kernel](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/README.md)
* [data](/Users/Shared/chroot/dev/nuislang/examples/yir/data/README.md)
* [demos](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/README.md)

Recommended starting points:

* [hello_yir.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/hello_yir.yir)
  smallest cross-domain example
* [window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir)
  current main `cpu + data + shader` control/render demo
* [shader_bindings_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/shader/shader_bindings_demo.yir)
  current shader resource/binding geometry path
* [kernel_auto_broadcast_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/kernel_auto_broadcast_demo.yir)
  current kernel tensor/broadcast path
* [data_fabric_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/data/data_fabric_demo.yir)
  current typed Fabric/data surface

Use:

```bash
cargo run -p yir-run -- examples/yir/demos/hello_yir.yir
cargo run -p yir-run -- examples/yir/demos/window_controls_demo.yir
cargo run -p yir-pack-aot -- examples/yir/demos/window_controls_demo.yir examples/bins/window_controls_demo 4
```

Also useful while comparing handwritten graphs against the `.ns` pipeline:

```bash
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/kernel_tensor_demo
```

Current reading guidance:

* prefer these files when you want to understand `YIR` graph shape directly
* prefer `examples/projects/*` when you want the current canonical end-to-end workflow
* use the invalid examples to understand verifier boundaries, especially for memory, data-plane, and ownership-sensitive paths

Generated bundle:

* [examples/bins/window_controls_demo/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo/window_controls_demo)
