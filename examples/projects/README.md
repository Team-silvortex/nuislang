# `nuis` Projects

This folder contains multi-file `nuis` project examples driven by `nuis.toml`.

Recommended starting point:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  three-file real-time ball demo:
  `main.ns`, `surface_shader.ns`, `fabric_plane.ns`

Use:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
```
