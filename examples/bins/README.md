# Built Example Bundles

This folder contains generated example outputs, not handwritten source examples.

Current kept bundles:

Canonical current build:

* [window_controls_demo_project](window_controls_demo_project/window_controls_demo)
  single-binary bundle built from multi-file project [window_controls_demo](../../examples/projects/window_controls_demo)
* [kernel_tensor_demo_project](kernel_tensor_demo_project/kernel_tensor_demo)
  native bundle built from multi-file project [kernel_tensor_demo](../../examples/projects/kernel_tensor_demo)

Notes:

* These folders may contain generated `.ast.txt`, `.nir.txt`, `.yir`, `.ll`, host stub, and bundle metadata files.
* Project-route bundles are the primary ones to keep aligned with the current `nuis.toml` workflow.
* Handwritten `YIR` and single-file `.ns` demo bundles should be rebuilt into a local output directory when needed, instead of being kept in-repo as compatibility artifacts.
* Real-time window/runtime output is now the preferred bundle mode; prerendered `ppm` assets are treated as fallback/reference artifacts when they still exist.
* Asset files inside a bundle should use the bundle name where possible; older anonymous leftovers such as `main.ppm` are treated as stale and can be cleaned.
* All `nuis build` outputs now emit `nuis.build.manifest.toml` with toolchain/profile info, loaded nustar list, effective project ABI mode, per-artifact FNV-1a hashes, and CPU target details such as ABI, machine, object format, calling ABI, clang triple, and cross-build flag.
* Project builds now also emit `nuis.project.host_ffi.txt` that records the host FFI contract surface used by the project entry.
* Project builds now also emit `nuis.project.abi.txt` for the resolved ABI entries and record the compact `abi_graph` summary in both `nuis.project.plan.txt` and `nuis.build.manifest.toml`.
* Current `project-status` and `build` output also surface the same ABI graph/target view, including per-domain backend family and whether a selected ABI is host-adaptive.
* Per-project `.nuis/` compile caches are generated locally and should not be checked into the repository.
