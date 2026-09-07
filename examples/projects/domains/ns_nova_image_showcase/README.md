# NS Nova Image Showcase

Nuis owns image generation, inline GPU image processing, and the three-frame
ns-nova application lifecycle. This complements, rather than replaces, the
small-uniform regression in `../ns_nova_showcase`.
An explicit registered window mode now keeps the same Nuis state and provider
connection across host events instead of replaying that three-frame program.

## Data Path

1. `PixelMagicPixels.fill_checkerboard` fills a 32x24 image using packed RGBA8
   integer pixels. The Nuis implementation uses bounded divide-and-conquer
   recursion; general buffer-writing `while` lowering is still incomplete.
2. `copy_bytes` creates an owned snapshot. The app overwrites the original first
   pixel and frees the original Buffer before binding the snapshot.
3. `shader_storage_binding(3, snapshot)` requests one immutable u32 array.
   Shader checks each snapshot element fits u32 and serializes little-endian
   words. Nuis `Bytes` currently stores i64 Buffer elements, so this is an
   explicit checked conversion, not a reinterpretation of host memory.
4. The 3,072-byte payload travels outside the 256-byte argument field in bounded
   IPC v3. A registered inherited-FD carrier forwards it to Metal; application
   paths and pointers cannot select device resources.
5. `PixelMagicImageSurface` reads the array and inverts RGB in inline WGSL.
   Generated MSL bounds-checks array reads, returning zero outside the array.
   Reflection checks the exact read-only u32 array length and slot before upload.
6. Full/clear/full GPU frames return through ns-nova completion, presentation,
   and commit. The first and third images invert complementary checkerboards.

## Verify

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 cargo run -q -p nuis -j 1 -- check examples/projects/domains/ns_nova_image_showcase
CARGO_INCREMENTAL=0 NUIS_TEST_QUIET_SUCCESS_LOGS=1 cargo test -q -p nuis --bin nuis artifact_device_sample_shader_render -j 1 -- --test-threads=1
```

## Run The Compiled Artifact

On an Apple Silicon host with the registered Metal adapter, run from the repository
root. `CARGO_BUILD_JOBS=1` also bounds the packer's nested runtime build:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- build examples/projects/domains/ns_nova_image_showcase build/ns-nova-image
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- run-artifact --export-frame build/ns-nova-image/result.ppm build/ns-nova-image
```

The executable runs one complete Nuis lifecycle without opening a window and
exports the last presented frame as native-resolution 160x120 binary PPM.
All three GPU dispatches, completion gates and GLM releases still execute.
Startup derives missing provider registration from the verified artifact plan;
it does not require a preceding `--json` inspection and does not overwrite an
existing registration to hide invalid evidence.
The output path must be new; use another filename for a subsequent export.
`--json` is inspection-only and cannot be combined with this execution option.
Missing or ambiguous IPC/replay sources fail rather than using reference pixels.

The device regression now launches this compiled binary through `run-artifact`,
not a test executable. It checks every GPU pixel
against the image algorithm, live/replay byte equality, compiled binding authority,
physical completion, graph release, and single-worker executable-cache reuse.
Replay stores input descriptors, not another image payload; changed input content
or layout must not consume an old frame. The same binary also exports identity-checked
replay with the external `.yir` file temporarily absent, proving embedded execution.
Failed dispatch does not create a partial output; existing files are not overwritten.

## Registered Window

After the same build, launch the event-driven path explicitly:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- run-artifact --window-session window build/ns-nova-image
# Initial redraw, space, one ignored non-BMP logical input, then explicit close:
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- run-artifact --window-session window --window-events 32,128578 build/ns-nova-image
```

Space toggles the GPU-processed checkerboard. The window's close button waits for
Nuis close and the provider acknowledgement. Scripted input traverses the actual
AppKit event queue as logical events, not synthesized physical keys. No application
names, fields or image policy are embedded in the generic host adapter.
Without `--window-session`, the old preview route is unchanged. This first window
profile still has the provider's 256-dispatch and 64 MiB replay limits. It can wait
idle between inputs without reconnecting or resetting clocks/budgets; an incoming
request has a 120-second deadline from its first byte, with separate reply/write
timeouts. It does not support general IME, pointer input or unlimited dispatches.
Replay capacity is reserved from the registered output extent before GPU work.
Loading checks the complete 64 MiB aggregate and bounded files before consuming
pixels; invalid replacement evidence no longer erases a prior valid replay.
The live session test includes idle gaps longer than an injected short request
deadline, retaining the same Metal worker and clock lineage. This is not yet a
long-duration window soak test or peer-failure recovery mechanism.
Window profile v3 calls `window_close(state, reason: i64, failure: i64)`. Nuis decodes
close intent and the separately observed failure category, saves the first fault
in `NovaAppState.failure_kind`, and marks failed status without replaying a frame
or clearing an earlier failure. Successful cleanup is not successful
execution: the host reports both separately, and late Finish failure still fails
the run without repeating close. Provider failure gets a five-second supervised
cleanup grace, not a retry or fresh budget. Rebuild older v1/v2 bundles, state
artifacts and close helpers together. The compiled-window test injects a provider reader failure and
verifies completed cleanup, failed execution and unchanged prior replay evidence.
It also exhausts a two-frame replay with a third draw in the same compiled binary.
The IPC rejection remains text-only: remote budget/device causes are not guessed
from its wording, and late Finish errors do not re-enter Nuis cleanup.
The bare aggregate-call return guard limitation remains; the event helper uses an
equivalent supported leading guard. See the
[window contract](../../../../docs/reference/nuis-yir-window-session-v3.md).

This is a **native host executable with an embedded YIR lifecycle runtime**, not
fully native CPU lowering, a self-contained Nsld image or a stable interactive app.
`nuis run-artifact` still supplies the registered provider session and artifact
sidecars. The compiler/runtime remain pre-self-host Rust; the existing Metal and
AppKit adapters remain OS compatibility glue, not implementations of the image app.
One fixed-size read-only array, one target, and one render stage pair are admitted.
Texture samplers, mixed bindings, persistent GPU contexts, dynamic extent, and
self-contained host-runner result injection remain open.
