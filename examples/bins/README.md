# Built Example Bundles

This folder contains generated example outputs, not handwritten source examples.

Current kept bundles:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo/window_controls_demo)
  single-binary bundle built from handwritten [window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir)
* [window_controls_demo_ns](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_ns/window_controls_demo)
  single-binary bundle built from single-file [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
* [window_controls_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
  single-binary bundle built from multi-file project [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* [kernel_tensor_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/kernel_tensor_demo_project/kernel_tensor_demo)
  native bundle built from multi-file project [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)

Notes:

* These folders may contain generated `.ast.txt`, `.nir.txt`, `.yir`, `.ll`, host stub, and bundle metadata files.
* Project builds now also emit `nuis.project.host_ffi.txt` that records the host FFI contract surface used by the project entry.
* Stale duplicate artifacts are periodically cleaned; the canonical output binary should match the folder name.
