# Native Scalar Value Returns V1

Status: private LLVM transport for the experimental
[native scalar session bridge](nuis-native-scalar-session-bridge-v1.md).
This is not a public ABI, a native provider callback contract, or a replacement
for the ordinary owned-aggregate ABI. It does not change application-host selection.

## Layout And Admission

Registered callback roots and ordinary scalar helpers return LLVM `[N x i64]`
values, not pointer bits to heap-owned aggregates. The transport accepts nonempty
nested nominal structs with 1..64 `bool`/`i32`/`i64`/`f32`/`f64` leaves. Empty structs,
resource fields, duplicate sibling names and duplicate flattened paths are rejected.
The shared owned-layout parser retains its byte, field, name and 64-level depth bounds.
The slot bound counts scalar leaves, not only the outer struct's field count.

Callback and helper admission remain distinct:

- Callback layouts bind to the registered session's flattened state signature.
- Each ordinary aggregate call requires exact agreement with its callee's declared
  nominal layout, nested names, field order, kinds and result ownership.
- User aggregate parameters are flattened to exact scalar parameters. General
  owned/resource inputs do not acquire scalar admission or implicit conversions.
- Every reachable function still passes CPU instruction, lane, dependency, parameter,
  graph, node and depth checks. Recursive calls and hidden effects remain rejected.
- Scoped iteration calls retain their separate flat-i64 carry schema; accepting a
  mixed helper return does not make it a valid mixed loop carry.

The producer validates layouts through
[aggregate admission](../../crates/yir-lower-llvm/src/native_session/aggregates.rs),
then selects the private return plan. Both helper and root paths share the existing
YIR scalar schema without requiring a helper to register as a lifecycle callback.
Duplicate nested parent names are checked at every struct level, even when the
resulting leaf paths would be different.

## Value Transport

The [return plan](../../crates/yir-lower-llvm/src/native_session/aggregate_values.rs)
matches runtime values against exact declared nominal layouts before packing them.
Constructor order may differ from declaration order; fields are matched by name,
then published in the declared order. Each leaf is initialized before return.

Packing reuses the existing owned-payload scalar bit rules: bool zero-extends,
i32 sign-extends, f32 retains its raw low 32 bits and f64 retains all 64 bits.
Calls extract typed independent values and reconstruct nested nominal structs.
Earlier results remain independent of subsequent calls, relay helpers and loops.
Callback roots invoked as ordinary helpers use the same private return signature.

Terminal and guarded early returns share this path. A selected early return packs
its own result; an unselected fallible suffix does not execute. Helper entries still
consume the same per-invocation budget as scalar-returning helpers, with no resets
or refunds at aggregate calls. Arithmetic or budget failure remains a process trap,
not a catchable status, transactional rollback or retry.

There are no returned stack pointers, shared scratch globals or aggregate heap
allocations in this admitted path. LLVM still decides target-specific register,
stack and implicit-result-address lowering. Zero aggregate allocation is not a
promise of zero stack traffic, whole-program memory safety or faster wall time.
Ordinary LLVM emission, resource aggregates, external FFI and task thunks retain
their separate ABI and ownership policies.

## Callback Publication

The wrapper checks argument/output shape and canonical scalar words before entry.
It loads every input, calls the root, extracts every result word, then publishes
output with alignment 1. Full overlap and forward/backward partial overlap therefore
preserve the input snapshot. Status 1/2 failures do not enter a helper or modify
output. Native traps do not publish a partial result.

The reference application host separately normalizes named state fields; it does
not become allocation-free merely because native LLVM returns use slot values.
Source evaluation order remains independent of declared field storage order.

## Guarded Local Values

Matching local bindings and rebindings now admit mixed/nested scalar records through
the [typed value profile](../../tools/nuisc/src/lowering/buffer_loop_outline/control_values/layouts.rs).
It resolves nominal layouts from leaves upward, rejecting cycles, unknown/resource
fields, generics, empty records and duplicate fields. New nested neutral initializer
expansion is bounded to 64 record levels and 4096 nodes; flat source admission is
unchanged. This source normalization bound is separate from native slot admission.

The predicate is evaluated once. Only already-bound captures enter the extracted
helper; calls and checked arithmetic remain behind their branch guard. Total field
projections of ready pure records may be moved into private capture transport.
One-sided rebindings preserve the old value, and discarded selected arithmetic is
not erased. Guard defaults independently require exact typed literal zeros, with
no calls or projections. Literal i64-to-i32 narrowing emits a wrapped i32 constant;
it does not widen the native instruction whitelist to dynamic conversions.

Typed helper discovery reuses the proven flat loop catalog without admitting new
mixed loop bodies. Existing direct typed returns do not acquire extra branch helpers
merely because the new profile can describe them. Resource/effectful arms remain
outside this pure-value extraction; i32/floating arithmetic is not newly generalized.

[Local-value probes](../../tools/nuisc/tests/native_application_bridge/typed_local_values.rs)
cover 1/6/63/64-slot records, matching and one-sided choices, reversed constructors and
YIR storage order, independent snapshots, raw floating bits, in-place publication and zero
aggregate allocations/drops, compared with reference open/event/close execution.
The former 64-leaf mixed capture plus predicate now fits through private boolean
capture transport. A helper that actually consumes a whole record with 64 independent
i64 leaves plus its predicate still needs 65 arguments and is rejected by the unchanged bound.

## Private Capture Transport

Only helpers identified by the outliner, not source names or annotations, acquire
the [capture plan](../../tools/nuisc/src/lowering/direct_calls/capture_params.rs).
User functions, callback roots, FFI and scoped iteration signatures stay unchanged.
Generated helpers used as scoped actions are also excluded, using the same target
discovery as scoped-call lowering; their per-trip induction/carry metadata is not packed.
The same plan lowers both the call arguments and the callee's parameters, using
declaration-order scalar leaves and exact nominal reconstruction. The normal owned
return ABI is unchanged; the native value-return path remains heap-free.

Two or more boolean captures share nonnegative i64 words of at most 63 bits.
Singleton tails keep their boolean type, while i32/i64/f32/f64 leaves keep their
original scalar kinds and bits. The first replaced leaf supplies a unique physical
parameter name. Packing uses canonical bool-to-word conversions and bounded
multiply/add; decoding uses positive constant division/remainder and word-to-bool.
The top bit is 62, so an all-true word is exactly i64::MAX, never signed overflow.
These existing YIR operations do not widen the native instruction whitelist.

Arguments are evaluated before packing, including the one-time predicate; no fallible
or effectful branch work is duplicated or hoisted. Decoding is total and precedes the helper guard.
Helper identities, entry charges, loop preflights and failure publication are unchanged.
Physical parameters still undergo the independent native 64-slot check; packing is
not a general large-argument ABI, a new mixed loop carry, or a claim of faster execution.

[Word probes](../../tools/nuisc/tests/native_application_bridge/typed_capture_words.rs)
cover independent boolean patterns, the highest nonnegative bit and a second word,
reversed storage order, in-place lifecycle publication and zero allocation/drop counters.
The [64-slot CLI workflow](../../tools/nuis/tests/native_session_workflow/capture_words.rs)
uses an independent predicate and a mixed record at exactly 64 physical arguments,
with build/run-artifact, malformed input, tamper, cache and source-free restoration checks.
## Field-Selective Captures

The [projection pass](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs)
runs before private boolean packing. It processes generated callees before their
callers, so a chain of private helpers does not keep forwarding an entire record
when its terminal users need only a few fields. Function storage order is irrelevant.
Only an actual reduction in scalar leaves changes the signature.

Each selected path becomes a hygienic private parameter. Repeated paths are shared;
using an entire nested subrecord subsumes its child paths and preserves its exact
nominal type. No missing fields are filled with synthetic zeros or partial records.
Definition, every call site and NIR verification move together before YIR lowering.
Unchanged user signatures still flatten in declaration order; projected parameters
use deterministic path order, and each retained subrecord keeps declaration order.

Only ready variable/field paths can be duplicated or discarded at a call boundary.
Computed record arguments veto projection rather than losing effects or failures.
Scalar arguments, including unused checked arithmetic and predicates, are not removed.
Whole-record returns/forwarding and control-flow-dependent rebinding conservatively keep
the required capture. Unsupported callers, scoped targets without the opt-in proof
below, cycles and their dependent helpers are not rewritten. The shared read-only expression walk is exhaustive,
including conversions, FFI operands and shader/kernel children.

[Sparse native probes](../../tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs)
execute a nested 64-i64 state with only four private selection arguments: one predicate
and three fields. The public helper still has 64 parameters. All lifecycle states
match independent expectations and reference execution with reversed YIR storage,
in-place event/close publication, whole-region sentinels and zero allocation/drop counters.
The [sparse CLI workflow](../../tools/nuis/tests/native_session_workflow/capture_fields.rs)
checks the same fixture through build, cache reuse, input/tamper rejection and
source-free materialization with byte-identical LLVM and unchanged lifecycle states.
Mixed sparse guard probes retain raw float bits, skipped division, selected zero/overflow
traps and unchanged output sentinels. Both guarded arms still cost entries: the fixture
needs exactly six, rejects five/zero, and rejects malformed input before helper entry.

The [alias normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_aliases.rs)
now resolves immutable Let/Const record chains rooted in stable helper parameters.
Nested field origins retain exact nominal and scalar types, including mixed records.
Distinct branch-local aliases have independent environments and do not escape.
Fallthrough cross-scope writes and rebound iteration-local records remain conservative;
a stable outer alias can still be read inside a loop. Same-name aliases in
independent branches now reduce to individual demanded fields through the lexical
identity pass below.

Alias normalization is transactional: it runs on a candidate copy and commits only
with actual leaf reduction and successful call-site preflight. Whole-record demand,
computed caller arguments, annotations that disagree with origins, resources and
reference types cannot silently change the original helper's contract.
The shared [alias fixture](../../tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs)
uses chains inside the generated branches of a 64-i64 helper. Those private branches
now take two and three physical arguments instead of 65, while the public helper
retains 64. Native/reference results, in-place publication and zero allocation/drop
counters agree. CLI cache reuse and source-free restoration retain identical LLVM
and states; selected zero-divisor and signed-overflow failures still trap, without
fallback, event publication or completion.

Forward branch substitution now materializes a binding before continuing when it
or any of its inputs is rebound later. Previously, substituting an old alias with
a live variable name could silently read the later value. The shared
[Nuis snapshot regression](../../tools/nuisc/tests/control_flow_syntax_native/alias_snapshots.ns)
covers origin/alias rebinding, guarded returns and branch binding chains in both
reference and ordinary native execution.

The [snapshot normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_snapshots.rs)
now gives straight-line record writes separate private identities before alias
discovery. It versions both rebound parameters and repeated local aliases, with
each initializer reading the preceding version, including self-rebinding.
Names reserve all existing bindings and reads before allocation. Every version
must retain the same canonical record type; untyped, reference and type-changing
versions are not rewritten. Any loop-local write or write in a non-returning child
scope vetoes all versions of the name, while nested reads can use an invariant version safely.

Pure-helper admission now accepts same-nominal-type record Let rebindings, retaining
dependency validation, checked arithmetic and existing scalar/loop restrictions.
Typed admission alone does not bypass local selection extraction; only the existing
full-control catalog authorizes that path, preserving mixed-value helper transport.
Actual signature projection remains private, transactional and reduction-only.
The 64-i64 rebound fixture keeps old snapshots and new records in the same expression,
with two/three private arguments, unchanged public inputs, reference/native parity,
in-place lifecycle publication and zero allocation/drop counters. Constructors and
calls are not expanded into aliases or erased when their result is unused: the
discarded-record regression retains selected division failure while skipping it on
the unselected path in reference and ordinary native execution.

## Branch-Local Capture Identities

The [binding normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_bindings.rs)
runs on the private candidate before snapshot versioning and alias discovery.
Fresh declarations below a branch/loop boundary receive independent identities;
parameters and already-visible bindings retain their identity on writes. Initializers
read the preceding scope. Siblings, nested scopes and later suffix declarations
do not share an identity just because their source spelling matches.

All bindings and variable reads reserve names before rewriting. Only variable uses
change; function symbols, field labels and nominal types remain untouched. Unsupported
expressions veto normalization, and the existing signature/call-site transaction
still commits only after a real leaf reduction. A computed caller or whole-record
demand therefore preserves the original body as well as its signature.

Same-name branch aliases that previously retained two complete Pair captures now
retain two demanded i64 fields. Tests cover nested aliases of different record types,
outer and parameter updates, loop writes, private-name collisions and before/after
reference execution. The shared scoped-alias source adds nested branches and a later
same-name declaration to the 64-i64 lifecycle fixture. Native and CLI regressions
retain four bounded private branches with 2/2/2/3 physical arguments, unchanged
64-parameter public inputs, exact state, zero aggregate allocations, skipped/reached
division failures, cache reuse and source-free restoration.

### Branch-Local Snapshot Versions

Straight-line rebindings of a branch-local record now receive private snapshot
identities before alias discovery. One iterative analysis tracks each hygienic
binding's write scope, exact type and loop ancestry. Writes in the owning non-loop
scope remain admissible; child writes now require the return proof below. Writes
with fallthrough joins and all iteration-local writes remain unchanged.

Each initializer sees its preceding version, including self-rebinding. Nested reads
inherit the version visible at entry; siblings and suffix declarations never inherit
another branch's versions. Constructor/call evaluation is not erased even when the
record is discarded. Unsupported types and computed callers retain the existing
conservative, transactional behavior. The direct nested-record regression reduces
one whole State capture to its two demanded i64 leaves, retaining the bool predicate.
Flat-record before/after execution covers nested branches, two self-rebindings,
sibling and suffix snapshots; selected checked constructor work still fails.

The shared scoped-rebound 64-i64 fixture combines old aliases with new values in
both nested arms and the suffix. Its native/CLI checks retain four private branches
with 2/2/2/3 physical arguments, the unchanged 64-argument public helper, zero aggregate
allocations/drops, exact lifecycle state, selected traps and source-free restoration.
This source route complements the direct NIR reduction proof; it is not a claim that
the whole frontend route was previously unbuildable.

Snapshot versioning adds no join/backedge rewrite or iteration-local write
versioning, and does not widen source control admission. Stable loop aliases use
the separate invariant-origin proof below.
This is not unrestricted scalar replacement, a wider native argument limit,
mixed loop carry admission or a measured speedup.

### Returning Child Snapshots

Cross-scope record writes can now receive separate versions when their immediate
non-loop scope is proven to return rather than rejoin its parent. This includes
parameter updates and ancestor-local updates, with preceding versions available to
initializers and old aliases. The enclosing continuation keeps its prior version;
sibling arms never inherit another arm's writes.

The [scope analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_snapshot_scopes.rs)
assigns lexical scope IDs iteratively and computes normal fallthrough and escaping
control bottom-up. Both returning arms can close a scope; a returning child loop
cannot because it may execute zero times. Break/continue do not count as function
returns, and loop ancestry is still an unconditional veto for snapshot candidates.
All writes must retain the exact declared pure-value type. A non-returning cross-scope
write vetoes every version of that binding rather than inventing a merge value.

[Direct projection regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_terminal_snapshots_tests.rs)
reduce a nested State capture to two demanded i64 fields plus its predicate. They
cover parameter/local writes, nested returning arms, top-level preceding versions,
unchanged fallthrough values, exact types, live joins, backedges and computed-caller
transactions. Before/after reference execution and LLVM emission retain selected
checked constructor work even when the constructed record is discarded.

The shared 64-field source combines an outer record/old alias, a returning child
update and a suffix update. Native/CLI execution retains two/three private branch
arguments, the unchanged 64-argument public helper, complete state, zero aggregate
allocations/drops, selected traps, cache reuse and source-free restoration. This
proves composition through the source pipeline, not that the fixture previously
failed to compile. General fallthrough joins, loop-written versions and resources
remain separate work; no native bound or source control admission is widened.

## Invariant Loop Aliases

Single-definition aliases inside loops can now expose demanded fields when their
canonical origin is a ready pure-value path rooted in an unwritten parameter.
The proof is checked explicitly before replacing an alias. Local computations,
constructors, calls, references, type mismatches and roots written anywhere in the
function cannot acquire invariant-origin authority. Rebound loop-local records
retain their per-trip snapshots; parameter and outer carry writes remain unchanged.

Child scopes inherit the preceding alias environment without exporting new locals.
The same rule handles chains and nested loops, with lexical identities separating
sibling declarations. Candidate transactions, whole-record demand, computed-caller
vetoes remain unchanged; scoped targets require the separate opt-in proof below. The
[loop-alias regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_loop_aliases_tests.rs)
project nested records to demanded scalar fields; flat-record before/after reference
execution and LLVM emission retain zero trips, selected division/overflow failures and calls.

The shared loop source adds a real repeated checked-division helper to the 64-i64
lifecycle fixture. It uses a separate two-field input record, not a wide iteration
capture. Native/CLI checks retain the unchanged public inputs, 2/3-argument private
branches, exact lifecycle state, zero aggregate allocations, selected traps, cache
reuse and source-free restoration. This is execution-composition evidence, not a
claim that every loop alias survives outlining or that wide scoped captures now fit.

That narrower fixture preceded the scoped iteration projection described below.
Fallthrough cross-scope writes and rebound iteration-local records remain conservative;
the native argument bound, backedge layouts, source loop admission and callback
budgets are not widened by alias discovery.

## Scoped Invariant Captures

Compiler-generated scoped helpers now opt into field projection separately from
ordinary generated helpers. Before rewriting signatures, the
[scoped input pass](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped.rs) examines every
scoped caller and unions protected argument positions: any computed argument or
path rooted in a loop-written binding stays whole. Induction, record/scalar carry
seeds and break-control values therefore retain their original identities. User
functions do not gain this private-signature permission.

Callees are still processed before callers. Signature and call-site changes commit
only together after real leaf reduction; whole uses, unsafe callers and cycles
remain conservative. Scoped candidates skip general local renaming and snapshot
versioning so nested break-control identities survive. Stable alias discovery can
still expose demanded fields without renaming any control binding.

[Scoped argument admission](../../tools/nuisc/src/lowering/scoped_loop_lowering/arguments.rs)
accepts field paths only from existing bindings unwritten anywhere
in that loop, including nested scopes. It materializes those ready inputs before
the loop and derives induction/carry argument positions from the rewritten calls.
Computed expressions and fields of carried records cannot use this invariant path.
Carry reconstruction, bool seed conversion, break slots and preflight are unchanged;
private boolean packing still excludes scoped targets.

The shared wide-loop source uses the original 64-field Payload inside the loop,
not a two-field wrapper. Its iteration signature drops from 66 physical parameters
to four, while the public helper retains 64. Native execution compares complete
lifecycle state against the reference, including reversed YIR storage, in-place
publication and zero aggregate allocations/drops. The CLI fixture covers cache,
tamper rejection, selected traps and byte-identical source-free restoration.
Focused NIR/reference/LLVM checks cover zero trips, leading/trailing steps,
multiple callers, record/bool carries and nested breaks. An initial nested-break
regression exposed the control-identity renaming problem and now guards its repair.
This is bounded correctness/transport evidence, not measured speedup or support
for unrestricted cross-scope record updates and mixed loop carries.

## Evidence And Limits

[Flat-return probes](../../tools/nuisc/tests/native_application_bridge/aggregate_values.rs)
execute 1/2/7/64-field helpers, early returns, reverse constructors and independent
snapshots, while verifying that ordinary LLVM still allocates owned aggregates.
The earlier four-trip nested helper change reduced 13 allocations to 1; callback
value transport subsequently removed that final allocation.

[Callback probes](../../tools/nuisc/tests/native_application_bridge/callback_values.rs)
cover 1/6/64 nested slots, all lifecycle roots, finite/negative-zero/NaN bit patterns,
unaligned full/partial overlap, disjoint buffers and whole-region sentinels.

[Typed helper probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_values.rs)
cover 1/6/64 mixed/nested slots, nested calls, flattened aggregate inputs, a callback
called as a helper, reversed constructors and reversed YIR node/function/body order.
They compare independent slot expectations with reference open/event/close execution
and real native execution, retaining zero aggregate allocations/drops. Two live
helper results contribute different fields to the final state. The open path fits
six shared entries, while event/close start fresh invocation budgets.

[Guard probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_guards.rs)
execute valid/skipped division, selected zero-divisor and signed-overflow traps,
exact three-entry success, two/zero-entry failure, and malformed-input rejection
before a zero entry budget. Trap observation retains the real trap and checks all
five output sentinels plus allocation/drop counters.

The [typed-helper CLI workflows](../../tools/nuis/tests/native_session_workflow.rs)
add mixed/nested State-returning helpers and guarded local rebindings to the multi-carry fixture.
It checks build/run-artifact, manifest identity, tamper rejection, cache reuse,
registration isolation and source-free standalone restoration, including
byte-identical restored LLVM and unchanged lifecycle states.
The guarded-local fixture also closes with a zero reason, proving that its
unselected zero-divisor state construction does not execute in the packaged binary.

[Admission probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_admission.rs)
reject nominal/kind/order/ownership/arity drift, nested resources, empty layouts,
duplicate parents, oversized/deep layouts, spoofed returned values and hidden effects.
The local-rebinding guard probes additionally check exact five-entry success,
four/zero-entry failure, unused selected results and malformed-input rejection,
with unchanged output sentinels and zero aggregate allocations/drops.
General mixed loop carries, resource state, provider dispatch, native image-host
selection, cross-target execution and performance still need separate evidence.

The 2026-09-23/24 macOS aarch64 alias/snapshot checkpoint passed 699 compiler/registry-unit,
47 native-bridge, ten ordinary native, five reference image/window cases and the host-path policy check.
Seven CLI workflows passed; exact-64-slot, sparse, alias and snapshot captures retain cache reuse,
input/tamper rejection, selected traps and source-free restoration with identical LLVM and states.
All 28 tensor tests passed; 1409 drift checks were clean (797 selected tests in total). No fresh GPU/Linux,
full-workspace, formal safety or performance result is inferred from this checkpoint.

The 2026-09-24 lexical-identity worktree follow-up to `9ec40a40` passed 601
lowering-unit tests, 36 selected native-bridge tests, two ordinary native snapshot
tests, four sparse-capture CLI workflows, 28 tensor tests and the host-path policy
case: 672 selected tests without counting overlapping reruns twice. The fresh CLI
reported 1416 clean drift checks; coverage, hierarchy and lineage remained clean,
with the session coordinate still `active/86`. This is not a full-workspace,
fresh GPU/Linux or performance result. Historical checkpoints above remain separate.

The later 2026-09-24 scope-confined snapshot follow-up passed 607 selected lowering
tests, 37 selected native-bridge tests, two ordinary native snapshot tests, five
sparse-capture CLI workflows, 26 tensor tests and the host-path policy case: 678
selected tests, excluding overlapping reruns. The new branch-local projection
regression failed before the implementation and passed afterward. The fresh CLI
reported 1418 clean drift checks, clean coverage/hierarchy/lineage and `active/86`.
All five workflows retained cache reuse, tamper rejection, selected traps and
source-free restoration. No full-workspace, fresh GPU/Linux or speedup is claimed.

The 2026-09-24 invariant-loop-alias follow-up passed 614 selected lowering tests,
38 selected native-bridge tests, two ordinary native snapshot tests, six
sparse-capture CLI workflows, 26 tensor tests and the host-path policy case: 687
selected tests, excluding overlapping reruns. The loop-local alias projection
regression failed before the implementation and passed afterward. The fresh CLI
reported 1420 clean drift checks, clean coverage/hierarchy/lineage and `active/86`.
All six workflows retained cache reuse, tamper rejection, selected traps and
source-free restoration. Direct NIR projection and narrow-loop execution evidence
do not imply field-selective scoped iteration inputs, a full-workspace result,
fresh GPU/Linux validation or a measured speedup.

The subsequent 2026-09-24 scoped-invariant-capture follow-up passed 622 selected
lowering tests, 134 selected native-bridge tests, two ordinary native snapshot
tests, seven sparse-capture CLI workflows, 26 tensor tests and the host-path policy
case: 792 selected tests without double-counting overlapping reruns. The new
64-field source regression failed with 66 iteration parameters before the change
and passes with four afterward, including a linked native executable. A separate
nested-break regression caught generic local renaming of control identities and
passes after preserving those identities. All seven CLI workflows retain cache,
tamper rejection, selected traps and byte-identical source-free restoration.
The fresh CLI reports 1426 clean drift checks, clean coverage/hierarchy/lineage
and `active/86`. This is not a full-workspace, fresh GPU/Linux or performance result.

The 2026-09-24 returning-child-snapshot follow-up passed 629 selected lowering
tests, 40 selected native-bridge tests, two ordinary native snapshot tests, eight
sparse-capture CLI workflows, 26 tensor tests and the host-path policy case: 706
selected tests, excluding overlapping reruns. The direct NIR projection regression
failed before the change and passes afterward; the shared 64-field source is
composition evidence, not a claim that the source pipeline previously rejected it.
All eight CLI workflows retain cache reuse, tamper rejection, selected traps and
byte-identical source-free restoration. The fresh CLI reports 1429 clean drift
checks, clean coverage/hierarchy/lineage and `active/86`. Returning child scopes
preserve old aliases and parent/suffix values without exporting branch versions.
Fallthrough joins and loop-written snapshots remain separate work. No full-workspace,
fresh GPU/Linux, formal safety or measured performance result is claimed.
