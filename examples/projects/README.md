# `nuis` Projects

This folder contains multi-file `nuis` project examples driven by `nuis.toml`.
This is the current canonical route for reading real `.ns` programs in this repo.

## What A Project Gives You

Compared with a single `.ns` file, project mode currently adds:

* `nuis.toml` manifest
* multi-file `mod cpu / mod data / mod shader / mod kernel` split
* project-level `links`
* project-level ABI locking or auto-resolution
* project metadata outputs during `build`
* compile-cache identity based on the whole project input set

Current project `links` are not only manifest hints anymore. They are checked
against final `YIR` as real `source -> data -> target` exchange structure.

Projects can lock required `nustar` ABI profiles per domain via:

```toml
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "data=data.fabric.macos.arm64.v1",
  "shader=shader.metal.msl2_4",
]
```

If `abi` is omitted, `nuisc/nuis` now auto-resolve a host-matching ABI set per
involved domain from the `abi_targets` registered by each `nustar` package.

Per-domain lane defaults are also declared by each `nustar` package through
`default_lanes = ["op.name=lane"]`, so project/profile lowering stays mod-owned
and `nuisc` only applies declared policy plus narrow fallback rules.

## Core Commands

Inspect project state:

```bash
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- project-lock-abi examples/projects/window_controls_demo
```

Validate and build:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project

cargo run -p nuis -- check examples/projects/kernel_tensor_demo
cargo run -p nuis -- build examples/projects/kernel_tensor_demo examples/bins/kernel_tensor_demo_project
```

Inspect cache and artifact metadata:

```bash
cargo run -p nuis -- cache-status examples/projects/window_controls_demo
cargo run -p nuis -- verify-build-manifest examples/bins/window_controls_demo_project/nuis.build.manifest.toml
```

Override CPU target when needed:

```bash
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project

cargo run -p nuis -- build --target aarch64-apple-darwin \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project
```

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
  data-plane graph. The data profile markers are now validated per link
  direction, so a `cpu <-> shader` fabric only needs its own sync pair.

Also included:

* [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
  three-file `cpu + data + kernel` demo:
  `main.ns`, `kernel_unit.ns`, `fabric_plane.ns`
  with project links:
  `cpu.Main -> kernel.KernelUnit via data.FabricPlane`
  `kernel.KernelUnit -> cpu.Main via data.FabricPlane`
  and kernel profile slots consumed from CPU via
  `kernel_profile_bind_core/kernel_profile_queue_depth/kernel_profile_batch_lanes`.
  Its `FabricPlane` now only declares the `cpu_to_kernel/kernel_to_cpu` sync
  markers required by that route.

Use:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- dump-nir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
cargo run -p nuis -- check examples/projects/kernel_tensor_demo
cargo run -p nuis -- build examples/projects/kernel_tensor_demo examples/bins/kernel_tensor_demo_project
```

Generated outputs to expect from a project build:

* [examples/bins/window_controls_demo_project/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
* [examples/bins/window_controls_demo_project/nuis.project.host_ffi.txt](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/nuis.project.host_ffi.txt)
  generated host-ffi contract index (abi/interface/symbol/signature) consumed by the project route
* `nuis.project.modules.txt`
  module index emitted by the project route
* `nuis.project.links.txt`
  link index emitted by the project route
* `nuis.project.abi.txt`
  effective ABI lock/auto-resolution summary
* `nuis.build.manifest.toml`
  build manifest including per-domain target/backend details
