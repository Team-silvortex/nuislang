# Registered Window Sessions

`nuis-yir-window-session-v3` is a bounded host adapter over the
[persistent application session](nuis-yir-application-session-v1.md). It is not a
new Nuis application language, a native CPU ABI or a general GUI toolkit.

## Binding And Input

The host explicitly selects an embedded `application-session` ID. It never guesses
an ID or application function name. Before provider admission or Nuis execution,
the worker checks the shared YIR session signature and this host profile:

| Helper | Scalar Arguments After State | Result |
| --- | --- | --- |
| Open | `width: i64`, `height: i64` (no incoming state) | Owned scalar aggregate |
| Event | `kind: i64`, `code: i64` | Same owned scalar aggregate |
| Close | `reason: i64`, `failure: i64` | Same owned scalar aggregate |

Version 3 requires rebuilding the host bundle and changing its registered Nuis
close helper to `close(state, reason: i64, failure: i64)`. Version 1 had no close
arguments; version 2 had only `reason`. Both older bundles are rejected by the
current launcher. The shared application-session v1 contract is unchanged;
the current transport uses IPC v4 and application-failure v2 independently of
this window-profile signature.

Event kinds are `0,0` for redraw and `1,code` for one Unicode scalar. Surrogates,
out-of-range codepoints and unknown kinds are rejected. Dimensions are restricted
to 1..=16384. The Nuis helpers own frame timing, rendering decisions and state;
the host does not inspect Ns-Nova fields. The example registers `window_open`,
`window_event`, and `window_close`: initial redraw renders phase zero, space toggles
the checkerboard, and other key scalars preserve state without drawing.

Each callback may present at most one frame. Close must not present any frame.
Frames require actual RGBA8 pixels; glyph-only surfaces are rejected rather than
rasterized as substitute images. Extent/length checks cap a frame at 16 MiB of
RGBA8. These checks execute in the
worker **before provider finish**, so an invalid close presentation cannot persist
a successful provider lifecycle. The host converts accepted frames to PPM at scale
one, then to its window image. There is no alternate/reference renderer on failure.

## Compiled Host

The packer includes the runtime for statically registered sessions even without a
`cpu.tick_i64` node. Its bundle advertises this optional window contract. The
registered launch mode bypasses both `nuis_yir_entry()` and whole-module timer
execution, starts one event pump, and polls without blocking the GUI thread.
There is one initial redraw, not a 30 Hz graph replay. Raw single-scalar AppKit
key events use the same Nuis ingress; command shortcuts are left to AppKit.
IME composition, multi-scalar text, pointer/resize input and general widgets are
not provided by this first profile.

The C ABI owns an opaque, single-host-thread handle. Source text and ID are copied
before asynchronous startup. The provider environment must select exactly one
IPC or replay path. Buffer outputs must be empty and use the existing runtime
free function. Freeing the handle abandons its scope, never implicitly calling
Nuis close. Applications still run in the Rust bootstrap YIR executor; AppKit is
OS compatibility code, not an alternative implementation of the image program.

Window close/quit defers termination until the existing close/finish gate replies.
The poll timer also runs in the modal run-loop mode used by
[AppKit termination](https://developer.apple.com/documentation/appkit/nsapplication/terminatereply/terminatelater?language=objc).
Script completion schedules the quit request outside the poll timer callback,
avoiding a wait that depends on the same timer firing recursively. Cleanup
preserves failure exit status in `applicationWillTerminate`; it must not rely on
code after `NSApplication.run` being reached. Failed callbacks remain failed even
when cleanup runs, and teardown does not retry a completed/failed close.

## Failure-Aware Close

The shared `nuis-yir-application-close-reason-v1` contract has three codes:

| Code | Reason | Meaning At Close Admission |
| --- | --- | --- |
| `0` | `Requested` | Ordinary host/user close request |
| `1` | `EventFailed` | An event failed, including provider rejection or exhausted dispatch budget |
| `2` | `HostFailed` | The host reports its own failure, such as image decoding |

These are close intents, not successful-completion receipts. An observed event
failure overrides an ordinary or host-failure request. A busy/rejected submission
does not set the reason. Host-reported failure is latched in the worker before
calling Nuis, preventing successful cleanup from authorizing provider Finish or
replacement replay evidence. The host ingress rejects unknown reason codes.

The Nuis `NovaCloseReason` and `NovaFailureKind` enums are consumed by
`NovaAppRuntime.close_with_failure` in the example's `window_close` helper. They
retain the last accepted frame identity, mark status `4` on failure and save the
first fault in `NovaAppState.failure_kind`. Later ordinary cleanup cannot clear
either the status or the first fault; a new fault cannot overwrite the old one.
The std decoder treats unknown codes as failure, not a normal close. No Rust or
AppKit code inspects Ns-Nova's state fields or chooses its failure-state policy.

Pump/window replies report `cleanup_completed` separately from their phase and
trace result. It means the Nuis close callback returned a state and passed the
host trace checks, not that owned resources were drained or execution succeeded.
The C host can query this flag and the admitted reason. `Closed` still requires
successful lifecycle checks and matching provider completion. An event failure,
failed close callback, invalid close presentation or failed Finish stays failed.
If Finish fails after successful Nuis cleanup, the flag stays true, the phase is
`Stopped`, and close is never called again. The original close reason is not
retroactively changed to a failure the callback could not yet observe.

The independent [terminal outcome](nuis-yir-application-outcome-v1.md) snapshots
status, cleanup and failure after the terminal reply. Its readonly runtime/FFI
getters and Nuis `NovaAppOutcome` decoder distinguish `[1,1,0]` success from,
for example, `[2,1,11]` late finalization failure. Reads do not rerun the closed
application or its global initialization. There is no automatic post-close
Nuis observer yet; parent-orchestrator delivery needs its own execution contract.

### Typed Failure Evidence

`nuis-yir-application-failure-v2` is a shared, backend-neutral category contract:

| Code | Kind | Producer Observation |
| --- | --- | --- |
| `0` | `None` | No recorded failure, not proof of successful completion |
| `1` | `Callback` | Nuis helper execution/state validation failed without a prior provider fault |
| `2` | `Host` | Host-reported failure or window presentation-contract rejection |
| `3` | `DispatchLimit` | Local IPC client refused a dispatch at its fixed invocation limit |
| `4` | `ProviderRejected` | Peer rejected request admission at the matching frontier |
| `5` | `ProviderExchange` | Request/reply transport, serialization or framing failed |
| `6` | `ProviderContract` | Provider request/result identity, extent or Finish acknowledgement disagreed |
| `7` | `ReplayExhausted` | An admitted replay queue has no next frame |
| `8` | `Unclassified` | Terminal failure without a more specific producer observation |
| `9` | `ProviderBudget` | Remote output reservation failed before execution |
| `10` | `ProviderExecution` | Remote execution operation failed, not necessarily a device fault |
| `11` | `ProviderFinalization` | Remote provider close or evidence publication failed |

Providers record the kind at the failing operation, before the executor's current
String diagnostic boundary. A scope-owned atomic latch carries that evidence to
the application session and pump; it is not process-global or thread-local state.
The first fault wins, reads do not consume it, and a second session starts empty.
Diagnostic strings are not parsed to select codes. A Request rejection mentioning
"budget" is still `ProviderRejected`, not an inferred `DispatchLimit`.
[IPC v4](nuis-yir-provider-runtime-ipc-v4.md) now carries producer-owned rejection
phase, sequence and code. Only an envelope matching the pending operation can
project its remote category; unknown codes and unrelated responses fail closed.
`ProviderExchange` deliberately groups I/O and wire decoding, rather than claiming
to distinguish errors after the lower-level diagnostic has lost that information.

The window snapshots the observed kind into its close arguments. The C host can
query `nuis_window_session_failure_kind`; terminal replies may reveal a later
Finish failure after Nuis cleanup has already run. That later observation does
not rewrite the earlier Nuis state or authorize a second close. If event execution
and cleanup both fail, the window's terminal diagnostic keeps the original event
error and appends the cleanup error, while retaining the first kind.

The Nuis decoder maps unsupported codes to `Unknown` (stored as `255`), never a
successful state. Legacy std `close`/`close_with_reason` helpers remain available
but cannot erase an existing fault. The added state field is part of the Nuis
library migration; applications and replay artifacts must be rebuilt together.
The close signature remains v3, but failure-v2 helpers and IPC-v4 peers must be
rebuilt together. This does not infer remote device causes, authorize recovery
or prove cancellation/resource retirement.

After its provider worker stops, the supervising launcher gives the child at most
five seconds to consume the error and run local cleanup, rather than killing it
immediately. The first failure latches this deadline; progress cannot reset it.
An explicit total child deadline remains stricter. The child is killed and reaped
if it does not exit in time. A zero child exit cannot erase a provider error.
Cleanup cannot reconnect or acquire new dispatch/replay capacity. This is bounded
failure teardown, not recovery or preemption of arbitrary in-flight callbacks.
Failures before successful open or fatal presentation-contract violations may
stop without a Nuis cleanup callback; the flag remains false in those cases.

## Running And Replaying Input

From the repository root on an Apple Silicon host with the registered Metal
adapter, after building the image example:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- build examples/projects/domains/ns_nova_image_showcase build/ns-nova-image
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- run-artifact --window-session window build/ns-nova-image
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- run-artifact --window-session window --window-events 32,128578 build/ns-nova-image
```

`--window-events` is a comma-separated sequence of up to 64 Unicode scalars. An
empty sequence still performs initial redraw and explicit close. It posts
**logical input events** to the actual AppKit queue, not fake physical keyboard
events; physical key maps cannot faithfully synthesize arbitrary Unicode text.
The sequence owns key delivery during replay, advances only after each reply, and
closes afterwards. The `run-artifact` replay frontdoor has a 180-second total child
deadline; expiry kills/reaps that child and closes its provider connection.
Interactive launch has no added total deadline. Both allow the provider connection
to wait idle between requests. Once its first byte arrives, the entire request
has a 120-second deadline; partial prefix/header/upload progress does not reset it.
The 256-dispatch limit, 64 MiB replay budget and separate reply/write timeouts
remain. No heartbeat, synthetic event or reconnect resets state, clocks or budgets.
EOF and supervisor shutdown wake idle waiting without certifying successful close.

The shared YIR `ReplayBudget` reserves the registered output byte extent before
each device invocation. Exhaustion rejects that invocation without running the
device; returned lengths must match the reservation. Failed admitted work does
not refund budget or authorize a successful lifecycle. This is a fixed bounded
session, not automatic budget extension or resumable execution.

Replay loading validates all declared payload sizes before opening any payload:
at most 256 frames, 16 MiB per frame and 64 MiB total. The textual manifest is
bounded to 4 MiB. Regular-file metadata and bounded reads prevent an oversized or
growing file from bypassing the declared byte cap; hashes still check identity.
Writing validates the complete replacement before removing old evidence. An
invalid replacement preserves existing files, but disk-write failures are not
yet handled by a transactional replacement protocol. These are retained-payload
and metadata caps, not a total process-RSS guarantee.

`--window-session` is mutually exclusive with `--json` and `--export-frame`.
The old no-option preview remains a compatibility route; it is **not migrated**
and must not be mistaken for this explicit registered mode. The one-shot compiled
frame export remains a separate complete-lifecycle regression. Provider sidecars
and the supervising launcher are still needed; this is not self-contained Nsld
injection, fully native CPU execution or a stable unlimited interactive engine.

## Evidence And Remaining Work

The compiled-window regression opens AppKit, posts logical space and non-BMP
input, verifies one open/two presentations/one close and successful exit, and
checks the actual persisted Metal pixels through identity-bound replay. It checks
one worker, sequences `0,1`, cache `compiled,hit`, physical fences, and an ignored
key with no GPU dispatch. Missing host profile/invalid script cases fail, and the
same test exercises the production `run-artifact` frontdoor. Protocol tests cover
signature drift, invalid Unicode, multiple presentations, close presentation
before finish, missing/malformed RGBA8, dimension overflow and C ABI output
ownership. This does not prove physical keyboard
layouts or IME behavior across systems.
Budget regressions cover pre-device rejection, no refund after failure, exact
dispatch-limit close, complete replay preflight, oversized sparse files, and
preservation of prior evidence on invalid replacement.
The live owned-session test also crosses an injected one-second request deadline
with 1.5-second gaps before each draw and close, preserving one Metal worker,
clock continuity and exact replay. Transport tests reject partial-message stalls
and exercise idle shutdown; this is not a long-duration window soak test or
proactive peer-health monitoring.

The same compiled AppKit binary is also run with an injected terminal provider
reader failure after dispatch receipt. It reports reason `1`, exactly one close
admission and completed Nuis cleanup, but no presentation or successful close;
the launcher retains the provider error and previous replay files. This is fault
injection, not a claim of a naturally failing Metal device or a live 64 MiB soak.
Protocol tests separately cover the rejected 257th dispatch, retained state,
host failure before cleanup, busy close admission, typed disconnect/identity
errors and late Finish rejection. A second failure run of the same compiled
window consumes its two saved frames and requests a third: it reports
`ReplayExhausted`, presents no substitute third frame, completes one failed-lifecycle
cleanup and exits nonzero without changing the prior replay evidence.
Compiled Nuis std tests verify failure status and no frame replay. Supervision
tests check graceful exit, a latched cleanup deadline and stricter total timeout.

The current Nuis event wrapper uses a leading guard returning existing state.
Bare `if { return aggregate_call(...); }` with no else is still unsupported by the
minimal lowering path; the equivalent guard form is not a compiler fix.
Close intent and observed failure kind are now separate. Device-specific causes,
terminal-outcome delivery into parent Nuis orchestration, application recovery, cancellation/resource retirement,
richer input, textures and non-Metal/native-CPU parity remain separate work.
