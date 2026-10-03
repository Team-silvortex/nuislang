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
  mixed helper return does not grant carry admission without a typed word map.

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
Alias discovery alone stays conservative for cross-scope writes and rebound iteration-local records;
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
failed to compile. Field-only fallthrough inputs use the separate reconstruction
proof below; no native bound or source control admission is widened.

### Record Input Reconstruction

The [copy-family proof](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_join_plan.rs)
collects full-record copies iteratively and unions all observed field paths across
each family, including initializers and old aliases. Every member must retain the
same declared pure-value type, and at least one must be an input. Whole-record
escapes and unknown or mismatched types veto the family. Field demand includes
loop conditions, every constructor RHS and all later copies. Nominal prefixes subsume
their descendant paths; candidates must reduce the number of input leaves.

The [input reconstruction](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_joins.rs)
creates a private nominal record from demanded input fields. Omitted fields receive
typed zero values only after proving they are unobservable throughout the family.
This is not defaulting an unknown runtime value. The original assignments, copies,
constructors, calls and branch shapes remain intact; their complete right-hand sides
still execute before their writes, including unused checked fields. Existing record
semantics retain old snapshots and branch-selected values. Public signatures,
backedges, scoped helper control identities and native limits remain unchanged.

[Direct regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_join_tests.rs)
cover parameter/local joins, nested and one-sided arms, multiple input copies,
nominal prefixes, mixed kinds, name hygiene, read-only loops and rejected callers.
Before/after reference execution and LLVM emission check old aliases, self-rebinding
and selected unused constructor calls. The source/native fixture combines both
reaching values with a pre-join alias, retaining 64 public inputs and three/four
private branch arguments. Source composition is separate from the direct reduction
proof. This does not implement general SSA joins, scoped loop-carried field interfaces,
resource transport or a measured speedup; ordinary owned-aggregate allocation is
not promised to improve through this private reconstruction.

The [loop-entry regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_loop_join_tests.rs)
extend this proof to loop-written families without replacing their original loop
bodies. Repeated writes within one loop scope qualify; single-definition invariant
aliases retain their separate path. Reference execution before and after projection,
plus LLVM emission, check seeded-local zero/multiple trips, per-trip old values,
nested loops, `continue`/`break`/`return`, and unused constructor work that can fail
on the second trip. Structural tests cover parameter updates, cyclic copies,
nominal prefixes, exact-type vetoes and unmodified loop bodies. Executable profiles
still use the existing seeded-local and ordered-update admission rules; parameter
loop reduction is not evidence for a wider source control-flow profile.

The shared native/CLI source composes two-trip record snapshots with a zero-trip
call whose divisor is zero. It retains 64 public inputs and two/three private branch
inputs, exact native/reference state and zero aggregate allocations/drops. Its CLI
workflow checks build/cache identity, selected arithmetic failures, tamper rejection
and byte-identical restoration without source files. This is source-composition
evidence in addition to direct NIR input reduction, not a claim that the source
pipeline previously rejected this fixture. Automatic scoped projection still keeps
loop-carried inputs whole; explicit field seeds use the separate slot-map proof below.

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
The invariant-alias pass leaves cross-scope writes and rebound iteration-local records conservative;
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

## Scoped Carry Field Seeds

The [scoped carry mapper](../../tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries.rs)
now accepts either a same-typed whole-record seed or direct flat-i64 field seeds.
One binding/type-to-slot range function is shared by admission, coverage validation
and argument encoding. Offsets follow the result layout, not helper parameter
order. Mapped fields are dynamic backedge operands, not invariant input snapshots.
Whole-record and field-mapped inputs can coexist for different carried bindings.

Every output slot still needs exactly one seed, including slots whose per-trip
values are overwritten without reading their initial values. This preserves the
zero-trip result. Repeated or overlapping argument coverage is rejected, as are
wrong scalar types and computed mutable-field arguments. Foreign/nested roots cannot name a carried slot.
Before flattening field seeds, the producer must also prove that the initial
record has the exact carried nominal type. Constructors, aliases, declared helper
returns and nested field origins retain that identity; mismatches or unproven
origins are rejected rather than changing record type on a zero-trip loop.
Bool carries retain explicit word conversion; a break flag still requires its
normalizer-proven identity and named zero seed. Nested and mixed records are not
admitted by this flat-i64 contract. Complete argument maps retain the legacy wire form.

[Mapping regressions](../../tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carry_seed_maps_tests.rs)
cover permuted arguments, whole/field combinations, bool/break identities, malformed
maps, initial nominal drift and reference execution with reversed YIR storage. A shared
[Nuis source](../../tools/nuisc/tests/control_flow_syntax_native/scoped_field_seeds.ns)
checks updated per-trip reads, an independent pre-loop snapshot and a zero-trip
zero-divisor call in both reference and ordinary native execution; selected division
failure remains a process failure. Native lifecycle and CLI fixtures use the slot
order `2,4,0,3,1`, preserve complete state with zero aggregate allocations, and check
cache identity, tamper rejection and source-free restoration. Input selection is
separate from the explicit loop driver; conditional helper admission is not widened.

### Independent Initial State

The shared [carry contract](../../crates/yir-core/src/loop_carry_contract/scoped_scalars.rs)
now optionally accepts `$carry_seeds N seed0 ... seedN-1` immediately after the
result layout. N equals the full layout width; every seed is a named input in
declaration order. Subsequent operands alone determine helper arity. A mapped
`$owned_struct_carry:index:seed` must match that slot's initial input, with no
duplicate slot mappings. Unknown markers, inconsistent counts, truncated prefixes,
out-of-range indices and seed drift reject. Legacy complete-map payloads still parse;
older consumers reject the explicit prefix rather than silently executing it.

Dependency and GLM reads include even the seeds not passed to the helper, never
the prefix/count metadata. Registered execution and ordinary/native LLVM share
the parsed state layout. All slots remain initialized, stored after each return
and reconstructed at exit; only mapped slots are loaded as call arguments. Native
state-width, helper-arity and execution-budget bounds remain independent and unchanged.
Break seeds/results are validated even when the control slot is not an operand.

Source admission currently permits partially consumed flat-i64 records, requiring
at least one mapped field per record and exact initial nominal provenance unless the
compiler carries the generated-helper elision proof described below. Whole-record,
scalar, bool and break inputs retain their existing admission. All initializers still execute, including checked
work in unpassed fields on zero-trip paths. The explicit lifecycle fixture declares
four iteration parameters for five state slots, rather than passing an unused field;
the ordinary source retains two slots while passing only one carried field.

### Generated Scoped Record Inputs

The [capture projection](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs)
now applies partial or entirely unread record-input elision to compiler-generated scoped
helpers. Each scoped caller must prove an exact flat-i64 record reconstruction and
complete original seed map through the same NIR projection rules used by lowering.
Constraints are unioned across callers; an unproven written input or an unrewritable
caller vetoes that projection. Source-declared scoped helper signatures are not rewritten by this pass.

Before demand analysis, eligible record parameters receive distinct snapshot versions
for their local updates. Initializers read the preceding value; immutable aliases can
then expose initial-field demand independently of the returned, fully updated record.
Only parameter records are versioned in scoped helpers: local control names, scalar
induction, bool conversions and named break seeds retain their identities. Unresolved
fallthrough joins and nested-loop writes remain whole. Entirely unread records may
leave the signature only with an internal proof of their nominal type, complete
width and starting result slot. Every scoped caller must agree on that proof.
It is published only when the signature/body/caller rewrite commits, then passed
from outlining to lowering. Source names cannot grant it; no wire-format change
or unchecked missing scalar/bool/break seed is admitted.

Caller initializers, complete seed storage, result layouts and record reconstruction
are unchanged. No seed expression or skipped-path computation is synthesized at a
call site. An omitted field can still be observed after a zero-trip loop, and checked
work in initializers or overwritten constructors must still execute when reached.
This does not implement sparse state storage, mixed/nested carries or a public ABI change.

[Projection regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped_carries_tests.rs)
exercise reverse function/YIR order, idempotence, aliases, cross-caller vetoes, empty
demand, ambiguous seed maps, whole uses, break/continue and bool identities. The shared
[source fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_projected_record_carries.ns)
retains three state slots with one carried operand and three total iteration arguments;
generated 7/64-field variants retain the same arity. Native lifecycle probes inspect
five complete state slots and five arguments instead of six, while still returning
all state fields and retaining ordinary/native execution's separate allocation behavior.

[Unread-record regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped_elision_tests.rs)
add whole-input elision, disjoint record slot ranges, caller disagreement, generated-name
spoofing and vetoed-proof checks. The [source fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_unread_record_carries.ns)
keeps three complete seeds and just two iteration arguments, with no record operand;
7/64-field variants keep that arity. The lifecycle variant keeps five state slots
with four arguments and maps only the separate scalar carry. Zero-trip initial values,
initializer/second-trip failures, bool words and break/continue remain covered.
Scoped whole-record inputs now have the bounded seed/carry mapping described below;
neither path relaxes native arity bounds or public/source/FFI signatures.

## Shared Pure-Value Codec

The lowering-private [record codec](../../crates/yir-lower-llvm/src/native_session/value_transport.rs)
now separates bounded nominal layout validation, preparation of an ordered scalar
snapshot, and LLVM packing/unpacking. Native helper and callback returns both use
it. Layouts retain the existing total bound of 1..64 scalar leaves across the whole
nested tree; resources, pointers, empty records and duplicate fields are rejected.
Preparation checks the complete nominal tree and exact scalar kinds before any
instruction or label is emitted. Guarded return packing stays inside the selected
branch, and an invalid value leaves the body and fresh register/block counters intact.
Pure records do not inherit owned-variant prefix conversion or synthetic zero fields.
A regression reproduces the previous erroneous acceptance of a variant as a differently
named record, at both the root and a nested field; the codec now rejects that conversion
while preserving exact records whose names happen to carry the internal-looking prefix.

The same codec can build and decode an LLVM `[N x i64]` argument without an owned
runtime pointer. [Direct LLVM probes](../../crates/yir-lower-llvm/src/native_session/value_transport/tests/native.rs)
exercise two independent record arguments plus a predicate, then verify every returned
word. Seven flat/nested shapes cover 1/2/5/7/64 leaves, eight input vectors and both
selected values at `-O0` and `-O2`. The bit oracle includes signed integer extrema,
boolean values, float negative zero, infinities, subnormals and quiet/signaling NaNs.
The emitted probe IR contains no loads, stores, pointers or runtime allocations;
this does not promise that a machine ABI never uses stack storage.

This codec is not signature authority. Source-declared functions still
flatten whole-record inputs; only proven compiler-generated capture plans opt
into the bounded parameter contract below. The direct LLVM probe is not evidence
for a new source ABI, external FFI contract, resource transport, general record-input
admission or a performance improvement.
Callback word canonicality and graph/ownership admission remain separate checks.

The host compiler probe is explicitly opt-in so portable emitter tests do not
require clang. It can use `CLANG` to select an installed host driver:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p yir-lower-llvm --lib -- typed_record_arguments_round_trip_through_host_llvm --ignored --test-threads=1
```

## Generated Record Inputs

The [capture planner](../../tools/nuisc/src/lowering/direct_calls/capture_params.rs)
first retains field projection and boolean compaction. Only when the resulting
signature would still exceed 64 parameters does it replace complete, exact,
resource-free record groups with private record-value parameters. Each record
retains the shared 1..64-leaf bound, nominal tree and exact scalar kinds. Source
function names cannot opt in; source-declared helpers and FFI stay unchanged.
Scoped targets require the separate all-caller seed proof described below.

`cpu.param_value_struct <index> <layout>` declares a bounded synchronous value
parameter, not a pointer or an owned handle. The CPU nustar registers its parameter
contract and runtime argument validator. The common verifier checks index, nominal
type, value ownership and unique function-table binding through registration hooks,
without a CPU opcode switch. The reference executor validates evaluated arguments
before changing a call frame; an unbound parameter cannot synthesize a default value.
The [CPU validator](../../crates/yir-domain-cpu/src/value_parameters.rs) rejects
missing/extra/duplicate fields, nominal or scalar-kind drift and owned-variant
conversion. The host registry wrapper forwards both hooks.

LLVM uses the shared checked snapshot codec for `[N x i64]` argument packing and
typed callee unpacking. Each record argument is validated before packing; no owned
aggregate allocation is introduced. Task thunks reject records; scoped calls require
explicit record maps rather than assuming that a scalar argument map still applies.
Native callback wrappers keep their original scalar word-buffer ABI and 64-slot
bound; helper-entry charging and selected-branch publication remain unchanged.

[Source/native probes](../../tools/nuisc/tests/native_application_bridge/typed_record_inputs.rs)
exercise the former 65-leaf capture failure as flat and nested 64-field records,
mixed nested snapshots, reverse constructors/YIR ordering, overlapping callback
input/output and raw negative-zero/NaN bits. A declared helper with a generated-style
name and 65 scalar leaves still rejects. [Guard probes](../../tools/nuisc/tests/native_application_bridge/typed_record_guards.rs)
cover nominal/kind drift, selected division traps, shared entry limits and unchanged
output sentinels. The [CLI workflow](../../tools/nuis/tests/native_session_workflow/record_inputs.rs)
checks build, cache reuse, input/artifact tamper rejection and source-free restoration
with identical LLVM and lifecycle states. These are bounded CPU-host proofs, not
new GPU/provider, resource-state or unrestricted whole-record transport claims.

## Scoped Record Inputs

Generated scoped helpers may compress a whole record when every admitted scoped
caller proves a complete seed map or an unwritten ready-input root. The planner
uses the compiler's generated-function set, never a name prefix. Existing scalar
projection, unread-input elision and separate full seed storage remain unchanged.
Independent boolean carries are not bit-packed by this path. Unproven inputs keep
their previous signature; source-declared helpers and callback/FFI ABI do not change.

The shared [loop contract](../../crates/yir-core/src/loop_carry_contract/scoped_record.rs)
encodes a single physical argument as
`$value_record:<layout>|<leaf0>|...`. Named captures retain exact bool/i32/i64/f32/f64
kinds; `$current` and `$owned_struct_carry:<slot>:<seed>` require i64 fields.
Only multi-carry scoped actions admit carry mappings. Plain/single-carry actions
admit named scalar captures and i64 induction inside records; `$carry` stays a
separate scalar argument. Parsing retains the 1..64-leaf bound and
rejects resources, non-i64 loop-state mappings, malformed operands and nested descriptors.
The enclosing carry contract validates all expanded mappings together, including
duplicate slots across record/scalar arguments, out-of-range indices and explicit
seed mismatches. Dependency/GLM reads are the actual leaves, not metadata strings.

The registered CPU executor reconstructs each record from current iteration state.
LLVM validates the descriptor against the callee's exact nominal layout and prepares
all record arguments before packing any of them through the shared value codec.
Carried fields load from current carry slots, not captured initial values. Zero trips
still return all initial state; break/control slots keep independent positions.

[Native/reference regressions](../../tools/nuisc/tests/native_application_bridge/typed_scoped_record_inputs.rs)
cover a 64-field record plus induction, bounded break loops, reverse source/YIR order,
exact callback state and overlapping input/output buffers. Descriptor/parameter drift,
selected arithmetic traps and work limits fail without publishing callback state.
The [CLI workflow](../../tools/nuis/tests/native_session_workflow/scoped_record_inputs.rs)
checks cache reuse, tamper rejection, source removal, standalone verification and
materialization with identical LLVM and lifecycle results.

Read-only transport grants no backedge or elision authority. Unmapped single/no-carry
actions, resources and task transport remain separate work. These are CPU-host proofs, not
fresh Linux/GPU evidence. The branch-helper input boundary recorded at the scoped
checkpoint is addressed by the following generated-branch slice.

## Generated Branch Inputs

The outliner now offers all of its actual generated helpers to the same private
capture planner, including branches created while outlining loops. Source names
are never generation authority. Pure-layout admission, field projection, boolean
compaction and the all-caller seed proof for scoped targets remain independent
requirements; references, optional values, resources and oversized records do not
gain admission from being passed to this planner.

The former 65-argument loop branch uses two parameters: its predicate and the
complete 64-field record. This changes neither the 64-parameter bound nor the
source/public/FFI or callback ABI. Branch calls still pass existing captured values;
calls and constructor arithmetic belonging to an arm stay behind that arm's guard.
Predicates are snapshotted before a selected arm updates a record. No new LLVM or
runtime transport is needed, and scoped induction/carry/seed maps stay separate.

The [branch regressions](../../tools/nuisc/tests/native_application_bridge/typed_scoped_record_inputs.rs)
check source/native/reference values, reversed YIR order, in-place callback output
and zero owned aggregate allocation/drop counters. Unselected helper division and
unselected record-field division remain lazy; selected failures and both work
budgets preserve all output sentinels. Planner tests reject source-name authority
and unsupported parameter layouts. The existing scoped-record CLI workflow also
exercises wide branches through two events, cache reuse, tamper rejection and
source-free restoration with identical LLVM and lifecycle states.

The first mixed-scalar carry slice follows below. Native bounds remain unchanged;
nested/resource carries and provider state remain separate admission work.

## Bool/I64 Record Carries

Generated scoped loops now carry flat records containing `bool` and `i64` fields.
The compiler creates private i64-word records for mixed input seeds and result
backedges. Each boolean is explicitly encoded as 0/1 and decoded inside the same
iteration helper, with no additional helper-entry debit. Pure i64 records retain
their existing projection/transport route. Layout planning is refreshed after
outlining so newly generated word-record definitions participate in bounded capture
planning; generated names are not authorization.

The [seed proof](../../tools/nuisc/src/lowering/scoped_loop_lowering/mixed_record_words.rs)
requires exact nominal input layout, declaration order, word width, source binding,
field kind and conversion. Missing, duplicate, reordered, unconverted, computed or
wrong-origin inputs fail closed. Initial record identity is checked before field
flattening, including on zero trips. Output reconstruction creates a new nominal
record and explicitly decodes boolean fields, preserving earlier snapshots.
Guard defaults admit only existing typed parameter fields and explicit boolean
word conversion. Calls, arithmetic, raw boolean words and resource/nested fields
remain forbidden there, including during counted-return normalization.
The existing YIR i64 carry contract, CPU executor, LLVM loop implementation,
source/public/FFI signatures and callback ABI are unchanged.

[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/mixed_scoped_record_carries.rs)
exercise wide mixed records, multiple flags/records, counted returns, guarded breaks, nested iterations,
immutable inputs, zero trips, reversed YIR order, overlapping outputs and zero
owned aggregate allocation/drop counters. Selected arithmetic and work-limit
failures leave output sentinels intact. Ordinary native entry and source-free CLI
restoration have separate regressions. Record seed projection remains conservative:
this slice transports the complete mixed record, not sparse mixed backedge inputs.

The subsequent signed-i32 slice follows below. f32/f64 require bit-preserving
transport, not numeric conversion; nested and resource records remain separate.
Existing non-loop mixed/nested values are not narrowed.

## Signed I32 Record Carries

Flat `bool/i32/i64` records and independent `i32` locals now use the same private
word transport. Each `i32` seed and backedge is sign-extended to i64, then explicitly
truncated to i32 inside the existing iteration helper and after the loop. Exact
nominal layouts, field order, scalar kinds, source identity and decode indices
remain mandatory. No extra helper-entry boundary or wider native slot limit is
introduced; sparse mixed record projection remains separate.

The native scalar bridge now admits the existing registered `cast_i32_to_i64` and
`cast_i64_to_i32` instructions. Shared YIR loop/runtime contracts and public/source,
FFI and callback signatures are unchanged. Guard defaults permit total typed i32
parameter reads followed by explicit signed encoding, not calls or arithmetic.
Wrong-kind casts and raw/misindexed reconstruction maps fail closed.

Execution testing also exposed a source arithmetic width bug: generic integer
operations return i64, so a previously accepted i32 sum could lose its declared
kind before its next use. Source i32 addition, subtraction and multiplication
now lower through explicit operand widening and result narrowing, preserving
wrapping i32 semantics and evaluating each operand once. This does not newly
admit i32 division/remainder into the scoped value profile.
Counted-return normalization also walks cast operands when reserving source names,
without turning hidden free references into generated initialized locals. A fallible
cast in an unselected return suffix remains lazy, rather than rejecting the loop.

[Signed record probes](../../tools/nuisc/tests/native_application_bridge/i32_scoped_record_carries.rs)
cover signed limits, out-of-range truncation, zero trips, snapshots, independent
scalar carries, nested iterations, early returns and 64-field mixed records.
The [ordinary native fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_i32_record_carries.ns)
checks break timing, signed comparisons, add/subtract/multiply wrapping and lazy
traps. CLI build/cache/tamper/source-free restoration has a separate signed-record
case, without changing the existing callback ABI or allocation-free transport.

The checkpoint's next step was to extend generated scoped record word transport to f32 leaves with bit-preserving seeds and backedges.
The following f32 and f64 slices implement that bit transport. Nested and resource-bearing
record carries remain separate, and the session coordinate stays `active/86`.

## Bit-Preserving F32 Carries

Flat `bool/i32/i64/f32` records and independent f32 locals now retain their exact
seed/backedge bits. Private NIR `PackF32Word` / `UnpackF32Word` expressions lower
to registered `cpu.pack_f32_word` / `cpu.unpack_f32_word` operations. These are not
source builtins, public ABI changes or numerical `cast_f32_to_i64` conversions.
Packing requires f32 and zero-extends its IEEE-754 u32 bits into i64. Unpacking
requires exactly i64 and reconstructs f32 from the low 32 bits. In particular,
bool, i32 and f64 operands receive no implicit conversion. External callback
words still require canonical zero upper bits under the existing scalar ABI.

The reference CPU executor uses `to_bits` / `from_bits`; LLVM uses `bitcast` with
`zext` / `trunc`, never `fptosi` / `sitofp`. Negative zero, NaN sign/payload,
infinities and subnormal bits survive transport. Arithmetic itself keeps normal
f32 semantics; this is not a guarantee of NaN payload stability across arithmetic.
Source value-loop admission adds f32 add/subtract/multiply, not float division,
remainder or comparisons. Existing non-loop float support is unchanged.

The exact source-field/slot/nominal proof now requires these explicit pack/unpack
forms for f32, rejecting numeric casts, missing encodes and misindexed decodes.
Only total typed parameter reads may be packed as guard defaults. Each iteration
decodes in the existing helper, so the loop metadata schema, work accounting,
64-slot bounds, immutable snapshots and source/public/FFI signatures stay unchanged.

[F32 native/reference probes](../../tools/nuisc/tests/native_application_bridge/f32_scoped_record_carries.rs)
cover zero trips, raw bit patterns, scalar companions, nested loops, early returns,
finite arithmetic, wide private record inputs and atomic selected failures.
The [ordinary native fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_f32_record_carries.ns)
also checks breaks and lazy traps. CLI build/cache/tamper/source-free restoration
has a dedicated f32 record case. This remains complete flat-record word transport,
not sparse mixed projection or nested/resource-bearing carry support.

That checkpoint selected: extend generated scoped record word transport to f64 leaves with bit-preserving seeds and backedges.

## Full-Width F64 Carries

Flat `bool/i32/i64/f32/f64` records and independent f64 locals now use the same
scoped word-map proof. Private NIR `PackF64Word` / `UnpackF64Word` expressions
lower to registered `cpu.pack_f64_word` / `cpu.unpack_f64_word` operations.
Packing requires exactly f64; unpacking requires exactly i64. All 64 bits are
transported, including the sign bit: a negative i64 word is not an invalid input.
No truncation, extension or numerical float/integer conversion is involved.
These internal operations are not source builtins or a new public ABI.

The reference executor uses `to_bits` / `from_bits`; LLVM uses `bitcast double`
and `bitcast i64`. Float-derived integer facts are not inferred. Negative zero,
quiet/signaling NaNs, infinities, subnormals and adjacent finite values are checked
as words, not by float equality. This is transport fidelity, not a promise to retain
NaN payloads across arithmetic. The bounded source value-loop profile adds f64
add/subtract/multiply, not float division, remainder or comparisons.

Seed/reconstruction checks require exact nominal fields, source bindings, slots
and f64 codecs. F32 codecs, numeric casts and missing encodes do not substitute for
f64 maps. Guard defaults still permit only total typed parameter projections.
The existing helper decodes each trip, without extra helper entries or changes to
loop metadata, work budgets, 64-slot limits or source/public/FFI/callback signatures.

[F64 native/reference probes](../../tools/nuisc/tests/native_application_bridge/f64_scoped_record_carries.rs)
cover full-width raw bits, all five scalar kinds in one record, independent f32/f64
locals, zero trips, snapshots, nested iterations, counted returns, finite arithmetic,
64-field private inputs and atomic selected failures. The
[ordinary native fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_f64_record_carries.ns)
checks break timing and selected versus skipped traps. A dedicated CLI workflow
covers build/cache identity, tamper rejection and source-free artifact restoration.
Loop-bound admission is unchanged: these carry probes bind a field-derived bound
to a local before the loop, rather than claiming arbitrary field-valued conditions.
At that checkpoint, sparse mixed projections, nested record carries and resources
remained separate admission work; the nested carry extension follows below.

That checkpoint selected: extend generated scoped record word transport to nested pure-scalar records with exact field-path seed and backedge maps.

## Nested Scoped Carries

Generated scoped carries now admit acyclic, nonempty nominal record trees with
`bool/i32/i64/f32/f64` leaves. The shared pure-shape proof rejects references,
optionals, generic arguments/definitions, duplicate fields, unknown/resource types
and cycles transitively. The entire tree must be pure, including unaccessed siblings.
Record depth is bounded at 64 and nested expansion at 4096 nodes. Explicit flat
record width retains its existing admission policy; native value limits remain 64 leaves.
Value eligibility and whole-function control authority remain separate: a new
nested type does not reroute existing non-loop helpers or inflate their entry budgets.
New control roots require an actual loop; their proven dependencies follow reachability.

Private word maps use ordered field segments, not concatenated path strings. Leaf
order follows the declared record tree, so `left.value` and `right.value` cannot
alias. Each seed must match the exact source binding, complete path, scalar codec
and word slot. Reconstruction checks each child's nominal type, field order/count
and exact decode before creating a fresh nested value tree. Wrong parent paths,
same-width child types, duplicate/missing fields and computed seeds cannot substitute.
Nested all-i64 records also require complete maps before sparse projection; projected
inputs use exact leaf-slot proofs rather than the flat-i64 shortcut. The existing YIR scoped action still transports flat i64
words, so no backend-specific loop opcode or wire-format revision is introduced.

Guard defaults permit only total typed parameter paths through a proven pure tree.
Calls, arithmetic and resource-bearing siblings remain rejected. Original values,
zero-trip seeds and per-trip snapshots stay independent; mixed leaves keep the
existing bool, signed-i32 and bit-preserving float codecs. Selected failures and
both work budgets retain the existing atomic callback publication contract.

[Nested native/reference probes](../../tools/nuisc/tests/native_application_bridge/nested_scoped_record_carries.rs)
cover all five scalar kinds, equal leaf names, raw float bits, three-level records,
nested loops, early returns, snapshots, 64-leaf private inputs and failure sentinels.
The [ordinary native fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_nested_record_carries.ns)
checks break timing and skipped versus selected traps. The nested CLI case retains
cache reuse, tamper rejection and source-free restoration with identical LLVM and
open/event/close states. These are host CPU transport tests, not fresh GPU/Linux or
performance evidence. Public/source/FFI/callback ABI and graph limits are unchanged.

That checkpoint selected sparse mixed/nested input projection, implemented below.

## Sparse Typed Inputs

Generated scoped iterations may now project a subset of mixed/nested record words,
or omit an entirely unread record input, while retaining the complete initial state.
The compiler first proves the complete source-to-word map and typed nominal backedge.
It removes only exact input reconstructions and proven ready record aliases, or
reconstructs unused input leaves with neutral values after separate field-demand proof;
computed constructors and their unaccessed checked fields still execute in source order.
Each projected operand uses the exact source binding, ordered field path and scalar
codec to identify its current backedge slot. This is not an invariant read of the
first iteration's snapshot. Scalar controls and induction retain independent maps.

Normalization is transactional across all callers. Every encoded argument must be
the full canonical map of one ready source record, including fields later omitted.
Raw word variables, calls/arithmetic in word constructors, changed codecs or paths,
duplicate slots and malformed reconstruction cannot grant sparse-input authority.
Unproven whole-input demand and exhausted loop-demand proofs conservatively keep
the complete input rather than publishing a partial fixed point. No new loop protocol,
wire representation, public/source/FFI/callback signature or backend rule is introduced.

[Compiler regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_words_tests.rs)
cover all-caller rollback, storage-order independence, flat/mixed/nested shapes,
subrecord aliases, entirely unread inputs and initializer/per-trip failures.
[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/sparse_typed_record_carries.rs)
keep 9/64 seed words with six iteration arguments, preserving signed i32, float bits,
zero-trip state and repeated boolean/float backedges without aggregate allocation.
An entirely unread nine-word carry retains all zero-trip state with two control inputs.
The [ordinary native fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_sparse_typed_record_carries.ns)
and [source-free CLI workflow](../../tools/nuis/tests/native_session_workflow/sparse_typed_records.rs)
exercise real execution, cache reuse, tamper rejection and identical restored LLVM.

### Child Snapshot Proof

Each child scope inherits only preceding visible bindings. New record/subrecord
aliases remain child-local; their names neither constrain sibling branches nor leak
into the parent suffix. Rebinding an inherited alias is removable only when its
complete decoded nominal leaf tree stays identical. This proves one immutable
version across fallthrough joins and loop backedges, including zero iterations.
It does not infer a mutable join or replace a changing value with its initial seed.

The alias-removal path rejects assignments to already materialized outer records or
parameters, different snapshot origins, whole escapes and incompatible nominal types.
The separate input-demand proof below may retain those assignments and project only
entry words; incompatible types and unproven whole-input demand remain conservative.
Computed constructors and unaccessed checked fields remain in the selected child.
The [scope regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_scopes_tests.rs)
cover identity writes, sibling names, invariant loop aliases, retained changed-origin assignments,
outer materialization, storage-order independence and transactional fallback.
The [branch fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_branch_typed_record_carries.ns)
alternates f32/f64 updates against current backedges. Native/reference checks preserve
complete 9/64-word state with six iteration arguments. Ordinary native execution
retains lazy/selected child traps; the nine-word source-free CLI probe preserves
repeated lifecycle events without adding backend-specific handling.

### Materialized Join Proof

The [input-demand analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_demand.rs)
uses a hygienic copy of the function; it never renames the actual scoped control
identities. Backward leaf demand follows ready record/subrecord copies, kills old
versions at assignments, unions both branch entry demands and treats each return
as an exit. One-sided assignments retain fallthrough input; overwrites on both arms
can remove demand for the preceding value without deleting either assignment.

Computed constructors and calls observe every operand, including unused checked
fields. A later overwrite cannot hide an earlier division, call or selected trap.
Only the exact canonical input reconstruction may replace unobserved leaves with
typed neutral values. All following statements, stores and joins remain unchanged;
complete carry seeds and current-trip field-slot maps still preserve zero-trip
state, stored snapshots and independent lifecycle events. Every caller must pass
the full canonical source/path/codec proof before any candidate change commits.

The analysis falls back without changing the candidate at depth 64 or after its
65,536-unit work budget. These are optimization bounds, not source restrictions.
Changing nested loops use bounded fixed-point and lexical exit-edge proof; the
existing immutable-identity alias path remains available for invariant loop copies.
Opaque whole-record reads keep all demanded leaves rather than assuming an escape
can be erased. Public/source/FFI/callback ABI and native graph limits are unchanged.

[Join regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_joins_tests.rs)
check one-sided versus two-sided overwrites, early returns, differently typed
sibling names, all-caller rollback and unused checked-field dependencies. The
[materialized-join fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_join_typed_record_carries.ns)
retains outer record assignments while alternating f32/f64 updates. Native/reference
probes keep complete 9/64-word state with six iteration arguments; the nine-word
CLI workflow retains cache/tamper checks and source-free restoration. This is host
CPU evidence, not fresh GPU/Linux execution, formal safety or measured speedup.

### Nested Loop Demand

Each loop header starts with the following suffix's demand and the condition's
reads, preserving the zero-trip/normal-exit path. Backward body analysis repeatedly
unions additional demand into that header until stable. This follows cyclic
record/subrecord copies across successive iterations, not just one body execution.
Break uses the nearest loop exit; continue uses that loop's header. Nested loops
have independent targets, while a function return observes its own result and
does not inherit unreachable suffix reads or writes.

All nested analyses share the existing depth/work budget. Exhaustion or a control
exit without a matching loop abandons input reconstruction, never granting partial
projection authority. Original assignments, typed codecs, complete seeds, control
identities and every caller's canonical map remain required. Loop conditions and
unused checked constructor fields contribute their operands even when the final
value is not selected. Opaque whole-input reads remain conservative.

[Loop-demand regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_loop_demand_tests.rs)
cover cyclic snapshots, nearest exit targets, zero trips, header reads, checked
unused fields, malformed callers and unchanged assignment tails. The before/after
reference execution probe uses an i64 marker because private bool-word casts must
not re-enter source-level admission; the actual native source fixture covers bool.
Direct NIR analysis does not widen source loop-shape admission or native scheduling.
The [nested-loop source fixture](../../tools/nuisc/tests/control_flow_syntax_native/scoped_loop_typed_record_carries.ns)
executes mixed typed updates with empty inner/outer loops and local break/continue
variants. Native/reference 9/64-word states retain six outer iteration arguments
and zero aggregate allocation/drop counters. Ordinary native tests retain selected
inner traps and skipped work; the CLI case checks repeated events, cache/tamper
handling and identical source-free restored LLVM/state.

### Nested Return And Budget Proof

The [nested-return source](../../tools/nuisc/tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns)
updates mixed leaf snapshots inside two counted loops, then returns the current
inner value before the outer update and a dead division suffix. Compiler proof
retains each lexical exit and the selected return payload. Native/reference
execution compares every state word for zero through four outer trips using
9/30/31/57/58/59/60/61/62/63/64-word states. Exact initialized mutable record storage may be reused
for a selected return, unobserved leading-index recovery is omitted, and proven
return-only exits share the pending/break signal. Invariant-field promotion reduces the inner backedge to N-1 words and the outer backedge to N. These are private representation
changes, not wider native limits or weaker public state validation.

Source admission first validates the independent payload rewrite. Only then can
[storage selection](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_storage.rs)
choose an exact nominal record from the straight-line function-entry bindings
which already participates in loop writes. Parameters, constants, scalars,
branch-local/late bindings and merely layout-compatible records are not candidates.
The complete selected expression is evaluated before overwriting that record or
setting the pending bit. Propagation evacuates all enclosing loops before any
normal suffix can observe the overwritten value; non-return paths keep the source
state and original initializer effects. No resource/destructor storage is admitted.

The coalesced body must pass the same control validator again. A newly introduced
inner write which invalidates source-ordered reads vetoes the optimization and
retains independent storage. Original invalid forward reads never gain assignment
self-read authority. [Storage regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_storage_tests.rs)
cover hygiene, nominal/mutability boundaries, rollback and differential execution
against independent payload storage, including leading/trailing steps and traps.

[Continuation observation](../../tools/nuisc/src/lowering/buffer_loop_outline/continuation_reads.rs)
conservatively includes all suffix reads, enclosing continuations and every
generated helper output. Only a leading index absent from this set can omit its
post-break recovery slot. The source step still executes; the canonical break bit,
complete loop reservation and helper-entry debits remain unchanged. Unknown syntax,
depth 64 or exhausted 65,536-unit scan/materialization work retains recovery.
Reads after redefinition and untaken-arm reads are deliberately not killed.
[Exit regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/index_recovery_tests.rs)
cover direct/outer suffixes, parent backedges and branch-helper outputs. Scalar
branch input collection also accepts already-lowered loop exit markers without
panicking or omitting expression-call inputs; source admission remains separate.
Whole-record break probes keep the observed-index 62-word record and 61-word
record-plus-bool cases. Unobserved-index variants now fit 63 and 62 words,
respectively; both still require actual scoped record transport and exact seeds.

[Return-signal proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_signal.rs)
uses an opaque identity minted by this pass's return rewrite, not a source-name
convention. Every break owned by the loop must immediately follow canonical pending
publication or be the exact child-return propagation guard. Child-loop breaks have
their own owner; any ordinary same-loop break vetoes sharing. Other reads/writes of
the signal, unknown syntax, depth 64 or exhausted 65,536-unit work retain separate
signals. Payload evaluation still completes before publication, mixed continues
keep their own running bit, and the single pending/break slot remains last and
zero-seeded. No source or shared YIR annotation grants this optimization authority.
[Differential regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_signal_tests.rs)
compare 300 inputs against both independently normalized control signals and
source-result oracles, with reversed YIR node order. Six shapes cover direct
returns, same-loop/child/parent breaks, continue and source-name spoofing, with
leading/trailing steps and enclosing branches. Ordinary native execution checks
36 corresponding cases. Pre-normalized input without this pass's provenance keeps
separate signals. The 57-word fixture takes flat input transport; 58 through 64-word
fixtures require actual compact-record transport, with an exact N-4-word private
parameter checked in LLVM rather than weakening the whole-record assertion.

[Invariant-field proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_facts.rs)
tracks exact entry binding/field identities through copies, nested constructors and
both branch arms. Every write must preserve a field, not just the final backedge;
temporary mutations, cross-root swaps, arithmetic identities and opaque calls do
not grant invariance. Unknown syntax, depth 64 or exhausted 65,536-unit analysis
and rewrite work retain the prior representation.

[Nested-loop proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_loops.rs)
joins zero trips and every intermediate write, including writes inside child loops.
Each entry origin can only become unknown. Repeated transfer reaches a bounded
fixed point without simulating a trip count. This retains transient snapshots that
an ordinary break or continue can expose before a restoring suffix, and invalidates
aliases that become different only on later iterations. Child-local bindings never
escape into the summary. Exhaustion discards the attempted rewrite transactionally.
At most two 65,536-unit attempts are made: nested promotion first, then the prior
innermost-only mode if needed. Rejection of an outer candidate must not erase an
admitted inner-only representation. Both attempts start from the same original
plan and install body and private definitions only after whole-function validation.

[Promotion](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariants.rs)
applies to counted loops in admitted return-containing functions, including ordinary child exits.
Per-write invariance is independent of return-signal sharing; the bounded fallback
retains its earlier innermost, return-owned restriction.
Invariant leaves stay captured outside the loop; changing leaves share a compact
private typed record. Reads and the loop exit reconstruct the original nominal
record. Each original assignment RHS still executes once at its original position,
before any compact record update. Source admission is unchanged, and the complete
candidate body plus private definitions must pass transactional revalidation.
Later private capture projection may eliminate an unread reconstruction only when
every field is a total copy from an unwritten pure-value parameter. Exact nominal
types, unique unread bindings, all-caller checks and a bounded proof are required.
Before this elimination, [record-view projection](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_views.rs)
resolves partial reads of exact total reconstructions to their immutable input fields.
Lexical copies and subrecords share the same proof; whole escapes retain their bindings.
Calls, arithmetic, codecs and resources cannot establish a total reconstruction.
Checked computations using projected fields remain at their source position. Work or
depth exhaustion installs no partial rewrite, and any caller may veto signature changes.
The original assignment site retains an explicit compact-state self-binding before
its RHS snapshot. This preserves the assignment's own-read authority when a child
loop resets ordered-read availability; it does not authorize a later sibling read,
parameter write or immutable induction/header mutation. The extra rewrite work is
charged to the same bounded budget.
No source/public/FFI type, callback ABI or shared YIR protocol is extended.

Scoped capture planning chooses complete proven record seed maps directly, rather
than first selecting a boolean-packed ordinary-call plan that scoped calls cannot
consume. Records without a seed proof remain flat; independent boolean slots are
not packed. This avoids trading fewer carried words for an over-bound helper
signature. The [proof and differential tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariants_tests.rs)
also retain zero trips, leading/trailing steps, hygiene and selected traps.
The [nested differential matrix](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_nested_tests.rs)
compares independent payload normalization and arithmetic oracles, including
observed outer indices, child breaks/continues and selected traps. Ordinary child
breaks do not gain return-signal authority; a candidate that invalidates ordered
reads falls back to an admitted inner-only plan, or the original body with no new
private definitions. The ordinary child-exit counterexample now admits the outer
plan; a [bounded exhaustion regression](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_admission_tests.rs)
separately verifies exact inner-plan retention and rejects invalid source writes.

Capture normalization preserves exact compiler-registered break identities for
the current helper's scoped callees, including a non-scoped branch helper's child
loops. Neither a prefix nor an unrelated function's same-spelled identity grants authority.
Nested loop seed proof also accepts existing compact value parameters only after
bounded scalar layout parsing and nominal type checks; no new YIR operation is added.
[Child-exit native probes](../../tools/nuisc/tests/native_application_bridge/sparse_typed_return_child_exits.rs)
exercise zero/one/two trips, break/continue and observed exit indices at 9/63/64 words.
The ordinary child break needs one word for control, or two with observed-index
recovery; continue needs no child record backedge. The outer N/inner N-1 return
carry widths remain unchanged. Opaque calls do not establish invariance. Partly
observed total reconstructions now execute at width 64 without deleting checked
field uses. Opaque snapshots in multi-carry breaks and no-carry/single-carry continues
now use read-only record transport without eliding calls. Checked constructors fit after
caching nominal layouts and field ranges within the unchanged proof budget. Value origins
remain fresh at every write; changing record backedges still retain native carry bounds.
The original selected child trap reserves five loop trips and enters eleven helpers before
failure, leaving aliased/disjoint state and border sentinels untouched with no
aggregate allocation/drop. These reservations include unexecuted planned trips.

[Budget and return probes](../../tools/nuisc/tests/native_application_bridge/sparse_typed_return_carries.rs)
check 9/63/64-word exact six-loop-reservation/24-helper-entry success, one-short failure,
zero budgets and fresh callback counters. Four requested outer trips still reserve
seven total trips, although the second outer iteration returns. Early exits do not
refund planned work. The ordinary 64-word nested-loop fixture separately retains
exact six-reservation/13-entry success and one-short failures without early returns.

The [native probe](../../tools/nuisc/tests/native_application_bridge/sparse_typed_return_probe.rs)
observes stack-owned budgets and all output words before retaining the real trap.
Disjoint output remains sentinel-filled; fully overlapping input/output remains
bit-identical to the supplied input on failure, with both border sentinels intact.
No aggregate allocation/drop calls are introduced. A selected return-expression
division fails before publishing state, while zero/one-trip paths skip it. The
reference host keeps its last accepted state after the semantic failure; its
execution fuel is not equated with native loop/entry budgets. Malformed f32/i32/bool
leaves reject before even a zero native entry budget, including initially unused
right-hand fields.

Ordinary native execution covers the same return/dead-suffix/selected-payload paths.
The 9/64-word CLI workflows check repeated events, cache reuse, tamper rejection and
identical LLVM/lifecycle state after removing source and restoring the artifact.
At the earlier checkpoint, private iteration arities were 12/12 at width 9 and 5/11 at width 64, where
complete record seed maps avoid exceeding the helper-argument bound.
This is host CPU evidence, not a new callback ABI, interpreter fallback or GPU proof.

Preheader snapshots now reduce the 64-word early-return fixture to outer width 63 and inner width 61.
The pending/break bit still occupies one slot. A changed outer tag now fits 64 outer words
because the saved count is provably unchanged; changing both tag and count
still rejects at 65 outer words, with private return/control state named in the
diagnostic. A public state within 64 words does not imply every lowered control
composition fits. Observed indices, ordinary exits and failed revalidation retain
their required state; no native bound is raised.

Next: reduce nested return snapshot carries while preserving observed exits;
broaden outer invariant admission across ordinary child exits while retaining
source-ordered read validation.

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
Sparse mixed/nested loop inputs, resource state, provider dispatch, native image-host
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
At that checkpoint, fallthrough joins and loop-written snapshots remained separate. No full-workspace,
fresh GPU/Linux, formal safety or measured performance result is claimed.

The 2026-09-24 fallthrough-input follow-up to `4257ebb6` passed 637 selected
lowering tests, 41 selected native-bridge tests, nine sparse-capture CLI workflows,
two ordinary native snapshot tests, 26 tensor tests and the host-path policy case:
716 selected tests without double-counting overlapping reruns. The direct NIR
reduction regression failed before implementation and passes afterward. An initial
field-assignment expansion failed existing lowering admission; input reconstruction
instead preserves original control shapes and complete record initializer work.
All nine CLI workflows retain cache/tamper checks, selected failures and byte-identical
source-free restoration. The fresh CLI reports 1433 clean drift checks, clean
coverage/hierarchy/lineage and `active/86`. At that checkpoint loop-written record
snapshots were next; that checkpoint does not certify fresh GPU/Linux, full-workspace,
ordinary owned-aggregate performance or formal safety.

The 2026-09-24 loop-written-input follow-up passed 643 selected lowering tests,
42 selected native-bridge tests, ten sparse-capture CLI workflows, two ordinary
native snapshot tests, 26 tensor tests and the host-path policy case: 724 selected
tests without double-counting overlapping reruns. Direct input-reduction regressions
failed before the change and pass afterward; source/native fixtures provide separate
composition evidence. Zero trips, per-trip aliases, nested exits and selected
second-trip constructor failures retain their original behavior. All ten CLI
workflows retain cache/tamper checks and byte-identical source-free restoration.
The fresh CLI reports 1437 clean drift checks, clean coverage/hierarchy/lineage and
`active/86`. The next boundary is field projection through scoped loop-carried
record interfaces, not another widening of ordinary entry reconstruction.
This checkpoint does not certify fresh GPU/Linux, full-workspace, formal safety
or measured performance; source admission and native graph limits remain separate.

The 2026-09-25 complete-field-seed follow-up passed 650 selected lowering tests,
49 selected native-bridge tests, six ordinary native tests, twelve distinct CLI
workflows, 26 tensor tests and the host-path policy case: 744 selected tests without
double-counting overlapping reruns. Field-map regressions failed before the change.
A further source regression exposed same-shaped nominal drift in initial records;
the producer now rejects it before flattening, with constructor, alias, helper-return
and nested-field checks. CLI branch inspection was corrected to include record
returns rather than assuming every generated branch returns i64. The field-seed and
legacy whole-record workflows both passed again after origin validation, preserving
cache identity, tamper rejection, selected traps and source-free restoration.
The fresh CLI reports 1443 clean drift checks, clean coverage/hierarchy/lineage and
`active/86`. Automatic carry-field elimination still awaits separate initial seeds
and demanded iteration operands. This macOS aarch64 checkpoint does not certify
fresh GPU/Linux, full-workspace, formal safety or measured performance.

The 2026-09-25 independent-initial-state follow-up passed 90 core-contract tests,
96 registered-CPU tests, 149 LLVM-lowering tests, 654 selected compiler-lowering
tests, 50 selected native-bridge tests, seven ordinary native tests, thirteen CLI
workflows, 26 tensor tests and the host-path policy case: 1086 selected tests without
double-counting overlapping reruns. The new partial-record fixture keeps five state
slots with four helper arguments; the ordinary native fixture returns 96 while
preserving zero-trip state and trapping on selected invalid computations. Unpassed
initializers are still evaluated, including failures before a zero-trip loop.
Legacy complete maps and the new explicit seed prefix retain cache reuse, tamper
rejection and byte-identical source-free restoration. The fresh CLI reports 1452
clean drift checks, clean coverage/hierarchy/lineage and `active/86`. Automatic
partial-record capture projection for compiler-generated scoped helpers is next;
partial declared interfaces do not yet imply automatic signature pruning or reduced
state storage. This macOS aarch64 checkpoint does not certify fresh GPU/Linux,
full-workspace, formal safety or measured performance.

The 2026-09-25 generated-scoped-record-projection follow-up passed 662 selected
compiler-lowering tests, 50 selected native-bridge tests, eight ordinary native
tests, five reference image/window tests, thirteen CLI workflows and 26 tensor
tests: 764 selected tests without double-counting overlapping reruns. The direct
NIR input-reduction regression failed before implementation and passes afterward.
Source/reference regressions retain 3/7/64 state slots with three iteration arguments;
the native lifecycle fixture retains all five slots while reducing six arguments
to five. Ordinary native execution returns 154 and preserves selected iteration,
zero-trip initializer and second-trip constructor failures. All thirteen workflows
retain cache reuse, tamper rejection and byte-identical source-free restoration;
the generated loop workflow also inspects its reduced LLVM signature. The fresh
CLI reports 1459 clean drift checks, clean coverage/hierarchy/lineage and `active/86`.
Entirely unconsumed generated flat-record inputs are next; this checkpoint does not
claim sparse state storage, fresh GPU/Linux execution, a full-workspace result,
formal safety or measured performance.

The unread-record follow-up completed on 2026-09-27 with 671 selected compiler-lowering
tests, 51 native-bridge tests, nine ordinary native tests, five reference image/window
tests, fourteen CLI workflows and 26 tensor tests: 776 distinct selected tests,
excluding overlapping reruns. The new input-elision regressions failed before the
implementation. Complete 3/7/64-slot seeds now coexist with two iteration arguments
and no record operand; two independent records retain disjoint slot ranges.
The native lifecycle fixture keeps five state slots and four arguments. Ordinary
native execution returns 132 and still traps for selected iteration, zero-trip
initializer and second-trip constructor failures. CLI restoration retains the exact
reduced LLVM signature, cache identity, tamper rejection and source-free execution.
Twenty maintenance tests and the host-path policy check also passed; documentation
links and UTF-8 checks are clean. The rebuilt CLI reports 1467 clean drift checks,
clean coverage/hierarchy/lineage and unchanged `active/86`. Whole-record uses in
generated private helpers remain the next bounded-transport boundary. This is macOS
aarch64 evidence, not fresh GPU/Linux, full-workspace, formal-safety or performance certification.

The 2026-09-27 shared-codec follow-up passed 160 LLVM emitter/host-ABI tests,
the explicit host-clang round-trip probe at both optimization levels, five selected
native lifecycle/argument-bound tests, one unread-record CLI build/cache/source-free
restoration workflow and 26 tensor tests: 193 distinct selected tests, excluding
overlapping reruns. The owned-variant prefix regression failed before the pure-value
validator was separated and passes afterward; valid nominal records retain their bits.
The rebuilt CLI reports 1473 clean drift checks, clean coverage/hierarchy/lineage
and unchanged `active/86`. This checkpoint verifies the shared codec and existing
return paths, not generated record-input signatures. The incompressible 65-argument
source fixture still rejects until compiler-private capture and YIR parameter/call
layouts are connected. No fresh Linux/GPU, full-workspace or performance result is claimed.

The 2026-09-27 generated-record-input integration supersedes that last source-admission
boundary for non-scoped generated helpers. All 673 compiler-lowering tests and all
246 native-bridge tests pass on macOS aarch64. The record-input and unread-record
CLI workflows both pass build/cache/tamper checks and source-free restoration with
identical LLVM and lifecycle states. Together with 435 core/registered-CPU/YIR/LLVM
checks (including the explicit host-clang probe) and 26 tensor tests, this is 1382
distinct selected tests, without counting repeated runs twice. Exact schema/argument rejection, independent
mixed snapshots, lazy arithmetic traps, shared-entry failures and untouched output
sentinels remain covered. Public/source/FFI and callback signatures are unchanged;
scoped whole-record transport and its induction/carry/seed maps were the next boundary
at that checkpoint; the subsequent scoped-record section above records the new slice.
The coordinate remains `active/86`. This is not a fresh Linux/GPU, full-workspace,
formal-safety or measured-performance result. The rebuilt CLI reports 1483 clean
drift checks and clean coverage, hierarchy and lineage.

The subsequent 2026-09-27 scoped-input checkpoint passes 420 core/CPU/verifier/LLVM
tests, 674 compiler-lowering tests, 89 distinct selected native/reference regressions
(including corrected-test reruns), two CLI record build/cache/tamper/source-free
restoration workflows and 26 tensor tests. These are 1211 selected tests without
double-counting reruns, not the full workspace or the full native-bridge suite.
The full 64-field record plus induction, 62-field break record and 61-field record
with independent bool/break carries match reference execution. Selected division,
shared-entry and loop-work failures retain every output sentinel, with zero owned
aggregate allocation/drop counters in the probes. Public/source/FFI and callback
signatures remain unchanged. The next boundary is the reproduced wide generated
branch-helper argument overflow, not a request to increase native bounds.
The rebuilt CLI reports 1492 clean drift checks, clean coverage/hierarchy/lineage
and unchanged `active/86`. This is macOS aarch64 evidence only; no fresh Linux/GPU,
full-workspace, formal-safety or measured-performance result is claimed.

The subsequent 2026-09-27 generated-branch checkpoint supersedes that branch-input
boundary. It passed 676 compiler-lowering tests, 93 selected native/reference bridge
tests, six ordinary native entry tests, five reference image/window tests, two CLI
workflows and 26 tensor tests: 808 distinct selected tests, excluding overlapping
reruns. The new branch regression failed before implementation at 65 rather than
two arguments, then passed with real native execution. One-time predicates remain
independent of selected record updates; unselected calls and constructor division
stay lazy, while selected failures and work-budget exhaustion preserve output
sentinels. Ordinary native executables retain zero-trip results and selected traps.
Both scoped-record and branch-record workflows preserve cache/tamper checks and
source-free restoration with identical LLVM and lifecycle states.

The rebuilt CLI reports 1496 clean drift checks with clean coverage, hierarchy and
lineage. The host-path policy, documentation links, UTF-8, changed-file formatting
and line budgets also pass. The coordinate stays `active/86`; mixed-scalar scoped
record seed/backedge transport is next. These are local macOS aarch64 results, not
a full-workspace run, fresh Linux/GPU acceptance or measured performance evidence.

The subsequent 2026-09-27 mixed-record checkpoint passed 679 compiler-lowering
tests, 99 distinct selected native/reference bridge tests, seven ordinary native
entry tests, five reference image/window tests, three CLI workflows and 26 tensor
tests: 819 distinct selected tests, excluding overlapping reruns. The mixed-record
source case failed before implementation. A counted-return regression then exposed
the guard-default check limited to pure-i64 parameter fields; exact typed field
reads and explicit bool-to-word conversion now pass without admitting speculative
arithmetic or calls. Early returns still skip unreachable division, and nested
iterations, independent records, multiple flags, zero-trip seeds and snapshots
retain their source values. Selected failures and work-budget exhaustion leave
output sentinels untouched, with zero owned-aggregate allocation/drop counters in
the native probes.

All three scoped, branch and mixed-record CLI workflows retain cache reuse, tamper
rejection and source-free restoration with identical LLVM and lifecycle states.
The rebuilt CLI reports 1502 clean drift checks and clean coverage, hierarchy and
lineage. The host-path policy check also passes. The coordinate stays `active/86`;
exact signed `i32` seed/backedge transport is next. Floating-point, nested and
resource-bearing scoped record carries remain outside this slice. This is local
macOS aarch64 evidence, not a full-workspace run, fresh Linux/GPU acceptance,
formal-safety certification or measured performance evidence.

The subsequent 2026-09-27 signed-i32 checkpoint passed 1582 frontend/lowering
tests, 106 distinct selected native/reference bridge tests after targeted repair
reruns, eight ordinary native entry tests, five reference image/window tests,
four CLI restoration workflows, 37 LLVM native-session/cast tests and 26 tensor
tests: 1768 distinct selected tests, excluding overlapping reruns and the opt-in
ignored codec probe. The initial source regression reproduced unsupported loop
lowering. Follow-up execution caught loss of i32 arithmetic width and counted-return
cast traversal; both repairs retain dedicated regressions rather than removing
the failing cases. Signed limits, explicit truncation, wrapping arithmetic, wide
records, scalar companions and zero-trip snapshots match the reference executor.
The ordinary native program returns 37 and still traps on selected invalid work.

Scoped, branch, bool/i64 and signed-i32 CLI cases each retain two events, exact typed
state rendering, cache identity, tamper rejection and source-free restoration with
byte-identical LLVM. The coordinate stays `active/86`; bit-preserving f32 seed and
backedge maps are next. This is local macOS aarch64 evidence, not a full-workspace
run, fresh Linux/GPU acceptance, formal-safety or measured-performance certification.

The rebuilt CLI reports 1511 clean drift checks with clean coverage, hierarchy and
lineage. The host-path policy check also passes; changed files remain within their
source/test/document line budgets, with clean formatting, local links and UTF-8.

The subsequent 2026-09-27 f32 checkpoint passed 1586 frontend/lowering/walker
tests, 64 NIR-verifier tests, 114 selected native/reference bridge tests, 23 ordinary
native tests, five CLI record restoration workflows, seven CPU scalar tests,
38 LLVM native-session/cast tests, five reference image/window tests and 26 tensor
tests: 1868 distinct selected tests, excluding overlapping reruns and one opt-in
ignored codec probe. The host-path policy check passed separately. The initial
f32 source case reproduced unsupported loop lowering before implementation.
Raw signed-zero, quiet/signaling NaN, infinity and subnormal words now survive
seeds/backedges and native publication; no float equality oracle substitutes for
bit comparisons. Strict word-kind tests also prevent permissive integer accessors
from accepting bool/i32 inputs differently in reference and native execution.

All five scoped-record CLI shapes pass build/cache/tamper checks and source-free
restoration with byte-identical LLVM and matching lifecycle states. The rebuilt
CLI reports 1521 clean drift checks and clean coverage, hierarchy and lineage;
the coordinate remains `active/86`, with f64 bit-preserving carries next. Changed
files respect source/test/document budgets, formatting, local links and UTF-8.
This is local macOS aarch64 evidence, not a full-workspace run, fresh Linux/GPU
acceptance, formal-safety certification or measured-performance evidence.

The subsequent 2026-09-27 f64 checkpoint passed 1652 frontend/lowering/NIR tests,
123 distinct selected native/reference bridge tests after targeted repair reruns,
24 ordinary native tests, six CLI record restoration workflows, nine CPU scalar
tests, 39 LLVM native-session/cast tests, five reference image/window tests and
26 tensor tests: 1884 distinct selected tests. Overlapping reruns and one opt-in
ignored codec probe are excluded; the host-path policy check passed separately.
The initial f64 source regression failed before implementation. Expanded tests
then exposed a probe assuming generated iteration arguments retained source order;
it now instruments the declared source helper and checks its exact signature.
LLVM checks allow register-valued operands while still requiring full-width
bitcasts and rejecting truncation, extension and numeric conversion. The mixed
five-kind source fixture retains the existing named-local loop-bound restriction.

Full-width signed-zero, quiet/signaling NaN, infinity, subnormal and adjacent-finite
words survive reference/native seeds, backedges and publication. All five scalar
kinds coexist in one record with independent f32/f64 locals. Selected failures
remain atomic, wide private inputs remain bounded and allocation-free, and the
ordinary native fixture returns 37 while retaining selected traps. All six CLI
shapes preserve cache reuse, tamper rejection and source-free restoration with
byte-identical LLVM and matching lifecycle states.

The rebuilt CLI reports 1531 clean drift checks with clean coverage, hierarchy and
lineage. The coordinate stays `active/86`; exact nested pure-scalar field-path seed
and backedge maps are next. Formatting, source/test/document budgets, local links
and UTF-8 pass. This is local macOS aarch64 evidence, not a full-workspace run,
fresh Linux/GPU acceptance, formal-safety certification or measured performance.

The subsequent 2026-09-27 nested-carry checkpoint passed 1660 frontend/lowering/NIR
tests, 129 selected native/reference bridge tests, 25 ordinary native tests, seven
CLI restoration workflows, five reference image/window tests and 26 tensor tests:
1852 distinct selected tests, excluding overlapping reruns. The host-path policy
test passed separately. The initial nested source probe reproduced unsupported
loop lowering before implementation. Expanded regression then caught value-type
admission accidentally rerouting existing non-loop helpers; separating control-root
authority restored the existing guarded selection routes and exact entry budgets.

Nested all-i64 and mixed five-kind trees preserve same-named leaf paths, raw float
bits, zero trips, snapshots, nested loops, early returns and bounded private inputs.
Selected failures and both work budgets preserve 64 output sentinels with zero
aggregate allocations/drops in the tested native profile. The ordinary nested
fixture returns 37 and traps only on the selected invalid path. All seven CLI
shapes retain cache identity, tamper rejection and source-free restoration with
identical LLVM and lifecycle states. The coordinate remains `active/86`; sparse
mixed/nested scoped input projection is next. This is local macOS aarch64 evidence,
not a full-workspace run, fresh Linux/GPU result or performance/safety certification.

The rebuilt CLI reports 1541 clean drift checks with clean coverage, hierarchy and
lineage. Source/test/document line budgets, formatting, local links, UTF-8 and
the host-independent documentation-path policy remain satisfied.

The subsequent 2026-09-27 sparse-input checkpoint passed 1668 frontend/lowering/NIR
tests, 133 distinct selected native/reference bridge tests, 26 ordinary native tests,
eight CLI restoration workflows, five reference image/window tests and 26 tensor
tests: 1866 distinct selected tests, excluding overlapping reruns. The host-path
policy test passed separately. All-caller negative probes reject wrong codecs,
source paths, computed words, malformed maps, whole escapes and unproven child writes
without partially changing the candidate. Flat, nested and entirely unread inputs
retain complete zero-trip state and all checked initializer/per-trip work.

Native 9/64-word state uses six iteration arguments; an unread nine-word group uses
two controls. Raw-bit transport, current-trip float accumulation, boolean backedges,
overlapping callback buffers and zero aggregate allocation/drop probes pass. The new
CLI case preserves two independent events, cache reuse, tamper rejection and identical
LLVM/state after removing all source/build files and restoring the standalone artifact.
The coordinate remains `active/86`; child-scope mixed/nested snapshot joins and loop
versions require separate proof. These are local macOS aarch64 results, not a full
workspace run, fresh Linux/GPU acceptance or a measured performance/safety guarantee.

The rebuilt CLI reports 1550 clean drift checks with clean coverage, hierarchy and
lineage. Formatting, 2934 local document links, UTF-8, host-independent paths and
the source/test/document line budgets pass.

The subsequent 2026-09-27 child-snapshot checkpoint passed 1674 frontend/lowering/NIR
tests, 134 selected native/reference bridge tests, 27 ordinary native tests, nine CLI
restoration workflows, five reference image/window tests, 26 tensor tests and the
host-path policy test: 1876 distinct selected tests, excluding overlapping reruns.
The six scope regressions were rerun after adding the preceding-materialized-binding
negative case. New child aliases and identical inherited alias writes project;
different origins, computed outer writes, whole escapes and malformed types do not.

Alternating f32/f64 branches retain 9/64 seeds with six iteration arguments across
zero through four trips. Selected child traps remain observable; unselected work
and zero-trip bodies remain lazy. The nine-word CLI case preserves two events,
cache reuse, tamper rejection and identical source-free restored LLVM/state.
The live CLI reports 1555 clean drift checks with clean coverage, hierarchy and
lineage. The coordinate remains `active/86`; materialized outer joins and genuinely
changing loop versions still need separate proof. These are local macOS aarch64
results, not a full-workspace run, fresh Linux/GPU proof or measured speedup.

The subsequent 2026-09-27 materialized-join checkpoint passed 1682 selected
frontend/lowering/NIR tests, 136 native/reference bridge tests, 28 ordinary native
tests, ten CLI restoration workflows, five reference image/window tests, 26 tensor
tests and the host-path policy case: 1888 distinct selected tests without counting
overlapping reruns twice. One-sided versus two-sided overwrites preserve the exact
outer assignment tail while demanding six versus five helper arguments. An unused
checked-field dependency adds the required seventh argument rather than disappearing.
The analysis keeps early-return/opaque whole-input demand and rejects a malformed
second caller transactionally; sibling binding hygiene never renames actual control.

Alternating materialized f32/f64 joins keep complete 9/64-word state with six
iteration arguments, current versions, zero-trip storage and zero aggregate
allocation/drop counters. Selected traps and skipped paths retain ordinary native
behavior. All ten CLI workflows preserve cache reuse, tamper rejection and identical
LLVM/lifecycle state after source-free artifact restoration.

The live CLI reports 1562 clean drift checks with clean coverage, hierarchy and
lineage. Formatting, 2939 local document links, UTF-8 and source/test/document line
budgets pass. The coordinate remains `active/86`; changing nested-loop snapshots
need a separate fixed-point and exit-edge proof. This is local macOS aarch64 CPU
evidence, not a full-workspace run, fresh Linux/GPU acceptance, formal safety or
measured speedup. No source/public/FFI/callback ABI or shared loop protocol changed.

The subsequent 2026-09-27 loop-demand checkpoint passed 1692 selected
frontend/lowering/NIR tests, 138 native/reference bridge tests, 29 ordinary native
tests, eleven CLI restoration workflows, five reference image/window tests,
26 tensor tests and the host-path policy case: 1902 distinct selected tests,
excluding overlapping reruns. Four new reduction regressions failed against the
previous loop-rejecting analysis and pass with bounded fixed-point demand.
The before/after reference oracle retains alternating 12/33 values over zero to
four trips. Its i64 marker avoids re-entering source admission with private bool
transport casts; the `.ns` native fixtures separately cover actual bool state.

Nested mixed-state native/reference probes preserve complete 9/64-word state and
six outer iteration operands, including zero-trip inner/outer loops and nearest
break/continue behavior. Selected nested division traps and skipped paths retain
ordinary native behavior. All eleven CLI workflows keep cache identity, tamper
rejection and identical LLVM/lifecycle state after source-free restoration.

The live CLI reports 1569 clean drift checks with clean coverage, hierarchy and
lineage. Formatting, 2941 local links, UTF-8 and source/test/document line limits
pass. Progress remains `active/86`; sparse typed nested-loop early returns and
exact shared-work-budget failure composition require dedicated native evidence.
This is local macOS aarch64 CPU evidence, not a full-workspace, fresh Linux/GPU,
formal-safety or performance result. Source loop-shape admission, public ABI and
the shared loop protocol remain unchanged.

The subsequent 2026-09-27 nested-return/budget checkpoint passed 1693 selected
frontend/lowering/NIR tests, 149 distinct native/reference bridge tests, 30 ordinary
native tests, twelve CLI workflows, 26 native LLVM unit tests, five reference
image/window tests, 26 tensor tests and the host-path policy case: 1942 distinct
selected tests, excluding repeated and overlapping runs. One ignored LLVM probe
was not executed. The focused return/work-budget suite was rerun after adding
same-process repeated invocation at exact budget exhaustion. An initial CLI assertion
counted only i64 parameters; the corrected assertion counts every typed parameter,
retains exact helper arities and passes all twelve workflows.

The 9/30-word return fixtures preserve current snapshots, lazy suffixes and all
output words. Exact work/entry failures, selected payload traps and invalid scalar
words preserve disjoint sentinels or the fully aliased input. The same callback can
re-enter with fresh counters after exact exhaustion. The 64-word ordinary nested
fixture separately retains exact work limits; its early-return variant is not
silently admitted. Expanded 31/64-word return states report 65/131 private carry
words against the unchanged 64-word bound before native emission.

The live CLI reports 1577 clean drift checks with clean coverage, hierarchy and
lineage. Formatting, 2944 local links, UTF-8 and changed-file line limits pass.
Progress remains `active/86`; the next weakness is private return/control carry
expansion, not an undifferentiated execution gap. No source/public/FFI/callback ABI
or shared loop protocol changed. This is local macOS aarch64 CPU evidence, not a
full-workspace run, fresh Linux/GPU acceptance, formal safety or a speedup claim.

The subsequent 2026-09-27 return-storage checkpoint passed 1699 selected
frontend/lowering/NIR tests, 149 distinct native/reference bridge tests, 31 ordinary
native tests, twelve CLI workflows, 26 native LLVM unit tests, five reference
image/window tests, 26 tensor tests and the host-path policy case: 1949 distinct
selected tests, excluding repeated/overlapping runs. One ignored LLVM probe was
not executed. Positive and negative width assertions were rerun after pinning both
outer N+3 and inner N+7 carries for this fixture.

Revalidated whole-record payload reuse now admits 9/30/31/57-word states. The
nine-word source has 12/16 private carry words and 14/18 iteration arguments,
down from maximum 21 carried words and 22/23 arguments. Full native work-budget,
trap and invalid-input probes cover 9/57 words; all output words, border sentinels,
zero allocation/drop counts and fresh invocation budgets remain checked.
Leading/trailing differential execution matches independent payload storage over
30 cases including selected traps. Unsafe-to-rewrite read order keeps independent
storage, while invalid source never acquires new write/read authority.

All twelve CLI workflows retain warm-cache behavior, pre-open tamper rejection and
identical LLVM/lifecycle state after source-free restoration. The 58/64-word return
variants still reject at 65/67 words in the first over-bound loop; their inner
widths reach 65/71. Further live-carry reduction remains a separate task.
The live CLI reports 1581 clean drift checks with clean coverage, hierarchy and
lineage. Formatting, 2946 local links, UTF-8 and changed-file line limits pass.
Progress remains `active/86`; no callback/source/FFI ABI or shared YIR protocol was
widened. This is local macOS aarch64 CPU evidence, not fresh Linux/GPU execution,
formal safety or measured speedup.

The subsequent 2026-09-27 index-recovery checkpoint passed 1706 selected
frontend/lowering/NIR tests, 149 distinct native/reference bridge tests, 33 ordinary
native tests, twelve CLI workflows, 26 native LLVM unit tests, five reference
image/window tests, 26 tensor tests and the host-path policy case: 1958 distinct
selected tests, excluding repeated/overlapping runs. One ignored LLVM probe was
not executed. The wide bridge run exposed two historical fixtures which now fit
the flattened-input path. Their observed-index cases remain, and larger dead-index
cases restore actual scoped record transport; both passed the focused rerun.

Bounded continuation observation removes only dead leading-index recovery.
Six source shapes preserve thirty independent results across zero through four
trips, including enclosing suffixes, generated branch outputs and parent backedges.
Ordinary native execution retains selected arithmetic traps and skipped work.
The same regressions exposed and repaired scalar branch input collection panicking
on already-lowered loop exits, without weakening original source admission.

The return fixture now executes 9/30/31/57/58-word state with outer N+2 and inner
N+6 carries. Nine-word carry widths fall from 12/16 to 11/15, and CLI iteration
arities from 14/18 to 13/17. Exact 9/58-word budgets, canonical input rejection,
selected payload traps and aliased/disjoint failure atomicity pass. The 59/64-word
variants still reject first at 65/66 private words; no native limit is raised.
All twelve CLI workflows retain cache reuse, pre-open tamper rejection and identical
LLVM/lifecycle state after source-free restoration.

The live CLI reports 1590 clean drift checks with clean coverage, hierarchy and
lineage. Formatting, 2948 local links, UTF-8 and changed-file line limits pass.
Progress remains `active/86`; further nested snapshot/control carry reduction is
separate. No source/public/FFI/callback ABI or shared YIR protocol changed. This is
local macOS aarch64 CPU evidence, not a full-workspace run, fresh Linux/GPU execution,
formal safety or a measured speedup.

### Return Signal Checkpoint (2026-10-02)

The return-signal checkpoint passed 1712 selected frontend/lowering/NIR tests,
149 native/reference bridge tests, 34 ordinary native tests, twelve CLI workflows,
26 native LLVM unit tests, five reference image/window tests, 26 tensor tests and
the host-path policy case: 1965 distinct selected tests, excluding overlapping
focused reruns. One ignored LLVM probe was not run.

Opaque return-rewrite provenance and bounded exit-ownership proof permit one
canonical pending/break word per return-only loop. Same-loop ordinary breaks,
unsupported signal reads/writes and exhausted proofs retain the independent
representation. Child breaks stay local, mixed continues retain their running
control, and payload evaluation still precedes publication. Three hundred
differential source inputs and 36 ordinary native cases retain independent
oracles, source-name hygiene, leading/trailing steps and enclosing branches.

The mixed return fixture now executes 9/30/31/57/58/59-word state with outer N+1
and inner N+5 carries. Nine-word widths fall from 11/15 to 10/14, and private CLI
iteration arities from 13/17 to 12/16. The 57-word case now proves flattened input;
58/59-word cases still require real whole-record transport. Exact 9/59-word shared
budgets, selected payload traps, canonical input rejection and fully aliased or
disjoint failure-atomic output pass. The 60/64-word variants still reject at the
first 65-word private loop; their inner widths reach 65/69. Native bounds are not
raised. All twelve CLI workflows retain cache reuse, pre-open tamper rejection
and identical LLVM/lifecycle state after source-free restoration.

The live CLI reports 1599 clean drift checks with clean coverage, hierarchy and
lineage. Formatting, 2950 local links, 4581 UTF-8 files and changed-file line limits
pass. Progress remains `active/86`; the next task is nested return snapshot carry
reduction with observed exits preserved. No source/public/FFI/callback ABI or
shared YIR protocol changed. This is local macOS aarch64 CPU evidence, not fresh
Linux/GPU execution, full-workspace validation, formal safety or a measured speedup.

### Invariant Snapshot Checkpoint (2026-10-02)

Per-write identity proof now promotes invariant leaves out of innermost return-owned
loops. Changing leaves remain grouped in compact private typed records, not an
unbounded list of independent parameters. Exact original RHS evaluation, source
admission, source-order revalidation and transactional private-type installation
are retained. Sixty differential source inputs compare independent payload
normalization and explicit result/trap oracles with reversed YIR order.

This work exposed a private capture-planning regression: ordinary boolean packing
could appear to fit the parameter bound, then be rejected as an invalid scoped
backedge representation. Scoped planning now selects only complete proven record
seed maps directly; unproven records and independent boolean slots stay flat.
No new loop protocol, special source naming authority or raised bound is used.

The mixed return fixture executes 9/30/31/57/58/59/60/61/62/63-word states, with
outer N+1 and inner N-1 carries. Nine-word widths fall from 10/14 to 8/10.
The 57-word input stays flat; 58 through 63-word cases check actual N-4-word
compact LLVM parameters. Exact 9/63-word budgets, aliased/disjoint atomic failures,
canonical validation before work, selected payload traps, negative-zero and NaN
payload preservation all pass. The 64-word variant retains inner width 63 but
rejects its 65-word outer carry. This is a fixture boundary, not a general
63-word language limit or evidence of measured runtime speedup.

All twelve CLI workflows retain cache reuse, pre-open tamper rejection and exact
LLVM/lifecycle state after source-free restoration. The nine-word return fixture
still has 12/16 private iteration parameters: fewer backedge words do not imply
that every captured input also disappears. Public/source/FFI/callback ABI is unchanged.

Progress remains `active/86`. Remaining outer return/control state and wider
observed-exit liveness are the next boundary; resources and live GPU scheduling
remain separate. This checkpoint is local macOS aarch64 CPU evidence.

Validation passed 1723 selected frontend/lowering/NIR tests, 151 native/reference
bridge tests, 34 ordinary native tests, twelve CLI workflows, 26 native LLVM unit
tests, five reference image/window tests, 26 tensor tests and the host-path policy
case: 1978 distinct selected tests, excluding overlapping focused reruns. One
ignored LLVM probe was not run. The live CLI reports 1607 clean drift checks with
clean coverage, hierarchy and lineage. Formatting, 2953 local links, 4585 UTF-8
files and changed-file line limits pass. This is not full-workspace validation,
fresh Linux/GPU execution, formal safety or measured performance evidence.

### Nested Invariant Checkpoint (2026-10-02)

Invariant promotion now reaches outer return-owned loops through a bounded
per-write fixed point. Summaries include zero trips and every intermediate write,
not just a final backedge. Delayed alias changes and snapshots exposed by ordinary
break/continue paths cannot become false invariants; local aliases do not escape.
Each entry origin only stays equal or becomes unknown. Work/depth exhaustion stays
conservative, without enumerating the number of runtime iterations.

At most two independently bounded attempts are made. If nested promotion fails
admission, the established innermost-only representation is tried from the original
plan. A dedicated regression verifies that valid inner bodies and private types
survive outer rejection, while failed candidates cannot install their definitions.
Original source admission, full RHS evaluation and whole-function revalidation
remain mandatory. The independent-return-storage counterexample still cannot reuse
invalid payload storage; its source carry can separately shed one proven invariant.

The mixed fixture now executes 9/30/31/57/58/59/60/61/62/63/64-word states with
outer N and inner N-1 carries. The former 65/63 boundary at width 64 is now 64/63.
The return/control bit is retained: a proven unchanged tag leaves the outer
backedge instead. Changing that tag still rejects 65 outer words. Exact budgets,
selected payload traps, malformed-input rejection before work, negative-zero/NaN
bits and disjoint/overlapping atomic publication retain their contracts.

The 144-input nested differential matrix checks independent return normalization
and hand-written result/trap oracles, including observed indices, leading/trailing
steps, child breaks/continues and zero trips. Native probes retain all state words
and no aggregate allocation/drop calls. The 9/64-word source-free CLI fixtures
inspect 12/16 and 5/11 private iteration arities respectively, while retaining
repeated events, cache hits, pre-open tamper rejection and exact restored LLVM/state.

Progress remains `active/86`. Broader ordered-read admission across ordinary child
exits is next; observed exits and changed fields keep their necessary state. No
source/public/FFI/callback ABI or shared YIR protocol changed. This is local macOS
aarch64 CPU evidence, not fresh Linux/GPU execution, formal safety or speedup data.

Validation passed 1729 selected frontend/lowering/NIR tests, 151 native/reference
bridge tests, 34 ordinary native tests, thirteen CLI workflows, 26 native LLVM
unit tests, five reference image/window tests, 26 tensor tests and one host-path
policy case: 1985 distinct selected tests, excluding overlapping focused reruns.
The 9/64-word CLI return pair reran after the fallback guard. One optional LLVM
probe remained ignored. This is targeted regression coverage, not a full-workspace
or cross-platform certification.

The freshly built CLI reports 1611 clean drift checks and clean coverage, hierarchy
and lineage. Formatting, 2955 local documentation links, 4588 UTF-8 files and the
line caps for 99 changed/new Rust and Markdown files pass. The selected coordinate
and its next-action lineage remain unchanged apart from the narrower child-exit task.

### Ordinary Child Exit Checkpoint (2026-10-02)

Nested return promotion now crosses the former ordinary-child own-read rejection.
The compact carry receives an explicit self-binding at the original assignment
site before evaluating the complete RHS once. Original source admission and
whole-function revalidation remain mandatory; future sibling reads, parameter
writes and immutable induction/header mutations still reject. A separate bounded
exhaustion fixture verifies exact inner-only body and private-type retention.

Following the accepted candidate through final lowering exposed two additional
gaps: capture hygiene renamed a registered child break identity, and nominal seed
proof did not recognize an existing compact value parameter. Hygiene now preserves
only identities registered for the current helper's callees, not prefixes or an
unrelated function's same-spelled variable. Seed proof reuses bounded scalar layout
parsing and exact nominal checks. No new YIR operation or Nustar dependency is added.

Native probes cover 9/63/64-word states with ordinary child breaks/continues and
zero/one/two child trips, preserving every initial/event/close word. Observed child
exit indices remain correct at 9/63 words. The current 64-word observed-index
fixture retains 63/64/65 carry widths and rejects; it cannot discard the visible
index to fit the bound. The original changed-outer-tag 65-slot rejection also stays.
Selected child traps leave aliased/disjoint output and border sentinels untouched,
with 59/53 remaining from 64/64 loop/helper budgets and zero aggregate allocations
or drops. The original 144-case matrix now requires promotion on child-exit cases,
not only semantic equivalence after a conservative fallback.

The full-width CLI fixture checks limits 0/2/3/4, so source-free restoration must
exercise real child exits as well as zero trips. Repeated events, exact 4/5/11
private iteration arities, cache hits, pre-open tamper rejection and byte-identical
restored LLVM/state remain required.

Progress remains `active/86`. The next task is to reduce invariant carries across
ordinary child exits without losing observed exit indices. Source/public/FFI and
callback ABI, ordered-read rules, private native bounds and resource boundaries
are unchanged. This is local macOS aarch64 CPU evidence, not fresh Linux/GPU,
formal safety, full-workspace certification or measured runtime speedup.

Validation across this turn passed 1734 selected frontend/lowering/NIR tests,
154 native/reference bridge tests, 35 ordinary native tests, fourteen CLI
workflows, 26 native LLVM unit tests, five reference image/window tests, 26 tensor
tests and one host-path policy case: 1995 distinct selected tests. The compiler,
three child-exit native and fourteen CLI gates reran after function-local control
hygiene was tightened; overlapping focused reruns are not counted twice. One
optional LLVM probe remained ignored.

The freshly built CLI reports 1619 clean drift checks, with clean coverage,
hierarchy and selected-task lineage. Formatting, 2963 local documentation links,
4592 UTF-8 files and the line limits for 32 changed/new Rust, Nuis and Markdown
files pass. The selected coordinate remains `active/86`; this checkpoint does not
raise the application-session score or certify a wider native profile.

### Observed Child Invariant Checkpoint (2026-10-02)

This supersedes the preceding checkpoint's full-width observed-index rejection.
The nested attempt now applies per-write invariant proof to ordinary child loops
as well as return-owned loops. Exit ownership remains an independent proof:
ordinary breaks do not borrow the function's pending-return signal. The bounded
innermost-only fallback and original source-order admission remain unchanged.

Following this through native lowering exposed an unread full-record reconstruction
that kept every invariant input live. The private
[ready-copy proof](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_dead_records.rs)
may remove only a unique unread record binding whose exact-typed fields are total
copies of unwritten pure-value parameters. All callers still preflight any changed
signature. Calls, arithmetic, codecs, resources, ambiguous names and partial reads
cannot use this rule. Work/depth exhaustion installs no partial elimination.

Native break/continue probes now execute 9/63/64-word state with observed child
indices across zero/one/two child trips. The child break carries one control word,
or two when index recovery is observed; continue needs no child record backedge.
Outer N and inner N-1 return carry widths stay unchanged. Selected child traps
retain the exact 59/53 residual loop/helper budgets from 64/64, unchanged aliased
and disjoint output, border sentinels and zero aggregate allocations/drops.

The additional [108-case differential matrix](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_child_tests.rs)
covers leading/trailing child steps, zero trips, partial mutation, old snapshots,
observed indices, break/continue and selected traps. Every case requires promotion
and agrees with independent return normalization and a handwritten arithmetic oracle.
Two CLI tests cover the full-width unobserved break and observed break/continue
workflows, repeated events at limits 0/2/3/4, exact private iteration arities
2/5/11, 3/5/11 and 1/5/11, cache hits, pre-open tamper rejection, and byte-identical
LLVM/state after removing source, project manifest and build output.

At this checkpoint, the next task was partly observed reconstructed child snapshots: the dedicated
64-word fixture still exceeds the helper-argument bound. Its checked field use
must not disappear merely to fit. Opaque helper calls do not establish invariance,
and the changed-outer-tag 65-carry rejection remains. Progress stays `active/86`;
no YIR operation, Nustar dependency, callback/public/FFI ABI or native bound changed.
Evidence here is local macOS aarch64 CPU execution, not fresh Linux/GPU execution,
formal safety certification, full-workspace coverage or a measured speedup.

Validation passed 1742 selected frontend/lowering/NIR tests, 156 native/reference
bridge tests, 35 ordinary native tests, fifteen CLI workflows, 26 native LLVM unit
tests, five reference image/window tests, 26 tensor tests and one host-path policy
case: 2006 distinct selected tests. Focused reruns are not counted twice. One
optional LLVM probe remains ignored. The freshly rebuilt CLI reports 1625 clean
drift checks, clean coverage/hierarchy/lineage and the same `active/86` coordinate.
Formatting, 2965 local documentation links, 4596 UTF-8 files and the line limits
for all 37 changed/new Rust, Nuis and Markdown files pass.

### Partial Child Snapshot Checkpoint (2026-10-02)

This supersedes the preceding checkpoint's partial-read rejection. Generated
private helpers now project fields of total input-record reconstructions before
capture planning. A shared exact nominal/type proof accepts only unwritten pure
inputs and their fields, with unique local bindings and lexical branch/loop views.
Local copy chains and subrecords may expose demand; whole escapes retain storage.
The pass is transactional, with 65,536 work units and depth 64. Unsupported syntax,
ambiguous writes, reference/resource inputs and exhausted proofs stay conservative.
Every caller must still accept the projected signature before the candidate is installed.

Native 9/63/64-word break/continue probes preserve observed indices and the checked
`10 / snapshot.count` use. Full-width child helpers use four/two arguments, not a
wider native ABI. The source-free workflow checks exact iteration arities 4/5/11
and 2/5/11, repeated events, cache reuse, pre-open tamper rejection and byte-identical
LLVM/state after deleting the source, project manifest and original build output.
The earlier nine-word nested-return workflow now uses 12/12 iteration arguments,
down from 12/16, while the full-width counterpart remains at 5/11.
Zero-trip calls skip the snapshot division; selected division-by-zero still traps
with residual budgets 59/53, unchanged aliased/disjoint output and border sentinels,
and zero aggregate allocations/drops. Outer N/inner N-1 carry widths are unchanged.

Computed snapshot inputs remain the next boundary. An opaque record-producing call
or an unobserved checked constructor field cannot use total-copy projection. The
opaque-call fixture retains 1/63/64 carry widths but exceeds the argument bound;
the checked-constructor fixture needs 63/65/65 carries and rejects at the carry bound. Original source
admission, changed-outer-tag rejection, shared YIR contracts, Nustar independence,
callback/public/FFI ABI and native bounds are unchanged. Progress stays `active/86`.
These are local macOS aarch64 CPU results, not new GPU/Linux, full-workspace,
performance or formal-safety claims.

Validation passed 1751 selected frontend/lowering/NIR tests, 158 native/reference
bridge tests, 35 ordinary native tests, sixteen CLI workflows, 26 native LLVM tests,
five reference image/window tests, 26 tensor tests and one host-path policy case:
2018 distinct selected tests. Focused reruns are not counted twice; one optional
LLVM probe remains ignored. The rebuilt CLI reports 1630 clean drift checks with
clean coverage, hierarchy and selected-task lineage. Formatting, 2966 local
documentation links, 4598 UTF-8 files and the line limits of all 40 changed/new
Rust, Nuis and Markdown files pass. The selected coordinate remains `active/86`.

### Opaque Child Snapshot Checkpoint (2026-10-02)

This supersedes the opaque child-break/continue argument rejections above. Generated
plain, single-carry and multi-carry helpers admit private read-only records when every caller
proves an unwritten ready-input root. This proof cannot elide a seed or authorize
a backedge. Unsupported caller shapes, computed arguments and nested writes veto it.
The shared descriptor admits exact bool/i32/i64/f32/f64 captures; dynamic induction
and carry mappings remain i64. Reference and LLVM execution share the layout and
word codec, including negative zero and NaN payloads, without owned allocation.

At this checkpoint, native 9/63/64-word child breaks retained opaque computation, observed exit indices and outer N/inner N-1 carries. The formerly 66-argument child needed four private arguments, or five with index recovery.
The observed 64-word CLI workflow kept 5/5/11 iteration arities, repeated events, cache hits, pre-open tamper rejection and byte-identical LLVM/state after deleting source, manifest and original output.
Selected opaque-call traps preserved aliased/disjoint outputs, sentinels and zero aggregate allocation/drop counters, with residual budgets 59/52. An eleven-entry budget trapped before the additional call; zero trips did not evaluate that call.

Single/no-carry continue actions now execute the same 9/63/64-word opaque snapshots.
Shared parsing, dependency/GLM leaf reads, reference execution and native preflight
agree on exact kinds and reject backedge/resource markers inside read-only records.
The shared [plain-call parser](../../crates/yir-core/src/loop_carry_contract/scoped_call.rs)
retains explicit resource capture modes outside records; the native scalar profile still rejects them.
An independent 63-field record plus scalar accumulator executes with either induction order.
Continue calls before/after the guard preserved first/second-trip traps with residual budgets 59/52 and 59/47. Eleven/sixteen-entry limits failed before the extra call; zero trips skipped it, and aliased/disjoint outputs, sentinels and allocation counters stayed unchanged.
The observed full-width continue artifact retained 3/5/11 private iteration arities, repeated events, cache hits, pre-open tamper rejection and byte-identical LLVM/state after deleting source, manifest and original build output.
The checked-constructor 63/65/65 rejection at this checkpoint is superseded below;
opaque carry mutation still cannot establish invariance. No ABI or native bound widened. This
additive shared contract does not introduce a Nustar dependency. Progress remains
`active/86`; this is local macOS aarch64 CPU evidence, not fresh Linux/GPU execution,
full-workspace coverage, a performance measurement or a formal safety claim.

The earlier opaque-continue checkpoint passed 2356 distinct selected tests, with one optional LLVM probe ignored, 1644 clean drift checks and clean coverage/hierarchy/lineage.

### Checked Child Constructor Checkpoint (2026-10-02)

Repeated whole-layout expansion exhausted invariant analysis for wide constructors, forcing the old 63/65/65 fallback. Per-proof immutable nominal layouts cache field ranges; field projection copies only current selected origins. The 65536-work/64-depth limits, source admission and transactional revalidation remain. At this checkpoint, native 9/63/64-word computed snapshots passed break/continue with observed indices; unobserved full-width breaks used 1/63/64 carries. Opaque carry mutations remained unknown. Unused-field traps preserved field order, zero trips, aliased/disjoint output, sentinels and zero aggregate allocation/drop, with 59/52 or 59/53 residual budgets. Checks before/after continue guards retained 59/53 and 59/48 budgets.
Observed source-free break/continue workflows used 5/5/11 and 3/5/11 private arities, with cache hits, repeated events, tamper rejection and byte-identical restored LLVM/state. Changing outer backedges still needed separate reduction. This was local CPU evidence, not new Linux/GPU, unrestricted ABI or formal-safety evidence.
This earlier checkpoint passed 1758 compiler unit tests, 82 native bridge tests, 24 ordinary native tests, one source-free workflow covering break/continue, five reference image sessions, 26 tensor tests and one host-path policy case: 1897 distinct tests, excluding focused reruns.
Its rebuilt CLI reported 1649 clean drift checks and clean coverage/hierarchy/lineage at `active/86`; formatting, 2967 documentation links, 4599 tracked UTF-8 files and all 63 changed/new file limits passed.

### Preheader Snapshot Checkpoint (2026-10-02)

Record, subrecord and field copies retain exact evaluated identities before loop entry. Each assignment's unknown computation gets a fresh snapshot version; opaque calls are not assumed to be identities. At this checkpoint, branch writes and parent-written bindings were conservatively invalidated before subsequent/child entry. Fixed points include zero trips, every intermediate publication and later-trip alias divergence; unknown values never establish equality. Old copies retain their own versions after rebinding.
The baseline return fixture uses inner N-3 and outer N-1 carries (61/63 at width 64). A changed outer tag runs with 61/64; changing both tag and count still rejects at 65 outer words. The return/control slot remains present.
Native/reference state, selected arithmetic, shared budget failures, overlap, sentinels and allocation/drop checks remain intact. Ordinary binaries also exercise delayed aliases, later parent trips and separately evaluated opaque preheader snapshots. The changed-tag full-width artifact uses 5/11 private iteration arguments; baseline 9/64-word workflows use 10/12 and 6/11, while checked child break/continue workflows use 6/6/11 and 4/6/11. All retain repeated events, cache reuse, pre-open tamper rejection and byte-identical LLVM/state after source/manifest deletion and standalone restoration. Private argument counts and carried-word widths are distinct; no speedup is inferred.
Source admission, whole-function revalidation, exact nominal qualifiers and the 65536-work/64-depth/64-word limits remain unchanged. Progress stays `active/86`; this local CPU proof does not certify Linux/GPU execution, performance or formal safety. Verification: 1765 compiler unit tests, 84 native bridge tests, 25 ordinary native tests, nine source-free workflow tests, five reference image sessions, 26 tensor tests and one host-path policy test passed: 1915 distinct selected tests, excluding focused reruns.

### Preheader Branch Join Checkpoint (2026-10-02)

Each admitted arm retains its evaluated snapshot environment. A transactional join keeps only entry-visible names with exact nominal type/width; identical known origins survive, and differing known origins share a fresh fact only when their ordered pair matches on every arm. Unknown leaves never share facts. This preserves relations within the selected arm, not equality between arms; nested joins, separate opaque calls, old versions, swapped-arm mappings, branch-local scope and exhausted proofs remain checked. Parent writes still invalidate inherited equality, and loop fixed points still check every intermediate publication and later-trip divergence.
The admitted 9/63/64-word preheader-if return fixture retains inner N-3/outer N carries after an outer-tag change (61/64 and 5/11 private arguments at width 64); a differing opaque count arm still rejects 65 outer words. Existing source admission, scalar rebinding restrictions, checked RHS evaluation, return/control provenance, budgets, ABI and resource/provider boundaries are unchanged. Progress remains `active/86`; local CPU execution is not Linux/GPU, benchmark or formal-safety evidence.
Verification: 1770 compiler unit tests, 86 native bridge tests, 26 ordinary native tests, three source-free workflow tests (four artifact variants), five reference image sessions, 26 tensor tests and one host-path policy case passed: 1917 distinct selected tests, excluding focused reruns. The restored artifacts retain cache reuse, repeated events, pre-open tamper rejection and byte-identical LLVM/state after deleting source, manifest and build directories. The CLI reports 1662 clean drift checks with clean coverage/hierarchy/lineage at `active/86`; formatting, 2967 local documentation links, tracked UTF-8 and changed-file line limits also passed.

### Parent Re-entry Checkpoint (2026-10-02)

Nested return proof now derives per-field parent-entry facts from the existing bounded loop summary, instead of invalidating every field of a parent-written record. The summary includes zero trips, every intermediate publication and all later trips. Stable leaves retain evaluated origins; varying leaves receive independent fresh identities per binding and leaf, never equality inferred from two unknowns. Exact nominal type/width checks, lexical scope and the 65536-work/64-depth limits remain. Exhaustion still retains an admitted inner-only plan or the original body. Post-loop invalidation at this historical checkpoint is superseded by the [2026-10-03 loop snapshot proof](nuis-native-scalar-loop-snapshots-v1.md).
The baseline carries inner N-4/outer N-1 words (60/63 at width 64), and changed-tag or joined-preheader cases use 60/64. A count-changing parent or one opaque count arm still needs 65 outer carries and rejects. Whole private record input transport begins at width 61 in this fixture, with N-7 mutable inner fields; this is a layout choice, not an ABI threshold change.
The immutable-parameter parent-entry fixture executes 9/63/64-word state across zero, first and later parent trips. A parent tag change prevents retaining first-trip-only equality: the inner/outer layout becomes N-2/N and later trips restore the original parameter tag. Baseline 9/64-word helpers use 9/12 and 6/11 private arguments; the full-width parent-entry case uses 7/12. Fewer carries do not necessarily mean fewer arguments or better performance.
Source admission, checked RHS evaluation, return/control provenance, native bounds, public/callback/FFI ABI and resource/provider boundaries are unchanged. This is a local macOS aarch64 CPU proof, not fresh Linux/GPU execution, a benchmark or formal-safety evidence. Progress remains `active/86`.

Verification: 1774 compiler unit tests, 87 native bridge tests, 26 ordinary native tests, three source-free workflow tests (four artifact variants), five reference image sessions, 26 tensor tests and one host-path policy case passed: 1922 distinct selected tests, excluding focused reruns. Parent-entry, narrow-return and checked-child artifacts retain cache reuse, repeated events, pre-open tamper rejection and byte-identical LLVM/state after deleting source, manifest and original build directories. The rebuilt CLI reports 1668 clean drift checks with clean coverage/hierarchy/lineage at `active/86`; formatting, 2967 local documentation links, 4523 tracked UTF-8 files and all 71 changed/new file limits pass.
