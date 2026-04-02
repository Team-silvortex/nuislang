# `.ns` Examples

This folder contains the current front-end examples for:

* `mod <domain> <unit>` parsing
* `AST -> NIR -> YIR` lowering
* lazy `nustar` binding through `nuis / nuisc`

Recommended starting points:

* [hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/hello_world.ns)
  minimal `mod cpu Main`
* [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/hello_ref_struct.ns)
  `struct` plus `ref` fields
* [hello_data.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/hello_data.ns)
  first front-end `data` link surface
* [hello_instantiate.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/hello_instantiate.ns)
  `cpu`-side instantiation of another domain unit
* [hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/hello_ffi.ns)
  first `extern "nurs" interface` CPU-side host bridge example
* [hello_c_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/hello_c_ffi.ns)
  same minimal host bridge path through explicit `extern "c"`
* [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/window_controls_demo.ns)
  current front-end `cpu + data + shader` real-time control/render demo, now with `extern "nurs" interface` host curve hooks

Use:

```bash
cargo run -p nuis -- dump-ast examples/ns/hello_world.ns
cargo run -p nuis -- dump-nir examples/ns/hello_ref_struct.ns
cargo run -p nuis -- dump-yir examples/ns/hello_data.ns
cargo run -p nuis -- build examples/ns/hello_instantiate.ns /tmp/nuis_hello_instantiate
cargo run -p nuis -- build examples/ns/window_controls_demo.ns examples/bins/window_controls_demo_ns
```
