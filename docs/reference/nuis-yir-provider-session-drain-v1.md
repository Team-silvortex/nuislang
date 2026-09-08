# Provider Session Drain

`nuis-yir-provider-session-drain-v1` is an explicit terminal extension of
[provider IPC v4](nuis-yir-provider-runtime-ipc-v4.md). It retires one admitted
provider-owned worker session without claiming successful application completion.
Its wire and provider implementation are independent of window state and parent
outcomes. A host cancellation ticket can explicitly request and observe it.

## Request And Receipt

`Drain(SessionDrain)` and `Drained(SessionDrain)` use the existing bounded IPC
framing with these header fields:

```text
nuis-yir-provider-runtime-ipc-v4
drain (or drained)
nuis-yir-provider-session-drain-v1
<completed-dispatch-count>
<source-yir-fnv1a64>
<module>
<instruction>
<node>
<resource>
```

There is no payload, new resource capability or additional dispatch budget.
The target must equal this connection's Hello and the count must equal its
completed Frame frontier, including zero or the exact 256-dispatch limit.
Malformed contracts, noncanonical counts, changed targets, stale/future counts
and unsolicited receipts fail closed. Unknown extensions fail on older peers;
there is no automatic negotiation, reconnect or fallback to Finish.
The receipt is connection-scoped, not a durable or transferable reuse token.

## Provider Ownership

The dispatch loop stops at a valid Drain, including before any already-buffered
subsequent request. Retained frame/replay material leaves scope. The provider
then consumes its registered session: logical leases close, worker close receipts
are validated, worker processes exit successfully, and the worker-image directory
is removed. Only successful close allows `Drained` to be written.

This reuses provider-owned registration and transport close behavior, not a
Metal-specific branch or a window-driven destructor. Existing work finishes at
the provider boundary; Drain is not instruction-level or device preemption.
Existing transport/worker waits still apply and are not a new wall-time guarantee.

The platform-neutral YIR contract `ProviderRuntimeSessionOutcome` distinguishes `Finished(count)` from
`Drained(receipt)`. The success-only launcher calls `into_finished_count`, which
rejects Drained. Drain never persists replacement result streams, output payloads,
successful completion records or parent deliveries. Normal Finish/Closed keeps
its independent lifecycle and publication requirements.

Close failure produces a `drain`/Finalization rejection and no receipt; receipt
write failure produces `drain`/Exchange. A previous dispatch/receive failure stays
the first failure even if cleanup also fails. EOF is still a receive/exchange
failure, not implicit Drain. Receiving a valid receipt does not prove application
success, retire caller-owned snapshots, authorize cross-session reuse, or certify
unrelated device allocations.

## Evidence And Remaining Work

[Core tests](../../crates/yir-core/src/provider_runtime_drain_tests.rs) exercise
framing, independent contract/target/count identity and phase-specific rejection.
[Provider tests](../../tools/nsdb/src/provider_runtime_ipc_drain_tests.rs) exercise
zero/midstream/exhausted-budget drain, no post-drain admission, close-before-ack,
no publication, mismatches, missing receipts and first-fault preservation.
The [Metal regression](../../tools/nuis/src/artifact_device_sample_shader_drain_tests.rs)
builds the Nuis image application, establishes completed output through its host
binary, then exercises direct registered-provider Drain after zero/two real GPU
frames. It checks exact pixels, physical completion, worker-image removal and
unchanged previously successful evidence. The same regression also invokes the
[host-library cancellation path](../../tools/nuis/src/artifact_device_sample_shader_host_drain_tests.rs)
with compiled Nuis callbacks: WindowSession forwards opt-in cancellation after
zero/two frames, its ticket observes exact scoped drain, and replay cancellation
reports ReplayOnly. The window can be dropped before reading the receipt; no
application outcome, parent delivery or replacement replay is produced.

The [host cancellation ticket](nuis-yir-application-cancellation-v1.md) now carries
an independent opt-in observation. A damaged/rejected exchange cannot be retried
or drained, and successful drain cannot erase a prior application fault. The
generic provider-scope admission policy seals abandonment before transport drop;
the window never receives provider internals.

## Packaged Launch Policy

The explicit scripted host accepts `--window-cancel-after-events --drain-provider`.
The CLI requires exactly one compatible
`application_provider_drain_contract=nuis-yir-provider-session-drain-v1`
bundle declaration before launch. Missing, old, suffixed, duplicate or conflicting
declarations reject rather than fall back to ordinary close.

The [shared launch policy](../../tools/nuis/src/artifact_runtime_provider_lifecycle.rs)
contains no socket, OS, window or concrete provider implementation. Its
`CompletionOnly` policy retains normal Finished accounting. `ExplicitDrain`
rejects Finish before the provider can publish completion, stops session admission
after Drained, and requires both that typed terminal and the child's cancellation
exit code. Missing sessions, EOF, exit 130 alone, normal Finished, a zero/error
exit or a late additional session cannot certify drain. The first policy error
is permanent. Shape checking a typed receipt does not authenticate it; the
registered provider remains responsible for admission and close-before-receipt.

`ArtifactRunOutcome::Cancelled(receipt)` preserves that non-success observation;
the CLI returns 130 without persisting successful launch/trace evidence. This is
a logical process status, not a platform signal or application outcome. The
AppKit adapter only forwards tickets and observations and calls the common
receipt-to-exit classifier. Replay-only, missing or failed observations and prior
application faults cannot obtain the confirmed-drain exit.

The [packaged regression](../../tools/nuis/src/artifact_device_sample_shader_packaged_drain_tests.rs)
runs the compiled Nuis image window after one/two real Metal frames and through
the production frontdoor. It checks typed cancellation, worker-image removal,
unchanged successful evidence, rejected ordinary Finish under drain intent, and
replay-only rejection. Portable policy tests cover malformed/missing terminals,
wrong exits, counts, first errors and implementation dependency boundaries.

Default host-only cancellation still sends no Drain and reports EOF as incomplete
provider execution. Ordinary quit retains close/Finish. The current packaged
window adapter is AppKit and the live transport is Unix-domain IPC: shared policy
does not certify a Windows transport, a non-AppKit host, non-Metal execution,
general interactive/parent cancellation or cross-session resource reuse.
