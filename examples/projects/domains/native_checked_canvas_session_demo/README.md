# Native Checked Canvas

Nuis owns Canvas dimensions, event count, cursor and Bytes pixels. All callbacks
return `Result<Canvas, i64>`. The host transports success/error; it does not
implement the dimension validation or canvas state machine.

From the repository root with the host Clang toolchain:

```sh
cargo run -p nuis -- build examples/projects/domains/native_checked_canvas_session_demo target/checked-canvas
cargo run -p nuis -- run-artifact target/checked-canvas \
  --resource-session canvas \
  --buffer-words 4278190080,4278190080,4278190080,4278190080,4278190080,4278190080 \
  --open-arg 3 --event-arg 4278190335 --close-arg 0 \
  --resource-frame-fields width,height --resource-frame-output target/checked-canvas.ppm
```

This produces a 3x2 image with one red pixel. Use a new output path for each run.
Changing the width to `0`, `-1`, `7` or `4` returns a clear Nuis application error,
without publishing State, dividing by zero or creating an image. Dimensions
remain read from the successfully closed Nuis State, not supplied to the exporter.

| Code | Nuis validation |
| --- | --- |
| 1 | Width is not positive. |
| 2 | Width exceeds the pixel buffer length. |
| 3 | Width does not divide the pixel count exactly. |
| 4 | Event color is outside the packed RGBA8 range. |
| 5 | Event cursor is outside the buffer. |
| 6 | Event count would overflow i64. |
| 7 | Close reason is negative. |

These are example-local error meanings, not a global compiler error registry.
Err preserves both accepted scalars and pixels. An event error remains a failed
session even after close succeeds, so no image is exported. The executable also
works without source files, and the NSB artifact can be relocated.

See [the checked resource contract](../../../../docs/reference/nuis-native-checked-resource-state-v1.md)
for bounds and proof scope. This is native CPU execution, not an interactive
window or GPU dispatch; arbitrary branch-local resource ownership remains closed.
