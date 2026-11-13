# Native Mixed-State Canvas

`main.ns` owns a `Canvas` record containing `width`, `height`, `events`, `cursor`
and an owned `Bytes` pixel resource. Open takes an initial width request and
computes height from the supplied Buffer. Events paint the current cursor,
advance it with wraparound, and increment the Nuis event counter. The host does
not implement those updates or store control values in hidden pixel cells.

From the repository root, using the host Clang toolchain:

```sh
cargo run -p nuis -- build examples/projects/domains/native_mixed_canvas_session_demo target/mixed-canvas
cargo run -p nuis -- run-artifact target/mixed-canvas \
  --resource-session canvas \
  --buffer-words 4278190080,4278190080,4278190080,4278190080,4278190080,4278190080 \
  --open-arg 3 --event-arg 4278190335 --event-arg 4278255360 \
  --close-arg 0 --resource-frame-fields width,height \
  --resource-frame-output target/mixed-canvas.ppm
```

The result is a 3x2 image with red and green pixels followed by black pixels.
The accepted close State has `events=2` and `cursor=2`. Changing the open request
to `2` produces 2x3 using the same binary; no export dimensions are supplied.
`--resource-frame-fields` selects registered scalar fields, not dimension values.
The mixed profile rejects the old `--resource-frame WIDTHxHEIGHT` override.

Only a successful complete close can export. Use a new output path each time.
Dimensions must be positive and cover all pixels exactly. This straight-line
demo does not yet return a typed configuration error: zero width traps, and a
non-dividing width fails final image validation. Neither produces an image.

The contract currently admits 1..=16 flat i64 fields plus one owned Bytes field.
Event/close State parameters follow the scalar field declaration order and names;
the final parameter is the per-call argument. Buffer is borrowed only for that
call. Scalar fields and the returned resource are accepted together or neither.

The same script works on the executable alone and after source-free NSB
materialization. This is synchronous CPU execution, not GPU dispatch, an
interactive ns-nova window, resizing or recoverable native traps.

See [the mixed State contract](../../../../docs/reference/nuis-native-mixed-resource-state-v1.md).
