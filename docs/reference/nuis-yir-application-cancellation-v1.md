# Application Cancellation And Host Retirement

`nuis-yir-application-cancellation-v1` adds explicit cooperative cancellation to
the owned [application event pump](nuis-yir-application-session-v1.md). It is a
host runtime boundary, not a device cancellation protocol or Nuis language change.
The bootstrap implementation lives in
[`application_cancellation.rs`](../../crates/yir-runtime-host/src/application_cancellation.rs).

## Admission

`ApplicationEventPump::cancel` is nonblocking and returns one owned
`ApplicationCancellation` ticket on acceptance. It may be called with Open,
Event or Close pending, including an unconsumed reply, or while idle/faulted.
Acceptance immediately stops further host admission, drops pending replies and
sets the local pump to `Stopped`. This local state is not a retirement receipt.

An atomic gate arbitrates cancellation against provider Finish admission:

| Winner | Result |
| --- | --- |
| Cancellation | No subsequent provider Finish; the host waits on its separate ticket. |
| Finish | Cancellation rejects without consuming or modifying the original pending reply. |
| Worker scope already returned | Cancellation rejects; it does not reconstruct a retired session. |

Already admitted callback work may finish, including its nested functions and
provider dispatches. Boundary checks skip work not yet entered when cancellation
is observed. This is not instruction-level preemption, rollback or a guarantee
that no physical dispatch can start after the host's cancellation call returns.
No new event or implicit close is enqueued. Explicit close already executing may
return, but accepted cancellation still prevents the subsequent Finish gate.

After provider admission returns, a checkpoint precedes application startup.
Further checkpoints guard command execution/reply delivery and the close/finish
boundary. Waiting for Hello, an in-flight provider reply, a blocking FFI call or
callback termination still obeys existing execution and transport constraints.
Cancellation does not increase budgets, reset deadlines, reconnect or retry.

## Dependency Boundary

The crate-local
[`ScopeAdmission`](../../crates/yir-runtime-host/src/application_scope_admission.rs)
interface exposes only `checkpoint` and `admit_finalization`. It owns neither
cancellation state nor a provider, window, registry, transport or result channel.
It is a host-scope interface, not a new YIR wire format or Nustar registration.

- The pump/worker boundary owns the concrete cancellation controller and
  retirement channel; the caller owns an issued ticket. The controller implements
  generic finalization admission, not IPC Finish.
- The provider session accepts a statically selected generic admission policy.
  It does not depend on `CancellationControl`, a pump, a window or a ticket.
  Only after lifecycle success and admission does it perform its own Finish.
- Existing synchronous callers use the unit policy with no additional admission
  restriction; application and provider checks remain mandatory. No dynamic
  trait dispatch, plugin loading or extra per-call policy allocation is added.

The window adapter forwards requests and tickets rather than borrowing provider
internals. Its generic ticket C ABI has no window, pump, controller or provider
dependency. Parent outcome delivery remains a separate value-only boundary;
it does not receive child registry, transport or retirement authority.

The [independent-policy tests](../../crates/yir-runtime-host/src/provider_application_session/admission_tests.rs)
substitute a recording policy with no cancellation implementation. It can reject
before provider I/O, after provider admission or before finalization. Driver and
close failures still prevent Finish even when the policy allows it. A direct
dependency guard accompanies these behavioral tests; it is not a complete
transitive dependency or extensibility proof.

## Acknowledgement

The ticket supports nonblocking `poll` and bounded `wait`. A timeout retains the
same ticket; it is neither failure nor retirement. Exactly one
`ApplicationHostRetirementAck` is delivered after the worker's borrowed execution
scope returns and its module, registry, execution state and provider transport
have been dropped. The worker thread publishes it after that scope returns;
this is not a thread join or a claim about unrelated allocations.

The acknowledgement contains two independent observations:

- `cleanup_completed()` is true only if an explicitly admitted Nuis close
  returned valid state and passed trace checks. Cancellation never runs close.
- `failure_kind()` retains the first latched execution/provider category,
  including a failure received after cancellation admission. `None` means no
  category was latched, not that the abandoned application succeeded.

No retained frame history, resource capability, parent event or application-state
mutation crosses this channel. A cancellation ticket is not an
[application outcome delivery](nuis-yir-application-outcome-v1.md) and cannot
certify success, authorize resource reuse or impersonate a provider completion.
It reports host ownership retirement, **not provider/device resource retirement**.
Caller-owned snapshots and resources outside this pump are not included.

Channel disconnection without an acknowledgement returns an error, not a fabricated
retirement receipt. A worker panic cannot publish the normal acknowledgement.
Dropping the ticket abandons observation without joining or running cleanup.
Existing `abort` and handle Drop remain unacknowledged abandonment: notably they
do not retract an already admitted close or promise to prevent its Finish.

## Window And C ABI

`WindowSession::cancel` forwards pump admission and returns the same independent
ticket. It may cancel pending open/event/close work, including an unconsumed
reply. Rejection preserves pending work and window observations. Accepted
cancellation sets the pump to `Stopped`, but does not invoke Nuis close, set a
close reason, publish an `ApplicationOutcome`, or issue parent outcome delivery.
Later rejected window calls must not manufacture any of those results either.

The window's cleanup/failure getters retain their last observed values. The
independent ticket can report a later worker fault or validated explicit cleanup
without mutating that abandoned snapshot, even after the window handle is freed.
Already returned caller-owned frame/state snapshots remain caller-owned.

The bootstrap host exports these thin adapters:

| Entry | Result |
| --- | --- |
| `nuis_window_session_cancel(session, ticket_out)` | `0` admitted; `-1` rejected. A null/nonempty output slot rejects before cancellation; it is never overwritten. No busy queue or implicit retry. |
| `nuis_application_cancellation_poll(ticket, cleanup_out, failure_out)` | `0` pending/already consumed; `1` exactly one receipt; `-1` invalid input or missing acknowledgement. |
| `nuis_application_cancellation_free(ticket_slot)` | Nulls the owned slot and abandons observation, without joining or cancelling additional work. |

Ticket poll outputs are `i32` cleanup and `i64` failure code, written only when
the result is `1`. Null outputs reject before consuming the receipt. Invalid
input, pending, already-consumed and disconnected paths leave output values
untouched; callers must check the return status rather than reuse stale values.
Handles are exclusively owned, with no concurrent calls or copied-handle frees.
The ticket has no window pointer and needs no provider access to be polled/freed.

The C ABI is a host boundary, not a new Nuis intrinsic or CFFI allowlist grant.
The packaged AppKit host now forwards and consumes tickets through the explicit
scripted path below. Normal window quit remains explicit close/Finish; no
implicit timeout-to-cancel conversion or successful exit is introduced.

## Packaged Host Policy

`--window-cancel-after-events` requires `--window-session ID --window-events
CODEPOINTS` and rejects `--window-parent-session`. An empty event script still
waits for the initial redraw. Cancellation begins only after the final event
reply is consumed; this entry does not interrupt a pending GPU draw. Both CLI and
embedded-host parsers reject unsupported combinations before opening a session.
The frontdoor also requires the bundle's exact
`window_cancellation_contract=nuis-yir-application-cancellation-v1` declaration;
old bundles must be rebuilt rather than silently taking normal close.

The [small host adapter](../../tools/yir-pack-aot/src/host_window_cancellation.rs)
only admits a ticket, frees the window, polls the receipt and terminates the host.
The timer stops ordinary session polling once a ticket exists; an OS termination
request during retirement is deferred until that observation completes. Pending
polls retain the ticket. One receipt logs cleanup/failure independently and exits
130, never zero; a missing receipt logs an error and exits 1 without claiming
retirement. Rejected cancellation preserves the window/reply for normal failure
handling and never creates a ticket receipt.

This is a bounded standalone scripted policy, not a general interactive or parent
cancellation controller. The live provider has no cancellation message and sees
EOF without Finish. Its existing supervision reports incomplete execution and
preserves prior replay evidence. No provider error is suppressed or reclassified
as successful cancellation; device retirement still requires provider-owned
protocol and resource-lifetime evidence.

## Evidence And Limits

[`cancellation.rs`](../../crates/yir-runtime-host/tests/provider_application_session/cancellation.rs)
uses gated IPC peers to test delayed Hello/frame replies, idle and in-flight
cancellation, unconsumed replies, explicit close versus cancellation, Finish
winning admission, ticket abandonment, typed first faults and one receipt only.
These are protocol fixtures, not new hardware cancellation evidence.
The shared gate tests race Finish and cancellation and reject lost receipts.

The [window cancellation tests](../../crates/yir-runtime-host/tests/provider_application_session/window/cancellation.rs)
check delayed admission/event cancellation, typed faults, Finish winning without
losing the original close reply, invalid output slots, no fabricated parent
outcome, a ticket outliving its window, and nonjoining abandonment.
The [ticket ABI tests](../../crates/yir-runtime-host/src/application_cancellation/ffi/tests.rs)
cover one receipt, invalid-output nonconsumption, disconnected channels, output
preservation and a direct dependency guard for the window/provider-neutral ABI.

[`ns_nova_application_cancellation.rs`](../../tools/nuisc/tests/ns_nova_application_cancellation.rs)
compiles the actual Nuis image showcase and validates its registered window
signature. Its opened Nuis state can retire without invoking close or sending
Finish; the independent explicit-close baseline still requires matching
Finish/Closed. The peer checks actual EOF rather than treating a timeout as release.
An additional compiled-source test crosses WindowSession and the cancellation
C ABI, frees the window, and consumes its independent ticket exactly once.
No GPU frame is dispatched by this cancellation fixture. Existing compiled Metal
window/export and provider failure regressions remain separate execution evidence.

The [generated host harness](../../tools/yir-pack-aot/src/host_window_cancellation_tests.rs)
compiles the actual adapter with stub tickets and no provider/close/outcome
implementation. It checks pending/once-only/missing receipts, late fault fields,
rejection and argument rules. The
[compiled-window cancellation regression](../../tools/nuis/src/artifact_device_sample_shader_cancellation_tests.rs)
uses actual Metal frames and the production frontdoor, requires EOF rather than
Finish, keeps old replay bytes, and separately verifies replay exit 130. It also
rejects invalid modes before window/parent admission. These tests do not prove
device interruption, device drain or general interactive cancellation.

The pump, registered `WindowSession` and its C ABI expose cancellation tickets;
AppKit has the explicit standalone scripted policy; parent-pump cancellation is not yet wired.
Normal window quit still uses explicit close. No IPC wire message is added and
there is no general provider drain/cancel acknowledgement. Device resource
retirement, cancellation of resource-capability state, recovery, multi-child
routing, fully native CPU execution and self-contained Nsld remain open.
