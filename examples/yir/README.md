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

* [cpu](cpu/README.md)
* [shader](shader/README.md)
* [kernel](kernel/README.md)
* [data](data/README.md)
* [demos](demos/README.md)

Recommended starting points:

* [hello_yir.yir](demos/hello_yir.yir)
  smallest cross-domain example
* [window_controls_demo.yir](demos/window_controls_demo.yir)
  current main `cpu + data + shader` control/render demo
* [shader_bindings_demo.yir](shader/shader_bindings_demo.yir)
  current shader resource/binding geometry path
* [shader_external_handle_bridge_probe.yir](shader/shader_external_handle_bridge_probe.yir)
  comment-only sketch probe for future render-side bridge vocabulary
* [kernel_auto_broadcast_demo.yir](kernel/kernel_auto_broadcast_demo.yir)
  current kernel tensor/broadcast path
* [data_fabric_demo.yir](data/data_fabric_demo.yir)
  current typed Fabric/data surface
* [data_external_handle_bridge_probe.yir](data/data_external_handle_bridge_probe.yir)
  comment-only sketch probe for future Fabric-side external-handle bridge work
* [cpu_task_external_handle_bridge_probe.yir](cpu/cpu_task_external_handle_bridge_probe.yir)
  comment-only sketch probe for future task-external-handle bridge vocabulary

Use:

```bash
cargo run -p yir-run -- examples/yir/demos/hello_yir.yir
cargo run -p yir-run -- examples/yir/demos/window_controls_demo.yir
cargo run -p yir-pack-aot -- examples/yir/demos/window_controls_demo.yir "$TMPDIR/window_controls_demo_yir" 4
```

Also useful while comparing handwritten graphs against the `.ns` pipeline:

```bash
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/kernel_tensor_demo
```

Current reading guidance:

* prefer these files when you want to understand `YIR` graph shape directly
* prefer `examples/projects/*` when you want the current canonical end-to-end workflow
* use comment-only sketch probes when you want to discuss future `GLM`/bridge
  directions without pretending the parser or verifier already understands them
* use the invalid examples to understand verifier boundaries, especially for memory, data-plane, and ownership-sensitive paths

When you want a native artifact from a handwritten graph, rebuild it into a
local output directory such as `$TMPDIR/window_controls_demo_yir`.
