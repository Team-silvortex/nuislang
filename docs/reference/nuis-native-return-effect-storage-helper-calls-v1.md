# Native Return Storage Helper-Call Prefix Writes V1

This bounded CPU follow-up extends the
[ordered reassignment proof](nuis-native-return-effect-storage-reassignment-v1.md).
The inspected Git identity is beta-0.17.4 at bef7a23d; this work makes no release.

## Existing Lowering, Not New Authority

The original effectful-call catalog, typed prefix scan and independent/shared
certificates already admit these record-returning calls. No production call,
purity, ownership, GLM or storage-sharing rule is widened. The only production
module edit registers a test module. Real source witnesses replace the previous
uncertainty about whether this existing lowering route can execute.

The [source adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs)
adds `HelperCalls`, a fifth explicit fixture mode. Two actual assignments call
`prefix_record(prefix_seed(carry), prefix_delta(...), prefix_gate(...))`.
All four helpers are real `@noinline` Nuis functions with distinct literal
prints. The aggregate argument is captured before the scalar arguments; the
callee prints before and after its checked division and returns a mixed
bool/i64/f32/f64 record. Integer +3/-3 and two float sign flips preserve the
established independent source oracle on its bounded inputs.

Successful argument/callee markers are `[710, 711, 714, 712, 713]` per call.
The first call precedes print 703; the second precedes print 702. The complete
successful prefix is `[5, 701, 710, 711, 714, 712, 713, 703, 710, 711, 714, 712, 713, 702]`.
The oracle constructs this order independently of generated YIR node storage.
The helper's `0 / root` must still check a zero divisor; a zero numerator does
not grant permission to eliminate a selected failure.

[Certificate controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_helper_call_tests.rs)
rename helpers and storage across eight variants, preserve the entire typed
prefix and source module, and retain the independent pending identity.
Removing any of the four callees from the catalog denies preparation.
Dynamic printing remains outside this literal-print profile; it does not gain
admission through an aggregate result. Type-valid reordering, deletion,
duplication, argument swapping and bypassing the aggregate-argument helper are
rejected by the structural proof. Failed-first/later-successful candidate
selection still composes with these calls.

[YIR comparisons](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
compare independent and shared lowering, actual entry values, ordered effects
and selected failures for all eight variants with normal/reversed YIR storage.
The four-word maximum scoped carry reduction remains a logical YIR fact,
not a physical-memory or speedup measurement.

## Argument And Callee Failure

The same source uses two additional selected inputs in its second assignment:
`root = -7` fails inside the second argument, after the first aggregate argument;
`root = -8` completes all arguments and fails inside the record-returning callee.
Neither input needs a separately privileged source program or runtime grant.

[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs)
check raw State words, actual previous-State handoff, local exits, full reached
loop reservations and selected failures. Two step representatives check these
exact flushed native prefixes:

```text
argument failure: [5, 701, 710, 711, 714, 712, 713, 703, 710, 711]
callee failure:   [5, 701, 710, 711, 714, 712, 713, 703, 710, 711, 714, 712]
```

The argument failure skips the last argument and callee entirely. The callee
failure skips its completion print, the subsequent print 702 and callback
publication. Reference sessions retain the last accepted State, enter Faulted,
reject subsequent events and refuse successful completion after close.
Native fatal observations prove flushed prefixes and absent normal publication;
they do not inspect memory after process termination.

## Source-Free Delivery

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
adds one helper-call representative to the four preceding source modes.
Fresh/cache/two restored images run healthy sequences around both prefix-helper
failures and the preceding selected loop/fallback failures. Each program retains
its own binary/container, host objects, envelope, lifecycle and packaging
identity. Original source, manifest, project cache and original build output are
removed before restoration. Artifact bytes remain unchanged after both restores.
Exact fatal stdout is checked by the flushed native probe, not buffered CLI output.
This is one helper-call source-free representative, not all eight variants.

## Remaining Frontier

The later [reached-loop follow-up](nuis-native-return-effect-storage-loop-calls-v1.md)
provides witnesses for the next task below. This receipt retains its historical
scope and counts.

The tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next establish equivalent real witnesses for effectful record-returning helper
assignments inside reached loop bodies, before selected return/break/continue:
once-only ordered work, selected helper failure, actual previous-State retention,
full reached-bound reservations and source-free delivery. Preserve original-read
authority, independent fallback and existing call grants; first determine what
the current lowering already handles rather than assuming it is unsupported.

Resource/I/O/async captures, stable ABI, device/cross-platform parity, physical
memory/performance gains, complete heterogeneous closure and finished
self-hosting remain separate gates.

## Validation

This follow-up passes 327 distinct focused tests, ten new: 246 compiler units,
20 native/reference/reservation/failure/preceding-break compatibility tests,
five CLI delivery tests, three host-fault tests, 24 repository-maintenance
tests, two line-budget tests and 27 tensor tests. The standalone initial
execution probe is included in the 246-unit result, not added again. Previous
receipts retain their own historical acceptance totals.

The compiler suite passes in 234.02 seconds, native suite in 385.94 seconds and
CLI delivery suite in 412.91 seconds on this M2. These are validation durations,
not runtime speedup measurements. All five fixture modes retain their prior
behavior, raw values, ordered work, local exits and failure boundaries.
The five CLI representatives cover 20 images total, including four images for
the helper-call source, not eight-variant source-free coverage. Both additional
helper failures are selected in that same program in every delivery image.

All 3,467 registered drift checks and 23,782 required patterns pass. The rebuilt
ordinary CLI exactly matches all six source tensor values and complete
predecessor hashes, with clean coverage/hierarchy/lineage and a ready task card.
The selected coordinate remains active/99, now pointing at reached-loop helper
assignments rather than already witnessed prefix calls. Shell syntax and
inert-history probes confirm that prior validation commands remain data.
Locked metadata for 29 packages, workspace formatting, UTF-8, relative document
links and source/test/document line budgets pass; no exception is enlarged.

The first standalone probe exposed a test-adapter insertion needle that omitted
the existing `@noinline` annotation. Correcting the fixture, not granting new
production authority, made the real program reach lowering. The repository
maintenance suite was rerun after the proof-index link was added and passes.
No compiler/native execution failure was hidden by weakening a gate.
No commit, push, tag or version bump is performed. No new Linux, Windows, GPU,
full-workspace semantic, stable-ABI, physical-memory/performance or completed
self-hosting acceptance is claimed.

The focused executable routes are:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_return_shared:: state_loop_return_parent_mixed_keeps_preceding_parent_break_native_profiles_unchanged --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_return_shared:: --test-threads=1 --nocapture
```

The six current tensor values retain these complete predecessor values, each
with its own SHA-256 and explicit checkpoint marker. The previous command chain
is retained inside an inert here-document rather than executed again.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `19efdde8cb5ed8830193f8807fac96f90e392eff0bcc44872f7d8bc0327dbc8d` |
| `next_step` | `d09ae34c62b1d9a3f94b69815c33a5dc81144f6e4416288e75072cbeedbbac6e` |
| `blocker` | `3a775d6cf8845d39ce3a07db1b83a2171ebd632b4413f8cb1c15121c8d96aa57` |
| `next_action` | `ab320d34d046523a53136a9027a9ae25c265eb4f50917057081150ba2e1589eb` |
| `validation_command` | `33755e95ffee038b15e23f3c0380bec832ee95b9c9c20ae7330e15c75f047491` |
| `expected_artifact` | `fe9ae3486dde9de4199123ef0699e0bde2ea700336e7b31e98d8354da933f952` |
