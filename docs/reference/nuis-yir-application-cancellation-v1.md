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
| Failed provider scope starts abandonment | Cancellation rejects before the provider is dropped; it cannot promise a late drain. |
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
interface exposes `checkpoint`, `admit_finalization` and `admit_abandonment`. It owns neither
cancellation state nor a provider, window, registry, transport or result channel.
It is a host-scope interface, not a new YIR wire format or Nustar registration.

- The pump/worker boundary owns the concrete cancellation controller and
  retirement channel; the caller owns an issued ticket. The controller implements
  generic finalization admission, not IPC Finish.
- The provider session accepts a statically selected generic admission policy.
  It does not depend on `CancellationControl`, a pump, a window or a ticket.
  Only after lifecycle success and admission does it perform its own Finish.
  On failure, abandonment seals admission before provider drop and can request
  a separate drain observation. The policy receives no transport or registry.
- Existing synchronous callers use the unit policy with no additional admission
  restriction; application and provider checks remain mandatory. No dynamic
  trait dispatch, plugin loading or extra per-call policy allocation is added.

The window adapter forwards requests and tickets rather than borrowing provider
internals. Its generic ticket C ABI has no window, pump, controller or provider
dependency. Parent outcome delivery remains a separate value-only boundary;
it does not receive child registry, transport or retirement authority.

The [independent-policy tests](../../crates/yir-runtime-host/src/provider_application_session/admission_tests.rs)
substitute a recording policy with no cancellation implementation. It can reject
before provider I/O, after provider admission or before finalization. An independent
abandonment policy can request drain without turning the failed driver into success. Driver and
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

The acknowledgement contains independent observations:

- `cleanup_completed()` is true only if an explicitly admitted Nuis close
  returned valid state and passed trace checks. Cancellation never runs close.
- `failure_kind()` retains the first latched execution/provider category,
  including a failure received after cancellation admission. `None` means no
  category was latched, not that the abandoned application succeeded.
- `provider_drain()` is `NotRequested` for ordinary cancellation. The explicit
  drain variant below reports a separately validated provider observation.

No retained frame history, resource capability, parent event or application-state
mutation crosses this channel. A cancellation ticket is not an
[application outcome delivery](nuis-yir-application-outcome-v1.md) and cannot
certify success, authorize resource reuse or impersonate a provider completion.
The host acknowledgement alone reports **no provider/device resource retirement**;
only its independently validated drain observation can describe the provider scope.
Caller-owned snapshots and resources outside this pump are not included.

Channel disconnection without an acknowledgement returns an error, not a fabricated
retirement receipt. A worker panic cannot publish the normal acknowledgement.
Dropping the ticket abandons observation without joining or running cleanup.
Existing `abort` and handle Drop remain unacknowledged abandonment: notably they
do not retract an already admitted close or promise to prevent its Finish.

## Optional Provider Drain

`ApplicationEventPump::cancel_with_provider_drain` uses the same atomic gate and
one-shot host ticket, but requests the independent
[provider-session drain extension](nuis-yir-provider-session-drain-v1.md).
`WindowSession::cancel_with_provider_drain` only forwards it. Ordinary `cancel`,
Drop/abort, successful close/Finish and existing synchronous callers are unchanged.

The provider session attempts Drain only after the admitted callback and its
borrowed application state leave scope. Its IPC client must still have a validated
Hello/completed-Frame frontier. The first attempted write invalidates that frontier;
only a fully validated Frame restores it. Rejected, partial, mismatched or terminal
exchanges cannot send another Dispatch, Finish or Drain. This is conservative:
even a fully parsed remote dispatch rejection does not authorize a new request.
Local validation/budget rejection before any exchange can retain the frontier.

The ticket exposes `ProviderDrainObservation`:

| Status / C code | Meaning |
| --- | --- |
| `NotRequested` / 0 | Ordinary host-only cancellation. |
| `NotObserved` / 1 | No drain observation was collected before scope exit, for example pre-admission failure. |
| `ReplayOnly` / 2 | Replay was abandoned; it is not a live provider receipt. |
| `Unavailable` / 3 | No trustworthy exchange frontier; no Drain sent. |
| `Failed(kind)` / 4 | Drain exchange/receipt failed. Its category is independent of the first application failure. |
| `Drained { completed_dispatches }` / 5 | Exact target and count validated on the original connection. |

A failed drain latches its category only if no earlier execution/provider fault
exists. Its separate category remains readable either way. A drain receipt never
manufactures Nuis cleanup, an application/parent outcome, successful publication or
resource-reuse authority. Host retirement is reported even when drain fails; the
ticket is delivered only after all of that worker's borrowed scope has returned.
Pending `poll`/`wait` does not renew a deadline, retry or certify provider retirement.

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
| `nuis_window_session_cancel_with_provider_drain(session, ticket_out)` | Opt-in drain cancellation with the same empty-slot ownership rules. |
| `nuis_application_cancellation_poll_with_provider(ticket, receipt_out)` | Same 0/1/-1 result, with independent host/provider observations in one output struct. |
| `nuis_application_provider_drain_exit_status(receipt)` | Pure portable classifier: 130 only for a valid drained observation with no host/provider fault; otherwise 1. It neither polls nor authenticates an arbitrary C struct. |

Ticket poll outputs are `i32` cleanup and `i64` failure code, written only when
the result is `1`. Null outputs reject before consuming the receipt. Invalid
input, pending, already-consumed and disconnected paths leave output values
untouched; callers must check the return status rather than reuse stale values.
Handles are exclusively owned, with no concurrent calls or copied-handle frees.
The ticket has no window pointer and needs no provider access to be polled/freed.

`NuisApplicationCancellationReceipt` has C-layout fields in order:
`cleanup_completed: i32`, `provider_status: i32`, `failure_kind: i64`,
`provider_failure_kind: i64`, `completed_dispatches: i64`. Count is -1 unless
status is 5; provider failure is zero unless status is 4. Both poll variants
consume the same receipt once. A null extended output is rejected before polling,
and its contents remain unchanged unless a receipt is returned.

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
cancellation controller. Without `--drain-provider`, it sends no provider drain request and the peer
sees EOF without Finish. Its existing supervision reports incomplete execution and
preserves prior replay evidence. No provider error is suppressed or reclassified
as successful cancellation; device retirement still requires provider-owned
protocol and resource-lifetime evidence.

Adding `--drain-provider` selects a separate policy, requiring exactly one matching
`application_provider_drain_contract=nuis-yir-provider-session-drain-v1` bundle
declaration. It retains the standalone/script requirement and independent ticket
ownership. The adapter forwards extended receipts to the portable Rust classifier;
it does not implement provider selection, resource release or error classification.
Only status 5, a count in 0..=256, valid cleanup representation and zero independent
host/provider failures yield 130. Missing, replay-only, unavailable and failed
observations yield 1, as does a prior application fault despite provider drain.
Cleanup remains an independent fact, not an application-success prerequisite.

The launcher also requires its own typed provider Drained terminal and that
non-success child exit; neither side's observation replaces the other. It rejects
Finish before provider publication under explicit drain intent and returns typed
`ArtifactRunOutcome::Cancelled` without writing successful launch/trace evidence.
EOF and exit 130 alone are not drain. The launch policy and terminal types are
platform-neutral; AppKit and Unix-domain transport are adapters, not the contract.

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

The [host drain tests](../../crates/yir-runtime-host/tests/provider_application_session/provider_drain.rs)
cover zero/two frames, delayed Hello/frame/Drain, explicit close and Finish races,
first faults, unavailable frontiers and nonjoining ticket drop. The
[window drain ABI test](../../crates/yir-runtime-host/tests/provider_application_session/window/provider_drain.rs)
checks invalid slots and independent receipts after window free. The
[Metal host regression](../../tools/nuis/src/artifact_device_sample_shader_host_drain_tests.rs)
uses compiled Nuis callbacks through WindowSession, verifies exact GPU pixels,
zero/two-frame drain, worker-image removal, no parent outcome and replay-only
observation. It runs inside the image drain regression with a successful compiled
binary baseline and unchanged prior output evidence.

The [packaged drain regression](../../tools/nuis/src/artifact_device_sample_shader_packaged_drain_tests.rs)
extends that same compiled binary with explicit one/two-frame Metal drain,
production-frontdoor typed cancellation, unchanged success evidence, pre-publication
Finish rejection and replay-only exit 1. Generated-host tests verify delegation,
pending/missing receipts and unchanged default cancellation. Portable policy tests
exercise terminal/count/exit mismatches without a window or backend dependency.

Non-AppKit packaged session entry and Windows transport remain unverified/missing,
not implied by portable types. Parent-pump cancellation is not yet wired. Normal window
quit still uses explicit close. General device preemption, cross-session resource
retirement, cancellation of resource-capability state, recovery, multi-child
routing, fully native CPU execution and self-contained Nsld remain open.
