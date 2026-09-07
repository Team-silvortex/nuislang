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

There is **no automatic post-close Nuis observer** yet. The separate consumer
fixture is not a second execution of the closed application's module. A fresh
function context executes global initialization, so using it as an observer
could repeat effects. Delivery into a live parent Nuis orchestrator needs an
explicit ownership/effect/execution-budget contract, not a fresh context or an
implicit callback. In-flight cancellation and resource retirement remain open.
