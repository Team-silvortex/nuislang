# Native Guarded Canvas

Nuis owns the Canvas dimensions, event count, cursor and Bytes pixels. The
[source](main.ns) returns `Result<Canvas, i64>` through the registered
open/event/close callbacks in [nuis.toml](nuis.toml).

This is the current native CPU ownership example: a heartbeat can return a held
snapshot early or preserve it across an outer branch that does not itself
return. The nonempty paint arm can also return early or continue. Both surviving
paths reach one shared function suffix, not duplicated copies. The host transports
State and exports the final image; it does not
implement the Nuis canvas state machine.

Both surviving arms use a local `width` for total snapshot words. The heartbeat
uses inference; the paint arm and event dimensions use the chained `Dimension`
alias. Shared type resolution confirms i64. Each shadow stays in its branch:
the shared suffix still returns canvas width 3 rather than the local word count 6.

## Run

From the repository root with Rust and the host Clang toolchain:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo run --locked -p nuis -- build examples/projects/domains/native_guarded_canvas_session_demo target/guarded-canvas
cargo run --locked -p nuis -- run-artifact target/guarded-canvas \
  --resource-session canvas \
  --buffer-words 4278190080,4278190080,4278190080,4278190080,4278190080,4278190080 \
  --open-arg 3 --event-arg 0 --event-arg 4278190335 --event-arg 0 \
  --event-arg 4278255360 --event-arg 0 \
  --close-arg 0 --resource-frame-fields width,height \
  --resource-frame-output target/guarded-canvas.ppm
```

Use a fresh output image path for each run. Expected result: a 3x2 PPM image
with a red first pixel, a green second pixel and four black pixels. The final
scalar State is `[width=3, height=2, events=5, cursor=2]`.

| Event | Path |
| --- | --- |
| First `0` | Heartbeat at cursor zero returns the held snapshot early; the suffix is skipped. |
| `4278190335` | The else arm paints red at cursor zero and returns early. |
| Second `0` | Heartbeat at cursor one keeps its snapshot across the nonterminal outer arm, then returns through the shared suffix. |
| `4278255360` | The else arm at cursor one reaches the shared suffix, paints green and advances. |
| Third `0` | Heartbeat at cursor two again preserves its snapshot through the shared suffix. |

Zero is reserved for heartbeat; this example cannot use it as a paint color.
Validation codes 1 through 7 match the [checked canvas](../native_checked_canvas_session_demo/README.md).
Code 8 reports a held-snapshot size inconsistent with width times height.
These codes are example-local. Failure preserves previously accepted State and
prevents image export, even if close subsequently succeeds.

## Scope And Verification

The [delivery regression](../../../../tools/nuis/tests/native_resource_session_workflow/early_ok.rs)
checks the exact pixels and scalar State after cache reuse, source-free NSB
relocation and detached binary launch with empty PATH:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p nuis --test native_resource_session_workflow -j1 -- shared_resource_continuation --test-threads=1
```

The [resolved capture proof](../../../../docs/reference/nuis-native-resource-inferred-shadowing-v1.md)
owns the current scope and links the resource/helper bounds. Captured writes, resource-changing joins
and general resource loops remain unsupported here.
This example is fixed-extent synchronous CPU execution, not GPU rendering,
native ns-nova window integration, pure Nsld finalization or an all-Nuis runtime.
