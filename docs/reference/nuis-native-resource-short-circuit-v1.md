# Native Resource Short-Circuit Values V1

This beta-0.19.0 worktree follow-up closes the pure scalar short-circuit boundary
from the [boolean-shadowing checkpoint](nuis-native-resource-boolean-shadowing-v1.md).
It also admits bounded nested resource-read predicates, selected read/helper
composition, logical bool call arguments, ready scalar selection, comparison/XOR
wrappers, direct i64 initialization and inline bool conditions described below.
The resource coordinate remains active/95. This is local native CPU evidence,
not native window integration, provider/device retirement or application closure.

## Shared Helper Contract

Resource callbacks now admit bounded `call_bool` and `call_i64` helpers alongside
`call_owned_struct`. All three share the
[value-call adapter](../../crates/yir-lower-llvm/src/native_resource_session/admission/value_helpers.rs),
canonical i64/bool value arguments, one reachable graph and the existing
root-inclusive depth 8 / 32-function limits. Recursive calls, other scalar call
kinds, resources in helper signatures, loops and effectful helper bodies are not
granted. Record results retain their existing bounds and allocation-free private
transport; scalar results use native `i1` or `i64` returns.

Short-circuit lowering reuses guarded scalar functions. The left value is passed
as an evaluated snapshot. An unselected RHS helper returns before computing its
RHS. The caller's scalar select combines values already protected by that helper;
it does not turn the original RHS into an eager computation.

Pure helpers reuse [strict native checks](../../crates/yir-lower-llvm/src/function_lowering/strict_scalar.rs)
without acquiring the scalar session's runtime work counters. Exact call and
return kinds, bool predicates, comparisons and selects cannot use compatibility
bool/integer coercions. Complete value materialization and terminal returns are
required. Exported callback signatures, owned State, resource guard/tail encoding
and ownership partitions are unchanged.

## Direct Negation

Resource branch-prefix initializers now accept `!(a || helper(...))` directly,
including nested negation and `!!` over AND/OR. The frontend's canonical
`value == false` representation enters the same
[guarded predicate normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical.rs).
Negation wraps the protected result; it neither hoists the RHS nor adds another
helper solely to invert the value. Existing exported signatures and owner edges
are unchanged.

The complete original expression is bounded and typed before helper names are
allocated. Wrapper nodes count against the existing 4096-node, 64-depth and
32-logical-edge limits. A separate predicate-shape permission leaves the older
prefix, loop and call-argument profiles unchanged. That negation-only change did
not grant comparison/XOR wrappers; the owned-resource normalizer below now handles
them through a separate complete-expression proof.
The [normalizer tests](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_negated_logical_tests.rs)
check those budgets, helper counts and rejection without partial mutation.

## Eager Fallback Fix

A negative probe exposed an existing incorrect fallback:
`mode != 0 && load_at(pixels, 99) > 0` emitted the read before ordinary `and`,
even when the left side was false. The
[core expression lowerer](../../tools/nuisc/src/lowering/core_exprs.rs) now rejects
a remaining logical RHS unless it is a bounded total value expression. Calls,
reads, moves, record construction, variant extraction and division/remainder
require guarded lowering.
The check runs before lowering either operand on that fallback path. Ready local
values and total comparisons retain eager value lowering; already outlined
short-circuit helpers retain lazy execution. This is shared compiler behavior,
not a canvas-specific exception.

## Selected Resource Reads

Bool `let`/`const` initializers with nested `&&`/`||` now admit `load_at` on
the borrowed Buffer and `bytes_len` on live Bytes, including canonical
negation and inferred/aliased declarations. For example:

```nuis
let valid = mode != 0 && (load_at(pixels, 0) > 0 || bytes_len(saved) == 48);
```

The [resource predicate normalizer](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs)
rewrites the declaration into existing owned branch pairs. The selected arm
evaluates the RHS, the skipped arm defines the logical result, and both tail-call
one shared continuation containing the original suffix. The LHS is evaluated
once before dispatch. Reads are not hoisted before selection; only their evaluated
scalar results may enter pure helper arguments. No new resource ABI, owner trailer
or runtime allocation is introduced.
Expressions whose reads occur only on a total LHS retain their existing lowering
route rather than consuming additional resource-helper depth.
Existing guard/tail admission independently verifies the complete owner partition.
The Buffer remains borrowed; live Bytes follow existing terminal owner transfers
and cleanup. A new scalar parameter transports the result to the shared suffix.

The complete original expression is checked before outlining: at most 4096 nodes,
depth below 64, at most 32 logical edges, exact bool/i64 operands, direct known Buffer variables and live
Bytes owners. Arithmetic, comparisons and proven pure helper calls may surround
reads inside each operand; resource moves/writes and nonlocal resource
expressions are not admitted by this path. Root-inclusive depth 8, 32 functions
and the existing capture bounds still apply, including generated continuations.
This representation trades helper depth for reuse of the established owner proof;
it is not an unlimited nested expression implementation.

For a lazy left subtree, the normalizer first creates a collision-free bool
snapshot in the old lexical scope, then requeues the enclosing declaration.
The right subtree stays inside its selected owned arm. Each rewrite reduces
the remaining logical subtree; complete suffix bodies are shared rather than
copied. An inner arm already returning to a registered owned continuation
forwards that terminal call directly only when it transfers the exact complete
live owner set and carries no cleanup obligation, avoiding identity helper chains. It does
not bypass guard/tail validation or transfer resources into pure value helpers.

The [nested regressions](../../tools/nuisc/tests/native_resource_session/nested_resource_predicates.rs)
cover five left/right/balanced/negated truth tables across three declaration
forms, each for 120 events, plus prior-write ordering and temporary-name
collisions. Application errors and failed Result allocation leave State
unpublished and zero tracked live blobs. Mutation tests still reject forged
owners, missing lifetime edges, borrowed Bytes parameters and non-bool guards.
The generated graph keeps one implementation per source store. A depth-8
witness succeeds, the next graph depth fails native admission, and deeper source
expansion fails the source bound; the limits are not increased for nesting.

The [read regressions](../../tools/nuisc/tests/native_resource_session/resource_predicates.rs)
exercise four predicates across three declaration forms, each for 120 events.
Early returns retain the original Bytes snapshot, continuing paths publish their
writes, and application errors or failed Result allocation preserve committed
State with zero tracked blobs. A separate 120-event write-order probe ensures
reads observe preceding writes and their bool results survive subsequent writes.

## Selected Helper Arguments

Read results can now compose with completed pure scalar/record helpers:

```nuis
let valid = mode != 0 && positive(load_at(pixels, 0));
let ready = mode != 0 && choose(bytes_len(saved) == 48).value;
```

The [completed-call facade](../../tools/nuisc/src/lowering/buffer_loop_outline/pure_value_calls.rs)
reuses the existing source purity catalog and exact call/field types. It does
not export provisional loop authority or introduce a second purity checker.
The resource predicate proof accepts only bool/i64 argument values. Resources
stay in owned regions; arguments and callee bodies execute only after selection.
Nested scalar calls, helper-computed indices and nested record projections retain
the same source budgets and independent native whole-record and closure checks.
Calls are potentially trapping even when their resource arguments are in bounds.

The [helper/read regressions](../../tools/nuisc/tests/native_resource_session/resource_helper_predicates.rs)
cover seven predicates across explicit, inferred and const-alias declarations,
each for 120 events. Read-after-write order, application errors, failed Result
allocation and zero tracked live blobs are checked. Mutated native signatures
and over-depth combined resource/value call graphs remain rejected. Source
proofs reject effects, unknown signatures, wrong kinds, borrowed signatures,
recursive helpers and ill-typed or effectful wrappers around logical arguments.

## Logical Call Arguments

Bool declarations in the owned resource profile now accept:

```nuis
let valid = mode != 0 && choose(mode < 3 && load_at(pixels, 0) > 0).value;
let ready = choose(valid || bytes_len(saved) == 0).value;
```

The [argument normalizer](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_predicate_arguments.rs)
first receives a completely typed, bounded source tree. It walks eager operands
left-to-right, retaining earlier reads and potentially trapping calculations in
fresh bool/i64 snapshots before later arguments branch. Logical operands become
declarations reprocessed by the existing owned branch outliner; neither their
RHS nor an enclosing logical RHS is hoisted. Nested record projections keep the
record inside its pure value call rather than adding record or resource captures.

Multiple bool arguments are independent: a false earlier argument does not
skip evaluation of the following argument. The original source work/depth/edge
budgets include every argument and field node. Pending call arguments are also
bounded before growing the traversal worklist. Additional snapshots and helper
nodes remain subject to existing capture and closure limits, not new allowances.

The [argument regressions](../../tools/nuisc/tests/native_resource_session/resource_argument_predicates.rs)
cover eight expressions across three declaration forms for 120 events each,
including nested calls, multiple arguments, negation, early returns and shared
continuations. Error and allocation failure retain State and release Bytes;
another 120-event probe checks prior-write order and source-name collisions.
Source tests verify that a rejected complete expression allocates no helper
names and does not change its suffix. Comparison/XOR wrappers around logical
children now reuse this ordered operand staging, as described below.

## Ready Scalar Selection

Conditional scalar helpers can inline to `cpu.select` within owned callbacks:

```nuis
fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }
let valid = load_at(pixels, 3) == word(valid && bytes_len(saved) == 48);
```

The [resource admission](../../crates/yir-lower-llvm/src/native_resource_session/admission.rs)
checks exactly three ready inputs: a bool condition and two values of the same
kind, either i64 or bool. It checks both arms even for a constant condition.
Buffer, Bytes, record and void values are not selectable. There is no ownership
transfer, implicit bool/integer conversion or change to the Bytes-only profile.
The existing LLVM scalar select emitter is reused without a new ABI or opcode.

Select consumes values already evaluated within the current owned region. It
does not make arbitrary operand evaluation lazy: guarded short-circuit reads
still use existing owned branches before their scalar result reaches select.
The native graph retains closure, owner, lifetime and input bounds independently
of source normalization.

The [selection regressions](../../tools/nuisc/tests/native_resource_session/scalar_selection.rs)
cover i64, bool and nested conditional helpers over 120 events each, YIR
round-tripping, exact State and zero tracked live blobs. Negative mutations
cover integer predicates, mixed kinds, resources, records, void, missing/cyclic
values and malformed arity. Constant-condition probes first admit the valid
modified graph, then reject an invalid unselected arm. Errors and failed Result
allocation retain the previous committed State and release owners.

Sixteen additional isolated probes check scalar-select composition with skipped
and selected out-of-bounds reads, negation, helper-body division and bool results.
Conditional helpers also keep trapping then/else return expressions guarded;
ready-value selection does not authorize hoisting their computations.
A canvas delivery variant covers cache reuse, source-free NSB relocation and
standalone binary launch with empty PATH, with exact pixel/scalar results.

## Comparison And XOR Wrappers

Bounded bool declarations in the owned-return resource profile now accept:

```nuis
let valid = (valid && load_at(pixels, 3) == 9) == (mode < 3);
let ready = (valid && load_at(pixels, 3) == 9) ^ (mode < 3 || bytes_len(saved) == 0);
let checked = choose((valid && bytes_len(saved) == 48) != (mode < 3)).value;
```

The complete expression is bounded before recursive type checks or name allocation.
Exact same-kind equality/inequality and bool-only XOR may contain logical children;
ordering comparisons still require i64 operands. The shared operand extractor
walks eager operands left-to-right. It freezes earlier reads/calculations before
later operands branch, but queues each complete logical subtree as a bool
declaration for the existing owned branch outliner. Neither a wrapper nor a helper
argument hoists a logical RHS. Canonical negation, nesting and helper-computed
indices compose through the same mechanism rather than a new opcode or ABI.
Admission still requires a resource read or the existing logical-call-argument
route. Ordinary pure predicates retain their scalar-helper lowering and its
independent signature/closure checks; detecting a logical node is not permission
to move every pure predicate into an owned helper.

XOR and comparisons are eager: a false first operand cannot skip the second one.
The compiler does not turn XOR into AND/OR or use equality to excuse a mandatory
read. All original node/depth/logical-edge budgets and native closure/capture/
owner/lifetime checks remain. Effects, unknown helpers, mixed scalar kinds,
bool ordering and dead resource owners still fail the complete-expression proof.

The [source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates_tests.rs)
check exact staging order, intact lazy subtrees, original-tree bounds and rejection
without partial name/suffix mutation. The [native wrapper regressions](../../tools/nuisc/tests/native_resource_session/resource_wrapped_predicates.rs)
cover twelve compositions across explicit, inferred and const-alias declarations,
each for 120 events. They check AST preservation, YIR round-trip, a single shared
suffix, exact pixel/scalar State and zero tracked live blobs. Write-order/shadowing
and failed Result allocation retain the existing ownership rules.

Thirty added isolated trap probes exercise both-sided logical trees, equality,
inequality, XOR, negation, outer short-circuit gates, mandatory eager operands,
helper-body division and nested buffer indices. A wrapped canvas is also checked
through cache reuse, source-free NSB relocation and empty-PATH binary launch.

## Direct I64 Initializers

The same owned-region normalization now accepts a canonical `i64` declaration:

```nuis
let choice: i64 = word((valid && bytes_len(saved) == 48) ^ (mode < 3));
let value = pick(load_at(pixels, 3), valid && bytes_len(saved) == 48, 0);
const result: Word = number(valid && bytes_len(saved) == 48).value;
```

Here `Word` is an i64 alias and `number` returns a pure scalar record. Shared
inference resolves inferred and aliased declarations before this pass. Admission
proves the complete initializer has exactly the declaration's kind: accepting
i64 does not coerce bools, grant other integer widths or admit resource selection.
Only then does the existing operand extractor freeze eager operands left-to-right
and queue intact lazy subtrees as bool declarations in owned regions. The outer
call, arithmetic, projection or indexed load consumes ready scalar results.
No new opcode, helper resource ABI or ownership path is introduced.

The [initializer source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_scalar_initializers_tests.rs)
check ordered typed temporaries, unchanged lazy subtrees, let/const identity and
atomic rejection before allocating names. Ordinary pure scalar initializers keep
their existing lowering route. The [native regressions](../../tools/nuisc/tests/native_resource_session/resource_i64_initializers.rs)
cover eight compositions in explicit, inferred and const-alias forms, each over
120 events. They retain AST/YIR round trips, one suffix, exact pixels/scalars and
zero tracked live blobs. Separate cases exercise both signed i64 extremes, old
scope reads, prior/later writes, temporary-name collisions, application errors
and injected Result allocation failures.

Twenty-four additional isolated probes distinguish skipped reads from selected
traps, including negation, XOR, nested logical arguments, helper-body division,
conditional scalar returns and computed indices. Mandatory eager arguments and
arithmetic operands still execute, even when a neighboring logical operand is
false. The delivery fixture uses direct i64 initialization through the same
cache, NSB relocation and source-free empty-PATH launch checks.

Closure limits are unchanged. A three-logical-argument composition inside the
outer resource branch exceeds the existing source depth bound and has an explicit
rejection regression. At that checkpoint the corresponding direct inline `if`
condition still rejected; the follow-up below closes the condition path.

## Direct Inline Conditions

The registered owned-return profile now accepts bounded bool conditions without
requiring an explicit local initializer:

```nuis
if word((valid && bytes_len(saved) == 48) ^ (mode < 3)) == 9 {
    return Result.Ok(Canvas { width: width + 10, pixels: move(saved) });
}
```

Complete expression/type/owner/budget proof runs before allocating any name.
A single hygienic bool declaration holds the intact original condition; the
original `if` keeps its original arms and reads that bool. Both statements are
requeued through the existing owned-region machinery. No lazy child is evaluated
during extraction, and neither arm nor the suffix is duplicated. Ready conditions,
plain resource comparisons and ordinary pure predicates keep their previous routes.
No new ABI, ownership partition or resource-carrying pure-helper signature is added.

The [source condition proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_condition_tests.rs)
check exact bool identity, intact lazy trees/arms, source-name collisions, unchanged
suffixes and rejection without partial mutation. Mixed kinds, dead owners, unknown
effects and excessive original-tree depth/work remain rejected. The internal
snapshot counts toward the original 33-input guard limit rather than bypassing it.

The [native condition regressions](../../tools/nuisc/tests/native_resource_session/resource_conditions.rs)
exercise eight truth tables over 120 events each, including negation, wrappers,
scalar/record helper calls and nested reads. Six paired-arm paths return incoming
or local Bytes, or keep incoming owners alive and execute one shared suffix.
Prior/later writes and colliding source names retain once-only predicate evaluation.
Explicit errors and failed Result allocation retain both committed State halves
and release owned values. Thirty-two additional isolated probes exercise direct
conditions using selected/skipped reads, helper-body traps, nested indices and
mandatory eager operands. The inline-condition canvas is also covered by cache,
source-free NSB relocation and empty-PATH delivery checks.

This is not normalization of every expression position. At the direct-condition
checkpoint, returned State fields were the next reproduced gap. The following
extension closes that gap without discarding field order or sibling Bytes transfer.

## Returned State Fields

Registered callbacks can now use resource short-circuit arguments in exact i64
fields of a directly returned mixed State, including a checked Ok payload:

```nuis
let saved: Bytes = copy_bytes(pixels);
return Result.Ok(Canvas {
    width: width + word(mode >= 0 && bytes_len(saved) == 48),
    pixels: move(saved)
});
```

The [field normalizer](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields.rs)
derives layouts from the shared `MixedResourceLayout` contract, not particular
record, field or result names. It admits the existing flat 1..=16 i64 fields plus
one Bytes resource, with exact checked-wrapper identity and generic arguments.
Scalar fields use the existing admitted read/helper expression vocabulary, not
arbitrary i64 expressions. The resource field must copy the borrowed Buffer or
explicitly move a live Bytes local.
The complete original aggregate shares the 4096-node, below-64-depth and
32-logical-edge budget. All field names, kinds, helper purity and live owner
reads are checked before creating any temporary.

Every field is staged in source order, not declaration-layout order. The scalar
initializer remains intact and reenters the existing selected owned-region path.
An earlier Bytes copy or move also stays before later scalar evaluation; a moved
owner cannot be read by a later field, even behind a constant false predicate.
The rebuilt record moves its staged Bytes exactly once. Pure helper signatures,
native closure/capture limits and independent ownership checks are unchanged.

Scope cleanup can bind the aggregate before dropping unused owners. Only a
terminal binding followed by owner drops and return of that same binding is
normalized; neither the binding nor its drops are reordered. This is not a
general aggregate-local or resource-phi grant.

[Source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields_tests.rs)
cover ordered staging, complete-record rejection, aggregate-wide budgets,
checked constructor identity and terminal cleanup. The
[native regressions](../../tools/nuisc/tests/native_resource_session/resource_return_fields.rs)
exercise root/early returns, copied and moved resources before/after scalar
fields, independent plain/checked layout names, 120-event State comparisons,
read-after-move rejection and failed aggregate allocation with spare owners.
Return-field trap probes reuse the selected/skipped i64 expression cases; the
canvas delivery test covers cache, source-free NSB and empty-PATH launch.

## Direct Checked Error Codes

The registered checked profile now accepts an exact i64 error expression with
resource short-circuit arguments, without requiring a source local:

```nuis
let saved: Bytes = copy_bytes(pixels);
return Result.Err(word(mode >= 0 && bytes_len(saved) == 48));
```

The shared return normalizer validates the actual registered Err constructor,
generic arguments, single `value` field, exact i64 result and complete original
expression budget before allocating a name. It requeues one intact typed i64
snapshot followed by the original Err wrapper. The same terminal cleanup-binding
route keeps reads ahead of drops. Invalid types, unknown/effectful helpers,
nonlive owner reads and excess work/depth remain rejected, including invalid
unselected children. Ready codes retain their existing route.

An error continuation still has an owned return ABI even when it captures no
Bytes. The compiler now excludes registered owned-return helpers from speculative
pure-return shortcuts by identity, not by generated-name spelling. This prevents
both arms from becoming eager `call_owned_struct` operations. Native admission
continues rejecting resource layouts on the scalar value-call path.

[Native error regressions](../../tools/nuisc/tests/native_resource_session/resource_error_codes.rs)
cover root and early errors, nested logical/comparison/XOR arguments, scalar record
projections, zero and full-width signed error codes, retained/branch-local/spare
Bytes, prior writes and last-committed-State preservation. Failed Result allocation
remains a transport failure, not an application code. Independent record/result
names and callbacks holding no Bytes exercise the same contracts.
Twenty-six additional isolated probes cover selected/skipped error-code reads,
helper traps, eager operands and borrowed-only returns. Checked-canvas delivery
exercises open/event/close error codes, cache reuse, source-free NSB relocation
and empty-PATH standalone execution without publishing an erroneous frame.
Rewriting every error in that multi-branch canvas exceeds the existing source
closure bound and remains a negative project-build regression. The executable
fixture uses codes 1, 4 and 7 across open/event/close, retaining the other ready
errors and the unchanged closure budget.

## Borrowed Buffer Length

The same guarded scalar vocabulary now admits `buffer_len` on a lexical binding
whose canonical type is exactly `ref Buffer`:

```nuis
let saved: Bytes = copy_bytes(pixels);
return Result.Ok(Canvas {
  width: buffer_len(pixels) + word(mode >= 0 && bytes_len(saved) == 48),
  pixels: move(saved)
});
```

Buffer length remains a word count; Bytes length remains a byte count. Length
reads share the same borrowed-Buffer proof as indexed reads, count toward the
original expression budget and stay inside the selected owned region. Eager
operands become scalar snapshots in source order; lazy subtrees remain intact
until branch outlining. No Buffer pointer enters a pure helper signature.
Non-reference, optional, generic, unknown and computed Buffer operands remain
rejected, including invalid operands hidden behind a constant false gate.
Ready length expressions retain their ordinary route.

The grant covers bool/i64 declarations, direct conditions, scalar fields of the
registered returned State and checked Err codes. Source type/budget checks and
independent native instruction arity/kind checks remain separate. It does not
admit arbitrary aggregate locals, change extents, permit escaping resources or
alter the ownership partition, capture/closure budgets or cleanup rules.

[Length regressions](../../tools/nuisc/tests/native_resource_session/resource_buffer_lengths.rs)
cover scalar/record helper composition, 120 events per declaration/truth-table
case, borrowed-only and owner-retaining conditions, prior/later writes, explicit
errors and failed Result allocation. The returned-field regression also runs
over 1-, 6- and 17-word buffers.
Native kind-mutation tests retain valid dependency edges so their rejection is
independent of missing-node/arity failures. Twelve length-sensitive trap expressions
are exercised as local i64 values, returned State fields and Err codes: skipped
reads remain skipped, selected out-of-range indices and helper-body divisions
trap, and mandatory eager operands are not skipped. The returned-field workflow
also composes word/byte lengths with a skipped invalid index through cache, NSB
relocation and empty-PATH standalone launch.

## Helper-Returned Record Locals

Completed pure helper results can now initialize a value-record local whose
arguments contain guarded resource expressions:

```nuis
let saved: Bytes = copy_bytes(pixels);
let local: Number = number(mode == 0 && buffer_len(pixels) == 6);
return Result.Ok(Canvas { width: local.value, pixels: move(saved) });
```

The original expression must pass the shared work/depth/logical-edge budget.
The source layout catalog must recognize the canonical declared record, and the
completed pure-call proof must return that exact nominal type. A projection from
a nested helper result uses the same proof; equal field shapes do not make two
record names interchangeable. Hidden resources and recursive/unknown layouts
remain outside the source catalog. Invalid unselected operands are still checked.

Only scalar arguments become source-ordered snapshots. Lazy children reenter the
existing owned-region pass intact; the original let/const declaration is then
constructed once in its continuation. Its type and declaration kind are retained.
Ready declarations keep their ordinary route. This does not add records to the
resource capture list or admit record/resource helper parameters. When a later
resource branch needs a result, project its fields into scalar locals first.

Native value-call admission remains separate: completed helper parameters are
i64/bool, record leaves remain i64/bool and the existing 64-slot limit still
applies. The source catalog is not a native ABI grant. Private helper results
remain allocation-free LLVM values; checked State/Result allocation and Bytes
cleanup retain their existing contracts.

[Source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_initializers_tests.rs)
cover exact nominal types, preserved let/const bindings, ordered scalar snapshots,
atomic rejection, whole-expression budgets and invalid hidden record layouts.
[Native regressions](../../tools/nuisc/tests/native_resource_session/resource_record_initializers.rs)
exercise nested records, explicit/inferred/const-alias declarations, multiple
logical arguments, read-before-write snapshots, early returns, projected scalar
continuations, errors and failed Result allocation. Borrowed-only callbacks with
renamed State/Result types retain error identity and zero tracked blobs. Native
slot tests accept 64 leaves and reject 65 even when only one field is projected.
Thirty-six isolated record-initializer probes reuse the existing i64/length trap
expressions. The nested validation record also has a cache/NSB/standalone workflow.

## Record Literal Locals

Nested value-record constructors can now initialize locals with guarded scalar
fields, without a helper wrapper:

```nuis
let saved: Bytes = copy_bytes(pixels);
let local: Flags = Flags { valid: mode == 0 && buffer_len(pixels) == 6 };
return Result.Ok(Canvas { width: word(local.valid), pixels: move(saved) });
```

The [constructor normalizer](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs)
checks the whole original tree before allocating names. Canonical record names,
exact field sets, nested literal identity and i64/bool leaf kinds must match the
source layout catalog. Duplicate, missing, extra and unknown fields are rejected,
as are hidden resources and invalid operands behind false gates. The original
4096-node, depth-below-64 and 32-logical-edge budgets are shared across all fields.

Every scalar leaf is staged in source field order, including mandatory eager
reads and checked arithmetic. Nested literals retain their structure; lazy
subtrees reenter the existing owned-region normalizer intact. The original
let/const identity is preserved. This does not capture whole records across
resource continuations. Record-valued helper fields use the additional once-only
materialization and scalar projection proof described below.

Independent [native admission](../../crates/yir-lower-llvm/src/native_resource_session/mixed_admission.rs)
distinguishes local pure records from owned mixed State. Pure i64/bool trees reuse
the shared 64-leaf and layout-depth limits. Mixed State still requires its exact
registered i64 fields and one live Bytes owner; bool/nested fields do not widen
that contract. This is local value construction, not a new external ABI.

[Source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals_tests.rs)
check atomic rejection, original-tree budgets, nominal identity, declaration
preservation and field order. [Native regressions](../../tools/nuisc/tests/native_resource_session/resource_record_literals.rs)
exercise four compositions across three declaration forms over 120 events each,
nested projections, early returns, projected scalar continuations, independent
registered names, errors and failed Result allocation. Forty isolated probes
cover skipped/selected reads and traps, including mandatory earlier/later fields.
The nested validation literal also has a cache/NSB/empty-PATH delivery witness.

## Record-Valued Helper Fields

Completed pure helper results can now fill record fields inside a local literal:

```nuis
let local: Envelope = Envelope {
    packet: packet(0, mode == 0 && buffer_len(pixels) == 6, 0),
    unused: mode < 3 && bytes_len(saved) > 0
};
```

The complete constructor is proved before any temporary name is allocated. Each
helper result must have the exact nominal field type; projected nested results
use the existing completed pure-call proof. Ready helper fields are also staged
when another field needs normalization, so their reads and traps cannot slide
past later logical fields. Resource/helper parameters and native ABI grants are
unchanged.

A helper executes once into a typed local value. Its i64/bool leaves are then
projected immediately into scalar snapshots; only those snapshots cross later
resource continuations. The final literal reconstructs the same nominal shape.
It does not invoke the helper again for each leaf or add whole records to the
resource capture list. Original expression budgets remain, and a separate
whole-constructor shape walk bounds reconstruction to 4096 nodes and depth below
64, rejects non-i64/bool leaves and counts repeated type-DAG children separately.
Existing native slot, function, capture and closure-depth checks still apply.

[Source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_field_values_tests.rs)
check one helper snapshot followed by exact typed projections, intact lazy
children, declaration identity, atomic rejection, ready routes and complete
expansion budgets. [Native regressions](../../tools/nuisc/tests/native_resource_session/resource_record_field_values.rs)
run five compositions across three declaration forms over 120 events each, with
one emitted helper call site rather than one per leaf. Multiple helper fields,
later scalar continuations, early errors, failed Result allocation and borrowed-only
callbacks with independent registered names retain State and owner checks.
Forty new isolated probes cover selected/skipped operands and mandatory helper
calls on either side of a lazy field, including a trap in an unprojected return
field. A nested validation helper field also has cache/NSB/empty-PATH delivery.

## Previously Bound Record Fields

A record already materialized in the current block can now fill a literal field
before later resource short-circuit logic:

```nuis
let prior: Packet = packet(0, true, 0);
let local: Envelope = Envelope {
    packet: prior,
    unused: mode == 0 && buffer_len(pixels) == 6
};
```

The owned-region walk keeps a block-local record type table separate from its
scalar capture scope. Only completed declarations with exact pure nominal layouts
are entered. Every rebinding first removes the old entry; nonrecord, unknown,
reference, optional and generic types cannot retain stale proof. If/while boundaries
clear the table. Outlined functions start with a fresh table, not their parent's
record bindings. This is not record capture or a record phi contract.

Direct names and nested record projections use this availability proof. The
existing complete-constructor and snapshot-shape checks still precede allocation.
Each local record field is materialized and projected to scalar leaves at its
source position, before later fields branch. Prior Buffer reads are not repeated
after intervening writes. Repeated fields can share the same completed source
value without repeating its helper call.

A prior record field after an earlier lazy field needs the separate liveness
proof below: the record itself cannot enter scalar-only continuation parameters.
Only safe completed projections may move, never eager computation or resource
reads. Scalar captures, helper parameters and native ABI bounds are unchanged.

[Source scope tests](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_local_fields_tests.rs)
cover exact nominal projections, invalidation, malformed later fields and rejection
before temporary names. [Native regressions](../../tools/nuisc/tests/native_resource_session/resource_record_local_fields.rs)
cover six binding forms across three declarations, 120 events per form, repeated
fields, scalar continuations, branch-local early returns, errors and failed
allocation. Borrowed-only callbacks retain independent State/Result names.
The local-field delivery witness follows cache hits, source-free NSB relocation
and empty-PATH binary launch. Isolated probes distinguish skipped/selected later
reads and arithmetic from mandatory earlier record initialization.

## Late Record Field Liveness

Previously bound pure records can now appear after a short-circuit field:

```nuis
let prior: Packet = packet(0, true, 0);
let local: Envelope = Envelope {
    unused: mode == 0 && buffer_len(pixels) == 6,
    packet: prior
};
```

After the complete constructor and expanded shape are proved, a separate planner
walks fields in source order, including nested literal fields. It identifies
existing record paths used after a guarded field. Only those completed i64/bool
projections are frozen before constructor evaluation; the original initializer
is not repeated. New helper calls, resource reads and checked arithmetic are
never moved. Ordinary field staging still evaluates them at their source positions.

A root/field-path cache shares repeated and overlapping scalar leaves. Rebuilt
nominal fields use those snapshots directly, avoiding redundant alias captures.
No record enters a resource continuation signature. The existing source tree,
shape, helper graph, native slot and 33-input guard bounds remain unchanged.
The repeated-wide-field regression accepts 29 leaves and rejects 30 under that
guard bound, rather than expanding its ABI to accommodate snapshots.

[Source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_liveness_tests.rs)
check exact early projections, unmodified eager/lazy expressions, preserved
declarations, overlapping paths, once-only calls and hygienic names. The existing
atomic-rejection proof now also places malformed fields after a lazy field.
[Native regressions](../../tools/nuisc/tests/native_resource_session/resource_record_liveness.rs)
exercise six binding forms across three declarations over 120 events each,
prior reads across later writes, nested and multiple lazy fields, repeated paths,
scalar continuations, branch-local early returns, errors and failed allocation.
Borrowed-only renamed callbacks retain their registered identities.

Late-field trap probes distinguish selected/skipped reads and arithmetic from
mandatory earlier record initialization. The late validation field also has a
cache, source-free NSB relocation and empty-PATH binary delivery witness. The next
section extends this record-valued-field grant to scalar operands inside predicates.

## Local Record Scalar Operands

Exact i64/bool fields of already initialized pure records can now participate in
resource predicates inside bounded record literal initializers:

```nuis
let prior: Packet = packet(0, mode == 0, 0);
let local: Envelope = Envelope {
    unused: prior.flags.valid && bytes_len(saved) == 48,
    packet: prior
};
```

The block-local nominal table proves the complete variable-rooted field path.
After bounding the original expression, admission substitutes type-only scalar
witnesses into a separate proof tree and checks the whole constructor using the
existing resource/helper/type rules. These witnesses are never emitted, evaluated
or constant-folded. Missing fields, wrong kinds, unavailable locals, dead owners,
unknown/effectful helpers and over-budget trees fail before allocating names.
Rebindings and control-flow boundaries still invalidate local record availability.

Only after complete proof does the liveness planner freeze the original completed
scalar projections. The same root/field-path cache serves expressions and later
record-valued fields, without duplicate alias captures. Negation, nested logic,
comparisons, XOR, indexed reads and scalar helper arguments preserve their original
syntax and evaluation order. Resource reads, new calls and checked arithmetic
stay in their original positions; only already computed pure fields move.
Original expression/shape budgets, native slots, helper limits, exact owners and
the 33-input guard bound remain unchanged.

[Source tests](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_operands_tests.rs)
check nominal/type proof, shared snapshots, source preservation, eager/lazy call
arguments, hygiene and atomic rejection.
[Native tests](../../tools/nuisc/tests/native_resource_session/resource_record_operands.rs)
exercise seven expressions across three declaration forms, 120 events each,
completed reads across writes, AST preservation and YIR roundtrips. Repeated-field
capture tests still accept 29 leaves and reject 30. Existing liveness error and
borrowed-only tests now also consume these local scalar operands, preserving
committed State, independent registration names and owner cleanup.

Eighteen isolated probes cover skipped/selected reads and division, negation,
nested conditions, XOR and helper arguments. Eager siblings and original record
initializers remain mandatory even when a later resource operand is skipped.
The project workflow checks the same predicate through cache reuse, source-free
NSB relocation and standalone empty-PATH launch.

The constructor grant does not imply whole-record resource captures or general
lexical joins. The next section extends its operand proof to scalar declarations.

## Local Record Scalar Initializers

The same exact local-record projections now work in standalone bool/i64 declarations:

```nuis
let prior: Packet = packet(0, mode == 0, 0);
let valid: bool = prior.flags.valid && bytes_len(saved) == 48;
return Result.Ok(Canvas { width: word(valid), pixels: move(saved) });
```

The local availability table routes canonical bool/i64 declarations through
complete expression admission. The original tree is bounded before type witnesses
are substituted; both sides of every logical operator retain exact resource,
owner and scalar-kind checks. After proof, the existing liveness planner saves
only referenced, completed scalar fields, deduplicated by root/field path.
The original let/const declaration, type, name and remaining expression are then
requeued into the existing owned-region machinery. No call, read or checked
arithmetic is moved into the pure projection prefix.

Ready/pure declarations that do not need this normalization keep their prior
route. If no local projections remain, the entry does not rewrite again or
allocate names; the ordinary scalar outliner owns the requeued expression.
Whole records do not enter resource captures. Later statements needing a record
across a resource continuation still need explicit scalar projections before
crossing that boundary. Original expression, helper, slot, owner and input bounds
are unchanged.

[Source proofs](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_scalar_initializers_tests.rs)
check bool/i64 and let/const identity, shadow initializer references, shared typed
projections, intact eager/lazy expressions, one-time rewriting and atomic rejection.
Invalid kinds, unavailable paths, invalidated scopes, dead owners, unknown/effectful
helpers and oversized source trees allocate no names.
[Native regressions](../../tools/nuisc/tests/native_resource_session/resource_record_scalar_initializers.rs)
exercise nine compositions across explicit, inferred and const-alias declarations
over 120 events each, including nested helper projections and computed indices.
Completed reads survive later Buffer writes; AST and YIR roundtrips remain intact.
Branch-local bool shadows restore outer fallthrough state. Checked errors,
injected Result allocation failures and independently named borrowed-only callbacks
retain committed State and zero tracked blobs. Repeated paths still accept 29
captured leaves and reject 30 under the existing 33-input guard bound.

Thirty-four additional isolated probes cover standalone bool and i64 forms,
including selected/skipped reads and division, negation, nested logic, XOR and
helper arguments. Bool fixtures require a complete outer `word(...)` call;
an i64 expression merely starting with that call is not a bool fixture.
Mandatory eager operands and original initializers still run.
Both declaration kinds have cache, source-free NSB relocation and empty-PATH
binary launch witnesses. The next section extends the same proof to direct
conditions, without an intermediate source declaration.

## Local Record Conditions

Exact scalar projections of already initialized local records can now appear
directly in a resource `if` condition:

```nuis
let prior: Packet = packet(0, mode == 0, 0);
if prior.flags.valid && bytes_len(saved) == 48 {
    return Result.Ok(Canvas { width: width + 1, pixels: move(saved) });
}
return Result.Ok(Canvas { width: width, pixels: move(saved) });
```

The block-local availability table sends the condition through the same bounded
exact-bool proof and completed-scalar projection planner as declarations. Only
the condition is replaced. Both arms remain intact, with no changes to the
pending suffix; the existing direct-condition route stages the bool and the
owned-region pass controls execution. No-projection and ready/pure paths retain
their old handling. Source budgets, owner checks and native ABI limits are not
relaxed, and no whole record is captured.

The native tests exposed an adjacent borrowed-only error path: an early registered
Err constructor followed by another Err, with no held Bytes, could miss owned-arm
outlining and reach scalar `select`. Both terminal error shapes now match the
registered return layout, variant, type arguments and payload field before the
early arm takes the owned-helper route. Ordinary early errors followed by a
success path retain their existing `guard_return` lowering, without extra helpers.
This is independent of conventional `Result`/`State` names; it does not add resource
selection to the scalar ABI or bypass the native ownership proof.

[Source tests](../../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_condition_tests.rs)
retain unchanged arms, source expressions, suffix and hygienic shared projections,
including one-time rewriting and atomic type/scope/owner/budget rejection.
[Native tests](../../tools/nuisc/tests/native_resource_session/resource_record_conditions.rs)
cover eight compositions across three record binding forms over 120 events each,
completed reads across later writes, AST/YIR roundtrips, both arms' local owners,
early returns and shared fallthrough. Explicit errors and injected Result allocation
failures preserve committed State and cleanup; borrowed-only renamed callbacks
retain registered error identity. The original direct-condition rejection now
executes 120 events.

Generated temporaries count against the unchanged 33-input guard limit. The flat
condition probe accepts 29 record leaves and rejects 30. The nested helper-argument
probe needs an argument bool, a call-result scalar and the condition bool, so it
accepts 27 leaves and rejects 28. Both maximum accepted cases assert exactly 33
guard inputs; repeated record paths still share one snapshot.

Eighteen added isolated probes cover selected/skipped reads, division, nested
logic, negation, XOR and helper arguments. Mandatory eager siblings and original
record initializers still execute. The direct-condition project witness retains
cache reuse, source-free NSB relocation, empty-PATH launch and exact final pixels.
Local-record operands in directly returned State fields remain the next reproduced
gap; later whole-record use across a resource continuation is not granted here.

## Execution Evidence

The [native regressions](../../tools/nuisc/tests/native_resource_session/boolean_short_circuit.rs)
run 120 events for each explicit, inferred and aliased shadow form. AND and OR
compose through bool, i64 and record helpers while four early-return/continuation
paths preserve pixels, scalar State and zero tracked live blobs. AST preservation
and YIR roundtrips are checked. A separately materialized Buffer-read bool feeds
a short-circuit initializer and survives later writes without recomputation.
A nested OR/AND case checks the selected truth table over another 120 events.
The [negated resource regressions](../../tools/nuisc/tests/native_resource_session/boolean_negated_short_circuit.rs)
add 120 events each for explicit, inferred and aliased declarations. Early
returns and continuations restore the outer bool, preserve held snapshots and
leave zero tracked blobs. Errors after prefix writes and injected Result
allocation failures preserve both halves of committed State.

Application errors after prefix writes and injected Result allocation failures
preserve committed State and release owners; close cannot erase earlier failure.
Mutation tests reject signature/ownership drift, wrong arguments, integer
predicates, mixed bool/integer selects and comparisons, effects and external
lifetime dependencies. Closure-limit witnesses use post-optimization YIR so
erased source wrappers cannot falsify the test.

Sixty-eight [isolated process probes](../../tools/nuisc/tests/native_resource_session/boolean_short_circuit_traps.rs)
put division by zero or an out-of-bounds Buffer read on the RHS inside a resource branch, with and without
negation. Skipped AND/OR cases finish with correct State
and cleanup; selected cases terminate by native trap before the post-event
marker. Traps remain process failures, not catchable Result errors or a promise
of in-process cleanup. Probes have a 60-second deadline and parent-owned
temporary-file cleanup.
Helper probes separately test skipped/selected out-of-bounds arguments and
division by zero inside pure helpers fed by valid Buffer/Bytes reads.
Argument probes distinguish outer/inner short-circuit selection and preserve
mandatory evaluation of earlier/later arguments, including two logical arguments.
Nested probes distinguish outer skips, inner AND/OR skips, left-subtree reads,
negated trees and balanced trees whose untaken operands would read out of bounds.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
materializes snapshot length before entering the branches, keeping their
validation prefixes scalar-only before AND/OR extent checks. The paint arm now
uses `const valid: bool = !(width <= 0 || validate_extent(width != words).extent.invalid)`
without a separate temporary. Its delivery witness
checks five events, State `[3, 2, 5, 2]` and exact 3x2 pixels through cache reuse,
source-free NSB relocation and binary-only launch with empty PATH.
The [nested delivery variant](../../tools/nuis/tests/native_resource_session_workflow/early_ok.rs)
places an out-of-bounds read behind a nested OR in the compiled callback. The
selected Bytes-length check skips that read, retaining the same exact State and
pixels through cache reuse, NSB relocation and standalone empty-PATH launch
after both source and project manifest have been removed.
The helper/read variant feeds the selected Bytes length into `nonempty` and
puts `nonempty(load_at(pixels, 99))` on the skipped nested RHS, retaining that
same cache, NSB relocation and source-free standalone result.
The logical-argument variant wraps the nested predicate in a helper returning a
nested validation record. It projects and negates that result while retaining
the same exact State and pixels across the complete delivery ladder.

## Validation

Run from the repository root, sequentially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- logical --skip print_epoch --skip aot_application_bundle --skip native:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Direct-negation checkpoint on 2026-11-13 (before the resource-read follow-up):

| Check | Result |
| --- | --- |
| Native resource session suite | 129 passed, including eight isolated selected/skipped trap probes and direct negation |
| Selected logical compiler regressions | 206 passed |
| Conditional-value compiler regressions | 73 passed, including the three new normalizer checks and existing native execution |
| LLVM lowerer library | 195 passed, 1 existing host-Clang opt-in test ignored |
| Native resource delivery workflow | 5 passed, including cache reuse, source-free NSB and empty-PATH binary launch |
| Tensor suite | 50 passed |
| Repository line budgets | 2 passed |

The two compiler selections overlap and are not an additive test count. The
negation follow-up uses the filters above rather than claiming a fresh full
`logical`-name sweep or a whole-workspace test run. Resource execution and
delivery are validated separately from those filtered compiler regressions.

That checkpoint reported 4603 drift checks, zero failures, clean coverage and
clean hierarchy validation. All other 36 coordinate values retain SHA-256
`b3e520bc09f004ae43d004c48106ee50ee202f6e21bcd4c8f8b672c59848e326`
over their sorted JSON array. The selected coordinate remains active/95 rather
than claiming general resource callback closure.

The single-edge resource-read follow-up uses these focused checks in addition
to the resource/session and delivery commands above:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 resource_predicate -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 resource_predicate -- --test-threads=1
```

It does not claim a fresh full compiler/library sweep, Linux/GPU execution or
measured speedup. The new path reuses the registered resource callback contract;
it does not make arbitrary CPU callbacks resource-safe.

Single-edge resource-read checkpoint on 2026-11-13 (before nested support):

| Check | Result |
| --- | --- |
| Resource predicate type/owner/depth/work checks | 2 passed, including exact 4096-node admission and left-only-read route preservation |
| Complete native resource suite | 132 passed, including 16 isolated selected/skipped trap probes |
| Resource delivery workflow | 5 passed, retaining the existing cache/source-free/empty-PATH witnesses |
| Tensor suite | 50 passed |
| Repository line budgets | 2 passed |
| Architecture, UTF-8 and documentation links | Passed; 29 packages, 6 domains, 74 normal/build edges and 5160 local links |

That checkpoint has 4607 drift checks with zero failures, clean coverage and
clean hierarchy validation. Active/95 and all other 36 coordinate values are
unchanged. Before/after comparison of the other cells, sorted by coordinate and
object key and serialized as compact JSON, gives SHA-256
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.
Its next reproduced gap was the nested resource tree now admitted above.

Nested resource-read verification on 2026-11-13:

| Check | Result |
| --- | --- |
| Resource predicate type/owner/depth/work checks | 4 passed, including exact identity-handoff restrictions and the 32-edge source bound |
| Complete native resource suite | 137 passed, including 32 isolated selected/skipped trap probes |
| Resource delivery workflow | 6 passed; nested source-free delivery rerun after the final handoff restriction also passed |
| Tensor suite | 50 passed |
| Repository line budgets | 2 passed |
| Architecture, UTF-8 and documentation links | Passed; 29 packages, 6 domains, 74 normal/build edges and 5163 local links |

That nested-read checkpoint reported 4611 drift checks with zero failures, clean coverage
and clean hierarchy validation. Resource callbacks remain active/95; the other
36 cells retained the compact-JSON hash recorded above. Read/helper composition was
the next reproduced gap. This receipt is local CPU evidence, not a full-workspace
compiler sweep, fresh Linux/GPU parity or measured performance improvement.

Selected helper/read verification on 2026-11-13:

| Check | Result |
| --- | --- |
| Resource predicate source proofs | 5 passed, including completed purity, exact signatures and unknown/recursive/effectful rejection |
| Complete native resource suite | 142 passed on the final rerun; 48 isolated selected/skipped trap probes |
| Resource delivery workflow | 7 passed, including helper/read cache, NSB and empty-PATH standalone launch |
| Tensor/CLI suite (`cargo test -p nuis --bin nuis dev_tensor`) | 51 passed, including the CLI parser case |
| Repository line budgets | 2 passed |
| Architecture, UTF-8 and documentation links | Passed; 29 packages, 6 domains, 74 normal/build edges, 6316 text files and 5167 local links |

The initial closure mutation probe assumed a scalar source helper survived
optimization. It now selects an actual retained record call and verifies that
the over-depth record chain is present in YIR before checking native rejection;
optimization is not disabled to make the witness pass.
That helper/read checkpoint reported 4615 drift checks, zero failures, clean coverage and clean
hierarchy validation. Resource callbacks remain active/95; all other 36 cells
retain the compact-JSON hash above. No full-workspace sweep, fresh Linux/GPU run
or performance improvement is claimed.

Logical argument verification on 2026-11-13:

| Check | Result |
| --- | --- |
| Resource predicate source proofs | 7 passed, including ordered snapshots, pre-mutation rejection and argument-inclusive budgets |
| Complete native resource suite | 146 passed; 68 isolated selected/skipped trap probes |
| Resource delivery workflow | 8 passed, including nested-record logical arguments through cache, NSB and empty-PATH standalone launch |
| Tensor/CLI suite | 51 passed |
| Repository line budgets | 2 passed |
| Architecture, UTF-8 and documentation links | Passed; 29 packages, 6 domains, 74 normal/build edges, 6318 text files and 5171 local links |

That logical-argument checkpoint reported 4620 drift checks, zero failures, clean coverage and clean
hierarchy validation. Resource callbacks remain active/95; all other 36 cells
retain SHA-256 `1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`
under the compact sorted-JSON comparison above. At that checkpoint, inlined
scalar `select` was an explicit negative regression; the ready-scalar follow-up
above replaces that rejection with execution evidence. Evidence is local native CPU only, without a new
full-workspace sweep, Linux/GPU parity or performance measurement.

Ready scalar selection verification on 2026-11-13:

| Check | Result |
| --- | --- |
| Complete native resource suite | 148 passed on the final rerun, including 84 isolated selected/skipped trap probes |
| Resource delivery workflow | 9 passed; scalar selection preserves exact canvas output through cache, NSB and empty-PATH standalone launch |
| Tensor/CLI suite | 51 passed |
| Repository line budgets | 2 passed |
| Architecture, UTF-8 and documentation links | Passed; 29 packages, 6 domains, 74 normal/build edges, 6319 text files and 5175 local links |

That ready-select checkpoint reported 4624 drift checks, zero failures, clean coverage and clean
hierarchy validation. Resource callbacks remain active/95; the other 36 cells
retain SHA-256 `1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`
under the compact sorted-JSON comparison above. At that checkpoint the next action
was the comparison-wrapper boundary, closed by the follow-up above.

Mutation fixtures preserve required dependency/lifetime edges and first admit
valid constant-condition, record and store nodes before corrupting select inputs.
This separates type rejection from unrelated graph rejection. No complete
workspace sweep, fresh Linux/GPU execution or performance measurement is claimed.

Comparison/XOR wrapper verification on 2026-11-13:

| Check | Result |
| --- | --- |
| Resource predicate source proofs | 9 passed, including eager operand order, intact lazy subtrees, atomic rejection and retained pure-helper routing |
| Complete native resource suite | 152 passed on the final rerun, including 114 isolated selected/skipped trap probes |
| Resource delivery workflow | 10 passed, including the wrapped canvas through cache, NSB relocation and empty-PATH standalone launch |
| Tensor/CLI suite | 51 passed |
| Repository line budgets | 2 passed |
| Architecture, UTF-8 and documentation links | Passed; 29 packages, 6 domains, 74 normal/build edges, 6320 text files and 5178 local links |

The fresh CLI reports 4629 drift checks, zero failures, clean coverage and clean
hierarchy validation. Resource callbacks remain active/95. All other 36 cells
retain SHA-256 `1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`
under the compact sorted-JSON comparison above. At this wrapper checkpoint,
direct i64 initialization was the next reproduced boundary; the follow-up above
closes that declaration path.

The initial broad logical scan also rerouted ordinary pure predicates and broke
two existing helper-shape regressions. Admission now retains its original
resource-read/logical-call-argument qualifier; the existing pure signature and
closure mutation checks pass unchanged. The staging proof also accounts for
separate read and comparison snapshots rather than assuming a fixed temporary
count. The final rerun validates these corrections. Evidence is local native CPU
only, without a full-workspace sweep, fresh Linux/GPU execution or performance
measurement.

### Direct I64 Initializer Validation

The direct-initializer follow-up was validated locally with incremental Cargo
disabled, one build job and serial tests:

| Check | Result |
| --- | --- |
| Source admission and staging proofs | 12 passed |
| Complete native resource suite | 158 passed, including 138 isolated trap probes |
| Focused direct-initializer rerun | 6 passed after strengthening the prior/later-write witness with distinct values |
| Native resource project workflow | 11 passed, including direct i64 initialization through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |
| Documentation host-path rejection | 1 passed |

Documentation links (5182), UTF-8 validation (6322 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages in six
domains with 74 normal/build edges.

The original over-budget three-logical-argument fixture correctly hit the source
closure-depth limit. The accepted matrix stays within that limit, while a
separate negative test preserves the rejection. No closure limit or ownership
partition was widened to accommodate it. At this initializer checkpoint, direct
inline resource conditions were explicitly reproduced as rejected; they are now
handled by the condition follow-up above.

The fresh CLI reports 4634 drift checks, zero failures, clean coverage and clean
hierarchy. The resource coordinate stays active/95. The other 36 complete cells
retain hash `1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`
under the compact sorted-JSON comparison above. This is local CPU evidence, not
a full-workspace run, fresh Linux/GPU validation or a performance measurement.

### Inline Condition Validation (2026-11-20)

Validation resumed after an interrupted run; its partial output is not counted
as a completed suite. The following results are from the fresh local serial run,
with incremental Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Complete native resource suite | 164 passed, including 170 isolated selected/skipped trap probes |
| Source admission and normalization proofs | 15 passed |
| Native resource project workflow | 12 passed, including inline conditions through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |
| Documentation host-path rejection | 1 passed |

The new condition cases retain exact State over 120 events per truth table,
six paired-arm paths with incoming/local Bytes, once-only writes, nonpublishing
errors and allocation-failure cleanup. The internal bool snapshot still consumes
the existing guard input budget. Direct short-circuit expressions in returned
State fields remained a reproduced rejection at this checkpoint; the later
returned-field extension above supersedes that boundary.

The freshly rebuilt CLI reports 4639 drift checks, zero failures, clean coverage
and clean hierarchy. The resource coordinate stays active/95. The other 36
complete cells retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`
under the compact sorted-JSON comparison above. The next action at that checkpoint was the reproduced
returned-State-field boundary, not another declaration or direct-condition gap.

Documentation links (5186), UTF-8 validation (6324 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages in six
domains with 74 normal/build edges. This is local native CPU evidence, not a
full-workspace run, fresh Linux/GPU validation or a performance measurement.

### Returned Field Validation (2026-11-20)

Fresh local serial validation, reusing the existing target directory with
incremental Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Complete native resource suite | 169 passed in 437.62 seconds, including 194 isolated selected/skipped trap probes |
| Source admission and normalization proofs | 21 passed |
| Native resource project workflow | 13 passed in 156.51 seconds, including returned fields through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The original returned-State-field rejection is now a positive native regression.
Root and early returns, fields before/after Bytes transfer, independent plain and
checked layout names, moved-owner rejection and failed Result allocation retain
exact State and zero tracked live blobs on normal and failure completion.
Direct checked Err code expressions remained a negative regression at this checkpoint;
the later error-code extension supersedes that boundary.

The rebuilt CLI reports 4644 drift checks, zero failures, clean coverage and
clean hierarchy. The resource coordinate remains active/95, and the other 36
complete cells retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.
Historical shadowing/helper and source-free evidence checks remain; next-step
assertions at that checkpoint followed the reproduced direct-Err-code boundary rather than
requiring already completed features to remain future work.

Documentation links (5192), UTF-8 validation (6327 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This is not a full-workspace run, fresh
Linux/GPU validation or a performance measurement.

### Error Code Validation (2026-11-20)

Fresh local serial validation reused the existing target directory, with
incremental Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 24 passed |
| Complete native resource suite | 173 passed in 514.24 seconds, including 220 isolated selected/skipped trap probes |
| Native resource project workflow | 14 passed in 194.52 seconds on the final rerun, including direct Err codes through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The borrowed-only Err case initially exposed owned continuations entering the
pure-helper shortcut. Registered owned-return identity now excludes that route;
the independent Outcome/Frame case and selected/skipped trap probes pass without
relaxing native scalar-layout admission. Root and early errors retain exact zero
and full-width signed codes, committed State and zero tracked live blobs.
Failed Result allocation remains distinct from an application error.

The first delivery run rejected the fixture that rewrote every error in the
multi-branch canvas: it exceeded the existing source closure bound. That complete
composition is now an explicit negative build regression. The successful fixture
exercises resource-computed codes 1, 4 and 7 across open/event/close while keeping
the other errors ready and the original closure limit unchanged. The failed run
is not counted as successful delivery evidence.

The rebuilt CLI reports 4650 drift checks, zero failures, clean coverage and clean
hierarchy. The resource coordinate remains active/95; the other 36 complete cells
retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.
At that checkpoint the next action followed the reproduced buffer-length
composition rejection; the length extension above supersedes that boundary.

Documentation links (5194), UTF-8 validation (6328 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This is local native CPU evidence, not a
full-workspace run, fresh Linux/GPU validation or a performance measurement.

### Buffer Length Validation (2026-11-20)

Local serial validation reused the existing target directory, with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 27 passed |
| Complete native resource suite | 178 passed in 672.12 seconds, including 256 isolated selected/skipped trap probes |
| Final focused length regressions | 6 passed in 31.67 seconds after strengthening extent and native-kind witnesses |
| Native resource project workflow | 15 passed in 203.27 seconds, including length composition through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The old length-composition rejection now runs 120 events at each of 1, 6 and
17 words. Seven scalar compositions across three declaration forms retain
120-event State/read-order/owner evidence; inline conditions cover borrowed-only
and held-owner returns. Application errors preserve committed State, failed
Result allocation remains separate, and completion leaves zero tracked blobs.

Malformed arity and missing dependencies are rejected before native kind
admission. The strengthened kind tests therefore retain valid dependency edges
while replacing a Buffer input with an available i64 or Bytes value, then require
the incompatible-input diagnostic. This is independent of source type checking.
The focused rerun includes these strengthened tests; no implementation or ABI
change was needed after the complete native run.

The rebuilt CLI reports 4655 drift checks, zero failures, clean coverage and clean
hierarchy. Resource callbacks remain active/95, and all other 36 complete cells
retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.
At that checkpoint the next action followed the reproduced helper-returned local
record boundary; the record-initializer extension above supersedes it.

Documentation links (5196), UTF-8 validation (6329 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This is local native CPU evidence, not a
full-workspace run, fresh Linux/GPU validation or a performance measurement.

### Record Local Initializer Validation (2026-11-20)

Local serial validation reused the existing target directory, with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 31 passed |
| Complete native resource suite | 184 passed in 569.96 seconds, including 292 isolated selected/skipped trap probes |
| Native resource project workflow | 16 passed in 192.91 seconds, including nested record initialization through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

Six record compositions across explicit, inferred and const-alias declarations
retain 120-event State, read-order and owner evidence per form. Nested record
projection, early returns and later scalar continuations preserve the original
binding and once-only evaluation. Errors and failed Result allocation retain
committed State and zero tracked blobs; borrowed-only callbacks with independent
State/Result names retain their registered error identity.

Independent native admission accepts 64 record leaves and rejects 65 even when
only one leaf is projected. No record-valued resource capture, resource helper
parameter or broader native ABI is granted. At that checkpoint the record-literal
field rejection was the next reproduced boundary; the constructor extension above
supersedes it without silently evaluating a short-circuit RHS.

The rebuilt CLI reports 4661 drift checks, zero failures, clean coverage and clean
hierarchy. Resource callbacks remain active/95, and all other 36 complete cells
retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.

Documentation links (5200), UTF-8 validation (6331 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This is local native CPU evidence, not a
full-workspace run, fresh Linux/GPU validation or a performance measurement.

### Record Literal Validation (2026-11-20)

Local serial validation reused the existing target directory, with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 34 passed |
| LLVM lowerer library | 197 passed, 1 existing host-Clang opt-in test ignored |
| Complete native resource suite | 189 passed in 614.43 seconds, including 332 isolated selected/skipped trap probes |
| Native resource project workflow | 17 passed in 212.40 seconds, including nested literal initialization through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The first minimal literal reached source lowering but failed native admission:
the constructor checker still applied the mixed-State field rules to local bool
records. Pure local construction now has its own bounded value-schema check,
while exact owned State validation still precedes any owner consumption. The
original reproducer and the complete suites above pass after that change.

Native mutation checks cover unavailable/duplicate/malformed fields, hidden
resources, layout delimiters, 64 versus 65 scalar leaves, total nested slot count
and 64 versus 65 layout levels. Failed admission leaves values and owners intact.
The extra eager-field probes prove that an unused earlier/later read or division
is still evaluated even when its neighboring short-circuit field skips its RHS.
Four constructor compositions across three declaration forms retain 120-event
State and owner evidence per form; error and allocation-failure paths retain
committed State and zero tracked blobs.

The rebuilt CLI reports 4667 drift checks, zero failures, clean coverage and clean
hierarchy. Resource callbacks remain active/95, and all other 36 complete cells
retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.
At that checkpoint the next action followed the record-valued helper field
rejection; the once-only field-value extension above supersedes it.

Documentation links (5206), UTF-8 validation (6334 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This is local native CPU evidence, not a
full-workspace run, fresh Linux/GPU validation or a performance measurement.

### Record Helper Field Validation (2026-11-20)

Local serial validation reused the existing target directory, with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 38 passed |
| Complete native resource suite | 194 passed in 663.79 seconds, including 372 isolated selected/skipped trap probes |
| Native resource project workflow | 18 passed in 209.36 seconds, including helper-valued fields through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

Five field compositions across three declaration forms retain 120-event State,
read-order and owner evidence per form. Emitted LLVM has one helper call site
rather than one per projected leaf. Multiple helper fields and later scalar
continuations preserve early returns; application errors and failed Result
allocation retain committed State and zero tracked blobs. Mandatory helper calls
on either side of a lazy field still execute, including a trap in an otherwise
unused helper return field.

Whole-constructor proof precedes temporary allocation. Original expression
budgets and independent native ABI bounds remain, with an additional complete
snapshot-shape bound before expansion. No record-valued resource capture or
resource helper parameter is granted. No LLVM backend implementation changed in
this extension; the earlier LLVM library result is historical, not a fresh run.

The rebuilt CLI reports 4672 drift checks, zero failures, clean coverage and clean
hierarchy. Resource callbacks remain active/95, and all other 36 complete cells
retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.
At that checkpoint the next action was the prior-record field scope boundary;
the block-local availability extension above supersedes it.

Documentation links (5210), UTF-8 validation (6336 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This is local native CPU evidence, not a
full-workspace run, fresh Linux/GPU validation or a performance measurement.

### Previously Bound Record Field Validation (2026-11-20)

Local serial validation reused the existing target directory, with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 41 passed |
| Complete native resource suite | 199 passed in 726.75 seconds, including 410 isolated selected/skipped trap probes |
| Native resource project workflow | 19 passed in 225.33 seconds, including bound-record fields through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The old prior-record rejection now executes 120 events. Six binding forms across
three declarations retain completed reads across later Buffer writes, exact nested
projections, State and zero tracked blobs. Repeated fields use one completed helper
result; later scalar continuations and branch-local early returns preserve cleanup.
Application errors and failed Result allocation never publish either half of State.
Borrowed-only callbacks preserve independently registered State/Result identity.

Thirty-eight added isolated probes retain selected-only later reads and arithmetic,
while mandatory earlier record initialization still traps even when its later
logical neighbor skips. Unit proofs invalidate old bindings and control-flow
boundaries and reject malformed later fields before allocating names. At this
checkpoint a prior record field after an earlier lazy field was still rejected;
the liveness extension above supersedes that boundary without record captures.

The rebuilt CLI reports 4678 drift checks, zero failures, clean coverage and clean
hierarchy. Resource callbacks remain active/95, and all other 36 complete cells
retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.
The next action at that checkpoint was the earlier-lazy-field liveness boundary.

Documentation links (5214), UTF-8 validation (6338 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. No LLVM backend or runtime implementation
changed in this extension. This is local native CPU and source-free delivery
evidence, not a full-workspace run, fresh Linux/GPU validation or a performance
measurement.

### Late Record Field Liveness Validation (2026-11-20)

Local serial validation reused the existing target directory, with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 44 passed |
| Complete native resource suite | 205 passed in 788.15 seconds, including 448 isolated selected/skipped trap probes |
| Native resource project workflow | 20 passed in 237.51 seconds, including late record fields through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The original earlier-lazy-field rejection now executes 120 events. Six binding
forms across three declarations retain completed reads across later writes,
exact nominal/nested projections, State and zero tracked blobs. Repeated and
overlapping paths share scalar snapshots across nested and multiple lazy fields.
Existing record helpers execute once; calls and resource/checked expressions are
not hoisted with their completed projections. Early errors and failed Result
allocation retain both halves of committed State and release every held owner.
Independently named borrowed-only callbacks retain registered error identity.

Thirty-eight added isolated probes exercise late record fields beside selected
and skipped operands, while prior initialization remains mandatory. The repeated
wide-field test accepts 29 captured leaves and rejects 30 under the unchanged
33-input guard bound. No record capture, helper parameter or native ABI grant was
added. At this checkpoint the next rejection was a local scalar record projection
inside a resource predicate; the later operand grant above closes constructor uses.

The rebuilt CLI reports 4684 drift checks, zero failures, clean coverage and clean
hierarchy. Resource callbacks remain active/95, and all other 36 complete cells
retain hash
`1c20aca14f35c3fa8b5831de4e0a652e398c7320ee1e4bb9d73e57b5a938a6d3`.

Documentation links (5219), UTF-8 validation (6341 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. No LLVM backend or runtime implementation
changed in this extension. This is local native CPU and source-free delivery
evidence, not a full-workspace run, fresh Linux/GPU validation or a performance
measurement.

### Local Record Operand Validation (2026-11-20)

Local serial validation reused the existing target directory with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 47 passed |
| Complete native resource suite | 208 passed in 814.31 seconds, including 466 isolated selected/skipped trap probes |
| Native resource project workflow | 21 passed in 247.60 seconds, including local-record predicates through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The original constructor-predicate rejection now executes 120 events. Seven
compositions across three declaration forms retain exact values, completed reads
across later writes, original AST, YIR roundtrips and zero tracked blobs. Shared
projection snapshots preserve the 29-leaf acceptance/30-leaf rejection boundary.
Branch-local checked errors and injected Result allocation failures preserve
committed State and owner cleanup; renamed borrowed-only callbacks keep their
registered identities. Eighteen added isolated probes retain selected-only reads
and traps without skipping mandatory eager siblings or original initializers.

The rebuilt CLI reports 4690 drift checks, zero failures, clean coverage and clean
hierarchy. Resource callbacks remain active/95. All other 36 complete cells match
the preceding CLI snapshot exactly. At this checkpoint the next gap was standalone
bool/i64 declaration admission; the later scalar-initializer grant above closes it.

Documentation links (5224), UTF-8 validation (6344 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This extension changes constructor admission
and scalar snapshot planning, not the LLVM backend, runtime or native ABI.
This is local native CPU and source-free delivery evidence, not a full-workspace
run, fresh Linux/GPU validation or a performance measurement.

### Local Record Scalar Initializer Validation (2026-11-20)

Local serial validation reused the existing target directory with incremental
Cargo disabled and one build job:

| Check | Result |
| --- | --- |
| Source admission and normalization proofs | 50 passed |
| Native resource suite excluding the isolated trap harness | 213 passed in the initial full-suite run |
| Corrected isolated trap harness and fixture-classification regression | 2 passed in 432.66 seconds, including 500 isolated selected/skipped probes |
| Native resource project workflow | 23 passed in 258.36 seconds, including both scalar declaration kinds through cache, source-free NSB relocation and empty-PATH launch |
| Tensor and CLI regressions | 51 passed |
| Repository line limits | 2 passed |

The initial full native run reported 213 passing tests and one harness failure:
`bool_operand_right_take` was misclassified because its i64 expression began
with `word(...)` but had an eager arithmetic suffix. The child failed in fixture
construction, before the event marker. The classifier now requires one complete
outer call, with a regression for arithmetic suffixes and unmatched parentheses.
The entire isolated harness was then rerun, including all 34 new scalar probes.
The other 213 tests were not rerun after this test-only correction; this is not a
claim of a single clean 215-test full-suite run.

The prior standalone-declaration rejection now executes 120 events. Nine
compositions across three declaration forms retain completed reads across later
writes, exact values, declaration identity, original AST, YIR roundtrips and zero
tracked blobs. Branch-local shadows, checked errors, failed Result allocation and
independently named borrowed-only callbacks preserve cleanup and committed State.
Shared projections retain the 29-leaf acceptance/30-leaf rejection boundary.

The rebuilt CLI reports 4696 drift checks, zero failures, clean coverage and clean
hierarchy. One old trap-routing evidence anchor was synchronized with the current
multiline method chain. Resource callbacks remain active/95, and all other 36
complete cells match the preceding CLI snapshot exactly. At this checkpoint the
next task was the direct resource condition using a local-record scalar operand;
the later condition grant above supersedes that negative test.

Documentation links (5228), UTF-8 validation (6346 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This extension reuses exact scalar projection
planning and the existing owned-region normalizer, without changing the LLVM
backend, runtime or native ABI. This is local native CPU and source-free delivery
evidence, not a full-workspace run, fresh Linux/GPU validation or a performance
measurement.

## Remaining Boundaries

The ready-scalar `select` grant does not admit resource or record selection.
Direct i64 initialization with guarded resource arguments is admitted above.
Direct inline bool conditions are admitted through the same bounded path.
Returned mixed-State i64 fields and their terminal cleanup bindings are admitted.
Direct checked Err codes and their terminal cleanup bindings are admitted.
Borrowed Buffer length now composes with the same expressions.
Helper-returned record locals now admit typed scalar argument staging.
Nested record literal locals now admit ordered scalar-leaf staging.
Record-valued helper fields now admit once-only materialization and scalar projection.
Previously bound record fields now admit block-local proof before later resource logic.
Previously bound record fields after earlier logic now use shared safe scalar projections.
Local-record scalar operands inside constructor predicates now share exact snapshots.
Standalone bool/i64 initializers now share those snapshots and retain declaration identity.
Direct resource conditions now reuse exact local-record projections with intact arms.
The next reproduced gap is a directly returned State field using a local-record operand:
`let prior: Packet = packet(0, mode == 0, 0); return Result.Ok(Canvas { width: word(prior.flags.valid && bytes_len(saved) == 48), pixels: move(saved) });`.
It still rejects with `short-circuit RHS requires guarded lowering` instead of
speculating reads. The nested/read-helper/logical-argument/wrapper grant applies
to bool/i64 declarations, direct bool conditions and the bounded mixed-State fields
and checked Err codes above, plus helper-returned and nested-literal record locals,
not arbitrary record expressions or record-valued resource captures, additional resource
primitives, loops or other callback profiles.
Unsupported expressions retain
`short-circuit RHS requires guarded lowering`; outer negation does not widen
their permissions. Unconditional scalar snapshots remain a
separate facility, not a substitute for conditional resource access.

General resource loops, escaping captured writes, broader State layouts,
resource-changing joins, resize/cancellation and provider/device completion
remain separate. Clang finalization and allocation compatibility functions
remain. No pure Nsld, all-Nuis runtime, fresh Linux/GPU parity or measured
performance improvement is claimed.
