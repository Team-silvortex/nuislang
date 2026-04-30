# `.ns` Examples

This folder contains the current front-end examples for:

* `mod <domain> <unit>` parsing
* `AST -> NIR -> YIR` lowering
* lazy `nustar` binding through `nuis / nuisc`
* current async/task surface at the source-language layer

Subdirectories:

* [core](/Users/Shared/chroot/dev/nuislang/examples/ns/core/README.md)
* [types](/Users/Shared/chroot/dev/nuislang/examples/ns/types/README.md)
* [data](/Users/Shared/chroot/dev/nuislang/examples/ns/data/README.md)
* [ffi](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)
* [memory](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/README.md)
* [demos](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/README.md)

Recommended starting points:

* [hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_world.ns)
  minimal `mod cpu Main`
* [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)
  `struct` plus `ref` fields
* [hello_data.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data.ns)
  first front-end `data` link surface
* [hello_instantiate.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_instantiate.ns)
  `cpu`-side instantiation of another domain unit
* [hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns)
  first `extern "nurs" interface` CPU-side host bridge example
* [hello_c_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_ffi.ns)
  same minimal host bridge path through explicit `extern "c"`
* [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
  current front-end `cpu + data + shader` real-time control/render demo, now with `extern "nurs" interface` host curve hooks
* [projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  current recommended multi-file project form once the demo grows beyond a single source file

Use:

```bash
cargo run -p nuis -- dump-ast examples/ns/core/hello_world.ns
cargo run -p nuis -- dump-nir examples/ns/types/hello_ref_struct.ns
cargo run -p nuis -- dump-yir examples/ns/data/hello_data.ns
cargo run -p nuis -- build examples/ns/data/hello_instantiate.ns /tmp/nuis_hello_instantiate
cargo run -p nuis -- build examples/ns/demos/window_controls_demo.ns examples/bins/window_controls_demo_ns
```

Project route is now preferred once a demo spans multiple domains or needs
real profile/link orchestration:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- project-status examples/projects/window_controls_demo
```

Current source-language notes worth keeping in mind while reading these files:

* `mod` is a top-level builtin declaration, not a nested construct
* `cpu` is currently the only domain that can declare `async fn`
* current explicit task-style async surface is intentionally small:
  `spawn`, `join`, `cancel`, `timeout`, `join_result`, and `task_*`
* cross-domain interaction is expected to route through project links and `mod data`, not direct nested mod definitions
