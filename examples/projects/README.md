# `nuis` Projects

This folder contains multi-file `nuis` project examples driven by `nuis.toml`.
Projects can now also declare a first project-level `links` list, so instance relations are not only implicit in file layout.
Those links are now checked against final `YIR` as real `source -> data -> target`
exchange structure, not only as loose metadata.
Projects can also lock required Nustar ABI profiles per domain via
`abi = ["cpu=...", "shader=...", "data=...", "kernel=..."]`.
If `abi` is omitted, `nuisc/nuis` now auto-resolve a host-matching ABI set
per involved domain and validate YIR against that effective ABI contract.

Recommended starting point:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  three-file real-time ball demo:
  `main.ns`, `surface_shader.ns`, `fabric_plane.ns`
  with project links:
  `cpu.Main -> shader.SurfaceShader via data.FabricPlane`
  `shader.SurfaceShader -> cpu.Main via data.FabricPlane`
  and per-mod `profile()` hooks in shader/data files that now also emit
  concrete `YIR` setup nodes during project compilation.
  `SurfaceShader` now contributes target/viewport/pipeline plus draw budget constants,
  plus inline WGSL source blocks via:
  `shader_inline_wgsl("entry", wgsl { ... })`
  while `FabricPlane` contributes bind-core, handle table, sync markers, and
  explicit uplink/downlink window policy nodes that are stitched into the final
  data-plane graph.

Also included:

* [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
  three-file `cpu + data + kernel` demo:
  `main.ns`, `kernel_unit.ns`, `fabric_plane.ns`
  with project links:
  `cpu.Main -> kernel.KernelUnit via data.FabricPlane`
  `kernel.KernelUnit -> cpu.Main via data.FabricPlane`
  and kernel profile slots consumed from CPU via
  `kernel_profile_bind_core/kernel_profile_queue_depth/kernel_profile_batch_lanes`.

Use:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
cargo run -p nuis -- check examples/projects/kernel_tensor_demo
cargo run -p nuis -- build examples/projects/kernel_tensor_demo examples/bins/kernel_tensor_demo_project
```

Output bundle:

* [examples/bins/window_controls_demo_project/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
* [examples/bins/window_controls_demo_project/nuis.project.host_ffi.txt](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/nuis.project.host_ffi.txt)
  generated host-ffi contract index (abi/interface/symbol/signature) consumed by the project route
