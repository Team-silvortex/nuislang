# Persistent YIR Application Sessions

`nuis-yir-application-session-v1` is a host-call boundary over the existing YIR
function table. It is a first persistent-session slice, not a new application
language, scheduler, native CPU ABI or device-specific engine implementation.

## Ownership And Execution

The caller supplies a verified module, an explicit Nustar registry and distinct
helper names for `open`, `event`, and `close`. No engine, galaxy, function name or
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
emitted YIR in the new scoped API and routes two separately delivered inputs
(0 and 2) through `nsdb::serve_runtime_provider_session` and the real registered
Nuis worker. It checks every RGBA8 pixel against the inverted Nuis checkerboard,
one frame per event, frame state 0 -> 1 -> 2, no `main` replay, increasing physical
completion clocks, one worker/lease, sequences `0,1` and cache `compiled,hit`.
Per-event replay matches both live frames; incomplete replay is rejected.
The CPU driver is still embedded-YIR execution in the integration process, not
the default compiled app entry or native CPU function lowering.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --test application_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --test provider_application_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test ns_nova_application_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-core -p yir-exec -p yir-runtime-host --lib -j 1
# Metal-capable macOS host; includes old compiled exports and the new scoped session.
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 NUIS_TEST_QUIET_SUCCESS_LOGS=1 cargo test -p nuis --bin nuis artifact_device_sample_shader_render -j 1 -- --test-threads=1
```

## Remaining Integration

The native window timer and default compiled-host entry are **not migrated** to
this API yet. They still execute a complete module. The existing compiled
`run-artifact --export-frame` Metal/replay path is a separate bounded regression;
it must keep its explicit provider admission, output validation and no-reference-
fallback policy. The carrier does not create a provider or choose a fallback.

The scoped API is synchronous. Existing IPC limits remain: one registered target,
256 dispatches, 64 MiB retained replay data and 120-second socket I/O timeouts.
Those bounds are not a sustained interactive-window protocol. Artifact lifecycle
metadata, an owned event pump/host handle, idle/cancellation policy and window
shutdown still need integration. Do not simply reopen a scope on every timer
tick or silently reset clocks/budgets. This must not introduce special function
names or backend combinations into the compiler. Native CPU dispatch remains
its own differential-execution step.

See the [mainline tensor](nuis-development-tensor-mainline.md) and
[application lifecycle contract](nuis-ns-nova-application-lifecycle-v1.toml).
