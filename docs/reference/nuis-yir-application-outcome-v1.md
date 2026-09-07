# Application Terminal Outcomes

`nuis-yir-application-outcome-v1` is an immutable, backend-neutral observation
of a terminal application result. It is separate from
[application state](nuis-yir-application-session-v1.md), close intent, device
completion receipts and resource ownership. It does not register a new callback.

## Canonical Fields

| Field Index | Field | Valid Terminal Value |
| --- | --- | --- |
| 0 | `status` | `1` succeeded, `2` failed |
| 1 | `cleanup_completed` | Exactly `0` or `1` |
| 2 | `failure_kind` | A registered application-failure-v2 code |

Success has exactly `[1,1,0]`. Failure requires status `2` and a known nonzero
failure code, with cleanup either `0` or `1`. Unknown codes, noncanonical flags
and contradictory combinations are rejected. An unavailable outcome is not a
successful or failed report: opening, open and faulted-awaiting-close have no
terminal outcome yet.

Cleanup means a validated Nuis close callback returned. It does not prove that
all resources were retired, that a device is idle, or that provider Finish
succeeded. For example `[2,1,11]` reports successful cleanup followed by a
provider finalization failure. The old Nuis state can still contain the normal
close result, because that callback ran before Finish failed.

## Runtime Observation

`ApplicationPumpReply::outcome()` validates terminal reply shape. `Closed`
requires a successful Close operation, retained state, completed cleanup, no
failure and a successful trace, after the existing provider Finish gate.
`Stopped` requires a failed trace and a nonzero failure; completed cleanup also
requires a Close reply with retained state. Inconsistent terminal replies stop
admission rather than manufacturing success.

`WindowSession` publishes one immutable snapshot after observing the terminal
reply, including a late Finish rejection. Its `outcome()` getter neither polls
nor executes code. A lost worker channel records failed/unclassified observation
unless an earlier category exists; it cannot certify cleanup that was never
observed. Host-side abort/failure does not prove in-flight work has stopped.
Freeing a handle is still abandonment, not implicit close or a resource-retirement
acknowledgement.

The readonly C ABI getter `nuis_window_session_outcome_field(session, field)`
returns the scalar field, or `-1` for a null handle, unavailable snapshot or
unknown field. The existing exclusive single-host-thread handle rule applies.
Repeated reads do not consume the snapshot, advance clocks, acquire budget,
repeat close or dispatch device work. The compiled window logs these fields;
its existing phase/trace/lifecycle checks still govern successful exit.

This additive value contract leaves window callback signatures at v3. Rebuild
the host and runtime together for the new getter; this is not a stable native
ABI promise. [IPC v4](nuis-yir-provider-runtime-ipc-v4.md) continues to own remote
rejection admission and request identity.

## Nuis Consumer

`NovaAppRuntime.terminal_outcome(status, cleanup, failure)` returns an owned
scalar `NovaAppOutcome`. Its valid codes match the shared contract. Invalid
input becomes the library-local sentinel `[3,0,255]`, not a wire terminal report.
Outcome status codes are independent of `NovaAppState`'s running/stopped codes.
`outcome_succeeded` and `outcome_failed` validate the fields rather than trusting
status alone. Neither helper mutates `NovaAppState` or invokes cleanup.

These values are descriptive, not unforgeable receipts. Copying, decoding or
constructing a report cannot authorize provider completion, resource release,
retry, recovery, replacement evidence or renewed execution.

## Evidence And Limits

Core tests cover the legal matrix and contradictory/unknown fields. Runtime
tests cover late Finish, failed cleanup, pre-open failure, repeated reads,
unavailable/invalid C ABI fields and rejection of repeated close/events.
Compiler tests feed an actual protocol-injected late Finish snapshot from the
compiled image module to a separate Nuis consumer and run the decoder as a
native binary. The injection is not a naturally failing device. The existing
compiled Metal window checks success, injected exchange failure and explicit
replay exhaustion through the same getter.

The consumer regression also exposed an execution-path mismatch: enum helpers
can return a canonical variant union through `call_owned_struct`, but the scoped
executor accepted only plain structs. It now validates the union's registered
layout, parent, active variant and payload shapes rather than rejecting every
union or bypassing type checks. Native and embedded-YIR decoder tests cover the
same valid and invalid outcome semantics.

There is **no automatic post-close Nuis observer**. A fresh function context
executes global initialization, so using it as an observer could repeat effects.
The explicit parent delivery below never constructs such a context. In-flight
cancellation and resource retirement remain open.

## Explicit Parent Delivery

`nuis-yir-application-outcome-delivery-v1` adds one runtime-owned delivery attempt
to an existing parent `ApplicationSession`. This is host wiring into a compiled
Nuis event, not a new source-language callback or a child-module observer.

1. The caller opens the independent parent once and keeps its execution context.
2. The child window observes a terminal result. Before then,
   `WindowSession::take_outcome_delivery()` returns `None` without spending the slot.
3. The first terminal take returns a non-cloneable `ApplicationOutcomeDelivery`.
   Readonly snapshots remain available, but all later takes return `None`.
4. `delivery.deliver(&mut parent, max_steps)` consumes the attempt and invokes the
   parent's registered event with its carried state, followed by three value-owned
   i64 arguments: status, cleanup and failure. No child handles, registry,
   completion witnesses, provider budget or execution context cross this boundary.

The caller explicitly chooses the parent and its event semantics. The event uses
the parent's existing registered effects, not borrowed child authority. It need
not be pure. Phase/signature rejection, callback failure and budget exhaustion
all consume the delivery; there is no automatic retry or refund. Dropping it
abandons delivery without running code. This is one issued attempt per live
window handle, not durable exactly-once delivery, authenticated child identity,
or protection against a caller manually submitting identical scalar data.

`ApplicationSession::event_budgeted` uses `FunctionSession::invoke_budgeted`.
One step is charged before every scoped function entry and graph node; nested
calls, recursion and scoped helper-loop calls share the same per-invocation fuel.
Parameter binding is not a separate step. Zero fuel fails before function entry.
Ingress validation still precedes execution. Fuel is cleared after return or
error, and partial traces are drained. An admitted failed event faults the parent
and preserves its last accepted state; already executed effects are not undone.
Explicit cleanup remains possible under the parent's ordinary lifecycle rules.

This is an executor admission budget, **not** a wall-time, allocation or device
preemption guarantee. Global initialization occurred when the parent was opened;
it is not covered retroactively. Registered operations' private loops, blocking
FFI and device work retain their own limits. The delivery does not connect new
providers, replenish their budgets or promise bounded GUI callback latency.

Tests preserve parent-owned effects without reinitialization, consume abandoned
attempts, reject closed/mismatched parents, exhaust nested/recursive fuel, retain
partial effects and allow cleanup without certifying successful completion. The
compiled image's injected late Finish now reaches a parent that was already open
and had accepted an earlier event: its counter advances from 43 to 44 while
recording `[2,1,11]`. The failed child remains failed even when its parent handles
that report successfully. Native Nuis execution also checks the same reducer.

The [parent pump](nuis-yir-application-outcome-pump-v1.md) now connects this API to
the compiled AppKit launcher through an explicit `--window-parent-session` ID.
It opens a root-scoped CPU parent before the child and waits for independent
delivery/cleanup without blocking GUI polling or clearing child failure.
General multi-child orchestration, native parent dispatch, durable routing
and in-flight resource retirement remain separate work. Window callbacks stay v3,
the outcome value stays v1, and the readonly C ABI is unchanged.
