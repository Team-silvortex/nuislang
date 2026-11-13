# Native Canvas Session

A bounded CPU-only canvas, not the GPU image showcase or an interactive window.
The Nuis callbacks modify packed RGBA8 pixels in persistent Buffer/Bytes State.
The host only validates the presentation view and serializes the final RGB image
after successful close. No callback is interpreted or implemented in C/Rust.

From the repository root, with the host Clang toolchain available:

```sh
cargo run -p nuis -- build examples/projects/domains/native_canvas_session_demo target/native-canvas
cargo run -p nuis -- run-artifact target/native-canvas \
  --resource-session canvas --buffer-words 4278190080,4278190080,4278190080,4278190080 \
  --open-arg 4278190335 --event-arg 8573222656 --event-arg 12884836352 \
  --close-arg 0 --resource-frame 2x2 --resource-frame-output target/native-canvas.ppm
```

This paints red, green and blue pixels and leaves the fourth black. Colors are
unsigned `0xAABBGGRR` values stored in i64 cells. An event packs the linear pixel
index into its upper 32 bits and color into its lower 32 bits. PPM discards alpha;
the native State retains it. Dimensions must exactly cover all cells, with at
most 4096 pixels. Output must be a new file; an existing path is never overwritten.
Malformed dimensions, initial colors or scripts fail before open. Invalid final
colors, failed close or native traps produce no frame. Traps terminate the process.

The same options work on the native executable alone and after NSB materialization.
No source checkout is needed at launch. This does not claim a GPU fence, native
ns-nova window dispatch, asynchronous ownership, resource loops or pure Nsld linking.

See [the resource artifact contract](../../../../docs/reference/nuis-native-resource-session-artifacts-v1.md).
