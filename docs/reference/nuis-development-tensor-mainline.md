# Application-Led Mainline Selection

`nuis-dev-tensor-mainline-v1` adds a dependency selection overlay to the existing
architecture/module/function tensor. The canonical plan is
[nuis-development-tensor.mainline.toml](nuis-development-tensor.mainline.toml).
It does not change language semantics, Nustar registration, compiler replacement
authority, or the recursive tensor hierarchy.

## Selection Contract

The plan declares an ID, ordered `goals`, ordered `interrupts`, and `nodes` with
explicit `depends_on` edges. Goal order is deliberate work ordering; node and
dependency registration order do not decide priority.

1. Validate the complete plan against registered tensor coordinates. Reject
   unknown fields, missing assignments, duplicate fields/nodes/array entries,
   unknown coordinates or dependencies, cycles, and unreachable nodes.
2. A node is closed only when its own cell is `stable/100` and every dependency
   is closed. Reopening a prerequisite reopens its dependent closure even if
   the dependent's historical evidence remains stable.
3. Select the first incomplete interrupt, otherwise the first incomplete goal.
   Interrupts are explicitly reviewed correctness regressions such as corruption,
   leaked resources, invalid synchronization, or broken execution. Their tensor
   cells must carry the reproduction and fix acceptance; a low score alone is
   not an interrupt.
4. Walk that root's dependency closure. Only incomplete nodes whose prerequisites
   are closed are actionable. Choose the weakest actionable node by
   `status_rank -> progress -> coordinate`.
5. Publish the goal, selected coordinate, pending goals and a deterministic
   shortest dependency path. Task/handoff ancestry still resolves through the
   existing hierarchy; dependency edges do not become hierarchy parent edges.

JSON and text expose `mainline_protocol`, `mainline_plan_source`,
`mainline_status`, `mainline_id`, `mainline_target`, `mainline_selected`,
`mainline_reason`, `mainline_dependency_path`, and `mainline_pending_goals`.
The existing `weakest_bootstrap_task_card_*` fields retain their transport names
but now carry the selected mainline task. Separate `weakest_bootstrap_*`
statistics remain preparation-gate diagnostics, not current task selection.
No hardware or application name is hardcoded in the algorithm.

Task-card source values are:

* `mainline-goal-dependency-frontier`: selected prerequisite or goal work;
* `mainline-blocking-regression`: an explicit interrupt or its prerequisite;
* `mainline-plan-invalid`: blocked, with no global fallback or actionable card;
* `mainline-plan-complete`: all declared goals and interrupts are closed, not
  every project capability and not the self-hosting programme.

A ready card still requires clean coverage, hierarchy and lineage validation.
A missing or invalid plan never silently falls back to another project area.
An invalid hierarchy or coordinate inventory also prevents a complete card.
The manifest uses a strict TOML subset: quoted ASCII protocol identifiers, string
arrays (inline or multiline), full-line comments, and `[[nodes]]` tables. Escapes,
inline comments and unknown syntax are rejected. It is bounded to 64 KiB and
256 nodes, and is read repository-relatively without writing caches.

## Recalibrated Coordinates

The previous `application-rendering-framework` score accumulated many bounded
proofs under one broad engine label. Its `active/99` is replaced by an
`active/20` scope marker. This is a conservative triage indicator, not a measured
percentage of engine completion or a regression in the verified three-frame
artifact. Existing Metal, replay, completion and GLM evidence stays intact.

That broad scope includes the planned rendering, control, ML and audio
[engine horizon](../versioning/nuis-beta-0.11-application-led-mainline.md#engine-capability-horizon).
The lifecycle manifest's `[engine_target]` records it without claiming integrated
execution. No progress score is raised for this direction update. ML, audio and
combined-workflow acceptance need separate coordinates before implementation is
scheduled; they are not extra prerequisites of the current image goal or the
finite compiler-module migration plan.

| Coordinate Suffix | Current Triage | Evidence Needed To Close |
| --- | --- | --- |
| `ns-nova/persistent-application-session` | active/86 | Packaged CPU parent and one outcome handoff are verified. Pump/window cancellation arbitrates with Finish and independently acknowledges host-scope retirement through the ticket C ABI. AppKit policy and provider/device retirement remain open, as do multi-child routing and recovery. |
| `application-session/lifecycle-failure-resource-safety` | early/0 | Scalar-state failed cleanup is tested, but cancellation, resize and in-flight close must still account for owned resource capabilities. |
| `shader/shader-resource-bindings` | early/15 | Image and parameter bindings execute through registered, reflected resource contracts. |
| `ns-nova/interactive-image-workflow` | early/0 | Load, zoom, parameter change, redraw and export operate in one Nuis-owned application. |
| `ns-nova/sustained-runtime-performance` | early/0 | Repeatable cold/warm and sustained measurements with semantic and resource checks. |
| `bootstrap/compiler-component-ownership-transfer` | early/10 | A named Nuis compiler component is selected in an application build with verified rollback. |
| `nuisc/native-cpu-frame-dispatch` | early/10 | Native stateful frame/event execution matches the retained embedded-YIR reference. |

These are new scope-specific coordinates, not retroactive changes to the five
self-hosting preparation gates. `required = false` and `bootstrap_critical = false`
mean they are outside that historical prerequisite set; they are nevertheless
mandatory dependencies/goals of the declared mainline plan.

The current goal is `standard-library/ns-nova/interactive-image-workflow`.
Its first actionable prerequisite is
`standard-library/ns-nova/persistent-application-session`. Shader resources and
failure testing follow that prerequisite, rather than their lower scores
prematurely displacing session work. A reopened shared std/runtime prerequisite
can correctly take priority within the same dependency chain.

The new [host cancellation boundary](nuis-yir-application-cancellation-v1.md)
does not raise this coordinate's score or close resource safety. One cancellation
ticket distinguishes admission from release of worker-owned host state; provider
retirement is not inferred from socket Drop. The registered window and C ABI
now forward that independent ticket without manufacturing any terminal outcome
or parent delivery, including after rejected window calls. Tickets can outlive
their window; typed late faults are delivered without mutating old snapshots.
Next wire explicit packaged-host cancellation policy, then establish explicit
provider-owned drain acknowledgement before resource reuse.
Provider sessions now depend only on the two-operation `ScopeAdmission` interface,
not on the concrete cancellation controller. Static policy-substitution tests
preserve the lifecycle gate. The generic ticket C ABI has no window, concrete
pump/controller or provider dependency; window adapters must not inherit provider
internals or merge parent outcome authority with resource retirement.

The [persistent session boundary](nuis-yir-application-session-v1.md) now calls
explicit compiled Nuis helpers in one execution context. It preserves scalar
aggregate state and provider clock frontiers between separate host calls, drains
per-call frames, and bounds close to one attempt. A second integration path now
uses one real Metal worker through one admitted IPC connection for two independent
events, validates every image pixel, and reproduces both events through replay.
Static `application_sessions` declarations now retain host-call helper roots,
survive canonical YIR/binary embedding and select the scoped session by ID. The
shared YIR contract rejects signature drift before provider connection; changing
the registration also changes source identity and invalidates old replay.
The owned event pump now keeps this registered scope in one worker across host
deliveries. Nonblocking admission/polling allows only one outstanding operation,
including an unconsumed reply; close success waits for the provider acknowledgement.
Protocol tests exercise delayed replies, failures, abandonment and unchanged
dispatch budgets, while the Metal test verifies both image deliveries and replay
through the owned handle. The explicit `--window-session ID` route now consumes
that handle inside the compiled AppKit process, bypassing native main and the
whole-module timer. Its queued logical input regression checks initial redraw,
space/Unicode input, two exact Metal images, one worker and confirmed close; the
production `run-artifact` frontdoor is included. The default legacy preview is
unchanged. Detailed dispatch/replay failure causes, recovery, cancellation
and resource retirement remain open. Shared YIR budget accounting now rejects replay exhaustion
before device effects, bounds manifest/payload reads before allocation, and
preserves old replay evidence when a replacement fails validation. Tests cover
exact limits and sparse oversized files without allocating full replay payloads.
Supervised Unix IPC now separates idle waiting from a whole-request deadline:
the first byte starts one 120-second budget for prefix/header/uploads, while
quiet intervals do not reconnect, advance clocks or consume dispatches. EOF and
supervisor shutdown wake the wait. A shortened-deadline live Metal test crosses
idle gaps before independent draws and finish with one worker, exact pixels and
monotonic clocks; transport tests reject partial-message stalls. IPC v4 retains
these deadlines and budgets while changing rejection framing.

[Window profile v3](nuis-yir-window-session-v3.md) now carries a shared requested,
event-failed or host-failed close intent into Nuis. Std cleanup preserves failed
status and accepted frame identity. The pump separately reports cleanup completion
and lifecycle success; host failure is latched before cleanup, and late Finish
failure never repeats it. A compiled-window injected provider failure completes
one Nuis cleanup while retaining failure and previous replay files. Protocol tests
also reject the 257th dispatch without losing the last state. The supervisor
allows a latched five-second cleanup grace; a stricter total child deadline still
wins. This is not recovery, resource retirement or a live full-budget soak test.
Producer-observed failure categories now cross the executor's String diagnostic
boundary through a scope-owned first-failure latch, not text matching or global
state. Nuis retains the category in its application state. The compiled window
also consumes two saved frames then rejects a third draw with `ReplayExhausted`;
there is no substitute frame, successful exit or replacement replay evidence.
Tests distinguish local dispatch limits, peer rejection, disconnect and result
drift; simultaneous event/cleanup errors keep both diagnostics and the first kind.
Late Finish is visible to the host without repeating Nuis close.
[IPC v4](nuis-yir-provider-runtime-ipc-v4.md) now admits producer-owned rejection
phase, sequence and code before delivery. Remote budget, execution and
finalization are distinct; incorrect phase/sequence is a contract failure, and
unknown/legacy framing is rejected. Provider-close failure cannot overwrite an
earlier execution fault or produce a successful acknowledgement. The compiled
window's injected request-reader failure now reaches Nuis as `ProviderExchange`,
not a text-only rejection. Exchange errors still group I/O and framing, and
execution errors do not identify hardware-specific causes.
The independent [outcome v1](nuis-yir-application-outcome-v1.md) now exposes a
stable status/cleanup/failure snapshot without changing the old Nuis state.
A protocol-injected late Finish from the compiled image reaches a compiled Nuis
parent that was already open and had accepted an earlier event. Explicit
outcome-delivery-v1 consumes one owned attempt under scoped function/node fuel;
no new context, child capability transfer or automatic retry occurs. Native
execution checks the Nuis reducer too. The runtime/FFI getters remain readonly.
The explicit [packaged parent profile](nuis-yir-application-outcome-pump-v1.md)
now runs that handoff on an independent worker, initialized from callback roots
before the child. Real Metal and compiled failure/replay cases retain one device
worker, one parent delivery/cleanup, and failed child exit despite parent success.
`active/86` is not an engine-completion percentage: multi-child routing,
packaged-host cancellation/provider retirement, long-duration soak and transactional publication
remain open. Fuel does not preempt provider-private work, blocking FFI or devices;
do not replace explicit delivery with an unchecked post-close child callback.

## Validation And Scope

Each cell separates existing `evidence`, the next action, baseline validation
commands, and the new `expected_artifact`. Passing an existing three-frame test
does not establish persistent state, sustained performance, or compiler ownership.
New acceptance tests must exist and execute before a cell can be closed.

Use `nuis dev-tensor --json` to inspect the plan and task card, and
`cargo test -p nuis --bin nuis dev_tensor -j 1 -- --test-threads=1` to check
selection, malformed plans, dependency regressions, hierarchy, coverage and drift.
Pattern drift checks bind declarations; they are not substitutes for runtime
or device execution tests.

Development policy and module-migration boundaries are described in
[the application-led roadmap](../versioning/nuis-beta-0.11-application-led-mainline.md).
