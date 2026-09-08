# Headless Application Scalar Script

Contract: `nuis-yir-application-scalar-script-v1`.

This is a bounded, non-window ingress profile over `ApplicationEventPump`.
It selects an embedded application registration and drives its Nuis open/event/close
callbacks. It neither uses `WindowSession` nor implements application state,
image algorithms, provider selection or device cleanup in a new host adapter.

## Packaging And Input

`yir-pack-aot <module.yir> <output-dir> [frame-scale] --headless` requires at
least one registered application session. The output contains an executable,
embedded YIR and the existing statically linked host runtime. Its generated C
source is only a process-entry bridge to `nuis_application_script_main`.
No AppKit/Objective-C host or prerendered fallback frame is selected.
Neither the standalone packager nor ordinary headless `nuis build` emits or
advertises unused CPU LLVM IR/shims, or claims AppKit's affinity worker. The compiler
selects an explicit verified-YIR checkpoint: frontend/NIR checks, registered Nustar
semantics, project links, ABI, application registration and FFI ownership checks
all precede packaging. CPU LLVM is not requested; failure is never caught and
converted into an empty intermediate. Native/window builds still require a real
LLVM checkpoint. Native lowering remains a separate path; an unbound
scalar parameter there now produces a function-context diagnostic, not a panic.

The bundle declares:

```text
cpu_host_binary_mode=embedded_yir_headless
runtime_bootstrap_mode=embedded_yir_session
application_script_contract=nuis-yir-application-scalar-script-v1
application_provider_drain_contract=nuis-yir-provider-session-drain-v1
application_yir_fnv1a64=<embedded-source-content-hash>
```

The process accepts:

- `--application-session ID`: required embedded registration ID.
- `--open-args I64,...`: required; an empty value means no arguments.
- Repeated `--event-args I64,...`: zero to 64 ordered event deliveries.
- `--close-args I64,...`: explicitly call close, then await provider Finish.
- Alternatively, `--cancel-after-events`: abandon without invoking close.
- `--drain-provider`: only with cancellation; observe explicit provider drain.

Each argument list has at most 16 values. The whole script has one 180-second
deadline, not a fresh budget per callback. IDs are 1..128 UTF-8 bytes; argv entries
are bounded to 512 bytes. Duplicate, unknown, excess, malformed and contradictory
options fail before session startup. All open/event/close arguments are checked
against the registered application signature before any callback is admitted.
This profile supports i64 ingress, not arbitrary
strings, pointer arguments, interactive stdin or a general scripting language.

## Termination And Ownership

There is only one outstanding operation and one consumed reply at a time.
The host observes accepted states and trace frames without retaining a history.
RGBA frames report byte count and content hash; they are observations, not a
success artifact or authority to reuse resources.

Close yields success only after lifecycle validation and provider Finish.
Cancellation occurs only after the preceding replies passed validation, drops
the pump and waits on its independent retirement ticket. It does not synthesize
Nuis close, a terminal application outcome or parent delivery. Host-only retirement
is not provider retirement. The headless and AppKit adapters use the same safe
receipt-to-exit classifier: an affirmed drain without a latched fault maps to 130,
not success. Replay-only, failed, missing or mismatched drain evidence maps to 1.
The supervising provider launch policy must also validate its typed Drained
terminal; neither exit130 nor EOF proves retirement.

A callback error or timeout aborts the script before its requested terminal
operation. There is no automatic cleanup, retry, reconnect, budget reset or Finish
fallback. Already admitted effects are not rolled back. An in-flight worker can
outlive a timeout; the bounded host wait does not preempt FFI or device work.

Exactly one nonempty provider source must be explicit:
`NUIS_YIR_PROVIDER_DISPATCH_SOCKET` for the current Unix IPC adapter, or
`NUIS_YIR_PROVIDER_RESULT_STREAM` for identity-bound replay. Neither source,
both sources, or an empty path fails. No reference-device fallback is selected.

## Ordinary Build And Launch

Select `--packaging-mode headless-aot-bundle` on `nuis build`, or the same
`packaging_mode` in the project manifest. It actually invokes the headless
packager; it does not relabel a window binary. Explicit packaging/CPU target
overrides participate in the compile-cache identity. A restored manifest must
match the requested profile; default builds cannot adopt a headless cache entry.
Project manifest selections already participate through the project fingerprint.

`nuis run-artifact` accepts the same scalar-script arguments as the binary.
It requires a verified headless build manifest and compiled artifact, one
hash-bound bundle/YIR/binary entry each, exact capability versions and a matching
executable image. A different binary in the same directory is not admitted.
The YIR content hash and all scripted signatures are checked before provider
preparation. These are consistency checks, not cryptographic publisher trust or
protection against concurrent hostile filesystem replacement.

The manifest's `nuis-headless-build-inputs-v2` section declares
`headless_compiler_checkpoint = "verified-yir-v1"` and carries exact bundle,
source, tokens, AST, NIR, YIR and compiler-stage-handoff bytes as hex. Bundle and
handoff are capped at 1 MiB each, source at 16 MiB, and each other input at 32 MiB;
the aggregate decoded limit is 64 MiB. Independent artifact verification and
relocation restore these inputs, verify hashes and the source-to-YIR SHA-256 chain,
check canonical projections and source/token consistency, and reject missing,
duplicate, changed inputs or an LLVM artifact claim. Rebuild older v1 headless
artifacts; an incompatible cached manifest triggers a fresh build.
They never rebuild capability declarations from the current installation or read
the original source directory. Failed verification also removes its owned
temporary directory.

Window, parent-window, JSON and frame-export options cannot be mixed with an
application script. A headless launch without a script is rejected. Live
frontdoor cancellation requires `--drain-provider`: host-only retirement remains
a direct-entry facility, not a substituted live provider receipt. Normal close
and typed non-success drain share the existing prepared-provider lifecycle and
180-second supervisor bound. Cancellation never publishes successful launch,
replay or trace replacement evidence.

## Reproduce

Run from the repository root with one Cargo job. These tests exercise a compiled
headless process with a protocol peer, without needing GPU hardware:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-pack-aot --test headless_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --test provider_application_session script:: -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test headless_checkpoint -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test checkpoint_inspection -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test checkpoint_workflow -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib aot_application_bundle -j 1
```

The separate real M2 Metal test builds the Nuis image showcase through the ordinary
headless build route, checks same-profile cache restoration and rejects tampered
capabilities, mismatched binaries and invalid scripts before provider work. Direct
and frontdoor launches verify normal Finish, one/two-frame typed drain, worker
removal and preservation of prior success evidence. The binary's GPU frames have
exact RGBA hashes, no AppKit dependency, and it rejects replay-only drain:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis headless_metal_session_uses_shared_lifecycle_without_appkit -j 1
```

For a live M2 Metal launch from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- build examples/projects/domains/ns_nova_image_showcase build/ns-nova-headless --packaging-mode headless-aot-bundle
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- run-artifact build/ns-nova-headless --application-session window --open-args 160,120 --event-args 0,0 --event-args 1,32 --close-args 1,0
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- run-artifact build/ns-nova-headless --application-session window --open-args 160,120 --event-args 0,0 --cancel-after-events --drain-provider
```

The final command intentionally returns 130, not success. `window` is the sample's
registered ID; the headless launcher does not interpret it as a window policy.

For a manual replay, first build the
[image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md)
and complete its `--window-events 32` route, producing two identity-matched frames.
Then package the same YIR and replay those callbacks without a window:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p yir-pack-aot -- build/ns-nova-image/ns_nova_image_showcase.yir build/ns-nova-headless --headless
env -u NUIS_YIR_PROVIDER_DISPATCH_SOCKET NUIS_YIR_PROVIDER_RESULT_STREAM=build/ns-nova-image/nuis.runtime.provider-result-stream.toml \
  build/ns-nova-headless/ns_nova_image_showcase --application-session window \
  --open-args 160,120 --event-args 0,0 --event-args 1,32 --close-args 1,0
```

## Source Inspection

Set `packaging_mode = "headless-aot-bundle"` in `nuis.toml` to select the
verified-YIR checkpoint for `nuis check`, `nuis dump-yir`, the `nuisc` AST/NIR/YIR
dumps, benchmark/binding inspection and `nuis workflow --json`. Both a project
directory and its manifest file select this profile. A bare `.ns` input, an
unmarked project, and native/window/self-contained-image profiles retain the
existing LLVM path. A build-only `--packaging-mode` override does not persist into
later inspection commands. Unknown packaging profiles fail rather than being
treated as native defaults.

The selection occurs before compilation; no LLVM error is caught as a headless
fallback. Input resolution, NIR checks, registered YIR semantics, ownership and
application-session registration are still required. Dumps report the checkpoint
on stderr so their stdout remains parseable. Inspection creates no build outputs.

Workflow JSON exposes `compile_pipeline_checkpoint = "verified-yir"`,
`llvm_emit` stage status `not_requested`, and
`compile_pipeline_llvm_ir_bytes = null`. The native AOT heuristic
`compile_pipeline_ready_for_aot` remains false, with `build_headless` as the next
step. Native reports retain their real LLVM byte counts. Neither kind of source
inspection certifies a runnable image; artifact hash/identity checks and provider
admission remain separate and unchanged.

## Bounded Buffer Loops

The [callback fixture](../../tools/nuisc/tests/fixtures/buffer_while.ns) allocates a
small `ref Buffer`, fills it in a counted loop, reads the result and frees the
buffer before returning scalar application state. Supported inline loop bodies
contain scalar temporary bindings and ordered `store_at`/`load_at` operations.
Scalar integer division and remainder are supported too: constant evaluation
uses checked operations, registered execution reports invalid divisors/overflow,
and native scalar lowering traps before division by zero or `MIN / -1` (also
`MIN % -1`). This does not certify all specialized loop arithmetic opcodes.
Induction is `i64`, with strict `<` and `+ 1`, or strict `>` and `- 1`, and an
invariant scalar bound. These combinations reach the bound without induction
overflow; they are not a general loop-termination proof.

Lowering outlines the body to a collision-safe private helper, revalidates NIR,
and uses the existing `cpu.loop_while_i64_effect ... cpu scoped_call` contract.
Borrowed captures retain GLM lifetime edges. CPU Nustar owns the resumable loop;
the generic executor invokes registered functions with the same invocation fuel.
Buffer reads/writes carry effect edges at the memory operation, including reads
nested in a larger expression, and helper returns follow their effects. Native
`cpu.load_at`/`cpu.store_at` require length metadata and trap on negative or
out-of-range indices; reference execution reports an error instead of accessing memory.
Buffer owner selection merges the pointer and length from the same branch, so a
subsequent loop capture cannot inherit the other candidate's bounds.
This is not a proof of all allocation, pointer or FFI memory safety.

The [regressions](../../tools/nuisc/tests/buffer_while.rs) compare reference/native
results, repeated application callbacks, zero/one/descending loops, per-iteration
read/write order, declaration-order independence, failed-state admission and shared
fuel. Allocation, ownership transfer, nested control flow or captured-state
rebinding inside an outlined body remain unsupported. General steps and mutable
header reads are not silently hoisted.

### Packaged Pixel Loop

[PixelMagic's generator](../../stdlib/pixelmagic/lib/pixels.ns) now uses this
straight-line loop instead of recursive pixel filling. The
[native/reference test](../../tools/nuisc/tests/pixelmagic_buffer_loop.rs) checks
every pixel for both 32x24 phases, a partial region and an empty region.
The [CLI regression](../../tools/nuis/tests/headless_image_loop.rs) builds the
ordinary image showcase with `headless-aot-bundle` and launches it through
`run-artifact`. On M2 it verifies two actual Metal output hashes, all output bytes
against direct-session replay, and all 768 writes plus the post-snapshot mutation
per callback. Wrong callback arguments or executable/YIR drift are rejected before
application effects. Exhausted packaged replay produces neither Close nor success,
and preserves the earlier replay stream.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test buffer_while --test pixelmagic_buffer_loop -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test headless_image_loop -j 1 -- --test-threads=1
```

The second command is a macOS Metal device regression, not Linux/Windows evidence.
CPU callbacks in the packaged host still execute embedded YIR; the native parity
test independently compiles the generator, not the complete live callback ABI.
Conditional pixel transforms with branch-local effects are the next boundary.

## Current Boundary

Ordinary `nuis build` and `nuis run-artifact` now select and admit this profile.
Headless build selection removes the unused CPU LLVM prerequisite without
fabricating an empty intermediate or weakening the verified stage handoff.
Checkpoint-aware source inspection now follows the explicit manifest selection.
Bounded Buffer-writing callbacks now pass the lowering/execution boundary above;
they still need packaged build/run-artifact evidence in the image workflow.
Compound-condition `while` callbacks now pass the verified-YIR boundary and execute
their current/carry state rather than returning trace-only unit values. CPU Nustar
owns the scalar loop state machine; the generic reference executor only drives
registered resumes and existing YIR function calls, sharing invocation fuel.
Synchronous and awaited steps, pre/post-update control, `&&`/`||`, mixed break/continue
and add/keep carries have reference/native parity tests. Registered callback tests
include zero iterations and untaken conditions. Arbitrary loop bodies, payload-bearing
carries and mixed-action `flow_and` descriptors are not certified by this fix.
Reference execution also has a 100000-resume per-node ceiling when no session fuel
is supplied; that ceiling is not a native ABI or native loop limit.
Standalone artifact verification preserves all compiler inputs; a relocated full
device launch still needs its provider environment and separate execution evidence.

This is embedded-YIR execution, not fully native CPU callback lowering or a
self-contained Nsld application image. The registered live provider remains an
external execution scope. macOS and Linux have host build branches; the recorded
real hardware evidence is M2 Metal, not Linux GPU or Windows transport certification.
Parent cancellation, arbitrary resource reuse and recovery remain separate work.

Related contracts: [application session](nuis-yir-application-session-v1.md),
[cancellation](nuis-yir-application-cancellation-v1.md),
[provider drain](nuis-yir-provider-session-drain-v1.md).
