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
| `ns-nova/persistent-application-session` | active/86 | Packaged CPU parent and one outcome handoff are verified. Pump/window cancellation arbitrates with Finish and acknowledges host-scope retirement; explicit scripted AppKit cancellation now consumes the ticket. Parent cancellation, provider/device retirement, multi-child routing and recovery remain open. |
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
The packaged host now admits `--window-cancel-after-events` with an explicit
event script and no parent. Live Metal/replay regressions verify no implicit close,
Finish, outcome or success; pending/lost receipts have generated-host tests.
Opt-in cancellation now reaches the packaged script through a declared capability
and a typed non-success launcher result. The shared policy rejects Finish under
drain intent before publication and never derives retirement from an exit code.
Without the option, EOF remains an incomplete live provider
lifecycle, not success. Provider sessions depend on the three-operation
`ScopeAdmission` interface (checkpoint, finalization and abandonment admission),
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
unchanged. Recovery, general interactive/parent cancellation
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
general cancellation/provider retirement, long-duration soak and transactional publication
remain open. Fuel does not preempt provider-private work, blocking FFI or devices;
do not replace explicit delivery with an unchecked post-close child callback.

## Validation And Scope

The [provider-session drain boundary](nuis-yir-provider-session-drain-v1.md) now
has target/count-bound terminal messages and a provider-owned close-before-receipt
path. Drained and Finished are separate outcomes; cancelled work cannot replace
successful replay. The selected session coordinate remains `active/86`: opt-in
host-library pump/window/C ABI tickets now request drain only at a validated
frontier, retain first faults and expose independent provider status/count.
Compiled Nuis callbacks plus registered Metal verify zero/two-frame cancellation,
worker-image removal and old replay preservation. Replay is only an observation,
not device retirement. Capability-gated packaged-host drain and a typed non-success
launcher result now have one/two-frame compiled Metal evidence, including
unchanged success evidence, pre-publication Finish rejection and replay-only exit 1.
Terminal types and launch policy are platform-neutral; AppKit only forwards the
ticket and common receipt-to-exit classification. The
[headless scalar-script entry](nuis-yir-application-scalar-script-v1.md) now reuses
the common pump without WindowSession or AppKit. Compiled protocol tests and M2
Metal verify stateful events, exact frame hashes, Finish/drain separation and
unchanged old evidence. Ordinary build/run-artifact now select the headless profile,
bind capabilities and YIR to the verified image, isolate explicit build selections
in cache and reuse the same provider lifecycle. Headless builds now stop at an
explicit verified-YIR checkpoint without CPU LLVM generation or dummy intermediates.
The v2 bounded inputs preserve the complete compiler-stage handoff through artifact
relocation. Manifest-selected check/dump, benchmark/binding inspection and workflow
JSON now select verified YIR before codegen, report LLVM as `not_requested` with
no fabricated byte count, and retain default native errors. Command regressions
cover directory/manifest inputs, dump roundtrips and rejected invalid inputs;
semantic inspection is not executable or provider evidence. The score remains
`active/86`: bounded Buffer-writing application callbacks now reuse private
helpers and the existing scoped-call loop contract. The registered CPU executor
performs actual helper calls under shared fuel. Memory-operation effect edges
prevent reads from preceding writes, including nested expressions, and helper
returns retain their effects. Native `load_at`/`store_at` require length metadata
and trap on invalid accesses. Reference/native tests and repeated application events
cover strict unit-step loops, ordered reads/writes, zero/one/descending iteration,
declaration-order independence and fail-closed unsupported bodies. PixelMagic's
checkerboard generator now uses this loop with checked scalar division/remainder.
Both constant evaluators retain invalid arithmetic rather than panicking; native
scalar integer operations trap on zero divisors and signed division overflow.
The [CLI integration](../../tools/nuis/tests/headless_image_loop.rs) now builds
the ordinary image project, runs two actual M2 Metal frames, compares every output
byte with direct-session replay and rejects executable/YIR or callback-argument
drift before application effects. Packaged replay exhaustion cannot close or publish
success. Separate native/reference tests compare all generated pixels, including
partial and empty regions. PixelMagic now writes colors through explicit `if`/`else`.
Branch-local reads, writes and nested conditions use private guarded functions,
one-time condition snapshots and the same shared callback fuel. Untaken invalid
accesses or arithmetic are not evaluated; selected invalid operations still fail.
Unused checked arithmetic survives dead-binding elimination and reordered YIR
declarations cannot skip it. Source-level scalar helper composition now admits
synchronous `i64`/`bool` bodies after transitive type/effect checks
and iterative acyclic dependency admission. Only reachable helpers become real
ordered YIR calls; Buffer-read arguments remain inside the current iteration and
selected branch. PixelMagic composes private coordinate and color helpers on this
path; imported helpers retain owner-local signatures without exposing their
private declarations to consumers or sibling modules. Source helpers now admit
nested statement `if`/`else` and early returns through typed guarded blocks and
shared fallthrough continuations. Unselected branch computation stays behind its
guard, source-call arguments still evaluate at entry, and function-local returns
do not escape into the caller. Scope, trap, shared-fuel and linear-growth regressions
accompany the same native/reference pixel and M2 packaged image checks.
Multiple explicit i64 accumulators now survive Buffer-writing iterations, including
zero trips, source-order rebinding, ordered reads and scalar helper control flow.
CPU and LLVM share layout validation and return LoopState fields without an
image-specific executor path. Native returns currently allocate/release an aggregate
per iteration and multi-state branch return. PixelMagic's fill-and-statistics API checks real scalar result
composition with its existing red-count/boolean wrappers; native/reference tests
compare pixels, red counts and packed-pixel sums.
The carried image passes ordinary M2 build/run-artifact again, retaining both exact
Metal frames, direct-session byte parity and failure-preserved replay evidence.
Branch-local scalar carry updates now preserve untaken seeds, snapshot conditions
once and compose nested/repeated updates in source order. PixelMagic counts red
pixels inside the matching write arm. The 32-branch growth regression also exposed
unbounded reverse substitution in pure-helper collection; body preflight and a
bounded expression-inlining analysis prevent that expansion without a source branch limit.
The branch-carry generator also passes the real M2 packaged workflow: both Metal
frames retain exact bytes, the counting call stays inside its guarded branch, and
pre-effect drift rejection plus failure-preserved replay remain intact. A warm-cache
development run of the 32-branch compilation/reference/native regression took 2.71
seconds with about 65 MiB peak RSS; this is a local measurement, not a general memory bound.
Nested bounded Buffer-writing loops now compose recursively with independent
counters and scalar carries, guarded child bounds, protected ancestor headers and
shared callback fuel. Eight native/reference regressions cover zero/descending
trips, reset/persistent counters, branch-visible locals, ordered traps and an
eight-level/four-sibling helper-growth case. This exposed stale NIR literals after
branch joins; join invalidation and enclosing-block liveness preserve outgoing state.
PixelMagic now uses clamped row/pixel loops instead of a flat pixel loop.
The nested generator passes ordinary M2 build/run-artifact: packaged row functions
retain their inner pixel loops and real source helper calls. Both 76,800-byte Metal
frames match direct-session bytes; argument/binary/YIR rejection happens before
effects, and exhausted replay preserves the previous stream. Packaged CPU callbacks
still execute embedded YIR, not a fully native callback ABI.
Guarded `continue` now skips each iteration's remaining suffix through existing
guarded functions, preserving earlier writes and scalar carry changes. The source
path must explicitly perform the matching unit step; the loop driver executes it
once. Loop-local control flags reset per iteration and never enter application state.
Nine reference/native and session regressions cover nested scope, signed boundary
steps, skipped traps and bounds, shared fuel, rejected updates and 32-guard linear
helper growth. PixelMagic exercises this path after red writes and statistics.
The continue-based generator passes ordinary M2 headless build/run-artifact. Its
guarded red branch returns both statistics plus private control state; both
76,800-byte Metal frames match the independent pixel oracle and direct-session
replay. Pre-effect identity rejection and exhausted-replay preservation still pass.
Guarded `break` now returns a private i64 control slot through the shared scoped-call
contract. CPU registered execution and LLVM exit before stepping, preserving all
prefix writes/carries. Eleven reference/native and session regression families cover
nested scope, mixed explicit-step continue, skipped traps and child bounds, zero trips,
malformed seed/return controls, GLM dependencies, reordered declarations and private
names with linear helper growth. A trillion-iteration bound exits on its third
invocation within 1000 shared fuel; exhaustion still preserves prior application state.
Partial helper lowering also honors its declared result and traps if that result
is unavailable, rather than returning an unrelated scalar as an aggregate pointer.
The malformed-return regression covers this native function-lane drift separately.
PixelMagic `recolor_run` now replaces only the matching prefix of a bounded range.
Twenty-two native/reference cases check stop index, write count, checksum, untouched
suffixes, empty ranges and invalid inputs. The image app marks four first-row pixels
green and checks all three statistics before GPU submission. Ordinary M2 headless
build/run-artifact now retains and executes the break action: two real Metal frames
include the expected magenta marker and match every direct-session replay byte.
Replay confirms five helper invocations, stop index 4 and exactly four marker writes
per callback. Pre-effect argument/binary/YIR rejection and exhausted-replay evidence
preservation still pass. The score remains `active/86`; next is native CPU lifecycle
callback dispatch, while retaining this packaged break proof.
This is not fully native CPU callbacks, arbitrary loop support or a
complete memory-safety proof.
The default-AOT image regression now passes its LLVM checkpoint and real Metal
export without changing packaging mode. Guarded and two-arm owned-Bytes cleanup
returns validate and pack the declared aggregate layout. Native/reference tests
cover nested fields, both paths, reordered declarations and zero live native Bytes;
layout drift and directly known dropped-Bytes aliases reject before cleanup.
The compiler records two terminal branches as a returned result, while registered
function-exit hooks replace CPU operation-name dispatch in the generic executor.
Other aggregate early-return operations remain incomplete. The compiled host still
executes embedded YIR callbacks, so native CPU callback dispatch is not yet proven.
Compound-condition `while` lowering now selects recursive descriptors,
normalizes linear carries consistently and deduplicates shared effect inputs.
CPU-owned cooperative execution replaces trace-only loop results; generic YIR
function calls and resumes share invocation fuel. Thirty-six synchronous/asynchronous
scalar-loop cases agree between reference execution and native M2 binaries, with
registered callback coverage for both `&&` and `||`. This covers bounded integer
induction and add/keep carries, not arbitrary loop bodies or every carry descriptor.
Linux hardware execution is unverified and Windows transport remains missing;
no cross-session reuse permission or general device
cancellation follows from this bounded boundary.

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
