# Persistent YIR Application Sessions

`nuis-yir-application-session-v1` is a host-call boundary over the existing YIR
function table. It is a first persistent-session slice, not a new application
language, scheduler, native CPU ABI or device-specific engine implementation.

## Ownership And Execution

The caller supplies a verified module, an explicit Nustar registry and either a
registered session ID or distinct helper names for `open`, `event`, and `close`.
No engine, galaxy, function name or
backend is hardcoded in the carrier. Application transitions remain in Nuis;
the current acceptance workload reuses `NovaAppRuntime` and the image showcase's
`render_showcase_frame` without translating their policy to Rust.

`yir-exec::FunctionSession` prepares one execution context and runs only nodes
outside all function bodies during initialization. It never implicitly runs an
entry function. Subsequent calls execute one named helper, including nested
helper calls, and restore its local value scope afterwards. Heap state, registered
provider clock frontiers and shared execution state remain in that context.

`yir-runtime-host::ApplicationSession` retains the last accepted Nuis result as
the next call's state. `open` receives scalar configuration arguments. Both
`event` and `close` take the same leading flattened state fields followed by their
own scalar arguments, and all three return the same owned aggregate type.

The boundary checks helper roles, distinct lifecycle names, parameter ownership,
scalar types and arity. Aliased parameter nodes are rejected. Recursive aggregate
leaves are matched against the registered dotted field paths, order and types;
there is no Rust list of Nova fields. Returned state is checked before replacing
the accepted state. Pointers, buffers, strings, device handles and other resource
capabilities are not accepted as session state in this first slice.

## Events, Failure And Close

* Invalid scalar input is rejected before callback execution and can be corrected.
* A callback or returned-state error faults the session and stops later events.
  The last accepted scalar state remains available, but there is **no rollback**
  of effects already performed by the failed callback. That failure remains
  latched even if subsequent explicit cleanup succeeds.
* Explicit close is permitted after an event failure and receives that last
  accepted scalar state. Once execution of close is attempted, no retry occurs,
  even if it fails. Repeated successful close is a no-op; repeated failed close
  returns the recorded error. Invalid close arguments do not consume the attempt.
* Dropping the carrier does not implicitly execute Nuis code. Callers must close
  explicitly; universal cleanup, cancellation and in-flight teardown guarantees
  remain in the separate lifecycle/resource-safety coordinate.
* Events, lane steps and presented frames are drained after each executed call,
  including failure. A successful result carries its own trace; failed-call
  traces are discarded, not attached to a later successful call. Completion
  witnesses are a snapshot of current node state, not an invocation history.
  Session traces do not clone/export persistent heap values.

The open trace also includes one-time global initialization. It is not repeated
in event or close traces.

This prevents carrier-owned frame history from growing with event count. It is
not a claim that arbitrary user allocations, backend resources or long-running
applications are leak-free.

## Verified Slice

The synthetic lifecycle regression runs 100 independent deliveries, checks
one-time global initialization, recursive scalar carry, invalid ingress,
signature/layout drift, event failure and non-retried close failure.

The compiler integration regression compiles the actual
`examples/projects/domains/ns_nova_image_showcase` Nuis sources, opens one session,
delivers image inputs 0 and 2 in separate host calls, and closes once. It verifies
frame state 0 -> 1 -> 2, two presentations, increasing completion clocks, stable
completion root, one returned frame per event and no entry-node execution.
This test explicitly uses the **reference provider**, not Metal hardware.

## Static Registration

Projects can declare lifecycle bindings in `nuis.toml` without hardcoding
application names in compiler or runtime logic:

```toml
application_sessions = [
  "image open=NovaAppRuntime.open event=render_showcase_frame close=NovaAppRuntime.close state=state"
]
```

Each string starts with an explicit session ID and contains exactly the four
named fields shown above, in any order. The array supports comments and trailing
commas; the declaration strings use unescaped, single-line UTF-8 text. Unknown
or duplicate fields, duplicate IDs, missing fields, malformed arrays and more
than 64 registrations are errors. The optional field defaults to no registration,
not to guessed helper names or a default application.

The compiler retains the three helpers as generic host-call roots, even if
`main` never calls them. It does not mark them as C exports or introduce dummy
calls. `yir-core::ApplicationSessionSignature` owns the common signature checks;
the verifier and runtime consume the same contract rather than separate copies.
The emitted YIR carries a non-executable record:

```text
application-session image nuis-yir-application-session-v1 NovaAppRuntime.open render_showcase_frame NovaAppRuntime.close state
```

Records may precede the function table; verification resolves them after parsing.
Unknown versions are rejected. The record travels in the canonical YIR payload,
including the current window artifact's embedded YIR, and participates in existing
stage/provider source identity checks. There is no independently editable host
mapping file. This is not yet a new Nsld-native session ABI or a native CPU caller.

`ApplicationSession::open_registered` selects a declared ID from an already
loaded module and explicit registry. `with_registered_provider_application_session`
does the same for an explicit IPC/replay source, with signature and scalar-input
preflight before connecting. The lower-level explicit-entry APIs remain available
for callers that deliberately supply bindings; neither API auto-selects a session.

## Provider-Backed Scope

`nuis-yir-provider-application-session-v1` adds
`with_provider_application_session`: one owned module, one explicitly selected
IPC/replay source, one registry and one Nuis application session for the duration
of a host driver. Signature/configuration preflight occurs before connection;
IPC admission and replay both verify the source identity. No environment-based
source selection is performed by this API.

Independent `event` calls use the same provider transport. The driver must close
the Nuis session explicitly. Only a successful driver and `completion_status()`
may finish the provider; swallowed event/close failures and missing close cannot
authorize successful provider evidence. Close acknowledgement errors and leftover
replay frames fail the whole call. The caller receives its result only after that
finish check, although already-performed driver/application effects cannot be
rolled back.

The supported provider result adapter remains the registered shader image path,
not arbitrary multi-provider execution. Shader Nustar declares its frame-producing
operations; unbound `draw_instanced`, `draw_ball`, `draw_sphere`, `clear` and
`overlay` are rejected in a provider-backed registry before reference computation.
Descriptor construction/forwarding remains available. The separate reference
executor is unchanged and must not be presented as device evidence.

The Metal integration regression builds the Nuis image project, consumes the
emitted YIR through the owned event pump over the scoped API, and routes two separately delivered inputs
(0 and 2) through `nsdb::serve_runtime_provider_session` and the real registered
Nuis worker. It checks every RGBA8 pixel against the inverted Nuis checkerboard,
one frame per event, frame state 0 -> 1 -> 2, no `main` replay, increasing physical
completion clocks, one worker/lease, sequences `0,1` and cache `compiled,hit`.
The driver selects only the declared `image` ID. The test also checks that the
registration survives binary embedding. Per-event replay matches both live
frames; incomplete replay and a renamed registration against old evidence are
rejected. A separate compiler regression retains and runs three helpers never
called by `main`, and rejects missing host roots and signature drift.
The CPU driver is still embedded-YIR execution in an integration-process worker, not
the default compiled app entry or native CPU function lowering.

## Owned Event Pump

`nuis-yir-application-event-pump-v1` exposes `ApplicationEventPump`, an owned host
handle rather than a borrowed driver closure. `spawn` takes owned source text,
an explicit provider source, a registered ID and scalar opening arguments. It
returns after starting a worker, not after provider admission. The worker owns
the source/module, registry, provider and borrowed application session in one
scope. No self-referential allocation, unsafe lifetime extension, application
field list or backend selection policy is introduced.

The host consumes the `Open` reply, submits an `event`, then consumes its reply.
`poll` never waits; `wait(Duration)` bounds only the host wait. An expired wait
leaves the original operation pending and does not cancel, retry, reconnect or
reset provider budgets. `phase()` is the last observed phase, so callers must
also inspect `pending()`. There is exactly one outstanding operation, including
an unconsumed reply. Both channels have capacity one; busy submission is rejected
without queuing or coalescing input. Per-event state snapshots and traces are
moved to the caller, not retained as pump-owned frame history. This bounds queue
count, not all application allocations or the size of arbitrary user input.

Replies identify `Open`, `Event` or `Close`, carry the last accepted scalar state
when available, and expose either that call's trace or an error. Invalid event or
close arguments leave the application phase unchanged and can be corrected.
Callback failures latch `Faulted`; later events are rejected, while one explicit
close attempt is still possible. Terminal admission/close/transport failure is
`Stopped`, never `Closed`. A successful `Closed` reply is published only after
the scoped lifecycle and provider finish checks, including complete replay
consumption. No cleanup success can hide an earlier event failure.

Replies also expose `cleanup_completed`: the close callback returned a state and
passed host trace checks. This is independent of lifecycle success and resource
retirement. A failed event or late provider Finish can yield a terminal `Stopped`
reply with this flag true. Failed close callbacks or invalid close presentations
leave it false. The window profile can latch a host failure before Nuis cleanup;
that failure likewise prevents successful Finish, even if the callback succeeds.

`failure_kind` carries the shared `nuis-yir-application-failure-v1` code separately
from text diagnostics and close intent. Producers record their observation before
crossing the executor's String error boundary; each scope owns its first-failure
latch. Callback/host cleanup cannot overwrite an earlier provider fault. Invalid
ingress arguments rejected before execution do not poison the latch. The terminal
reply includes late Finish failure even though Nuis close cannot be repeated.
The window terminal diagnostic retains both an event error and a later cleanup
error. Text-only peer rejection and exchange/framing failures remain coarse
categories; no remote cause is guessed from diagnostic wording.

`abort` and dropping the handle disconnect the channels without joining the
worker or implicitly executing Nuis close. An idle worker wakes and drops its
scope without successful provider finish. An already admitted callback may still
run; an already requested close may still finish and persist provider evidence
even if its caller stops waiting. Neither operation preempts arbitrary YIR code
or guarantees immediate device/transport cleanup. They are abandonment, not a
general cancellation protocol.

Protocol regressions cover 100 independent deliveries without frame/witness
history growth, pending-reply backpressure, delayed admission/frame/close replies,
recoverable input errors, latched callback failures, failed close acknowledgements,
idle/in-flight abandonment and the unchanged per-session dispatch limit. The live
Metal regression now holds this owned handle between deliveries, verifies both
images and replays them through another owned pump. An explicit registered
[window mode](nuis-yir-window-session-v3.md) now uses this handle in the compiled
AppKit process; the no-option legacy preview timer remains unchanged.

### Idle And Request Deadlines

The supervised Unix provider now waits for a request's first byte without an
idle deadline. A quiet application no longer fails simply because 120 seconds
pass between events. This is the same connection, application scope, worker,
clock frontier and replay budget: idle waiting issues no heartbeat, device work
or synthetic Nuis event. The IPC v3 wire format is unchanged.

After the first byte, the complete incoming request, including length prefix,
header and uploads, shares one 120-second deadline. Partial progress cannot
restart that deadline. A malformed, truncated or timed-out request terminates
the provider lifecycle rather than retrying a partially consumed stream. Existing
reply/write and device-operation timeouts remain separate; this is not arbitrary
callback preemption. Nsdb accepts a transport-specific request reader while
retaining ownership of dispatch admission, accounting and close.

EOF or supervisor socket shutdown wakes idle waiting without a success
acknowledgement. Explicit child wall-clock limits still apply to scripted runs;
interactive runs do not gain an implicit lifetime limit. There is no automatic
reconnect or proactive idle health probe. If the provider itself fails, recovery
is still a separate protocol, not authorization to restart clocks or budgets.

On provider-worker failure, the launcher allows at most five seconds for the
child to observe the error and perform local Nuis cleanup before kill/reap. An
explicit total child deadline is not extended, and successful child cleanup or
zero exit cannot replace the provider error with success. This bounded grace is
not reconnection, resource retirement or arbitrary callback cancellation.

Fast transport tests cover repeated idle waits, buffered requests, incomplete
prefix/header/uploads, cumulative deadlines, EOF and supervisor shutdown. The
real Metal owned-session regression injects a one-second request deadline and
waits 1.5 seconds before both draws and close, verifying exact pixels, one worker,
monotonic clocks and unchanged replay. This is shortened-deadline evidence, not a
multi-hour window soak test.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --test application_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --test provider_application_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test ns_nova_application_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib project::application_sessions -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-syntax --test application_sessions -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-core -p yir-exec -p yir-runtime-host --lib -j 1
# Unix host transport: idle waits, partial-message deadlines and supervisor shutdown.
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis artifact_runtime_provider_results -j 1 -- --test-threads=1
# Metal-capable macOS host; includes old compiled exports and the owned event pump.
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 NUIS_TEST_QUIET_SUCCESS_LOGS=1 cargo test -p nuis --bin nuis artifact_device_sample_shader_render -j 1 -- --test-threads=1
```

## Remaining Integration

The default legacy window timer is **not migrated**; no-option preview still
executes a complete module. Explicit `--window-session ID` instead selects the
window profile, bypasses main/tick replay, delivers logical inputs and waits for
confirmed close. The existing compiled
`run-artifact --export-frame` Metal/replay path is a separate bounded regression;
it must keep its explicit provider admission, output validation and no-reference-
fallback policy. The carrier does not create a provider or choose a fallback.

The inner scoped API is synchronous; the owned pump keeps it off the host event
thread. Existing IPC limits remain: one registered target,
256 dispatches, 64 MiB retained replay data and 120-second socket I/O timeouts.
Incoming requests have the whole-message deadline described above, not an idle
timeout between messages.
Shared YIR budget accounting now reserves the registered extent before device
execution. Replay readers validate the full declared aggregate before payload
reads, bound each actual read and the 4 MiB manifest, and retain identity checks.
Writers validate replacement evidence before removing the prior stream; this
does not yet provide crash-atomic or disk-error-atomic publication.
Those bounds are not a sustained interactive-window protocol. The lifecycle
registration, owned event pump and explicit window startup/input/shutdown route
now exist, including same-connection idle waiting, but user-visible budget
exhaustion, peer-failure recovery and cancellation/resource retirement
still need integration. Do not simply reopen a scope
on every timer tick or silently reset clocks/budgets. This must not introduce special function
names or backend combinations into the compiler. Native CPU dispatch remains
its own differential-execution step.

See the [mainline tensor](nuis-development-tensor-mainline.md) and
[application lifecycle contract](nuis-ns-nova-application-lifecycle-v1.toml).
