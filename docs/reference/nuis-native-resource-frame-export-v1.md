# Native Resource Frame Export V1

Follow-up: [mixed scalar/resource State](nuis-native-mixed-resource-state-v1.md)
adds Nuis-owned frame dimensions and control through a distinct callback contract.
The active/65 checkpoint below remains historical; the resource cell is now active/75.

This beta-0.18.9 worktree follow-up adds a bounded CPU canvas consumer of the
[native Buffer/Bytes artifact profile](nuis-native-resource-session-artifacts-v1.md).
Nuis callbacks paint pixels. The host checks the final accepted State and writes
PPM only after successful close. The image is not synthesized by a Rust callback
or obtained from embedded-YIR execution.

The resource coordinate advances from active/60 to active/65, a scoped engineering
label rather than a measured completion percentage. Native CPU frame dispatch
remains early/10; the other 36 tensor coordinates retain their existing evidence
and scores. The selected prerequisite remains native resource callbacks.

## Runnable Caller

The [native canvas project](../../examples/projects/domains/native_canvas_session_demo)
contains all application callbacks in `main.ns`:

* `canvas_open` writes the initial mark and returns an owned full-buffer copy.
* `canvas_event` decodes one index/color command, updates that pixel, and returns
  the full buffer. The event script supplies iteration; this does not add loops
  or helper calls to resource callback admission.
* `canvas_close` returns the last accepted canvas unchanged.

Its manifest registers these roots under `canvas`, using the existing
`native-resource-session-aot-bundle:canvas` mode. There is no ns-nova-specific
compiler branch, shader registration, provider sidecar or new application C/ObjC
source. The existing Rust resource host and allocation compatibility ABI remain.

From the repository root:

```sh
cargo run -p nuis -- build examples/projects/domains/native_canvas_session_demo target/native-canvas
cargo run -p nuis -- run-artifact target/native-canvas \
  --resource-session canvas --buffer-words 4278190080,4278190080,4278190080,4278190080 \
  --open-arg 4278190335 --event-arg 8573222656 --event-arg 12884836352 \
  --close-arg 0 --resource-frame 2x2 --resource-frame-output target/native-canvas.ppm
```

Use a new destination on each run. This produces red, green, blue and black
pixels, in row-major order. The demo command encodes
`index * 4294967296 + unsigned_color`; negative commands are not a public drawing
operation. The general resource callback argument remains ordinary i64.

## Presentation Contract

The optional bundle field is:

```text
native_resource_frame_contract=nuis-native-resource-frame-rgba8-v1
```

It is a consumer capability, not a new callback ABI or authority over resources.
Existing resource-only artifacts without this field remain valid for their old
scripts. The launcher requires exactly one matching capability before admitting
frame options; duplicate, unknown-version and unexpected resource claims fail
checkpoint validation. The field and generated binary remain covered by the
ordinary artifact identity checks. These checks establish consistency, not trust
or authentication of an untrusted producer.

`--resource-frame WIDTHxHEIGHT` and `--resource-frame-output PATH` must appear
together, once each. They are distinct from embedded-YIR `--export-frame` and
cannot be combined with that route, window dispatch or provider draining.

The frame view has these rules:

* Each i64 cell must contain an unsigned 32-bit `0xAABBGGRR` pixel. Negative or
  larger values are rejected, not wrapped, clamped or silently truncated.
* Positive canonical dimensions must exactly cover the entire Buffer, with no
  multiplication overflow and at most 4096 cells. There is no padding, repetition
  or inference of missing pixels.
* Byte extraction is explicitly R/G/B/A, independent of host endianness. PPM
  stores RGB only; alpha stays in accepted native State, without compositing.
* Complete script syntax, dimensions, initial pixels and destination existence
  are checked before open. The existing limit of 256 events is unchanged.
* Only the final successfully closed State is converted. Intermediate states
  are not exported or independently accepted as images. A temporarily invalid
  pixel may be repaired before close; an invalid final pixel fails export.
* A failed event/close or incomplete session cannot export. Native traps remain
  process failures, not recoverable callbacks with invented cleanup guarantees.
* The output is created exclusively after successful pixel conversion. Existing
  files and symlinks are not overwritten, including a file created during render.
  The path is not reserved during callbacks; a later filesystem error still
  means failed export, without rollback of already completed callbacks.

On success, diagnostics include the frame contract, execution mode
`native-cpu-resource-session`, and then `native_resource_completed=1`.
That completion is synchronous CPU completion, not a device fence.

## Validation

`tools/nuis/tests/native_resource_session_workflow/frame.rs` builds the checked-in
Nuis canvas and checks each native State against separate reference executions.
It also compares all output bytes against an independent 2x2 pixel oracle,
including a second write to the same pixel. Reference evaluation is test-only;
it is not part of the artifact launch path.

The workflow covers bad dimensions/colors/options, pre-existing destinations,
dangling symlinks on Unix, late script errors, invalid final colors, an out-of-range
native write, 256/257-event boundaries, optional/duplicate/incorrect capability
claims, fresh/cache launch, NSB relocation and a binary-only launch with empty
PATH after original source files and build output are removed. The previous
resource workflow continues to cover full-width non-pixel State and tampering.

Resource library tests additionally require incomplete and faulted-then-closed
sessions to refuse frame export, preserving their original completion error.
Shared frame-writer regressions retain no-clobber, UTF-8 path, failed-lifecycle
and destination-created-during-render coverage.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_resource_session_workflow -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_resource_session -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p yir-runtime-host --lib -- native_resource_session frame_export --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --bin nuis -- dev_tensor mainline resource_scripts --test-threads=1
```

No fresh Linux/GPU execution is claimed. The local native workflow is checked on
the current macOS host; hardware absence does not become a successful parity
claim. These tests are correctness evidence, not a performance benchmark.

Local validation passed: two resource artifact workflows, 14 resource library
tests, 12 frame/native-host tests, the retained scalar artifact workflow, 52
tensor/mainline/CLI tests, 40 repository script tests and both file-budget tests.
Live status reports 37 cells and 4531 clean drift checks. The other 36 cells'
serialized content matches the pre-change digest. Architecture, UTF-8, local
documentation links and host-absolute-path checks also pass.
Targeted Clippy completes with existing lowering/runtime/debugger warnings; this
receipt does not claim a warning-free workspace.

## Remaining Boundaries

This is final-state CPU image export, not interactive ns-nova rendering or a
replacement for the existing embedded-YIR Metal showcase. Mixed scalar/resource
State, application-owned dimensions, resource helper/control support, resizing,
cancellation and provider/device completion/retirement remain open. Current
finalization still uses Clang and the existing blob shim, not pure Nsld or an
all-Nuis host runtime.

Next, use this concrete caller to establish mixed scalar/resource State and its
frame handoff without coupling the compiler to ns-nova. Keep exact pixels and
source-free delivery as regression gates while extending admission only where
the caller needs it. Self-hosting remains gradual, not a claim made by this export.
