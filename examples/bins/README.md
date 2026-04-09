# Built Example Bundles

This folder contains generated example outputs, not handwritten source examples.

Current kept bundles:

Canonical current build:

* [window_controls_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
  single-binary bundle built from multi-file project [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* [kernel_tensor_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/kernel_tensor_demo_project/kernel_tensor_demo)
  native bundle built from multi-file project [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)

Compatibility / reference bundles:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo/window_controls_demo)
  single-binary bundle built from handwritten [window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir)
* [window_controls_demo_ns](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_ns/window_controls_demo)
  single-binary bundle built from single-file [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)

Notes:

* These folders may contain generated `.ast.txt`, `.nir.txt`, `.yir`, `.ll`, host stub, and bundle metadata files.
* Project-route bundles are the primary ones to keep aligned with the current `nuis.toml` workflow.
* Real-time window/runtime output is now the preferred bundle mode; prerendered `ppm` assets are treated as fallback/reference artifacts when they still exist.
* Asset files inside a bundle should use the bundle name where possible; older anonymous leftovers such as `main.ppm` are treated as stale and can be cleaned.
* All `nuis build` outputs now emit `nuis.build.manifest.toml` with toolchain/profile info, loaded nustar list, effective project ABI mode, and per-artifact FNV-1a hashes.
* Project builds now also emit `nuis.project.host_ffi.txt` that records the host FFI contract surface used by the project entry.
* Project builds now also emit `nuis.project.abi.txt` that records required ABI profile locks per domain.
* Per-project `.nuis/` compile caches are generated locally and should not be checked into the repository.
