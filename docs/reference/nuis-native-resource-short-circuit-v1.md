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
The next action follows the reproduced buffer-length composition rejection below.

Documentation links (5194), UTF-8 validation (6328 files), scoped formatting and
diff whitespace checks pass. Workspace architecture remains 29 packages, six
domains and 74 normal/build edges. This is local native CPU evidence, not a
full-workspace run, fresh Linux/GPU validation or a performance measurement.

## Remaining Boundaries

The ready-scalar `select` grant does not admit resource or record selection.
Direct i64 initialization with guarded resource arguments is admitted above.
Direct inline bool conditions are admitted through the same bounded path.
Returned mixed-State i64 fields and their terminal cleanup bindings are admitted.
Direct checked Err codes and their terminal cleanup bindings are admitted.
The next reproduced gap is inline Buffer length composed with resource short-circuit arguments:
`return Result.Ok(Canvas { width: buffer_len(pixels) + word(mode >= 0 && bytes_len(saved) == 48), pixels: move(saved) });`.
It still rejects with `short-circuit RHS requires guarded lowering` instead of
speculating reads. The nested/read-helper/logical-argument/wrapper grant applies
to bool/i64 declarations, direct bool conditions and the bounded mixed-State fields
and checked Err codes above, not arbitrary aggregate locals, additional resource
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
